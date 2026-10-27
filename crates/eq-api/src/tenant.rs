//! Request tenant gates for the API edge (ARC-04, SEC-05).

use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request, Response};
use axum::middleware::Next;
use axum::response::IntoResponse;
use eq_core::error::AppError;
use eq_core::model::TenantStatus;
use eq_core::tenant::{TenantCtx, TenantResolver};

/// ARC-04: resolves tenant context before tenant-facing routes and enforces setup/read-only gates.
pub async fn tenant_guard(
    State(resolver): State<TenantResolver>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let path = String::from(request.uri().path());
    if matches!(path.as_str(), "/healthz" | "/readyz") {
        return next.run(request).await;
    }

    let (mut parts, body) = request.into_parts();
    let context = match resolver.resolve(&parts).await {
        Ok(context) => context,
        Err(error) => return error.into_response(),
    };

    match &context {
        TenantCtx::Resolved(info) => {
            if info.status == TenantStatus::Suspended && !matches!(parts.method, Method::GET | Method::HEAD) {
                return AppError::Locked { until: None }.into_response();
            }
        }
        TenantCtx::Unprovisioned if !setup_route(&path) => {
            return AppError::SetupRequired.into_response();
        }
        TenantCtx::Unprovisioned => {}
    }

    parts.extensions.insert(context);
    next.run(Request::from_parts(parts, body)).await
}

fn setup_route(path: &str) -> bool {
    path == "/api/v1/meta/app" || path.starts_with("/api/v1/setup/")
}

#[cfg(test)]
mod tests {
    use super::setup_route;

    #[test]
    fn setup_route_only_allows_the_declared_paths() {
        assert!(setup_route("/api/v1/meta/app"));
        assert!(setup_route("/api/v1/setup/status"));
        assert!(!setup_route("/api/v1/setup"));
        assert!(!setup_route("/api/v1/people"));
        assert!(!setup_route("/"));
    }
}
