//! Router assembly for the Ritma API (spec/02 §2, ARC-01).

pub use eq_core::app_state::AppState;

#[cfg(feature = "attendance")]
use eq_attendance as _;
use eq_audit as _;
#[cfg(feature = "billing")]
use eq_billing as _;
#[cfg(feature = "enrollment")]
use eq_enrollment as _;
#[cfg(feature = "feedback")]
use eq_feedback as _;
use eq_identity as _;
#[cfg(feature = "notify")]
use eq_notify as _;
#[cfg(feature = "rental")]
use eq_rental as _;
#[cfg(feature = "reports")]
use eq_reports as _;
use eq_scheduling as _;

mod health;
pub mod meta;
pub mod middleware;
pub mod modules;
pub mod settings;
mod spa;
pub mod tenant;

/// API-14, OPS-06: builds the process health and readiness routes.
pub fn router(state: AppState) -> axum::Router {
    let registry = std::sync::Arc::new(modules::registry());
    let manifests = registry.manifests();
    let enabled: std::sync::Arc<dyn eq_core::module::gate::EnabledModuleSet> = std::sync::Arc::new(
        eq_core::module::gate::EnabledModulesCache::new(state.pool.clone(), &manifests),
    );
    let login_runtime = std::sync::Arc::new(eq_identity::api_auth::LoginHttpState::new(
        state.pool.clone(),
        std::sync::Arc::clone(&state.clock),
        std::sync::Arc::clone(&registry),
        std::sync::Arc::clone(&enabled),
        std::sync::Arc::clone(&state.auth),
        state.tenant_resolver.clone(),
    ));
    let module_routes = modules::routes(
        std::sync::Arc::clone(&registry),
        enabled,
        login_runtime,
        state.currency_lock_check.clone(),
    );
    health::router(state, module_routes)
}
