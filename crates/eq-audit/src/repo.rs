//! Audit persistence adapter (AUD-02, ERD-04).

use crate::record::AuditEvent;
use chrono::{DateTime, Utc};
use eq_core::actor::ActorRole;
use eq_core::db::Tx;
use eq_core::error::AppError;
use eq_core::ids::TenantId;
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

pub(super) async fn record(tx: &mut Tx, event: AuditEvent<'_>) -> Result<(), AppError> {
    let context = sqlx::query!(
        "SELECT NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid AS tenant_id, NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::uuid AS actor_id"
    )
    .fetch_one(tx.conn())
    .await?;

    if context.actor_id != event.actor.person_id.map(|person_id| person_id.0) {
        return Err(AppError::Internal(
            std::io::Error::other("audit actor does not match transaction context").into(),
        ));
    }
    let tenant_id = context.tenant_id.ok_or_else(|| {
        AppError::Internal(std::io::Error::other("audit tenant is missing from transaction context").into())
    })?;

    let actor_role = actor_role(event.actor.role)?;
    let id = Uuid::now_v7();
    let actor_id = event.actor.person_id.map(|person_id| person_id.0);
    let action = event.action;
    let entity_type = event.entity_type;
    let entity_id = event.entity_id;
    let category = event.category.as_str();
    let before = event.before.map(Json);
    let after = event.after.map(Json);
    let ip = event.actor.ip.map(|ip| ip.to_string());
    let request_id = event.actor.request_id;

    sqlx::query!(
        "INSERT INTO audit.entries (id, tenant_id, at, actor_id, actor_role, action, entity_type, entity_id, category, \"before\", \"after\", ip, request_id) VALUES ($1, $2, pg_catalog.statement_timestamp(), $3, $4, $5, $6, $7, $8, $9, $10, $11::text::inet, $12)",
        id,
        tenant_id,
        actor_id,
        actor_role,
        action,
        entity_type,
        entity_id,
        category,
        before as Option<Json<serde_json::Value>>,
        after as Option<Json<serde_json::Value>>,
        ip,
        request_id
    )
    .execute(tx.conn())
    .await?;

    Ok(())
}

pub(super) struct SealRow {
    pub(super) seq: i64,
    pub(super) id: Uuid,
    pub(super) tenant_id: Uuid,
    pub(super) at: DateTime<Utc>,
    pub(super) actor_id: Option<Uuid>,
    pub(super) actor_role: String,
    pub(super) action: String,
    pub(super) entity_type: String,
    pub(super) entity_id: Option<Uuid>,
    pub(super) category: String,
    pub(super) before_json: Option<Value>,
    pub(super) after_json: Option<Value>,
    pub(super) ip: Option<String>,
    pub(super) request_id: Option<Uuid>,
}

pub(super) struct VerifyRow {
    pub(super) seq: i64,
    pub(super) id: Uuid,
    pub(super) tenant_id: Uuid,
    pub(super) at: DateTime<Utc>,
    pub(super) actor_id: Option<Uuid>,
    pub(super) actor_role: String,
    pub(super) action: String,
    pub(super) entity_type: String,
    pub(super) entity_id: Option<Uuid>,
    pub(super) category: String,
    pub(super) before_json: Option<Value>,
    pub(super) after_json: Option<Value>,
    pub(super) ip: Option<String>,
    pub(super) request_id: Option<Uuid>,
    pub(super) prev_hash: Option<Vec<u8>>,
    pub(super) hash: Option<Vec<u8>>,
    pub(super) anonymized_at: Option<DateTime<Utc>>,
    pub(super) redaction_proof: Option<Vec<u8>>,
}

pub(super) async fn check_transaction_tenant(tx: &mut Tx, tenant: TenantId) -> Result<(), AppError> {
    let transaction_tenant = sqlx::query_scalar!(
        "SELECT NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid AS tenant_id"
    )
    .fetch_one(tx.conn())
    .await?;
    if transaction_tenant != Some(tenant.0) {
        return Err(AppError::Internal(
            std::io::Error::other("audit tenant does not match transaction context").into(),
        ));
    }
    Ok(())
}

pub(super) async fn lock_tenant(tx: &mut Tx, tenant: TenantId) -> Result<(), AppError> {
    let lock_key = format!("audit.seal:{}", tenant.0);
    sqlx::query!(
        "SELECT pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtext($1)::bigint)",
        lock_key
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}

pub(super) async fn unsealed_entries(tx: &mut Tx, tenant: TenantId) -> Result<Vec<SealRow>, AppError> {
    let rows = sqlx::query_as!(
        SealRow,
        "SELECT seq, id, tenant_id, at, actor_id, actor_role, action, entity_type, entity_id, category, \
                \"before\" AS before_json, \"after\" AS after_json, ip::text AS ip, request_id \
         FROM audit.entries WHERE tenant_id = $1 AND hash IS NULL ORDER BY seq",
        tenant.0
    )
    .fetch_all(tx.conn())
    .await?;
    Ok(rows)
}

pub(super) async fn has_sealed_after(tx: &mut Tx, tenant: TenantId, seq: i64) -> Result<bool, AppError> {
    let sealed_after = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM audit.entries WHERE tenant_id = $1 AND seq > $2 AND hash IS NOT NULL) \
         OR EXISTS(SELECT 1 FROM audit.checkpoints WHERE tenant_id = $1 AND seq > $2) AS sealed_after",
        tenant.0,
        seq
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(sealed_after)
}

pub(super) async fn persist_seals(
    tx: &mut Tx,
    tenant: TenantId,
    updates: Vec<Value>,
) -> Result<(), AppError> {
    let payload = Json(Value::Array(updates));
    sqlx::query!("SELECT audit.seal($1, $2)", tenant.0, payload)
        .execute(tx.conn())
        .await?;
    Ok(())
}

pub(super) async fn previous_hash_before(
    tx: &mut Tx,
    tenant: TenantId,
    before_seq: i64,
) -> Result<(Vec<u8>, bool), AppError> {
    let anchor = sqlx::query!(
        "SELECT chain.hash FROM (\
           SELECT seq, hash FROM audit.entries WHERE tenant_id = $1 AND seq < $2 \
           UNION ALL \
           SELECT seq, hash FROM audit.checkpoints WHERE tenant_id = $1 AND seq < $2\
         ) AS chain ORDER BY chain.seq DESC LIMIT 1",
        tenant.0,
        before_seq
    )
    .fetch_optional(tx.conn())
    .await?;
    let anchor_is_unsealed = anchor.as_ref().is_some_and(|row| row.hash.is_none());
    let previous_hash = anchor.and_then(|row| row.hash).unwrap_or_else(|| vec![0; 32]);
    Ok((previous_hash, anchor_is_unsealed))
}

pub(super) async fn verify_rows(
    tx: &mut Tx,
    tenant: TenantId,
    from_seq: i64,
    to_seq: i64,
) -> Result<Vec<VerifyRow>, AppError> {
    let rows = sqlx::query_as!(
        VerifyRow,
        "SELECT seq, id, tenant_id, at, actor_id, actor_role, action, entity_type, entity_id, category, \
                \"before\" AS before_json, \"after\" AS after_json, ip::text AS ip, request_id, \
                prev_hash, hash, anonymized_at, redaction_proof \
         FROM audit.entries WHERE tenant_id = $1 AND seq >= $2 AND seq <= $3 ORDER BY seq",
        tenant.0,
        from_seq,
        to_seq
    )
    .fetch_all(tx.conn())
    .await?;
    Ok(rows)
}

fn actor_role(role: ActorRole) -> Result<&'static str, AppError> {
    let role = match role {
        ActorRole::Anonymous => {
            return Err(AppError::Internal(
                std::io::Error::other("anonymous actor role is not mapped by the audit schema").into(),
            ));
        }
        ActorRole::Owner => "owner",
        ActorRole::Admin => "admin",
        ActorRole::Instructor => "instructor",
        ActorRole::Guardian => "guardian",
        ActorRole::Student => "student",
        ActorRole::System => "system",
        ActorRole::Platform => "platform",
    };
    Ok(role)
}

#[cfg(test)]
mod tests {
    use super::actor_role;
    use eq_core::actor::ActorRole;

    #[test]
    fn anonymous_actor_role_is_rejected() {
        let result = actor_role(ActorRole::Anonymous);
        assert!(result.is_err());
    }
}
