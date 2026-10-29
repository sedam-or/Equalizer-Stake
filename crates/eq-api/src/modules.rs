//! Feature-gated module registry and route assembly (MOD-04/05).

use std::sync::Arc;

use eq_core::app_state::AppState;
use eq_core::module::gate::{EnabledModuleSet, ModuleGate};
use eq_core::module::{Module, ModuleId, ModuleRegistry};
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

/// MOD-04/05: wraps every compiled module route in its tenant-scoped runtime gate.
pub(super) fn routes(
    registry: &ModuleRegistry,
    enabled: Arc<dyn EnabledModuleSet>,
    login_runtime: Arc<LoginHttpState>,
) -> axum::Router<AppState> {
    let mut routes = OpenApiRouter::<AppState>::new();
    for module in &registry.modules {
        let gate = ModuleGate::new(module.manifest().id, Arc::clone(&enabled));
        let module_routes = module.routes().layer(gate);
        let module_routes = if module.manifest().id == ModuleId::Identity {
            module_routes.layer(axum::Extension(Arc::clone(&login_runtime)))
        } else {
            module_routes
        };
        routes = routes.merge(module_routes);
    }
    let (routes, _openapi) = routes.split_for_parts();
    routes
}
