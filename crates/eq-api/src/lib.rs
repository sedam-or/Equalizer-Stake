//! Router assembly for the Ritma API (spec/02 §2, ARC-01).

use eq_core::clock::Clock;
use eq_core::config::Config;
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;

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
