//! In-memory module side-effect sinks for tests (MOD-06).

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eq_core::actor::ActorCtx;
use eq_core::db::Tx;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::module::ModuleId;
use eq_core::ports::sinks::{AuditSink, EventSink, PolicyChange, PolicyInit};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleToggleRecord {
    pub tenant: TenantId,
    pub actor: Option<PersonId>,
    pub module: ModuleId,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyChangeRecord {
    pub tenant: TenantId,
    pub actor: Option<PersonId>,
    pub module: ModuleId,
    pub version: i32,
    pub previous: Option<Value>,
    pub document: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyChangedEventRecord {
    pub tenant: TenantId,
    pub actor: Option<PersonId>,
    pub module: ModuleId,
    pub version: i32,
    pub effective_from: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleSinkRecords {
    pub policies_initialized: Vec<(TenantId, ModuleId)>,
    pub audits: Vec<ModuleToggleRecord>,
    pub events: Vec<ModuleToggleRecord>,
    pub policy_audits: Vec<PolicyChangeRecord>,
    pub policy_events: Vec<PolicyChangedEventRecord>,
}

/// Test-only in-memory implementations sharing one inspectable record set.
#[derive(Clone, Default)]
pub struct ModuleSinkDoubles {
    records: Arc<Mutex<ModuleSinkRecords>>,
}

impl std::fmt::Debug for ModuleSinkDoubles {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModuleSinkDoubles")
            .field("records", &"[in-memory test records]")
            .finish()
    }
}

impl ModuleSinkDoubles {
    pub fn policy_init(&self) -> Arc<dyn PolicyInit> {
        Arc::new(self.clone())
    }

    pub fn audit_sink(&self) -> Arc<dyn AuditSink> {
        Arc::new(self.clone())
    }

    pub fn event_sink(&self) -> Arc<dyn EventSink> {
        Arc::new(self.clone())
    }

    pub fn records(&self) -> Result<ModuleSinkRecords, AppError> {
        self.records
            .lock()
            .map(|records| records.clone())
            .map_err(|_| AppError::Internal(anyhow::anyhow!("module sink test records are unavailable")))
    }
}

#[async_trait]
impl PolicyInit for ModuleSinkDoubles {
    async fn ensure_initial(&self, _tx: &mut Tx, tenant: TenantId, module: ModuleId) -> Result<(), AppError> {
        self.records
            .lock()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("module sink test records are unavailable")))?
            .policies_initialized
            .push((tenant, module));
        Ok(())
    }
}

#[async_trait]
impl AuditSink for ModuleSinkDoubles {
    async fn record_module_toggle(
        &self,
        _tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        enabled: bool,
    ) -> Result<(), AppError> {
        self.records
            .lock()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("module sink test records are unavailable")))?
            .audits
            .push(ModuleToggleRecord {
                tenant,
                actor: actor.person_id,
                module,
                enabled,
            });
        Ok(())
    }

    async fn record_policy_change(
        &self,
        _tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        change: PolicyChange,
    ) -> Result<(), AppError> {
        self.records
            .lock()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("module sink test records are unavailable")))?
            .policy_audits
            .push(PolicyChangeRecord {
                tenant,
                actor: actor.person_id,
                module: change.module,
                version: change.version,
                previous: change.previous,
                document: change.document,
            });
        Ok(())
    }
}

#[async_trait]
impl EventSink for ModuleSinkDoubles {
    async fn emit_module_toggled(
        &self,
        _tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        enabled: bool,
    ) -> Result<(), AppError> {
        self.records
            .lock()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("module sink test records are unavailable")))?
            .events
            .push(ModuleToggleRecord {
                tenant,
                actor: actor.person_id,
                module,
                enabled,
            });
        Ok(())
    }

    async fn emit_policy_changed(
        &self,
        _tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        version: i32,
        effective_from: DateTime<Utc>,
    ) -> Result<(), AppError> {
        self.records
            .lock()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("module sink test records are unavailable")))?
            .policy_events
            .push(PolicyChangedEventRecord {
                tenant,
                actor: actor.person_id,
                module,
                version,
                effective_from,
            });
        Ok(())
    }
}
