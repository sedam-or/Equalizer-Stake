//! SQL for the core-owned policies table and preset resolver (CA-30, POL-03..08).

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Uuid;

use crate::db::Tx;
use crate::error::AppError;
use crate::ids::TenantId;
use crate::module::ModuleId;

#[derive(Clone, Debug)]
pub(super) struct PolicyRow {
    pub version: i32,
    pub schema_version: i32,
    pub effective_from: DateTime<Utc>,
    pub document: Value,
}

pub(super) struct NewPolicyRow {
    pub tenant: TenantId,
    pub module: ModuleId,
    pub version: i32,
    pub schema_version: i32,
    pub effective_from: DateTime<Utc>,
    pub document: Value,
    pub note: Option<String>,
    pub created_by: Option<Uuid>,
}

pub(super) struct TenantPreset {
    pub preset_id: String,
    pub preset_version: i32,
}

pub(super) async fn history(
    tx: &mut Tx,
    tenant: TenantId,
    module: ModuleId,
) -> Result<Vec<PolicyRow>, AppError> {
    Ok(sqlx::query!(
        "SELECT version, schema_version, effective_from, document FROM public.policies WHERE tenant_id = $1 AND module_id = $2 ORDER BY effective_from ASC, version ASC",
        tenant.0,
        module.as_str()
    )
    .fetch_all(tx.conn())
    .await?
    .into_iter()
    .map(|row| PolicyRow {
        version: row.version,
        schema_version: row.schema_version,
        effective_from: row.effective_from,
        document: row.document,
    })
    .collect())
}

pub(super) async fn tenant_context_matches(tx: &mut Tx, tenant: TenantId) -> Result<bool, AppError> {
    let tenant_id = tenant.0.to_string();
    let row = sqlx::query!(
        "SELECT COALESCE(pg_catalog.current_setting('app.tenant_id', true), '') = $1::text AS \"matches!\"",
        tenant_id
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(row.matches)
}

pub(super) async fn transaction_has_writes(tx: &mut Tx) -> Result<bool, AppError> {
    let row =
        sqlx::query!("SELECT pg_catalog.pg_current_xact_id_if_assigned() IS NOT NULL AS \"has_writes!\"")
            .fetch_one(tx.conn())
            .await?;
    Ok(row.has_writes)
}

pub(super) async fn next_version(tx: &mut Tx, tenant: TenantId, module: ModuleId) -> Result<i32, AppError> {
    let lock_key = format!("{}:{}", tenant.0, module.as_str());
    sqlx::query!(
        "SELECT pg_advisory_xact_lock(pg_catalog.hashtext($1)::bigint)",
        lock_key
    )
    .execute(tx.conn())
    .await?;
    let row = sqlx::query!(
        "SELECT COALESCE(MAX(version), 0) AS \"max_version!\" FROM public.policies WHERE tenant_id = $1 AND module_id = $2",
        tenant.0,
        module.as_str()
    )
    .fetch_one(tx.conn())
    .await?;
    row.max_version
        .checked_add(1)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("policy version overflow")))
}

pub(super) async fn previous_document(
    tx: &mut Tx,
    tenant: TenantId,
    module: ModuleId,
) -> Result<Option<Value>, AppError> {
    Ok(sqlx::query!(
        "SELECT document FROM public.policies WHERE tenant_id = $1 AND module_id = $2 ORDER BY version DESC LIMIT 1",
        tenant.0,
        module.as_str()
    )
    .fetch_optional(tx.conn())
    .await?
    .map(|row| row.document))
}

pub(super) async fn insert(tx: &mut Tx, row: NewPolicyRow) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO public.policies (tenant_id, module_id, version, schema_version, effective_from, document, note, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        row.tenant.0,
        row.module.as_str(),
        row.version,
        row.schema_version,
        row.effective_from,
        row.document,
        row.note,
        row.created_by
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}

pub(super) async fn exists(tx: &mut Tx, tenant: TenantId, module: ModuleId) -> Result<bool, AppError> {
    let row = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM public.policies WHERE tenant_id = $1 AND module_id = $2) AS \"exists!\"",
        tenant.0,
        module.as_str()
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(row.exists)
}

pub(super) async fn tenant_preset(tx: &mut Tx, tenant: TenantId) -> Result<TenantPreset, AppError> {
    let row = sqlx::query!(
        "SELECT preset_id, preset_version FROM public.resolve_tenant_preset($1)",
        tenant.0
    )
    .fetch_optional(tx.conn())
    .await?;
    let Some(row) = row else {
        return Err(AppError::Internal(anyhow::anyhow!(
            "tenant preset identity is unavailable"
        )));
    };
    Ok(TenantPreset {
        preset_id: row
            .preset_id
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("tenant preset id is unavailable")))?,
        preset_version: row
            .preset_version
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("tenant preset version is unavailable")))?,
    })
}

pub(super) async fn notify_changed(tx: &mut Tx, tenant: TenantId, module: ModuleId) -> Result<(), AppError> {
    let payload = format!("{}:{}", tenant.0, module.as_str());
    sqlx::query!("SELECT pg_catalog.pg_notify('eq_policy_changed', $1)", payload)
        .execute(tx.conn())
        .await?;
    Ok(())
}
