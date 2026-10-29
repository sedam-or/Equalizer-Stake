use eq_core::crypto::{Aad, CryptoError, Keyring, Keys};
use eq_core::db::begin_system;
use eq_core::ids::TenantId;
use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
#[expect(
    clippy::panic,
    reason = "the shared database fixture reports only redacted setup failures"
)]
mod shared_test_db;

use shared_test_db::TestDb;

const MASTER_A: &[u8] = b"test-secret-master-a-32-bytes-long";
const MASTER_B: &[u8] = b"test-secret-master-b-32-bytes-long";

async fn fixture() -> (TestDb, TenantId, Keyring) {
    let db = TestDb::new().await;
    let _credential_fields = (&db.url, &db.app_url);
    let tenant = TenantId(Uuid::now_v7());
    insert_tenant(&db.pool, tenant).await;
    (db, tenant, Keyring::new(Keys::derive(MASTER_A, None)))
}

#[expect(
    clippy::expect_used,
    reason = "test fixture setup must fail immediately with synthetic context"
)]
async fn insert_tenant(pool: &PgPool, tenant: TenantId) {
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Crypto Test School', 'ID', 'test', 1)",
    )
    .bind(tenant.0)
    .bind(format!("crypto-{}", tenant.0.simple()))
    .execute(pool)
    .await
    .expect("insert synthetic tenant");
}

fn aad(tenant: TenantId, row_id: Uuid) -> Aad<'static> {
    Aad {
        tenant,
        table: "persons",
        column: "legal_name_enc",
        row_id,
    }
}

#[expect(
    clippy::expect_used,
    reason = "test fixture crypto must fail immediately with synthetic context"
)]
fn wrapped_test_dek(keys: &Keys, dek: &[u8; 32]) -> Vec<u8> {
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

    let cipher = Aes256Gcm::new_from_slice(keys.kek.expose_secret().as_ref())
        .expect("derived AES-256 key has the correct length");
    let mut nonce = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let nonce = Nonce::from(nonce);
    let ciphertext = cipher.encrypt(&nonce, dek.as_slice()).expect("test DEK encrypts");
    [&nonce[..], ciphertext.as_slice()].concat()
}

mod crypto {
    use super::*;

    mod tests {
        use super::*;
        use std::io::Cursor;

        #[tokio::test]
        async fn roundtrip() {
            let (db, tenant, keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let row = Uuid::now_v7();
            let aad = aad(tenant, row);
            let plaintext = b"synthetic encrypted field";

            let ciphertext = keyring
                .encrypt_field(&mut tx, &aad, plaintext)
                .await
                .expect("encrypt field");
            let decrypted = keyring
                .decrypt_field(&mut tx, &aad, &ciphertext)
                .await
                .expect("decrypt field");

            assert_eq!(decrypted.as_slice(), plaintext);
            tx.commit().await.expect("commit tenant tx");
        }

        #[tokio::test]
        async fn aad_binding() {
            let (db, tenant, keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let row = Uuid::now_v7();
            let ciphertext = keyring
                .encrypt_field(&mut tx, &aad(tenant, row), b"synthetic field")
                .await
                .expect("encrypt field");

            let result = keyring
                .decrypt_field(&mut tx, &aad(tenant, Uuid::now_v7()), &ciphertext)
                .await;
            assert!(matches!(result, Err(CryptoError::Integrity)));
        }

        #[tokio::test]
        async fn aad_binding_covers_table_and_column() {
            let (db, tenant, keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let row = Uuid::now_v7();
            let ciphertext = keyring
                .encrypt_field(&mut tx, &aad(tenant, row), b"synthetic field")
                .await
                .expect("encrypt field");

            let other_column = Aad {
                tenant,
                table: "persons",
                column: "email_enc",
                row_id: row,
            };
            assert!(matches!(
                keyring.decrypt_field(&mut tx, &other_column, &ciphertext).await,
                Err(CryptoError::Integrity)
            ));

            let other_table = Aad {
                tenant,
                table: "memberships",
                column: "legal_name_enc",
                row_id: row,
            };
            assert!(matches!(
                keyring.decrypt_field(&mut tx, &other_table, &ciphertext).await,
                Err(CryptoError::Integrity)
            ));
        }

        #[tokio::test]
        async fn aad_binding_covers_tenant() {
            let (db, tenant, keyring) = fixture().await;
            let other_tenant = TenantId(Uuid::now_v7());
            insert_tenant(&db.pool, other_tenant).await;
            let row = Uuid::now_v7();
            let original_aad = aad(tenant, row);
            let mut tx = begin_system(&db.app_pool, tenant)
                .await
                .expect("begin source tenant tx");
            let ciphertext = keyring
                .encrypt_field(&mut tx, &original_aad, b"synthetic cross-tenant field")
                .await
                .expect("encrypt field");
            tx.commit().await.expect("commit source tenant tx");

            let (wrapped_dek,) = sqlx::query_as::<_, (Vec<u8>,)>(
                "SELECT dek_enc FROM tenant_keys WHERE tenant_id = $1 AND key_version = 1",
            )
            .bind(tenant.0)
            .fetch_one(&db.pool)
            .await
            .expect("read test DEK envelope");
            sqlx::query("INSERT INTO tenant_keys (tenant_id, key_version, dek_enc) VALUES ($1, 1, $2)")
                .bind(other_tenant.0)
                .bind(wrapped_dek)
                .execute(&db.pool)
                .await
                .expect("share test DEK envelope");

            let mut tx = begin_system(&db.app_pool, other_tenant)
                .await
                .expect("begin target tenant tx");
            let other_aad = aad(other_tenant, row);
            assert!(matches!(
                keyring.decrypt_field(&mut tx, &other_aad, &ciphertext).await,
                Err(CryptoError::Integrity)
            ));
        }

        #[tokio::test]
        async fn mismatched_transaction_tenant_is_rejected() {
            let (db, tenant, keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let mismatched_aad = aad(TenantId(Uuid::now_v7()), Uuid::now_v7());
            assert!(matches!(
                keyring
                    .encrypt_field(&mut tx, &mismatched_aad, b"synthetic field")
                    .await,
                Err(CryptoError::TenantMismatch)
            ));
        }

        #[tokio::test]
        async fn key_versioning() {
            let (db, tenant, keyring) = fixture().await;
            let row = Uuid::now_v7();
            let aad = aad(tenant, row);
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let old_ciphertext = keyring
                .encrypt_field(&mut tx, &aad, b"old DEK value")
                .await
                .expect("encrypt with first DEK");
            tx.commit().await.expect("commit first version");

            let keys = Keys::derive(MASTER_A, None);
            let mut new_dek = [0_u8; 32];
            OsRng.fill_bytes(&mut new_dek);
            let wrapped = wrapped_test_dek(&keys, &new_dek);
            sqlx::query("UPDATE tenant_keys SET retired_at = now() WHERE tenant_id = $1 AND key_version = 1")
                .bind(tenant.0)
                .execute(&db.pool)
                .await
                .expect("retire old test DEK");
            sqlx::query("INSERT INTO tenant_keys (tenant_id, key_version, dek_enc) VALUES ($1, 2, $2)")
                .bind(tenant.0)
                .bind(wrapped)
                .execute(&db.pool)
                .await
                .expect("insert second test DEK");

            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let new_ciphertext = keyring
                .encrypt_field(&mut tx, &aad, b"new DEK value")
                .await
                .expect("encrypt with current DEK");
            assert_eq!(u32::from_be_bytes(old_ciphertext[1..5].try_into().unwrap()), 1);
            assert_eq!(u32::from_be_bytes(new_ciphertext[1..5].try_into().unwrap()), 2);
            assert_eq!(
                keyring
                    .decrypt_field(&mut tx, &aad, &old_ciphertext)
                    .await
                    .expect("old DEK remains readable")
                    .as_slice(),
                b"old DEK value"
            );
            assert_eq!(
                keyring
                    .decrypt_field(&mut tx, &aad, &new_ciphertext)
                    .await
                    .expect("new DEK is readable")
                    .as_slice(),
                b"new DEK value"
            );
            tx.commit().await.expect("commit tenant tx");
        }

        #[tokio::test]
        async fn previous_master_unwraps() {
            let (db, tenant, old_keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let aad = aad(tenant, Uuid::now_v7());
            let ciphertext = old_keyring
                .encrypt_field(&mut tx, &aad, b"wrapped under previous master")
                .await
                .expect("encrypt with master A");
            tx.commit().await.expect("commit tenant tx");

            let current_keyring = Keyring::new(Keys::derive(MASTER_B, Some(MASTER_A)));
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let decrypted = current_keyring
                .decrypt_field(&mut tx, &aad, &ciphertext)
                .await
                .expect("previous master unwraps the tenant DEK");
            assert_eq!(decrypted.as_slice(), b"wrapped under previous master");
        }

        #[tokio::test]
        async fn rollback_does_not_cache_uncommitted_dek() {
            let (db, tenant, keyring) = fixture().await;
            let field_aad = aad(tenant, Uuid::now_v7());
            {
                let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
                let ciphertext = keyring
                    .encrypt_field(&mut tx, &field_aad, b"rolled back DEK")
                    .await
                    .expect("encrypt field");
                assert_eq!(
                    keyring
                        .decrypt_field(&mut tx, &field_aad, &ciphertext)
                        .await
                        .expect("decrypt field")
                        .as_slice(),
                    b"rolled back DEK"
                );
            }

            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let ciphertext = keyring
                .encrypt_field(&mut tx, &field_aad, b"committed DEK")
                .await
                .expect("encrypt with committed DEK");
            tx.commit().await.expect("commit tenant tx");

            let fresh_keyring = Keyring::new(Keys::derive(MASTER_A, None));
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            assert_eq!(
                fresh_keyring
                    .decrypt_field(&mut tx, &field_aad, &ciphertext)
                    .await
                    .expect("fresh keyring unwraps committed DEK")
                    .as_slice(),
                b"committed DEK"
            );
        }

        #[test]
        fn bidx_normalized_and_tenant_scoped() {
            let keys = Keys::derive(MASTER_A, None);
            let tenant_a = TenantId(Uuid::now_v7());
            let tenant_b = TenantId(Uuid::now_v7());

            let formatted =
                eq_core::crypto::bidx_email(&keys, tenant_a, " Ana@Mail.COM ").expect("index email");
            let normalized =
                eq_core::crypto::bidx_email(&keys, tenant_a, "ana@mail.com").expect("index normalized email");
            let other_tenant = eq_core::crypto::bidx_email(&keys, tenant_b, "ana@mail.com")
                .expect("index email for other tenant");
            assert_eq!(formatted, normalized);
            assert_ne!(formatted, other_tenant);

            let (phone_index, e164) = eq_core::crypto::bidx_phone(&keys, tenant_a, "202-555-0123", "US")
                .expect("normalize and index phone");
            assert_eq!(e164, "+12025550123");
            assert!(eq_core::crypto::bidx_phone(&keys, tenant_a, "not a phone", "US").is_err());
            assert_ne!(phone_index, [0_u8; 16]);
        }

        #[tokio::test]
        async fn stream_roundtrip() {
            let (db, tenant, keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let mut plaintext = vec![0_u8; 3 * 1024 * 1024 + 512 * 1024];
            OsRng.fill_bytes(&mut plaintext);
            let mut encrypted = Vec::new();
            keyring
                .encrypt_stream(&mut tx, tenant, Cursor::new(&plaintext), &mut encrypted)
                .await
                .expect("encrypt stream");
            let mut decrypted = Vec::new();
            keyring
                .decrypt_stream(&mut tx, tenant, Cursor::new(&encrypted), &mut decrypted)
                .await
                .expect("decrypt stream");
            assert_eq!(decrypted, plaintext);

            let mut empty_encrypted = Vec::new();
            keyring
                .encrypt_stream(&mut tx, tenant, Cursor::new(Vec::new()), &mut empty_encrypted)
                .await
                .expect("encrypt empty stream");
            let mut empty_decrypted = Vec::new();
            keyring
                .decrypt_stream(
                    &mut tx,
                    tenant,
                    Cursor::new(&empty_encrypted),
                    &mut empty_decrypted,
                )
                .await
                .expect("decrypt empty stream");
            assert!(empty_decrypted.is_empty());

            let mut tampered = encrypted;
            let last = tampered.len() - 1;
            tampered[last] ^= 1;
            assert!(matches!(
                keyring
                    .decrypt_stream(&mut tx, tenant, Cursor::new(&tampered), &mut Vec::new())
                    .await,
                Err(CryptoError::Integrity)
            ));

            let mut invalid_header = empty_encrypted.clone();
            invalid_header[0] = 0x03;
            assert!(matches!(
                keyring
                    .decrypt_stream(&mut tx, tenant, Cursor::new(&invalid_header), &mut Vec::new())
                    .await,
                Err(CryptoError::InvalidEnvelope)
            ));

            let truncated_header = &empty_encrypted[..8];
            assert!(matches!(
                keyring
                    .decrypt_stream(&mut tx, tenant, Cursor::new(truncated_header), &mut Vec::new())
                    .await,
                Err(CryptoError::InvalidEnvelope)
            ));
            tx.commit().await.expect("commit tenant tx");
        }

        #[tokio::test]
        async fn tampered_field_ciphertext_is_integrity_error() {
            let (db, tenant, keyring) = fixture().await;
            let mut tx = begin_system(&db.app_pool, tenant).await.expect("begin tenant tx");
            let aad = aad(tenant, Uuid::now_v7());
            assert!(matches!(
                keyring.decrypt_field(&mut tx, &aad, &[0x01]).await,
                Err(CryptoError::InvalidEnvelope)
            ));
            let mut ciphertext = keyring
                .encrypt_field(&mut tx, &aad, b"synthetic tamper test")
                .await
                .expect("encrypt field");
            let last = ciphertext.len() - 1;
            ciphertext[last] ^= 1;
            assert!(matches!(
                keyring.decrypt_field(&mut tx, &aad, &ciphertext).await,
                Err(CryptoError::Integrity)
            ));
        }

        #[test]
        fn key_debug_is_redacted() {
            let keys = Keys::derive(MASTER_A, None);
            let debug = format!("{keys:?}");
            assert!(!debug.contains("test-secret-master-a"));
            assert!(debug.contains("redacted"));

            let keyring = Keyring::new(Keys::derive(MASTER_A, None));
            let debug = format!("{keyring:?}");
            assert!(!debug.contains("test-secret-master-a"));
            assert!(debug.contains("redacted"));
        }
    }
}
