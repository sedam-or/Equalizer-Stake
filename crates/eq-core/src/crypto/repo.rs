//! CA-30: SQL adapter for the core crypto module's `tenant_keys` table.

use crate::db::Tx;
use crate::ids::TenantId;

pub(super) async fn current_version(tx: &mut Tx, tenant: TenantId) -> Result<Option<i32>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT key_version FROM tenant_keys \
         WHERE tenant_id = $1 AND retired_at IS NULL \
         ORDER BY key_version DESC LIMIT 1 FOR UPDATE",
        tenant.0
    )
    .fetch_optional(tx.conn())
    .await?;
    Ok(row.map(|row| row.key_version))
}

pub(super) async fn next_version(tx: &mut Tx, tenant: TenantId) -> Result<i32, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT COALESCE(MAX(key_version), 0) + 1 AS \"next_version!\" \
         FROM tenant_keys WHERE tenant_id = $1",
        tenant.0
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(row.next_version)
}

pub(super) async fn insert_key(
    tx: &mut Tx,
    tenant: TenantId,
    version: i32,
    wrapped: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO tenant_keys (tenant_id, key_version, dek_enc) \
         VALUES ($1, $2, $3) ON CONFLICT (tenant_id, key_version) DO NOTHING",
        tenant.0,
        version,
        wrapped
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}

pub(super) async fn wrapped_key(
    tx: &mut Tx,
    tenant: TenantId,
    version: i32,
) -> Result<Option<(Vec<u8>, bool)>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT dek_enc, \
                xmin::text::bigint = (txid_current() % $3) AS \"uncommitted!\" \
         FROM tenant_keys WHERE tenant_id = $1 AND key_version = $2",
        tenant.0,
        version,
        super::POSTGRES_XID_MODULUS
    )
    .fetch_optional(tx.conn())
    .await?;
    Ok(row.map(|row| (row.dek_enc, row.uncommitted)))
}

pub(super) async fn transaction_tenant(tx: &mut Tx) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!("SELECT NULLIF(current_setting('app.tenant_id', true), '') AS current_tenant")
        .fetch_one(tx.conn())
        .await?;
    Ok(row.current_tenant)
}
