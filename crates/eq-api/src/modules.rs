//! Feature-gated module registry and route assembly (MOD-04/05).

use std::sync::Arc;

use async_trait::async_trait;
use eq_core::app_state::AppState;
use eq_core::module::gate::{EnabledModuleSet, ModuleGate};
use eq_core::module::{Module, ModuleId, ModuleRegistry};
use eq_core::ports::{CurrencyLockCheck, PortRegistry};
use eq_identity::api_auth::LoginHttpState;
use utoipa_axum::router::OpenApiRouter;

/// MOD-04: returns compiled modules in the canonical `ModuleId` order.
pub fn registry() -> ModuleRegistry {
    let mut modules: Vec<Arc<dyn Module>> = vec![
        Arc::new(eq_core::module::CoreModule),
        Arc::new(eq_identity::IdentityModule),
        Arc::new(eq_audit::AuditModule),
        Arc::new(eq_scheduling::SchedulingModule),
    ];

    #[cfg(feature = "enrollment")]
    modules.push(Arc::new(eq_enrollment::EnrollmentModule));
    #[cfg(feature = "billing")]
    modules.push(Arc::new(eq_billing::BillingModule));
    #[cfg(feature = "attendance")]
    modules.push(Arc::new(eq_attendance::AttendanceModule));
    #[cfg(feature = "rental")]
    modules.push(Arc::new(eq_rental::RentalModule));
    #[cfg(feature = "notify")]
    modules.push(Arc::new(eq_notify::NotifyModule));
    #[cfg(feature = "reports")]
    modules.push(Arc::new(eq_reports::ReportsModule));
    #[cfg(feature = "feedback")]
    modules.push(Arc::new(eq_feedback::FeedbackModule));

    let registry = ModuleRegistry::new(modules);
    registry.validate_graph();
    registry
}

/// MOD-04: builds the same compiled route set used by the runtime for OpenAPI generation.
pub fn openapi_router() -> OpenApiRouter<AppState> {
    registry()
        .modules()
        .iter()
        .fold(OpenApiRouter::new(), |routes, module| {
            routes.merge(composed_module_routes(module.as_ref()))
        })
        .merge(crate::meta::public_routes())
}

fn composed_module_routes(module: &dyn Module) -> OpenApiRouter<AppState> {
    if module.manifest().id == ModuleId::Core {
        module
            .routes()
            .merge(crate::settings::routes())
            .merge(crate::settings::module_routes())
            .merge(crate::meta::module_routes())
    } else {
        module.routes()
    }
}

/// MOD-04/05: wraps every compiled module route in its tenant-scoped runtime gate.
pub(super) fn routes(
    registry: Arc<ModuleRegistry>,
    enabled: Arc<dyn EnabledModuleSet>,
    login_runtime: Arc<LoginHttpState>,
    fallback_currency_lock: Arc<dyn CurrencyLockCheck>,
) -> axum::Router<AppState> {
    let mut ports = PortRegistry::default();
    for module in registry.modules() {
        module.register_ports(&mut ports);
    }
    let currency_lock = ports
        .get::<dyn CurrencyLockCheck>()
        .unwrap_or(fallback_currency_lock);
    let metadata = crate::meta::MetadataState::new(
        Arc::clone(&registry),
        Arc::clone(&enabled),
        Arc::clone(&login_runtime),
    );
    let mut routes = OpenApiRouter::<AppState>::new();
    for module in &registry.modules {
        let gate = ModuleGate::new(module.manifest().id, Arc::clone(&enabled));
        let actor_auth = axum::middleware::from_fn_with_state(
            Arc::clone(&login_runtime),
            eq_identity::api_auth::actor_auth_middleware,
        );
        let module_routes = composed_module_routes(module.as_ref());
        let module_routes = if module.manifest().id == ModuleId::Core {
            module_routes
                .layer(axum::Extension((
                    Arc::clone(&login_runtime),
                    Arc::clone(&currency_lock),
                )))
                .layer(axum::Extension(metadata.clone()))
        } else {
            module_routes
        }
        .layer(actor_auth)
        .layer(gate);
        let module_routes = if module.manifest().id == ModuleId::Identity {
            module_routes.layer(axum::Extension(Arc::clone(&login_runtime)))
        } else {
            module_routes
        };
        let module_routes = if module.manifest().id == ModuleId::Audit {
            let policy_provider: Arc<dyn eq_audit::AuditPolicyProvider> =
                Arc::new(RuntimeAuditPolicyProvider {
                    runtime: Arc::clone(&login_runtime),
                });
            module_routes.layer(axum::Extension(policy_provider))
        } else {
            module_routes
        };
        routes = routes.merge(module_routes);
    }
    routes = routes.merge(crate::meta::public_routes().layer(axum::Extension(metadata)));
    let (routes, _openapi) = routes.split_for_parts();
    routes
}

struct RuntimeAuditPolicyProvider {
    runtime: Arc<LoginHttpState>,
}

#[async_trait]
impl eq_audit::AuditPolicyProvider for RuntimeAuditPolicyProvider {
    async fn admin_categories(
        &self,
        tx: &mut eq_core::db::Tx,
        tenant: eq_core::ids::TenantId,
    ) -> Result<Vec<eq_audit::Category>, eq_core::error::AppError> {
        let policy = self
            .runtime
            .policy_store()
            .await?
            .current::<eq_audit::policy::AuditPolicy>(tx, tenant)
            .await?;
        Ok(policy.doc.admin_categories().to_vec())
    }
}
