//! SQL adapter for core-owned attention items (ERD-04, STM-14).

use crate::db::Tx;
use crate::error::AppError;
use crate::model::Severity;
use crate::module::ModuleId;
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

pub(super) async fn raise(
    tx: &mut Tx,
    module_id: ModuleId,
    kind: &str,
    payload: Value,
    severity: Severity,
    dedupe_key: &str,
) -> Result<(), AppError> {
    let tenant_id = tx.tenant().0;
    let severity = match severity {
        Severity::Low => "low",
        Severity::Normal => "normal",
        Severity::High => "high",
    };
    let payload = Json(payload);
    sqlx::query!(
        "INSERT INTO public.attention_items \
             (id, tenant_id, module_id, kind, payload, severity, dedupe_key) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (tenant_id, dedupe_key) DO UPDATE SET \
             payload = EXCLUDED.payload, severity = EXCLUDED.severity, \
             resolved_at = NULL, resolved_by = NULL",
        Uuid::now_v7(),
        tenant_id,
        module_id.as_str(),
        kind,
        payload as Json<Value>,
        severity,
        dedupe_key
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}
