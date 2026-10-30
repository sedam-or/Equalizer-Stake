//! Core-owned tenant metadata reads (CA-30, MOD-09).

use crate::db::Tx;
use crate::error::AppError;
use crate::ids::TenantId;

/// MOD-09: returns the public tenant display name through the caller's RLS-scoped transaction.
pub(super) async fn public_name(tx: &mut Tx, tenant: TenantId) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT public.resolve_tenant_name($1)", tenant.0)
        .fetch_one(tx.conn())
        .await?
        .ok_or(AppError::NotFound { entity: "tenant" })
}
