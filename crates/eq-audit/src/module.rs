//! Always-on audit module composition (MOD-01/04, AUD-01/02/07).

use crate::jobs::{AuditRetentionJob, AuditVerifyRecentJob};
use crate::policy::AuditPolicy;
use crate::{AuditEvent, Category, record};
use chrono::DateTime;
use chrono_tz::Tz;
use eq_core::actor::ActorCtx;
use eq_core::app_state::AppState;
use eq_core::db::Tx;
use eq_core::error::{AppError, FieldError, NOT_ALLOWED};
use eq_core::ids::TenantId;
use eq_core::jobs::{CronSpec, JobHandler};
use eq_core::module::{Module, ModuleId, ModuleManifest, NavItem, PermissionDef, SummaryLine};
use eq_core::outbox::EventHandler;
use eq_core::policy::PolicyDoc;
use eq_core::ports::PortRegistry;
use eq_core::ports::sinks::{AuditSink, PolicyChange};
use serde_json::{Map, Value, json};
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;

static MANIFEST: ModuleManifest = ModuleManifest {
    id: ModuleId::Audit,
    name_key: "module.audit.name",
    description_key: "module.audit.description",
    icon: "scroll-text",
    always_on: true,
    requires_all: &[],
    requires_any: &[],
    policy_schema_version: AuditPolicy::SCHEMA_VERSION,
};

/// AUD-01/MOD-01: audit is available to every tenant and cannot be toggled off.
#[derive(Debug, Default)]
pub struct AuditModule;

impl Module for AuditModule {
    fn manifest(&self) -> &'static ModuleManifest {
        &MANIFEST
    }

    fn policy_schema(&self) -> Value {
        AuditPolicy::json_schema()
    }

    fn default_policy(&self) -> Value {
        let policy = AuditPolicy::default();
        json!({
            "retention_years": policy.retention_years(),
            "anonymize_pii_after_years": policy.anonymize_pii_after_years(),
            "admin_categories": policy.admin_categories(),
        })
    }

    fn validate_policy(&self, document: &Value) -> Result<(), Vec<FieldError>> {
        match eq_core::policy::validate_document::<AuditPolicy>(document.clone()) {
            Ok(_) => Ok(()),
            Err(AppError::Validation { errors }) => Err(errors),
            Err(_) => Err(vec![FieldError {
                pointer: String::new(),
                code: NOT_ALLOWED.to_owned(),
                params: Map::new(),
            }]),
        }
    }

    fn summarize_policy(&self, document: &Value) -> Vec<SummaryLine> {
        serde_json::from_value::<AuditPolicy>(document.clone())
            .map(|policy| policy.summarize())
            .unwrap_or_default()
    }

    fn migrate_policy(&self, from_version: u32, document: Value) -> anyhow::Result<Value> {
        if from_version == AuditPolicy::SCHEMA_VERSION {
            Ok(document)
        } else {
            Err(anyhow::anyhow!("unsupported audit policy schema version"))
        }
    }

    fn routes(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
    }

    fn event_handlers(&self) -> Vec<Box<dyn EventHandler>> {
        Vec::new()
    }

    fn jobs(&self) -> Vec<Box<dyn JobHandler>> {
        vec![Box::new(AuditRetentionJob), Box::new(AuditVerifyRecentJob)]
    }

    fn cron(&self) -> Vec<CronSpec> {
        vec![
            CronSpec {
                kind: "audit.retention",
                module: ModuleId::Audit,
                schedule: "0 3 * * *",
                dedupe: retention_dedupe,
            },
            CronSpec {
                kind: "audit.verify_recent",
                module: ModuleId::Audit,
                schedule: "30 3 * * *",
                dedupe: verify_recent_dedupe,
            },
        ]
    }

    fn register_ports(&self, ports: &mut PortRegistry) {
        let audit_sink: Arc<dyn AuditSink> = Arc::new(AuditModule);
        ports.register(audit_sink);
    }

    fn permissions(&self) -> &'static [PermissionDef] {
        &[]
    }

    fn nav(&self) -> &'static [NavItem] {
        &[]
    }
}

#[async_trait::async_trait]
impl AuditSink for AuditModule {
    async fn record_module_toggle(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        enabled: bool,
    ) -> Result<(), AppError> {
        crate::repo::check_transaction_tenant(tx, tenant).await?;
        let action = if enabled {
            "core.module.enabled"
        } else {
            "core.module.disabled"
        };
        record(
            tx,
            AuditEvent {
                action,
                category: Category::Security,
                entity_type: "module",
                entity_id: None,
                before: Some(json!({
                    "module": module.as_str(),
                    "enabled": !enabled,
                })),
                after: Some(json!({
                    "module": module.as_str(),
                    "enabled": enabled,
                })),
                actor,
            },
        )
        .await
    }

    async fn record_policy_change(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        change: PolicyChange,
    ) -> Result<(), AppError> {
        crate::repo::check_transaction_tenant(tx, tenant).await?;
        if change.version < 1 {
            return Err(AppError::Internal(
                std::io::Error::other("audit policy version must be positive").into(),
            ));
        }
        let module = change.module.as_str();
        let before = match change.previous {
            Some(document) => {
                let previous_version = change
                    .version
                    .checked_sub(1)
                    .filter(|version| *version > 0)
                    .ok_or_else(|| {
                        AppError::Internal(
                            std::io::Error::other("previous audit policy version is invalid").into(),
                        )
                    })?;
                Some(json!({
                    "module": module,
                    "version": previous_version,
                    "document": document,
                }))
            }
            None => None,
        };
        let after = Some(json!({
            "module": module,
            "version": change.version,
            "document": change.document,
        }));
        record(
            tx,
            AuditEvent {
                action: "core.policy.changed",
                category: Category::Security,
                entity_type: "policy",
                entity_id: None,
                before,
                after,
                actor,
            },
        )
        .await
    }
}

fn retention_dedupe(tenant: TenantId, scheduled_at: DateTime<Tz>) -> String {
    format!("audit.retention:{}:{}", tenant.0, scheduled_at.timestamp_micros())
}

fn verify_recent_dedupe(tenant: TenantId, scheduled_at: DateTime<Tz>) -> String {
    format!(
        "audit.verify_recent:{}:{}",
        tenant.0,
        scheduled_at.timestamp_micros()
    )
}

#[cfg(test)]
mod tests {
    use super::AuditModule;
    use eq_core::module::{Module, ModuleId};
    use eq_core::ports::{PortRegistry, sinks::AuditSink};
    use serde_json::json;

    #[test]
    fn audit_module_registers_its_always_on_jobs_and_schedules() {
        let module = AuditModule;
        let manifest = module.manifest();
        assert_eq!(manifest.id, ModuleId::Audit);
        assert!(manifest.always_on);
        assert_eq!(manifest.policy_schema_version, 1);
        assert!(manifest.requires_all.is_empty());
        assert!(manifest.requires_any.is_empty());

        let mut ports = PortRegistry::default();
        module.register_ports(&mut ports);
        assert!(ports.get::<dyn AuditSink>().is_some());

        let jobs = module.jobs();
        assert_eq!(
            jobs.iter().map(|job| job.kind()).collect::<Vec<_>>(),
            vec!["audit.retention", "audit.verify_recent"]
        );
        assert_eq!(
            module
                .cron()
                .iter()
                .map(|spec| (spec.kind, spec.module, spec.schedule))
                .collect::<Vec<_>>(),
            vec![
                ("audit.retention", ModuleId::Audit, "0 3 * * *"),
                ("audit.verify_recent", ModuleId::Audit, "30 3 * * *"),
            ]
        );
        assert!(module.event_handlers().is_empty());
        assert!(module.permissions().is_empty());
        assert!(module.nav().is_empty());

        let default = module.default_policy();
        assert_eq!(default["retention_years"], 7);
        assert_eq!(default["anonymize_pii_after_years"], 2);
        assert!(module.validate_policy(&default).is_ok());
        assert_eq!(
            module.summarize_policy(&default)[0].key,
            "policy.audit.summary.retention"
        );
        assert_eq!(
            module.policy_schema()["properties"]["retention_years"]["x-ui"]["group"],
            "advanced"
        );

        let invalid = json!({
            "retention_years": 2,
            "anonymize_pii_after_years": 3,
            "admin_categories": ["operational"]
        });
        let Err(errors) = module.validate_policy(&invalid) else {
            panic!("audit policy cross-field limits must be validated");
        };
        assert_eq!(errors[0].pointer, "/anonymize_pii_after_years");
        assert_eq!(errors[0].code, eq_core::error::MUST_BE_LTE);
    }
}
