//! Router assembly for the Ritma API (spec/02 §2, ARC-01).

use axum::extract::FromRef;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::tenant::TenantResolver;
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;

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
pub mod middleware;
mod spa;
pub mod tenant;

/// ARC-01, ARC-04: shared infrastructure passed to API handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub clock: Arc<dyn Clock>,
    pub tenant_resolver: TenantResolver,
}

impl AppState {
    /// ARC-01, ARC-04: builds shared application state and its bounded tenant resolver.
    pub fn new(pool: PgPool, config: Arc<Config>, clock: Arc<dyn Clock>) -> Self {
        let tenant_resolver = TenantResolver::new(pool.clone(), Arc::clone(&config));
        Self {
            pool,
            config,
            clock,
            tenant_resolver,
        }
    }
}

impl FromRef<AppState> for TenantResolver {
    fn from_ref(state: &AppState) -> Self {
        state.tenant_resolver.clone()
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("pool", &"[configured]")
            .field("config", &"[REDACTED]")
            .field("clock", &self.clock)
            .field("tenant_resolver", &self.tenant_resolver)
            .finish()
    }
}

/// API-14, OPS-06: builds the process health and readiness routes.
pub fn router(state: AppState) -> axum::Router {
    health::router(state)
}
