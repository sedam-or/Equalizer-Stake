//! Tenant-scoped envelope encryption and blind indexes (SEC-01..04, SBD-48..50).

use crate::db::Tx;
use crate::ids::TenantId;
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::stream::{DecryptorBE32, EncryptorBE32};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use moka::future::Cache;
use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretBox};
use sha2::Sha256;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
use zeroize::Zeroizing;

// technical: SEC-01 uses 256-bit derived keys.
const KEY_BYTES: usize = 32;
// technical: AES-256-GCM uses a 96-bit nonce.
const GCM_NONCE_BYTES: usize = 12;
// technical: the GCM authentication tag is 128 bits.
const GCM_TAG_BYTES: usize = 16;
// technical: PostgreSQL xmin is a 32-bit xid while txid_current returns xid8.
const POSTGRES_XID_MODULUS: i64 = 4_294_967_296;
// technical: SEC-03 fixes file streaming chunks at 1 MiB.
const STREAM_CHUNK_BYTES: usize = 1024 * 1024;
// technical: XChaCha20-Poly1305's 24-byte nonce minus STREAM BE32's 5-byte counter/flag.
const STREAM_NONCE_PREFIX_BYTES: usize = 19;
// technical: version byte for the SEC-02 field envelope format.
const FIELD_ENVELOPE_VERSION: u8 = 0x01;
// technical: version byte for the SEC-03 file envelope format.
const FILE_ENVELOPE_VERSION: u8 = 0x02;
// technical: T-103 requires DEKs to expire from the in-memory cache after ten minutes.
const KEY_CACHE_TTL: Duration = Duration::from_secs(10 * 60);

type SecretKey = SecretBox<Zeroizing<[u8; KEY_BYTES]>>;
type CachedDek = Arc<SecretKey>;
type HmacSha256 = Hmac<Sha256>;

mod repo;

/// SEC-01, SBD-50: master-derived keys are zeroized on drop and redacted on Debug.
pub struct Keys {
    pub jwt: SecretKey,
    pub kek: SecretKey,
    pub bidx_base: SecretKey,
    pub filesig: SecretKey,
    pub backup: SecretKey,
    pub csrf: SecretKey,
    pub ics: SecretKey,
    pub kek_previous: Option<SecretKey>,
}

impl Keys {
    /// SEC-01: derives all keys with their fixed, versioned HKDF labels.
    pub fn derive(master: &[u8], previous: Option<&[u8]>) -> Self {
        Self {
            jwt: derive_key(master, b"eq/jwt/v1"),
            kek: derive_key(master, b"eq/kek/v1"),
            bidx_base: derive_key(master, b"eq/bidx/v1"),
            filesig: derive_key(master, b"eq/filesig/v1"),
            backup: derive_key(master, b"eq/backup/v1"),
            csrf: derive_key(master, b"eq/csrf/v1"),
            ics: derive_key(master, b"eq/ics/v1"),
            kek_previous: previous.map(|master| derive_key(master, b"eq/kek/v1")),
        }
    }
}

impl fmt::Debug for Keys {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Keys([redacted])")
    }
}

#[expect(
    clippy::expect_used,
    reason = "SEC-01 uses fixed 32-byte HKDF outputs, below HKDF-SHA256's maximum"
)]
fn derive_key(master: &[u8], label: &[u8]) -> SecretKey {
    let hkdf = Hkdf::<Sha256>::new(None, master);
    let mut output = Zeroizing::new([0_u8; KEY_BYTES]);
    // A 32-byte output with a fixed, short label cannot exceed HKDF's limit.
    hkdf.expand(label, &mut output[..])
        .expect("fixed HKDF output length is valid");
    SecretBox::new(Box::new(output))
}

/// SEC-02: authenticated context binding a field ciphertext to its owning row.
#[derive(Debug, Clone, Copy)]
pub struct Aad<'a> {
    pub tenant: TenantId,
    pub table: &'a str,
    pub column: &'a str,
    pub row_id: Uuid,
}

/// SEC-01..04: crypto failures without key material or plaintext.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("crypto database operation failed")]
    Database(#[from] sqlx::Error),
    #[error("ciphertext authentication failed")]
    Integrity,
    #[error("malformed encrypted envelope")]
    InvalidEnvelope,
    #[error("crypto operation failed")]
    Operation,
    #[error("stream I/O failed")]
    Io(#[from] io::Error),
    #[error("phone number is invalid for the supplied region")]
    InvalidPhone,
    #[error("transaction tenant does not match the crypto request")]
    TenantMismatch,
}

/// SEC-02: field encryption interface shared by domain modules.
#[async_trait::async_trait]
pub trait FieldCipher {
    /// SEC-02: encrypts bytes with the tenant DEK and row-bound AAD.
    async fn encrypt(&self, tx: &mut Tx, aad: &Aad<'_>, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// SEC-02: authenticates AAD and returns zeroizing plaintext bytes.
    async fn decrypt(
        &self,
        tx: &mut Tx,
        aad: &Aad<'_>,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError>;
}

/// SEC-02..03, SBD-50: provides versioned tenant DEKs and authenticated field/file envelopes.
pub struct Keyring {
    keys: Arc<Keys>,
    cache: Cache<(TenantId, i32), CachedDek>,
}

impl Keyring {
    /// SEC-02: creates a keyring with a per-tenant, ten-minute DEK cache.
    pub fn new(keys: Keys) -> Self {
        Self {
            keys: Arc::new(keys),
            cache: Cache::builder().time_to_live(KEY_CACHE_TTL).build(),
        }
    }

    /// Encrypts a field with its current tenant DEK and the supplied row context.
    pub async fn encrypt_field(
        &self,
        tx: &mut Tx,
        aad: &Aad<'_>,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        ensure_tx_tenant(tx, aad.tenant).await?;
        let (version, dek) = self.current_dek(tx, aad.tenant).await?;
        let cipher =
            Aes256Gcm::new_from_slice(dek.expose_secret().as_ref()).map_err(|_| CryptoError::Operation)?;
        let mut nonce = [0_u8; GCM_NONCE_BYTES];
        OsRng.fill_bytes(&mut nonce);
        let context = aad_bytes(aad);
        let nonce = Nonce::from(nonce);
        let encrypted = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: context.as_bytes(),
                },
            )
            .map_err(|_| CryptoError::Operation)?;

        let version = u32::try_from(version).map_err(|_| CryptoError::InvalidEnvelope)?;
        let mut envelope = Vec::with_capacity(1 + 4 + GCM_NONCE_BYTES + encrypted.len());
        envelope.push(FIELD_ENVELOPE_VERSION);
        envelope.extend_from_slice(&version.to_be_bytes());
        envelope.extend_from_slice(&nonce);
        envelope.extend_from_slice(&encrypted);
        Ok(envelope)
    }

    /// Decrypts a field using the DEK version embedded in its envelope.
    pub async fn decrypt_field(
        &self,
        tx: &mut Tx,
        aad: &Aad<'_>,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        ensure_tx_tenant(tx, aad.tenant).await?;
        if ciphertext.len() < 1 + 4 + GCM_NONCE_BYTES + GCM_TAG_BYTES
            || ciphertext[0] != FIELD_ENVELOPE_VERSION
        {
            return Err(CryptoError::InvalidEnvelope);
        }
        let version = i32::try_from(u32::from_be_bytes(
            ciphertext[1..5]
                .try_into()
                .map_err(|_| CryptoError::InvalidEnvelope)?,
        ))
        .map_err(|_| CryptoError::InvalidEnvelope)?;
        let dek = self.dek_for_version(tx, aad.tenant, version).await?;
        let cipher =
            Aes256Gcm::new_from_slice(dek.expose_secret().as_ref()).map_err(|_| CryptoError::Operation)?;
        let nonce_bytes: [u8; GCM_NONCE_BYTES] = ciphertext[5..5 + GCM_NONCE_BYTES]
            .try_into()
            .map_err(|_| CryptoError::InvalidEnvelope)?;
        let nonce = Nonce::from(nonce_bytes);
        let context = aad_bytes(aad);
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &ciphertext[5 + GCM_NONCE_BYTES..],
                    aad: context.as_bytes(),
                },
            )
            .map_err(|_| CryptoError::Integrity)?;
        Ok(Zeroizing::new(plaintext))
    }

    /// Encrypts arbitrary input in bounded 1 MiB XChaCha20-Poly1305 STREAM chunks.
    pub async fn encrypt_stream<R, W>(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        mut reader: R,
        mut writer: W,
    ) -> Result<(), CryptoError>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        ensure_tx_tenant(tx, tenant).await?;
        let (version, dek) = self.current_dek(tx, tenant).await?;
        let version = u32::try_from(version).map_err(|_| CryptoError::InvalidEnvelope)?;
        let cipher = XChaCha20Poly1305::new_from_slice(dek.expose_secret().as_ref())
            .map_err(|_| CryptoError::Operation)?;
        let mut nonce_prefix = [0_u8; STREAM_NONCE_PREFIX_BYTES];
        OsRng.fill_bytes(&mut nonce_prefix);
        let mut header = Vec::with_capacity(1 + 4 + STREAM_NONCE_PREFIX_BYTES);
        header.push(FILE_ENVELOPE_VERSION);
        header.extend_from_slice(&version.to_be_bytes());
        header.extend_from_slice(&nonce_prefix);
        writer.write_all(&header).await?;

        let nonce = (&nonce_prefix).into();
        let mut encryptor = EncryptorBE32::from_aead(cipher, nonce);
        let mut pending = None;
        loop {
            let (chunk, last) = read_bounded(&mut reader, STREAM_CHUNK_BYTES, &mut pending).await?;
            if last {
                let encrypted = encryptor
                    .encrypt_last(chunk.as_slice())
                    .map_err(|_| CryptoError::Operation)?;
                writer.write_all(&encrypted).await?;
                break;
            }
            let encrypted = encryptor
                .encrypt_next(chunk.as_slice())
                .map_err(|_| CryptoError::Operation)?;
            writer.write_all(&encrypted).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    /// Decrypts a bounded-memory XChaCha20-Poly1305 STREAM envelope.
    pub async fn decrypt_stream<R, W>(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        mut reader: R,
        mut writer: W,
    ) -> Result<(), CryptoError>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        ensure_tx_tenant(tx, tenant).await?;
        let mut header = [0_u8; 1 + 4 + STREAM_NONCE_PREFIX_BYTES];
        read_header(&mut reader, &mut header).await?;
        if header[0] != FILE_ENVELOPE_VERSION {
            return Err(CryptoError::InvalidEnvelope);
        }
        let version = i32::try_from(u32::from_be_bytes(
            header[1..5]
                .try_into()
                .map_err(|_| CryptoError::InvalidEnvelope)?,
        ))
        .map_err(|_| CryptoError::InvalidEnvelope)?;
        let dek = self.dek_for_version(tx, tenant, version).await?;
        let cipher = XChaCha20Poly1305::new_from_slice(dek.expose_secret().as_ref())
            .map_err(|_| CryptoError::Operation)?;
        let nonce_prefix: [u8; STREAM_NONCE_PREFIX_BYTES] =
            header[5..].try_into().map_err(|_| CryptoError::InvalidEnvelope)?;
        let nonce = (&nonce_prefix).into();
        let mut decryptor = DecryptorBE32::from_aead(cipher, nonce);
        let mut pending = None;
        loop {
            let (chunk, last) =
                read_bounded(&mut reader, STREAM_CHUNK_BYTES + GCM_TAG_BYTES, &mut pending).await?;
            if chunk.len() < GCM_TAG_BYTES {
                return Err(CryptoError::InvalidEnvelope);
            }
            if last {
                let plaintext = decryptor
                    .decrypt_last(chunk.as_slice())
                    .map_err(|_| CryptoError::Integrity)?;
                let plaintext = Zeroizing::new(plaintext);
                writer.write_all(&plaintext).await?;
                break;
            }
            if chunk.len() != STREAM_CHUNK_BYTES + GCM_TAG_BYTES {
                return Err(CryptoError::InvalidEnvelope);
            }
            let plaintext = decryptor
                .decrypt_next(chunk.as_slice())
                .map_err(|_| CryptoError::Integrity)?;
            let plaintext = Zeroizing::new(plaintext);
            writer.write_all(&plaintext).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    /// Encrypts UTF-8 field content through the same authenticated field envelope.
    pub async fn encrypt_text(
        &self,
        tx: &mut Tx,
        aad: &Aad<'_>,
        plaintext: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        self.encrypt_field(tx, aad, plaintext.as_bytes()).await
    }

    /// Decrypts a UTF-8 field and returns zeroizing bytes for the caller to decode.
    pub async fn decrypt_text(
        &self,
        tx: &mut Tx,
        aad: &Aad<'_>,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        self.decrypt_field(tx, aad, ciphertext).await
    }

    async fn current_dek(&self, tx: &mut Tx, tenant: TenantId) -> Result<(i32, CachedDek), CryptoError> {
        if let Some(version) = repo::current_version(tx, tenant).await? {
            return Ok((version, self.dek_for_version(tx, tenant, version).await?));
        }

        let mut raw_dek = Zeroizing::new([0_u8; KEY_BYTES]);
        OsRng.fill_bytes(raw_dek.as_mut());
        let wrapped = wrap_dek(&self.keys, &raw_dek)?;
        let next = repo::next_version(tx, tenant).await?;
        repo::insert_key(tx, tenant, next, wrapped.as_slice()).await?;

        let Some(active_version) = repo::current_version(tx, tenant).await? else {
            return Err(CryptoError::Operation);
        };
        let active_dek = self.dek_for_version(tx, tenant, active_version).await?;
        Ok((active_version, active_dek))
    }

    async fn dek_for_version(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        version: i32,
    ) -> Result<CachedDek, CryptoError> {
        if version <= 0 {
            return Err(CryptoError::InvalidEnvelope);
        }
        let cache_key = (tenant, version);
        if let Some(dek) = self.cache.get(&cache_key).await {
            return Ok(dek);
        }
        let Some((dek_enc, uncommitted)) = repo::wrapped_key(tx, tenant, version).await? else {
            return Err(CryptoError::Integrity);
        };
        let unwrapped = unwrap_dek(&self.keys, &dek_enc)?;
        let dek = Arc::new(SecretBox::new(Box::new(unwrapped)));
        if !uncommitted {
            self.cache.insert(cache_key, Arc::clone(&dek)).await;
        }
        Ok(dek)
    }
}

impl fmt::Debug for Keyring {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Keyring([redacted])")
    }
}

#[async_trait::async_trait]
impl FieldCipher for Keyring {
    async fn encrypt(&self, tx: &mut Tx, aad: &Aad<'_>, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.encrypt_field(tx, aad, plaintext).await
    }

    async fn decrypt(
        &self,
        tx: &mut Tx,
        aad: &Aad<'_>,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        self.decrypt_field(tx, aad, ciphertext).await
    }
}

/// SEC-04: canonical email form (trim, lowercase, NFC).
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase().nfc().collect()
}

/// SEC-04: parses a phone in the tenant's ISO alpha-2 region and returns E.164.
pub fn normalize_phone(phone: &str, default_region: &str) -> Result<String, CryptoError> {
    if phone.chars().any(char::is_alphabetic) {
        return Err(CryptoError::InvalidPhone);
    }
    let region = region_from_iso_alpha2(default_region)?;
    let parsed = rlibphonenumber::PHONE_NUMBER_UTIL
        .parse(phone, Some(region))
        .map_err(|_| CryptoError::InvalidPhone)?;
    if !parsed.is_valid() {
        return Err(CryptoError::InvalidPhone);
    }
    Ok(parsed
        .format_as(rlibphonenumber::enums::PhoneNumberFormat::E164)
        .into_owned())
}

fn region_from_iso_alpha2(country_code: &str) -> Result<rlibphonenumber::enums::Region, CryptoError> {
    if country_code.len() != 2 || !country_code.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        return Err(CryptoError::InvalidPhone);
    }
    let country_code = country_code.to_ascii_uppercase();
    rlibphonenumber::PHONE_NUMBER_UTIL
        .get_supported_regions()
        .find(|region| region.as_region_str().eq_ignore_ascii_case(&country_code))
        .ok_or(CryptoError::InvalidPhone)
}

/// SEC-04: returns a tenant-scoped, 128-bit blind index for normalized email.
pub fn bidx_email(keys: &Keys, tenant: TenantId, email: &str) -> Result<[u8; 16], CryptoError> {
    blind_index(keys, tenant, b"email", normalize_email(email).as_bytes())
}

/// SEC-04: returns a tenant-scoped phone blind index and its normalized E.164 value.
pub fn bidx_phone(
    keys: &Keys,
    tenant: TenantId,
    phone: &str,
    default_region: &str,
) -> Result<([u8; 16], String), CryptoError> {
    let normalized = normalize_phone(phone, default_region)?;
    let index = blind_index(keys, tenant, b"phone", normalized.as_bytes())?;
    Ok((index, normalized))
}

/// SEC-04/CAN-10: returns a tenant-scoped blind index for an already-canonical E.164 number.
pub fn bidx_phone_e164(keys: &Keys, tenant: TenantId, phone: &str) -> Result<[u8; 16], CryptoError> {
    let canonical =
        crate::fields::Phone::try_from(phone.to_owned()).map_err(|_| CryptoError::InvalidPhone)?;
    blind_index(keys, tenant, b"phone", canonical.as_str().as_bytes())
}

fn blind_index(
    keys: &Keys,
    tenant: TenantId,
    kind: &[u8],
    normalized: &[u8],
) -> Result<[u8; 16], CryptoError> {
    let hkdf = Hkdf::<Sha256>::new(None, keys.bidx_base.expose_secret().as_ref());
    let mut tenant_key = Zeroizing::new([0_u8; KEY_BYTES]);
    hkdf.expand(tenant.0.as_bytes(), tenant_key.as_mut())
        .map_err(|_| CryptoError::Operation)?;
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(tenant_key.as_ref()).map_err(|_| CryptoError::Operation)?;
    mac.update(kind);
    mac.update(&[0]);
    mac.update(normalized);
    let full = mac.finalize().into_bytes();
    let mut index = [0_u8; 16];
    index.copy_from_slice(&full[..16]);
    Ok(index)
}

async fn ensure_tx_tenant(tx: &mut Tx, tenant: TenantId) -> Result<(), CryptoError> {
    let current_tenant = repo::transaction_tenant(tx).await?;
    let expected = tenant.0.to_string();
    if current_tenant.as_deref() != Some(expected.as_str()) {
        return Err(CryptoError::TenantMismatch);
    }
    Ok(())
}

fn aad_bytes(aad: &Aad<'_>) -> String {
    format!("{}|{}|{}|{}", aad.tenant.0, aad.table, aad.column, aad.row_id)
}

fn wrap_dek(keys: &Keys, dek: &[u8; KEY_BYTES]) -> Result<Vec<u8>, CryptoError> {
    let cipher =
        Aes256Gcm::new_from_slice(keys.kek.expose_secret().as_ref()).map_err(|_| CryptoError::Operation)?;
    let mut nonce = [0_u8; GCM_NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce);
    let nonce = Nonce::from(nonce);
    let encrypted = cipher
        .encrypt(&nonce, dek.as_slice())
        .map_err(|_| CryptoError::Operation)?;
    let mut wrapped = Vec::with_capacity(GCM_NONCE_BYTES + encrypted.len());
    wrapped.extend_from_slice(&nonce);
    wrapped.extend_from_slice(&encrypted);
    Ok(wrapped)
}

fn unwrap_dek(keys: &Keys, wrapped: &[u8]) -> Result<Zeroizing<[u8; KEY_BYTES]>, CryptoError> {
    if wrapped.len() < GCM_NONCE_BYTES + GCM_TAG_BYTES {
        return Err(CryptoError::Integrity);
    }
    let nonce_bytes: [u8; GCM_NONCE_BYTES] = wrapped[..GCM_NONCE_BYTES]
        .try_into()
        .map_err(|_| CryptoError::Integrity)?;
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = &wrapped[GCM_NONCE_BYTES..];
    let decrypt = |key: &SecretKey| {
        Aes256Gcm::new_from_slice(key.expose_secret().as_ref())
            .map_err(|_| CryptoError::Operation)?
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CryptoError::Integrity)
    };
    let plaintext = match decrypt(&keys.kek) {
        Ok(plaintext) => plaintext,
        Err(_) => match &keys.kek_previous {
            Some(previous) => decrypt(previous)?,
            None => return Err(CryptoError::Integrity),
        },
    };
    let plaintext = Zeroizing::new(plaintext);
    if plaintext.len() != KEY_BYTES {
        return Err(CryptoError::Integrity);
    }
    let mut dek = Zeroizing::new([0_u8; KEY_BYTES]);
    dek.copy_from_slice(&plaintext);
    Ok(dek)
}

async fn read_bounded<R>(
    reader: &mut R,
    limit: usize,
    pending: &mut Option<Zeroizing<u8>>,
) -> Result<(Zeroizing<Vec<u8>>, bool), CryptoError>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = Zeroizing::new(vec![0_u8; limit + 1]);
    let mut filled = 0;
    if let Some(byte) = pending.take() {
        buffer[0] = *byte;
        filled = 1;
    }
    while filled < buffer.len() {
        let read = reader.read(&mut buffer[filled..]).await?;
        if read == 0 {
            buffer.truncate(filled);
            return Ok((buffer, true));
        }
        filled += read;
    }
    *pending = Some(Zeroizing::new(buffer[limit]));
    buffer.truncate(limit);
    Ok((buffer, false))
}

async fn read_header<R>(reader: &mut R, header: &mut [u8]) -> Result<(), CryptoError>
where
    R: AsyncRead + Unpin,
{
    let mut filled = 0;
    while filled < header.len() {
        let read = reader.read(&mut header[filled..]).await?;
        if read == 0 {
            return Err(CryptoError::InvalidEnvelope);
        }
        filled += read;
    }
    Ok(())
}
