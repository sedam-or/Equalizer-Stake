//! Tenant and module discovery endpoints (API-14, MOD-07/09, VAL-23).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use eq_core::app_state::AppState;
use eq_core::auth::Actor;
use eq_core::db::{Tx, begin_read_only};
use eq_core::error::AppError;
use eq_core::ids::TenantId;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::module::{ModuleId, ModuleRegistry};
use eq_core::policy::CorePolicy;
use eq_core::tenant::{RequireTenant, TenantCtx};
use eq_identity::api_auth::LoginHttpState;
use serde_json::{Value, json};
use utoipa_axum::router::OpenApiRouter;

/// MOD-07/09: shared compiled module, enablement and policy runtime for metadata reads.
#[derive(Clone)]
pub struct MetadataState {
    pub(crate) registry: Arc<ModuleRegistry>,
    pub(crate) enabled: Arc<dyn EnabledModuleSet>,
    pub(crate) runtime: Arc<LoginHttpState>,
}

impl std::fmt::Debug for MetadataState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MetadataState")
            .field("registry", &self.registry)
            .field("enabled", &"[tenant module set]")
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl MetadataState {
    /// MOD-07/09: builds metadata state from the API's process-scoped module runtimes.
    pub fn new(
        registry: Arc<ModuleRegistry>,
        enabled: Arc<dyn EnabledModuleSet>,
        runtime: Arc<LoginHttpState>,
    ) -> Self {
        Self {
            registry,
            enabled,
            runtime,
        }
    }
}

/// MOD-09: authenticated tenant module catalog and permission-filtered navigation.
pub fn module_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(utoipa_axum::routes!(get_modules))
}

/// API-14: public application metadata for provisioned and unprovisioned tenants.
pub fn public_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(utoipa_axum::routes!(get_app))
}

#[utoipa::path(
    get,
    path = "/api/v1/meta/modules",
    responses((status = 200, description = "Compiled and tenant-enabled module catalog", body = Value)),
    security(("bearer" = [])),
    tag = "core"
)]
async fn get_modules(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Extension(metadata): Extension<MetadataState>,
) -> Result<Json<Value>, AppError> {
    ensure_tenant(&actor, tenant.id)?;
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    let body = module_document(
        &metadata.registry,
        metadata.enabled.as_ref(),
        metadata.runtime.as_ref(),
        &mut tx,
        tenant.id,
        Some(&actor),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(body))
}

#[utoipa::path(
    get,
    path = "/api/v1/meta/app",
    responses((status = 200, description = "Public application and tenant metadata", body = Value)),
    tag = "core"
)]
async fn get_app(
    State(state): State<AppState>,
    tenant: TenantCtx,
    Extension(metadata): Extension<MetadataState>,
) -> Result<Json<Value>, AppError> {
    match &tenant {
        TenantCtx::Unprovisioned => Ok(Json(app_document(state.config.mode, true, None))),
        TenantCtx::Resolved(info) => {
            let actor = eq_core::actor::ActorCtx::anonymous(None, None);
            let mut tx = begin_read_only(&state.pool, info.id, &actor).await?;
            let policy_store = metadata.runtime.policy_store().await?;
            let policy = policy_store.current::<CorePolicy>(&mut tx, info.id).await?;
            tx.commit().await?;
            let name = state.tenant_resolver.public_name(info.id).await?;
            let identity = policy_store_for_identity(&metadata.runtime, &state, info.id).await?;
            Ok(Json(app_document(
                state.config.mode,
                false,
                Some((name, policy.doc, identity.password_min_length())),
            )))
        }
    }
}

async fn policy_store_for_identity(
    runtime: &LoginHttpState,
    state: &AppState,
    tenant: TenantId,
) -> Result<eq_identity::policy::IdentityPolicy, AppError> {
    let actor = eq_core::actor::ActorCtx::anonymous(None, None);
    let mut tx = begin_read_only(&state.pool, tenant, &actor).await?;
    let store = runtime.policy_store().await?;
    let identity = store
        .current::<eq_identity::policy::IdentityPolicy>(&mut tx, tenant)
        .await?;
    tx.commit().await?;
    Ok(identity.doc)
}

fn app_document(
    mode: eq_core::config::Mode,
    setup_required: bool,
    tenant: Option<(String, CorePolicy, u8)>,
) -> Value {
    let mode = match mode {
        eq_core::config::Mode::Single => "single",
        eq_core::config::Mode::Multi => "multi",
    };
    let mut body = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "commit": option_env!("RITMA_COMMIT").unwrap_or("unknown"),
        "mode": mode,
        "brand": {
            "name": eq_core::brand::NAME,
            "logo_url": Value::Null,
            "brand_hue": CorePolicy::default().branding.brand_hue.get(),
            "corner_style": CorePolicy::default().branding.corner_style,
        },
        "features": {"captcha": false, "guests": false},
        "rules": {"password_min_length": eq_identity::policy::IdentityPolicy::default().password_min_length()},
        "setup_required": setup_required,
    });
    if let Some((name, policy, password_min_length)) = tenant {
        body["brand"]["brand_hue"] = json!(policy.branding.brand_hue.get());
        body["brand"]["corner_style"] = json!(policy.branding.corner_style);
        body["rules"]["password_min_length"] = json!(password_min_length);
        if !setup_required {
            body["tenant"] = json!({
                "name": name,
                "locale": policy.locale,
                "currency": policy.currency.code(),
                "timezone": policy.timezone.as_str(),
                "terminology": policy.terminology,
            });
        }
    }
    body
}

/// MOD-07/09: serializes the catalog with tenant enablement, policy versions and allowed navigation.
pub(crate) async fn module_document(
    registry: &ModuleRegistry,
    enabled: &dyn EnabledModuleSet,
    runtime: &LoginHttpState,
    tx: &mut Tx,
    tenant: TenantId,
    actor: Option<&Actor>,
    enabled_override: Option<&std::collections::BTreeSet<ModuleId>>,
) -> Result<Value, AppError> {
    let policy_store = runtime.policy_store().await?;
    let compiled = registry
        .modules()
        .iter()
        .map(|module| module.manifest().id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut modules = Vec::with_capacity(11);
    let mut nav = Vec::new();

    for module in registry.modules() {
        let manifest = module.manifest();
        let is_enabled = match enabled_override {
            Some(set) => manifest.always_on || set.contains(&manifest.id),
            None => enabled.is_enabled(tenant, manifest.id).await?,
        };
        let has_data = module.has_data(tx, tenant).await.map_err(AppError::Internal)?;
        let policy_version = policy_store.current_version(tx, tenant, manifest.id).await?;
        modules.push(json!({
            "id": manifest.id,
            "compiled": true,
            "enabled": is_enabled,
            "always_on": manifest.always_on,
            "requires_all": manifest.requires_all,
            "requires_any": manifest.requires_any,
            "has_data": has_data,
            "policy_version": policy_version,
        }));
        if is_enabled && let Some(actor) = actor {
            nav.extend(module.nav().iter().filter(|item| actor.has(item.perm)).cloned());
        }
    }

    for id in all_module_ids() {
        if !compiled.contains(&id) {
            modules.push(json!({
                "id": id,
                "compiled": false,
                "enabled": false,
                "always_on": false,
                "requires_all": [],
                "requires_any": [],
                "has_data": false,
                "policy_version": Value::Null,
            }));
        }
    }
    nav.sort_by_key(|item| item.order);
    Ok(json!({"modules": modules, "nav": nav}))
}

fn all_module_ids() -> [ModuleId; 11] {
    [
        ModuleId::Core,
        ModuleId::Identity,
        ModuleId::Audit,
        ModuleId::Scheduling,
        ModuleId::Enrollment,
        ModuleId::Billing,
        ModuleId::Attendance,
        ModuleId::Rental,
        ModuleId::Notify,
        ModuleId::Reports,
        ModuleId::Feedback,
    ]
}

fn ensure_tenant(actor: &Actor, tenant: TenantId) -> Result<(), AppError> {
    if actor.tenant() == tenant {
        Ok(())
    } else {
        Err(AppError::NotFound { entity: "tenant" })
    }
}
