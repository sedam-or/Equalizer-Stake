//! Liveness and readiness endpoints (OPS-06, API-14).

use crate::{AppState, middleware::request_id};
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode};
use axum::middleware::{self, Next};
use axum::routing::get;
use axum::{Router, response::IntoResponse};
use serde::Serialize;
use sqlx::migrate::Migrator;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;
use uuid::Uuid;

const BODY_LIMIT_BYTES: usize = 1024 * 1024; // technical: ARC-04 default request body cap
const DB_PING_TIMEOUT: Duration = Duration::from_millis(500); // technical: OPS-06 readiness bound
const MIGRATION_CHECK_TIMEOUT: Duration = Duration::from_millis(500); // technical: bound readiness migration reads
const MIGRATION_CACHE_TTL: Duration = Duration::from_secs(10); // technical: task cache interval
const MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[derive(Clone)]
struct HealthState {
    app: AppState,
    migration_cache: Arc<Mutex<Option<CachedMigrationState>>>,
}

#[derive(Clone, Copy)]
struct CachedMigrationState {
    checked_at: Instant,
    pending: bool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct ReadyResponse {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

/// API-14, OPS-06: process liveness and database readiness routes.
pub(super) fn router(state: AppState, module_routes: Router<AppState>) -> Router {
    let csrf_origin = state.config.public_url.origin().ascii_serialization();
    let tenant_resolver = state.tenant_resolver.clone();
    let health_state = HealthState {
        app: state,
        migration_cache: Arc::new(Mutex::new(None)),
    };

    let module_routes = module_routes.with_state(health_state.app.clone());
    let router = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(module_routes)
        .fallback(crate::spa::fallback)
        .with_state(health_state)
        .layer(CatchPanicLayer::custom(handle_panic))
        .layer(middleware::from_fn_with_state(
            tenant_resolver,
            crate::tenant::tenant_guard,
        ))
        .layer(middleware::from_fn_with_state(
            csrf_origin,
            eq_identity::api_auth::role_switch_csrf_guard,
        ))
        .layer(RequestBodyLimitLayer::new(BODY_LIMIT_BYTES))
        .layer(middleware::from_fn(security_headers_stub))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = request
                        .extensions()
                        .get::<Uuid>()
                        .map_or_else(|| "unknown".to_owned(), Uuid::to_string);
                    let path = request
                        .extensions()
                        .get::<axum::extract::MatchedPath>()
                        .map_or_else(|| request.uri().path(), axum::extract::MatchedPath::as_str);
                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        tenant_id = tracing::field::Empty,
                        actor_id = tracing::field::Empty,
                        method = %request.method(),
                        path = %path,
                        status = tracing::field::Empty,
                        latency_seconds = tracing::field::Empty,
                    )
                })
                .on_request(())
                .on_response(|response: &Response<Body>, latency: Duration, span: &Span| {
                    span.record("status", response.status().as_u16());
                    span.record("latency_seconds", latency.as_secs_f64());
                    tracing::info!(
                        parent: span,
                        status = response.status().as_u16(),
                        latency_seconds = latency.as_secs_f64(),
                        "request completed"
                    );
                })
                .on_failure(()),
        );

    request_id::apply(router)
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn readyz(State(state): State<HealthState>) -> impl IntoResponse {
    let db_ping =
        tokio::time::timeout(DB_PING_TIMEOUT, sqlx::query("SELECT 1").execute(&state.app.pool)).await;
    if !matches!(db_ping, Ok(Ok(_))) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse {
                status: "not_ready",
                reason: Some("db_unreachable"),
            }),
        );
    }

    if migrations_pending(&state).await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse {
                status: "not_ready",
                reason: Some("migrations_pending"),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(ReadyResponse {
            status: "ready",
            reason: None,
        }),
    )
}

async fn migrations_pending(state: &HealthState) -> bool {
    let mut cache = state.migration_cache.lock().await;
    if let Some(cached) = *cache
        && cached.checked_at.elapsed() < MIGRATION_CACHE_TTL
    {
        return cached.pending;
    }

    let applied = tokio::time::timeout(
        MIGRATION_CHECK_TIMEOUT,
        sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_all(&state.app.pool),
    )
    .await;
    let pending = match applied {
        Ok(Ok(versions)) => MIGRATOR
            .migrations
            .iter()
            .any(|migration| !versions.contains(&migration.version)),
        Ok(Err(_)) | Err(_) => true,
    };

    *cache = Some(CachedMigrationState {
        checked_at: Instant::now(),
        pending,
    });
    pending
}

fn handle_panic(_panic: Box<dyn std::any::Any + Send + 'static>) -> Response<Body> {
    eq_core::error::AppError::Internal(anyhow::anyhow!("request handler panicked")).into_response()
}

async fn security_headers_stub(request: Request<Body>, next: Next) -> Response<Body> {
    // TODO(T-115): install the SEC-20 response header policy.
    next.run(request).await
}
