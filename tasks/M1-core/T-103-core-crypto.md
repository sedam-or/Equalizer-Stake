---
id: T-103
title: Add core crypto (HKDF sub-keys, tenant DEK envelope, field encryption with AAD, blind index, file streams)
milestone: M1
status: todo
depends_on: [T-101, T-102]
spec_refs: [SEC-01, SEC-02, SEC-03, SEC-04]
estimated_size: M
---

## Goal
Any module can encrypt and decrypt PII fields and files for a tenant, and compute blind indexes, with key versioning in place for rotation.

## Context
- SEC-01: sub-keys via HKDF-SHA256 from `EQ_SECRET_KEY` with `info` labels `"eq/jwt/v1"`, `"eq/kek/v1"`, `"eq/bidx/v1"`, `"eq/filesig/v1"`, `"eq/backup/v1"`, `"eq/csrf/v1"`, `"eq/ics/v1"`. `EQ_SECRET_KEY_PREVIOUS` is tried when unwrapping fails.
- SEC-02: one row per tenant per DEK version in `tenant_keys(tenant_id, key_version, dek_enc, created_at, retired_at)`.
  - The DEK is 32 random bytes, wrapped with AES-256-GCM under the KEK (nonce || ct).
  - Field ciphertext = `0x01 || key_version u32 BE || nonce(12) || ct+tag`.
  - AAD = `"{tenant_id}|{table}|{column}|{row_id}"`.
  - The current version is the max with `retired_at IS NULL`.
- SEC-03: file encryption uses the same DEK and STREAM construction (`aead::stream::EncryptorBE32` with XChaCha20Poly1305 or AES-GCM-SIV; choose `chacha20poly1305::XChaCha20Poly1305` STREAM BE32) with 1 MiB chunks. The header is `0x02 || key_version || nonce_prefix(19)`.
- SEC-04: `bidx(tenant, kind, normalized) = HMAC-SHA256(key = HKDF(bidx_base, info = tenant_id bytes), msg = kind || 0x00 || normalized)[0..16]`. `kind` is `"email"` or `"phone"`. Normalisation: email is trimmed, lowercased and NFC (crate `unicode-normalization`). Phone is parsed with `phonenumber` using the tenant country (ISO alpha-2) as default region and formatted as E.164. An invalid phone is a validation error.

## Scope
### In
- `eq_core::crypto::{Keys, Keyring, FieldCipher, Aad, bidx_email, bidx_phone, normalize_email, normalize_phone}`.
- `Keyring` is cached per tenant (DEKs unwrapped in memory as `Zeroizing<[u8;32]>`, `moka` cache with 10-min TTL). It creates the tenant's first DEK lazily, inside the provided tx.
- `encrypt_field(tx, tenant, aad, plaintext: &[u8]) -> Vec<u8>`, `decrypt_field(tenant, aad, ct) -> Zeroizing<Vec<u8>>`, plus string helpers.
- `encrypt_stream(reader, writer)` / `decrypt_stream` for files.

### Out
- Key rotation CLI (T-705). The re-encrypt job (T-705).

## Interfaces
```rust
pub struct Keys { pub jwt: Zeroizing<[u8;32]>, pub kek: Zeroizing<[u8;32]>, pub bidx_base: Zeroizing<[u8;32]>, pub filesig: Zeroizing<[u8;32]>, pub backup: Zeroizing<[u8;32]>, pub csrf: Zeroizing<[u8;32]>, pub ics: Zeroizing<[u8;32]>, pub kek_previous: Option<Zeroizing<[u8;32]>> }
impl Keys { pub fn derive(master: &[u8], previous: Option<&[u8]>) -> Keys; }
pub struct Aad<'a> { pub tenant: TenantId, pub table: &'a str, pub column: &'a str, pub row_id: Uuid }
#[async_trait] pub trait FieldCipher {
  async fn encrypt(&self, tx: &mut Tx, aad: &Aad<'_>, pt: &[u8]) -> Result<Vec<u8>, CryptoError>;
  async fn decrypt(&self, tx: &mut Tx, aad: &Aad<'_>, ct: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError>;
}
pub fn bidx_email(keys: &Keys, tenant: TenantId, email: &str) -> Result<[u8;16], CryptoError>;
pub fn bidx_phone(keys: &Keys, tenant: TenantId, phone: &str, default_region: &str) -> Result<([u8;16], String /*E.164*/), CryptoError>;
```

## Implementation notes
- `tenant_keys` has no RLS (T-101). Access it only through this module, with queries filtered by `tenant_id`.
- Decryption with a wrong AAD must fail (tag mismatch) → `CryptoError::Integrity`.
- Never log key material or plaintext. Implement `Debug` for key structs as `"[redacted]"`.

## Acceptance criteria
- AC1. Round trip: encrypt, then decrypt with the same AAD → equal. → `crypto::tests::roundtrip`
- AC2. Decrypt with a different `row_id` in the AAD → `Integrity` error. → `aad_binding`
- AC3. After inserting a new DEK version, old ciphertexts still decrypt and new ones use version 2. → `key_versioning`
- AC4. With master key B and previous = A, DEKs wrapped under A unwrap. → `previous_master_unwraps`
- AC5. `bidx_email(" Ana@Mail.COM ")` equals `bidx_email("ana@mail.com")`, and differs across tenants. → `bidx_normalized_and_tenant_scoped`
- AC6. File stream round trip of 3.5 MiB of random bytes. → `stream_roundtrip`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/crypto.rs` | the AC names above |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `feat(core): envelope encryption and blind indexes [T-103]`, signed off

## Report

## Questions
