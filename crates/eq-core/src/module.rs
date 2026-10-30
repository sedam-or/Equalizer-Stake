//! Module contracts, runtime gates and tenant enablement (MOD-01..08).

pub mod domain;
pub mod gate;
pub mod registry;
mod repo;
pub mod service;

pub use registry::ModuleRegistry;

#[macro_export]
macro_rules! impl_module_stub {
    ($module_type:ident, $id:ident, $id_key:literal, $icon:literal, $always_on:expr, $requires_all:expr, $requires_any:expr) => {
        $crate::impl_module_stub!(
            $module_type,
            $id,
            $id_key,
            $icon,
            $always_on,
            $requires_all,
            $requires_any,
            &[]
        );
    };
    ($module_type:ident, $id:ident, $id_key:literal, $icon:literal, $always_on:expr, $requires_all:expr, $requires_any:expr, $permissions:expr) => {
        #[derive(Debug)]
        pub struct $module_type;

        static MANIFEST: $crate::module::ModuleManifest = $crate::module::ModuleManifest {
            id: $crate::module::ModuleId::$id,
            name_key: concat!("module.", $id_key, ".name"),
            description_key: concat!("module.", $id_key, ".description"),
            icon: $icon,
            always_on: $always_on,
            requires_all: $requires_all,
            requires_any: $requires_any,
            policy_schema_version: 1,
        };

        impl $crate::module::Module for $module_type {
            fn manifest(&self) -> &'static $crate::module::ModuleManifest {
                &MANIFEST
            }

            fn policy_schema(&self) -> $crate::module::serde_json::Value {
                $crate::module::serde_json::json!({})
            }

            fn default_policy(&self) -> $crate::module::serde_json::Value {
                $crate::module::serde_json::json!({})
            }

            fn validate_policy(
                &self,
                _document: &$crate::module::serde_json::Value,
            ) -> Result<(), Vec<$crate::module::FieldError>> {
                Ok(())
            }

            fn summarize_policy(
                &self,
                _document: &$crate::module::serde_json::Value,
            ) -> Vec<$crate::module::SummaryLine> {
                Vec::new()
            }

            fn migrate_policy(
                &self,
                _from_version: u32,
                document: $crate::module::serde_json::Value,
            ) -> anyhow::Result<$crate::module::serde_json::Value> {
                Ok(document)
            }

            fn routes(&self) -> utoipa_axum::router::OpenApiRouter<$crate::app_state::AppState> {
                utoipa_axum::router::OpenApiRouter::new()
            }

            fn event_handlers(&self) -> Vec<Box<dyn $crate::outbox::EventHandler>> {
                Vec::new()
            }

            fn jobs(&self) -> Vec<Box<dyn $crate::jobs::JobHandler>> {
                Vec::new()
            }

            fn cron(&self) -> Vec<$crate::jobs::CronSpec> {
                Vec::new()
            }

            fn register_ports(&self, _ports: &mut $crate::ports::PortRegistry) {}

            fn permissions(&self) -> &'static [$crate::module::PermissionDef] {
                $permissions
            }

            fn nav(&self) -> &'static [$crate::module::NavItem] {
                &[]
            }
        }
    };
}

pub use serde_json;

use crate::app_state::AppState;
use crate::db::Tx;
use crate::ids::TenantId;
use crate::jobs::{CronSpec, JobHandler};
use crate::outbox::EventHandler;
use crate::policy::PolicyDoc;
use crate::ports::PortRegistry;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

pub use crate::error::FieldError;

/// MOD-01: closed module identity set in canonical enum order.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    ToSchema,
    sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ModuleId {
    Core,
    Identity,
    Audit,
    Scheduling,
    Enrollment,
    Billing,
    Attendance,
    Rental,
    Notify,
    Reports,
    Feedback,
}

impl ModuleId {
    /// MOD-01: returns the canonical wire and database identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Identity => "identity",
            Self::Audit => "audit",
            Self::Scheduling => "scheduling",
            Self::Enrollment => "enrollment",
            Self::Billing => "billing",
            Self::Attendance => "attendance",
            Self::Rental => "rental",
            Self::Notify => "notify",
            Self::Reports => "reports",
            Self::Feedback => "feedback",
        }
    }

    /// MOD-01: parses a canonical module identifier from persisted text.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "core" => Some(Self::Core),
            "identity" => Some(Self::Identity),
            "audit" => Some(Self::Audit),
            "scheduling" => Some(Self::Scheduling),
            "enrollment" => Some(Self::Enrollment),
            "billing" => Some(Self::Billing),
            "attendance" => Some(Self::Attendance),
            "rental" => Some(Self::Rental),
            "notify" => Some(Self::Notify),
            "reports" => Some(Self::Reports),
            "feedback" => Some(Self::Feedback),
            _ => None,
        }
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// MOD-03: immutable module catalog entry.
#[derive(Clone, Debug)]
pub struct ModuleManifest {
    pub id: ModuleId,
    pub name_key: &'static str,
    pub description_key: &'static str,
    pub icon: &'static str,
    pub always_on: bool,
    pub requires_all: &'static [ModuleId],
    pub requires_any: &'static [ModuleId],
    pub policy_schema_version: u32,
}

/// MOD-07: navigation group rendered by the SPA.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NavSection {
    Main,
    Admin,
    Settings,
}

/// MOD-07: permission-filtered navigation item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema, ToSchema)]
pub struct NavItem {
    pub id: &'static str,
    pub label_key: &'static str,
    pub icon: &'static str,
    pub path: &'static str,
    pub perm: &'static str,
    pub section: NavSection,
    pub order: u16,
}

/// IDN-10: permission key and its default role grants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionDef {
    pub key: &'static str,
    pub roles: &'static [crate::actor::ActorRole],
}

/// POL-07: an i18n summary key and interpolation arguments.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct SummaryLine {
    pub key: String,
    pub args: Map<String, Value>,
}

/// T-314: one generated post-setup checklist item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ChecklistItem {
    pub key: String,
    pub done: bool,
    pub href: String,
    pub order: u16,
}

/// MOD-01, MOD-03: module behavior and its normative API route type.
#[async_trait]
pub trait Module: Send + Sync + 'static {
    fn manifest(&self) -> &'static ModuleManifest;

    /// POL-06: returns the module policy's JSON Schema.
    fn policy_schema(&self) -> Value;

    /// POL-09: returns the built-in default policy document.
    fn default_policy(&self) -> Value;

    /// POL-10: validates a full policy document.
    fn validate_policy(&self, document: &Value) -> Result<(), Vec<FieldError>>;

    /// POL-10: validates policy values that require tenant data; default accepts the schema-valid document.
    async fn validate_policy_in_context(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
        _document: &Value,
    ) -> Result<(), Vec<FieldError>> {
        Ok(())
    }

    /// MOD-07: reports whether the tenant has module-owned data.
    async fn has_data(&self, _tx: &mut Tx, _tenant: TenantId) -> anyhow::Result<bool> {
        Ok(false)
    }

    /// SET-01: seeds this module's preset catalog section.
    async fn seed_from_preset(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
        _catalog: &Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// T-314: returns generated post-setup checklist items.
    async fn checklist(&self, _tx: &mut Tx, _tenant: TenantId) -> anyhow::Result<Vec<ChecklistItem>> {
        Ok(Vec::new())
    }

    /// PRD-06: returns i18n keys and arguments, not server-built sentences.
    fn summarize_policy(&self, document: &Value) -> Vec<SummaryLine>;

    /// POL-08: migrates a stored policy document between schema versions.
    fn migrate_policy(&self, from_version: u32, document: Value) -> anyhow::Result<Value>;

    /// MOD-04: returns routes with state compatible with the shared API router.
    fn routes(&self) -> OpenApiRouter<AppState>;

    /// MOD-04: handlers subscribed to this module's domain events.
    fn event_handlers(&self) -> Vec<Box<dyn EventHandler>>;

    /// MOD-04: job handlers owned by this module.
    fn jobs(&self) -> Vec<Box<dyn JobHandler>>;

    /// ARC-09: schedules owned by this module.
    fn cron(&self) -> Vec<CronSpec>;

    /// ARC-08: registers typed synchronous ports implemented by this module.
    fn register_ports(&self, ports: &mut PortRegistry);

    /// IDN-10: permissions and default role grants.
    fn permissions(&self) -> &'static [PermissionDef];

    /// MOD-07: navigation items, filtered by enabled state and permissions at runtime.
    fn nav(&self) -> &'static [NavItem];

    /// MOD-06: performs module-owned work inside the caller transaction.
    async fn on_enable(&self, _tx: &mut Tx, _tenant: TenantId) -> anyhow::Result<()> {
        Ok(())
    }

    /// MOD-06: performs module-owned work inside the caller transaction.
    async fn on_disable(&self, _tx: &mut Tx, _tenant: TenantId) -> anyhow::Result<()> {
        Ok(())
    }
}

/// MOD-01/POL-06: the always-on tenant Core policy module.
#[derive(Debug, Default)]
pub struct CoreModule;

static CORE_MANIFEST: ModuleManifest = ModuleManifest {
    id: ModuleId::Core,
    name_key: "module.core.name",
    description_key: "module.core.description",
    icon: "settings",
    always_on: true,
    requires_all: &[],
    requires_any: &[],
    policy_schema_version: crate::policy::CorePolicy::SCHEMA_VERSION,
};

static CORE_POLICY_READ_ROLES: &[crate::actor::ActorRole] =
    &[crate::actor::ActorRole::Owner, crate::actor::ActorRole::Admin];
static CORE_POLICY_WRITE_ROLES: &[crate::actor::ActorRole] = &[crate::actor::ActorRole::Owner];
static CORE_MODULE_READ_ROLES: &[crate::actor::ActorRole] =
    &[crate::actor::ActorRole::Owner, crate::actor::ActorRole::Admin];
static CORE_MODULE_WRITE_ROLES: &[crate::actor::ActorRole] = &[crate::actor::ActorRole::Owner];
static CORE_PERMISSIONS: [PermissionDef; 4] = [
    PermissionDef {
        key: "core.policy.read",
        roles: CORE_POLICY_READ_ROLES,
    },
    PermissionDef {
        key: "core.policy.write",
        roles: CORE_POLICY_WRITE_ROLES,
    },
    PermissionDef {
        key: "core.modules.read",
        roles: CORE_MODULE_READ_ROLES,
    },
    PermissionDef {
        key: "core.modules.write",
        roles: CORE_MODULE_WRITE_ROLES,
    },
];

#[async_trait]
impl Module for CoreModule {
    fn manifest(&self) -> &'static ModuleManifest {
        &CORE_MANIFEST
    }

    fn policy_schema(&self) -> Value {
        crate::policy::CorePolicy::json_schema()
    }

    fn default_policy(&self) -> Value {
        serde_json::to_value(crate::policy::CorePolicy::default()).unwrap_or(Value::Null)
    }

    fn validate_policy(&self, document: &Value) -> Result<(), Vec<FieldError>> {
        crate::policy::validate_core_document(document)
    }

    fn summarize_policy(&self, document: &Value) -> Vec<SummaryLine> {
        serde_json::from_value::<crate::policy::CorePolicy>(document.clone())
            .map(|policy| PolicyDoc::summarize(&policy))
            .unwrap_or_default()
    }

    fn migrate_policy(&self, from_version: u32, document: Value) -> anyhow::Result<Value> {
        if from_version == crate::policy::CorePolicy::SCHEMA_VERSION {
            Ok(document)
        } else {
            Err(anyhow::anyhow!("unsupported Core policy schema version"))
        }
    }

    fn routes(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
    }

    fn event_handlers(&self) -> Vec<Box<dyn EventHandler>> {
        Vec::new()
    }

    fn jobs(&self) -> Vec<Box<dyn JobHandler>> {
        Vec::new()
    }

    fn cron(&self) -> Vec<CronSpec> {
        Vec::new()
    }

    fn register_ports(&self, _ports: &mut PortRegistry) {}

    fn permissions(&self) -> &'static [PermissionDef] {
        &CORE_PERMISSIONS
    }

    fn nav(&self) -> &'static [NavItem] {
        &[]
    }
}
