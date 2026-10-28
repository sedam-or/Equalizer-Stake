//! Tenant-scoped attention-item operations (STM-14, OPS-08).

mod repo;

use crate::db::Tx;
use crate::error::AppError;
use crate::model::Severity;
use crate::module::ModuleId;
use serde_json::Value;

/// STM-14: raises or refreshes one tenant-scoped attention item in the caller's transaction.
///
/// A repeated dedupe key updates the open item and reopens a resolved one.
///
/// # Errors
/// Returns the persistence error if the item cannot be stored in the caller transaction.
pub async fn raise(
    tx: &mut Tx,
    module_id: ModuleId,
    kind: &str,
    payload: Value,
    severity: Severity,
    dedupe_key: &str,
) -> Result<(), AppError> {
    repo::raise(tx, module_id, kind, payload, severity, dedupe_key).await
}
