//! Transactional sinks for module side effects (MOD-06, ARC-05/07).

use crate::actor::ActorCtx;
use crate::db::Tx;
use crate::ids::TenantId;
use crate::module::ModuleId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// POL-03: policy documents passed together to the transactional audit sink.
pub struct PolicyChange {
    pub module: ModuleId,
    pub version: i32,
    pub previous: Option<Value>,
    pub document: Value,
}

impl std::fmt::Debug for PolicyChange {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PolicyChange")
            .field("module", &self.module)
            .field("version", &self.version)
            .field("previous", &self.previous.as_ref().map(|_| "[REDACTED]"))
            .field("document", &"[REDACTED]")
            .finish()
    }
}

/// MOD-06: creates a module's initial policy only when its first enabled.
#[async_trait]
pub trait PolicyInit: Send + Sync {
    async fn ensure_initial(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        module: ModuleId,
    ) -> Result<(), crate::error::AppError>;
}

/// MOD-06: records one module state change in the caller's transaction.
#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn record_module_toggle(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        enabled: bool,
    ) -> Result<(), crate::error::AppError>;

    /// POL-03: records the named policy-change audit action in the caller's transaction.
    async fn record_policy_change(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        change: PolicyChange,
    ) -> Result<(), crate::error::AppError>;
}

/// MOD-06: emits one versioned module-toggle event in the caller's transaction.
#[async_trait]
pub trait EventSink: Send + Sync {
    async fn emit_module_toggled(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        enabled: bool,
    ) -> Result<(), crate::error::AppError>;

    /// POL-03: emits `core.policy_changed.v1` in the caller's transaction.
    async fn emit_policy_changed(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        version: i32,
        effective_from: DateTime<Utc>,
    ) -> Result<(), crate::error::AppError>;
}
