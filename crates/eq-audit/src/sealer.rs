//! Tenant-serialized audit-chain sealing and verification (AUD-04/06).

use crate::repo::{self, VerifyRow};
use chrono::SecondsFormat;
use eq_core::db::{self, Tx};
use eq_core::error::AppError;
use eq_core::ids::TenantId;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::fmt;
use uuid::Uuid;

/// AUD-04: result of checking a tenant's audit chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyReport {
    pub checked: usize,
    pub anonymized: usize,
    pub first_mismatch: Option<i64>,
}

/// AUD-04: serializes sealing and verification work for one tenant.
pub struct Sealer {
    pool: PgPool,
}

impl fmt::Debug for Sealer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Sealer")
            .field("pool", &"[configured application pool]")
            .finish()
    }
}

impl Sealer {
    /// Creates a sealer backed by the application pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// AUD-04: seals visible unsealed rows in sequence order for one tenant.
    pub async fn seal_tenant(&self, tenant: TenantId) -> Result<(), AppError> {
        let mut tx = db::begin_system(&self.pool, tenant).await?;
        repo::check_transaction_tenant(&mut tx, tenant).await?;
        repo::lock_tenant(&mut tx, tenant).await?;

        let rows = repo::unsealed_entries(&mut tx, tenant).await?;
        let Some(first) = rows.first() else {
            tx.commit().await?;
            return Ok(());
        };
        let (mut previous_hash, has_unsealed_predecessor) =
            repo::previous_hash_before(&mut tx, tenant, first.seq).await?;
        if has_unsealed_predecessor || repo::has_sealed_after(&mut tx, tenant, first.seq).await? {
            return Err(AppError::Internal(anyhow::anyhow!(
                "audit chain cannot be sealed in sequence order"
            )));
        }

        let mut updates = Vec::with_capacity(rows.len());
        for row in &rows {
            let document = HashDocument {
                id: row.id,
                tenant_id: row.tenant_id,
                at: row.at.to_rfc3339_opts(SecondsFormat::Micros, true),
                actor_id: row.actor_id,
                actor_role: &row.actor_role,
                action: &row.action,
                entity_type: &row.entity_type,
                entity_id: row.entity_id,
                category: &row.category,
                before: row.before_json.as_ref(),
                after: row.after_json.as_ref(),
                ip: row.ip.as_deref(),
                request_id: row.request_id,
            };
            let hash = entry_hash(&previous_hash, &document)?;
            updates.push(serde_json::json!({
                "seq": row.seq,
                "prev_hash": hex(&previous_hash),
                "hash": hex(&hash),
            }));
            previous_hash = hash;
        }

        repo::persist_seals(&mut tx, tenant, updates).await?;
        tx.commit().await?;
        Ok(())
    }

    /// AUD-04: verifies an inclusive sequence range against its preceding row or checkpoint.
    pub async fn verify(
        &self,
        tenant: TenantId,
        from_seq: i64,
        to_seq: i64,
    ) -> Result<VerifyReport, AppError> {
        let mut tx = db::begin_system(&self.pool, tenant).await?;
        repo::check_transaction_tenant(&mut tx, tenant).await?;
        repo::lock_tenant(&mut tx, tenant).await?;
        let report = verify_in_tx(&mut tx, tenant, from_seq, to_seq).await?;
        tx.commit().await?;
        Ok(report)
    }
}

#[derive(Serialize)]
struct HashDocument<'a> {
    id: Uuid,
    tenant_id: Uuid,
    at: String,
    actor_id: Option<Uuid>,
    actor_role: &'a str,
    action: &'a str,
    entity_type: &'a str,
    entity_id: Option<Uuid>,
    category: &'a str,
    before: Option<&'a Value>,
    after: Option<&'a Value>,
    ip: Option<&'a str>,
    request_id: Option<Uuid>,
}

async fn verify_in_tx(
    tx: &mut Tx,
    tenant: TenantId,
    from_seq: i64,
    to_seq: i64,
) -> Result<VerifyReport, AppError> {
    let rows = repo::verify_rows(tx, tenant, from_seq, to_seq).await?;

    let mut report = VerifyReport {
        checked: 0,
        anonymized: 0,
        first_mismatch: None,
    };
    let Some(first) = rows.first() else {
        return Ok(report);
    };
    let (mut previous_hash, anchor_is_unsealed) = repo::previous_hash_before(tx, tenant, first.seq).await?;
    if anchor_is_unsealed {
        report.checked = 1;
        if first.anonymized_at.is_some() {
            report.anonymized = 1;
        }
        report.first_mismatch = Some(first.seq);
        return Ok(report);
    }

    for row in rows {
        report.checked += 1;
        if row.anonymized_at.is_some() {
            report.anonymized += 1;
        }
        if !verify_row(&row, &previous_hash)? {
            report.first_mismatch = Some(row.seq);
            break;
        }
        if let Some(hash) = row.hash {
            previous_hash = hash;
        }
    }
    Ok(report)
}

fn verify_row(row: &VerifyRow, previous_hash: &[u8]) -> Result<bool, AppError> {
    let Some(stored_hash) = row.hash.as_deref() else {
        return Ok(false);
    };
    if previous_hash.len() != 32 || stored_hash.len() != 32 {
        return Ok(false);
    }
    if row.prev_hash.as_deref() != Some(previous_hash) {
        return Ok(false);
    }
    if row.anonymized_at.is_some() {
        return Ok(row.redaction_proof.is_some());
    }

    let document = HashDocument {
        id: row.id,
        tenant_id: row.tenant_id,
        at: row.at.to_rfc3339_opts(SecondsFormat::Micros, true),
        actor_id: row.actor_id,
        actor_role: &row.actor_role,
        action: &row.action,
        entity_type: &row.entity_type,
        entity_id: row.entity_id,
        category: &row.category,
        before: row.before_json.as_ref(),
        after: row.after_json.as_ref(),
        ip: row.ip.as_deref(),
        request_id: row.request_id,
    };
    let calculated_hash = entry_hash(previous_hash, &document)?;
    Ok(calculated_hash == stored_hash)
}

fn entry_hash(previous_hash: &[u8], document: &HashDocument<'_>) -> Result<Vec<u8>, AppError> {
    let canonical = serde_jcs::to_vec(document)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("audit entry canonicalization failed")))?;
    let mut hasher = Sha256::new();
    hasher.update(previous_hash);
    hasher.update(canonical);
    Ok(hasher.finalize().to_vec())
}

fn hex(bytes: &[u8]) -> String {
    // technical: lowercase hexadecimal is the input format for PostgreSQL decode().
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
