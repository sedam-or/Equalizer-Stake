//! Persistence adapter for the core-owned `tenant_modules` table (CA-30, MOD-06).

use std::collections::BTreeSet;

use crate::actor::ActorCtx;
use crate::db::Tx;
use crate::error::AppError;
use crate::ids::TenantId;

use super::ModuleId;

/// MOD-05/06: loads this tenant's enabled optional modules under its RLS-scoped transaction.
pub(super) async fn enabled(tx: &mut Tx, tenant: TenantId) -> Result<BTreeSet<ModuleId>, AppError> {
    let rows = sqlx::query!(
        "SELECT module_id FROM tenant_modules WHERE tenant_id = $1 AND enabled = TRUE",
        tenant.0
    )
    .fetch_all(tx.conn())
    .await?;

    rows.into_iter()
        .map(|row| {
            ModuleId::parse(&row.module_id).ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("tenant_modules contains an unknown module id"))
            })
        })
        .collect()
}

/// MOD-06: persists an optional module state without deleting its row or module-owned data.
pub(super) async fn set_enabled(
    tx: &mut Tx,
    tenant: TenantId,
    actor: &ActorCtx,
    module: ModuleId,
    enabled: bool,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO tenant_modules (tenant_id, module_id, enabled, changed_by) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (tenant_id, module_id) DO UPDATE \
         SET enabled = EXCLUDED.enabled, changed_at = now(), changed_by = EXCLUDED.changed_by",
        tenant.0,
        module.as_str(),
        enabled,
        actor.person_id.map(|person| person.0)
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}

/// MOD-05/06: schedules tenant-specific cache invalidation after this transaction commits.
pub(super) async fn notify_changed(tx: &mut Tx, tenant: TenantId) -> Result<(), AppError> {
    sqlx::query!(
        "SELECT pg_notify('eq_modules_changed', $1) AS notified",
        tenant.0.to_string()
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(())
}
