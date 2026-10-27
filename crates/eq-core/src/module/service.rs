//! Transactional module enablement orchestration (MOD-06, ARC-04/05).

use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::{Map, Value};

use crate::actor::ActorCtx;
use crate::db::Tx;
use crate::error::{AppError, codes};
use crate::ids::TenantId;
use crate::ports::sinks::{AuditSink, EventSink, PolicyInit};

use super::ModuleId;
use super::domain::{ToggleError, TogglePlan, plan_toggle};
use super::registry::ModuleRegistry;
use super::repo;

/// MOD-06: applies module callbacks, persistence, audit and event effects in the caller's transaction.
#[derive(Clone)]
pub struct ModuleService {
    registry: Arc<ModuleRegistry>,
    policy_init: Arc<dyn PolicyInit>,
    audit: Arc<dyn AuditSink>,
    events: Arc<dyn EventSink>,
}

impl std::fmt::Debug for ModuleService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModuleService")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl ModuleService {
    /// MOD-06: creates a service backed by the compiled registry and transactional side-effect ports.
    pub fn new(
        registry: Arc<ModuleRegistry>,
        policy_init: Arc<dyn PolicyInit>,
        audit: Arc<dyn AuditSink>,
        events: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            registry,
            policy_init,
            audit,
            events,
        }
    }

    /// MOD-06: toggles the full requested optional-module set without committing the caller's transaction.
    pub async fn set_enabled(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        requested: &BTreeSet<ModuleId>,
    ) -> Result<TogglePlan, AppError> {
        let current = repo::enabled(tx, tenant).await?;
        let manifests = self.registry.manifests();
        let plan = plan_toggle(&current, requested, &manifests).map_err(map_toggle_error)?;

        for change in &plan.changes {
            let Some(module) = self.registry.get(change.module) else {
                return Err(AppError::Internal(anyhow::anyhow!(
                    "validated module is missing from the compiled registry"
                )));
            };

            if change.enabled {
                self.policy_init.ensure_initial(tx, tenant, change.module).await?;
                module.on_enable(tx, tenant).await.map_err(AppError::Internal)?;
            } else {
                module.on_disable(tx, tenant).await.map_err(AppError::Internal)?;
            }

            repo::set_enabled(tx, tenant, actor, change.module, change.enabled).await?;
            self.audit
                .record_module_toggle(tx, tenant, actor, change.module, change.enabled)
                .await?;
            self.events
                .emit_module_toggled(tx, tenant, actor, change.module, change.enabled)
                .await?;
        }

        if !plan.changes.is_empty() {
            repo::notify_changed(tx, tenant).await?;
        }

        Ok(plan)
    }
}

fn map_toggle_error(error: ToggleError) -> AppError {
    match error {
        ToggleError::UnknownModule(_) => AppError::NotFound { entity: "module" },
        ToggleError::Dependency { module, missing } => {
            let mut missing_by_module = Map::new();
            missing_by_module.insert(
                module.to_string(),
                Value::Array(
                    missing
                        .into_iter()
                        .map(|id| Value::String(id.to_string()))
                        .collect(),
                ),
            );
            let mut extra = Map::new();
            extra.insert("missing".to_owned(), Value::Object(missing_by_module));
            AppError::Unprocessable {
                code: &codes::MODULE_DEPENDENCY,
                detail: String::from("module requirements are not satisfied"),
                extra,
            }
        }
        ToggleError::RequiredBy { required_by, .. } => {
            let mut extra = Map::new();
            extra.insert(
                "required_by".to_owned(),
                Value::Array(
                    required_by
                        .into_iter()
                        .map(|id| Value::String(id.to_string()))
                        .collect(),
                ),
            );
            AppError::Unprocessable {
                code: &codes::MODULE_REQUIRED_BY,
                detail: String::from("module is required by an enabled module"),
                extra,
            }
        }
    }
}
