//! Transactional sinks for module side effects (MOD-06, ARC-05/07).

use crate::actor::ActorCtx;
use crate::db::Tx;
use crate::ids::TenantId;
use crate::module::ModuleId;
use async_trait::async_trait;

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
}
