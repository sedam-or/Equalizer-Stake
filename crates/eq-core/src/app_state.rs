//! Shared API state used by module routes (ARC-01, ARC-04).

use crate::auth::AuthState;
use crate::clock::Clock;
use crate::config::Config;
use crate::tenant::TenantResolver;
use axum::extract::FromRef;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;

/// ARC-01, ARC-04: shared infrastructure passed to API handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub clock: Arc<dyn Clock>,
    pub tenant_resolver: TenantResolver,
    pub auth: Arc<AuthState>,
}

impl AppState {
    /// ARC-01, ARC-04: builds shared application state and its bounded tenant resolver.
    pub fn new(pool: PgPool, config: Arc<Config>, clock: Arc<dyn Clock>) -> Self {
        let tenant_resolver = TenantResolver::new(pool.clone(), Arc::clone(&config));
        let previous = config.secret_key_previous.as_ref().map(|key| key.expose_secret());
        let auth = Arc::new(AuthState::new(config.secret_key.expose_secret(), previous));
        Self {
            pool,
            config,
            clock,
            tenant_resolver,
            auth,
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
            .field("auth", &self.auth)
            .finish()
    }
}
