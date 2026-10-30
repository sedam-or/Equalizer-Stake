//! Identity module manifest and policy integration (MOD-01, POL-01/06/07/10).

use eq_core::actor::ActorRole;
use eq_core::app_state::AppState;
use eq_core::error::{AppError, FieldError, NOT_ALLOWED};
use eq_core::jobs::{CronSpec, JobHandler};
use eq_core::module::{Module, ModuleId, ModuleManifest, NavItem, PermissionDef, SummaryLine};
use eq_core::outbox::EventHandler;
use eq_core::policy::{Group, PolicyDoc, XUi, validate_document};
use eq_core::ports::PortRegistry;
use serde_json::{Map, Value, json};
use utoipa_axum::router::OpenApiRouter;

use crate::policy::IdentityPolicy;

const POLICY_SCHEMA_FIELDS: [(&str, u32); 8] = [
    ("idle_timeout_minutes", 1),
    ("absolute_session_days", 2),
    ("lockout", 3),
    ("password_min_length", 4),
    ("invite_expires_days", 5),
    ("reset_token_ttl_minutes", 6),
    ("reset_max_per_hour", 7),
    ("totp", 8),
]; // technical: preserve the policy field order in the settings schema

const ROLE_FIELDS: [(&str, u32); 5] = [
    ("owner", 1),
    ("admin", 2),
    ("instructor", 3),
    ("guardian", 4),
    ("student", 5),
]; // technical: keep role values in their declared RoleMap order

const LOCKOUT_FIELDS: [(&str, u32); 4] = [
    ("max_failed", 1),
    ("window_minutes", 2),
    ("lock_minutes", 3),
    ("notify_admins", 4),
]; // technical: keep nested lockout settings in declaration order

static IDENTITY_WRITE_ROLES: &[ActorRole] = &[ActorRole::Admin];
static IDENTITY_ROLE_WRITE_ROLES: &[ActorRole] = &[ActorRole::Owner, ActorRole::Admin];
static IDENTITY_ADMIN_MANAGE_ROLES: &[ActorRole] = &[ActorRole::Owner];
static IDENTITY_READ_ROLES: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static IDENTITY_READ_PII_ROLES: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Guardian,
    ActorRole::Student,
];
static IDENTITY_PERMISSIONS: [PermissionDef; 5] = [
    PermissionDef {
        key: "identity.people.read",
        roles: IDENTITY_READ_ROLES,
    },
    PermissionDef {
        key: "identity.people.read_pii",
        roles: IDENTITY_READ_PII_ROLES,
    },
    PermissionDef {
        key: "identity.people.write",
        roles: IDENTITY_WRITE_ROLES,
    },
    PermissionDef {
        key: "identity.roles.write",
        roles: IDENTITY_ROLE_WRITE_ROLES,
    },
    PermissionDef {
        key: "identity.admins.manage",
        roles: IDENTITY_ADMIN_MANAGE_ROLES,
    },
];

static MANIFEST: ModuleManifest = ModuleManifest {
    id: ModuleId::Identity,
    name_key: "module.identity.name",
    description_key: "module.identity.description",
    icon: "users",
    always_on: true,
    requires_all: &[],
    requires_any: &[],
    policy_schema_version: IdentityPolicy::SCHEMA_VERSION,
};

/// Identity's policy schema and module-level contract.
#[derive(Debug, Default)]
pub struct IdentityModule;

impl Module for IdentityModule {
    fn manifest(&self) -> &'static ModuleManifest {
        &MANIFEST
    }

    fn policy_schema(&self) -> Value {
        identity_policy_schema()
    }

    fn default_policy(&self) -> Value {
        let policy = IdentityPolicy::default();
        json!({
            "idle_timeout_minutes": {
                "owner": policy.idle_timeout_minutes().owner,
                "admin": policy.idle_timeout_minutes().admin,
                "instructor": policy.idle_timeout_minutes().instructor,
                "guardian": policy.idle_timeout_minutes().guardian,
                "student": policy.idle_timeout_minutes().student,
            },
            "absolute_session_days": policy.absolute_session_days(),
            "lockout": {
                "max_failed": policy.lockout().max_failed(),
                "window_minutes": policy.lockout().window_minutes(),
                "lock_minutes": policy.lockout().lock_minutes(),
                "notify_admins": policy.lockout().notify_admins(),
            },
            "password_min_length": policy.password_min_length(),
            "invite_expires_days": policy.invite_expires_days(),
            "reset_token_ttl_minutes": policy.reset_token_ttl_minutes(),
            "reset_max_per_hour": policy.reset_max_per_hour(),
            "totp": policy.totp(),
        })
    }

    fn validate_policy(&self, document: &Value) -> Result<(), Vec<FieldError>> {
        match validate_document::<IdentityPolicy>(document.clone()) {
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
        serde_json::from_value::<IdentityPolicy>(document.clone())
            .map(|policy| policy.summarize())
            // POL-07 exposes no error channel; invalid documents must not yield partial summaries.
            .unwrap_or_default()
    }

    fn migrate_policy(&self, from_version: u32, document: Value) -> anyhow::Result<Value> {
        if from_version == IdentityPolicy::SCHEMA_VERSION {
            Ok(document)
        } else {
            Err(anyhow::anyhow!("unsupported identity policy schema version"))
        }
    }

    fn routes(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
            .routes(utoipa_axum::routes!(crate::api_auth::login))
            .routes(utoipa_axum::routes!(crate::api_auth::refresh))
            .routes(utoipa_axum::routes!(crate::api_auth::switch_role))
            .routes(utoipa_axum::routes!(crate::api_invites::preview))
            .routes(utoipa_axum::routes!(crate::api_invites::accept))
            .routes(utoipa_axum::routes!(crate::api_reset::forgot))
            .routes(utoipa_axum::routes!(crate::api_reset::reset))
            .routes(utoipa_axum::routes!(crate::api_me::get_me))
            .routes(utoipa_axum::routes!(crate::api_me::patch_me))
            .routes(utoipa_axum::routes!(crate::api_me::change_password))
            .routes(utoipa_axum::routes!(crate::api_me::setup_totp))
            .routes(utoipa_axum::routes!(crate::api_me::confirm_totp))
            .routes(utoipa_axum::routes!(crate::api_me::disable_totp))
            .routes(utoipa_axum::routes!(crate::api_me::list_sessions))
            .routes(utoipa_axum::routes!(crate::api_me::revoke_session))
            .routes(utoipa_axum::routes!(crate::api_people::list_people))
            .routes(utoipa_axum::routes!(crate::api_people::get_person))
            .routes(utoipa_axum::routes!(crate::api_people::grant_role))
            .routes(utoipa_axum::routes!(crate::api_people::remove_role))
            .routes(utoipa_axum::routes!(crate::api_people::create_guardian_link))
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
        &IDENTITY_PERMISSIONS
    }

    fn nav(&self) -> &'static [NavItem] {
        &[]
    }
}

fn identity_policy_schema() -> Value {
    let Ok(mut schema) = serde_json::to_value(schemars::schema_for!(IdentityPolicy)) else {
        return json!({"not": {}});
    };
    add_xui_to_properties(&mut schema, "#", &POLICY_SCHEMA_FIELDS);
    add_referenced_property_metadata(&mut schema, "idle_timeout_minutes", &ROLE_FIELDS);
    add_referenced_property_metadata(&mut schema, "lockout", &LOCKOUT_FIELDS);
    set_role_map_bounds(&mut schema);
    schema
}

fn add_xui_to_properties(schema: &mut Value, reference: &str, fields: &[(&str, u32)]) {
    let Some(definition) = schema_definition_mut(schema, reference) else {
        return;
    };
    let Some(properties) = definition.get_mut("properties").and_then(Value::as_object_mut) else {
        return;
    };
    for (field, order) in fields {
        if let Some(property) = properties.get_mut(*field).and_then(Value::as_object_mut) {
            property.insert("x-ui".to_owned(), XUi::new(Group::Advanced, *order).into_value());
        }
    }
}

fn add_referenced_property_metadata(schema: &mut Value, field: &str, nested_fields: &[(&str, u32)]) {
    let reference = schema
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get(field))
        .and_then(Value::as_object)
        .and_then(|property| property.get("$ref"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(reference) = reference {
        add_xui_to_properties(schema, &reference, nested_fields);
    }
}

fn schema_definition_mut<'a>(schema: &'a mut Value, reference: &str) -> Option<&'a mut Value> {
    if reference == "#" {
        return Some(schema);
    }
    let name = reference.strip_prefix("#/$defs/")?;
    schema.get_mut("$defs")?.get_mut(name)
}

fn set_role_map_bounds(schema: &mut Value) {
    let reference = schema
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get("idle_timeout_minutes"))
        .and_then(Value::as_object)
        .and_then(|property| property.get("$ref"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let Some(reference) = reference else {
        return;
    };
    let Some(definition) = schema_definition_mut(schema, &reference) else {
        return;
    };
    let Some(properties) = definition.get_mut("properties").and_then(Value::as_object_mut) else {
        return;
    };
    for (field, _) in ROLE_FIELDS {
        if let Some(property) = properties.get_mut(field).and_then(Value::as_object_mut) {
            property.insert("minimum".to_owned(), json!(5));
            property.insert("maximum".to_owned(), json!(1440));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IdentityModule, LOCKOUT_FIELDS, POLICY_SCHEMA_FIELDS, ROLE_FIELDS, identity_policy_schema};
    use crate::policy::{IdentityPolicy, TotpPolicy};
    use eq_core::actor::ActorRole;
    use eq_core::module::{Module, ModuleId};
    use eq_core::policy::PolicyDoc;
    use serde_json::{Value, json};

    #[test]
    fn defaults_schema_and_summary_match_contract() {
        let policy = IdentityPolicy::default();
        assert_defaults(&policy);

        let module = IdentityModule;
        assert_manifest(&module);
        let schema = assert_policy_schema(&module);
        let default_policy = module.default_policy();
        assert_eq!(default_policy, serde_json::to_value(&policy).unwrap());
        assert_policy_validation_and_migration(&module, &default_policy);
        assert_empty_surfaces(&module);

        let summaries = policy.summarize();
        assert_eq!(module.summarize_policy(&default_policy), summaries);
        assert_summary_values(&summaries);
        assert_eq!(identity_policy_schema(), schema);
    }

    fn assert_defaults(policy: &IdentityPolicy) {
        assert_eq!(policy.idle_timeout_minutes().owner, 30);
        assert_eq!(policy.idle_timeout_minutes().admin, 30);
        assert_eq!(policy.idle_timeout_minutes().instructor, 60);
        assert_eq!(policy.idle_timeout_minutes().guardian, 120);
        assert_eq!(policy.idle_timeout_minutes().student, 120);
        assert_eq!(policy.absolute_session_days(), 30);
        assert_eq!(policy.lockout().max_failed(), 5);
        assert_eq!(policy.lockout().window_minutes(), 15);
        assert_eq!(policy.lockout().lock_minutes(), 15);
        assert!(policy.lockout().notify_admins());
        assert_eq!(policy.password_min_length(), 10);
        assert_eq!(policy.invite_expires_days(), 7);
        assert_eq!(policy.reset_token_ttl_minutes(), 60);
        assert_eq!(policy.reset_max_per_hour(), 3);
        assert_eq!(policy.totp(), TotpPolicy::OptionalForStaff);
        assert_eq!(IdentityPolicy::MODULE, ModuleId::Identity);
        assert_eq!(IdentityPolicy::SCHEMA_VERSION, 1);
    }

    fn assert_manifest(module: &IdentityModule) {
        let manifest = module.manifest();
        assert_eq!(manifest.id, ModuleId::Identity);
        assert!(manifest.always_on);
        assert_eq!(manifest.policy_schema_version, 1);
        assert!(manifest.requires_all.is_empty());
        assert!(manifest.requires_any.is_empty());
    }

    fn assert_policy_schema(module: &IdentityModule) -> Value {
        let schema = module.policy_schema();
        for (field, order) in POLICY_SCHEMA_FIELDS {
            assert_eq!(schema["properties"][field]["x-ui"]["group"], "advanced");
            assert_eq!(schema["properties"][field]["x-ui"]["order"], order);
        }
        assert_bounds(&schema["properties"]["absolute_session_days"], 1, 90);
        assert_bounds(&schema["properties"]["password_min_length"], 8, 64);
        assert_bounds(&schema["properties"]["invite_expires_days"], 1, 30);
        assert_bounds(&schema["properties"]["reset_token_ttl_minutes"], 10, 1440);
        assert_bounds(&schema["properties"]["reset_max_per_hour"], 1, 10);
        let totp_values = referenced_schema(&schema, "totp")["oneOf"]
            .as_array()
            .map(|variants| {
                variants
                    .iter()
                    .map(|variant| variant["const"].clone())
                    .collect::<Vec<_>>()
            });
        assert_eq!(
            totp_values,
            Some(vec![
                json!("off"),
                json!("optional_for_staff"),
                json!("required_for_owner_admin"),
            ]),
            "TOTP schema reference: {}; resolved definition: {}",
            schema["properties"]["totp"],
            referenced_schema(&schema, "totp")
        );

        let role_definition = referenced_schema(&schema, "idle_timeout_minutes");
        for (field, order) in ROLE_FIELDS {
            assert_bounds(&role_definition["properties"][field], 5, 1440);
            assert_eq!(role_definition["properties"][field]["x-ui"]["group"], "advanced");
            assert_eq!(role_definition["properties"][field]["x-ui"]["order"], order);
        }
        let lockout_definition = referenced_schema(&schema, "lockout");
        for (field, order) in LOCKOUT_FIELDS {
            assert_eq!(
                lockout_definition["properties"][field]["x-ui"]["group"],
                "advanced"
            );
            assert_eq!(lockout_definition["properties"][field]["x-ui"]["order"], order);
        }
        assert_bounds(&lockout_definition["properties"]["max_failed"], 3, 20);
        assert_bounds(&lockout_definition["properties"]["window_minutes"], 1, 120);
        assert_bounds(&lockout_definition["properties"]["lock_minutes"], 1, 1440);
        schema
    }

    fn assert_policy_validation_and_migration(module: &IdentityModule, default_policy: &Value) {
        assert!(module.validate_policy(default_policy).is_ok());

        let mut invalid_policy = (*default_policy).clone();
        invalid_policy["reset_max_per_hour"] = json!(0);
        assert!(module.validate_policy(&invalid_policy).is_err());
        assert!(module.summarize_policy(&invalid_policy).is_empty());

        let mut invalid_policy = (*default_policy).clone();
        invalid_policy["totp"] = json!("unknown_policy");
        assert!(module.validate_policy(&invalid_policy).is_err());

        let mut invalid_policy = (*default_policy).clone();
        invalid_policy["unknown"] = json!(true);
        assert!(module.validate_policy(&invalid_policy).is_err());

        assert_eq!(
            module
                .migrate_policy(IdentityPolicy::SCHEMA_VERSION, (*default_policy).clone())
                .unwrap(),
            (*default_policy).clone()
        );
        assert!(module.migrate_policy(0, json!({})).is_err());
        assert!(module.migrate_policy(2, json!({})).is_err());
    }

    fn assert_empty_surfaces(module: &IdentityModule) {
        let (_, routes_openapi) = module.routes().split_for_parts();
        assert!(routes_openapi.paths.paths.contains_key("/api/v1/auth/login"));
        assert!(routes_openapi.paths.paths.contains_key("/api/v1/auth/refresh"));
        assert!(
            routes_openapi
                .paths
                .paths
                .contains_key("/api/v1/auth/switch-role")
        );
        assert!(routes_openapi.paths.paths.contains_key("/api/v1/me"));
        assert!(routes_openapi.paths.paths.contains_key("/api/v1/me/password"));
        assert!(routes_openapi.paths.paths.contains_key("/api/v1/me/sessions"));
        assert!(module.event_handlers().is_empty());
        assert!(module.jobs().is_empty());
        assert!(module.cron().is_empty());
        assert_eq!(module.permissions().len(), 5);
        assert_eq!(module.permissions()[0].key, "identity.people.read");
        assert_eq!(module.permissions()[0].roles.len(), 5);
        assert_eq!(module.permissions()[1].key, "identity.people.read_pii");
        assert_eq!(module.permissions()[1].roles.len(), 4);
        assert_eq!(module.permissions()[2].key, "identity.people.write");
        assert_eq!(module.permissions()[2].roles, &[ActorRole::Admin]);
        assert_eq!(module.permissions()[3].key, "identity.roles.write");
        assert_eq!(
            module.permissions()[3].roles,
            &[ActorRole::Owner, ActorRole::Admin]
        );
        assert_eq!(module.permissions()[4].key, "identity.admins.manage");
        assert_eq!(module.permissions()[4].roles, &[ActorRole::Owner]);
        assert!(module.nav().is_empty());
    }

    fn assert_summary_values(summaries: &[eq_core::module::SummaryLine]) {
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].key, "policy.identity.summary.sessions");
        assert_eq!(summaries[0].args["idle_admin"], 30);
        assert_eq!(summaries[0].args["idle_guardian"], 120);
        assert_eq!(summaries[0].args["absolute_days"], 30);
        assert_eq!(summaries[1].key, "policy.identity.summary.lockout");
        assert_eq!(summaries[1].args["max_failed"], 5);
        assert_eq!(summaries[1].args["window"], 15);
        assert_eq!(summaries[1].args["lock"], 15);
    }

    fn assert_bounds(schema: &Value, minimum: u64, maximum: u64) {
        assert_eq!(schema["minimum"], minimum);
        assert_eq!(schema["maximum"], maximum);
    }

    fn referenced_schema<'a>(schema: &'a Value, property: &str) -> &'a Value {
        let reference = schema["properties"][property]["$ref"]
            .as_str()
            .unwrap_or_default();
        let definition = reference.strip_prefix("#/$defs/").unwrap_or_default();
        &schema["$defs"][definition]
    }
}
