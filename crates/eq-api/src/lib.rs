//! Router assembly for the Ritma API (spec/02 §2, ARC-01).

use eq_core::clock::Clock;
use eq_core::config::Config;
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

/// ARC-01, ARC-04: shared infrastructure passed to API handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub clock: Arc<dyn Clock>,
}

impl fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("pool", &"[configured]")
            .field("config", &"[REDACTED]")
            .field("clock", &self.clock)
            .finish()
    }
}

/// API-14, OPS-06: builds the process health and readiness routes.
pub fn router(state: AppState) -> axum::Router {
    health::router(state)
}
