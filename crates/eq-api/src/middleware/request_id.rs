//! Server-generated request correlation (ARC-04, CAN-30).

use axum::Router;
use axum::body::Body;
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::{Request, Response};
use axum::middleware::{self, Next};
use eq_core::error::REQUEST_ID;
use uuid::Uuid;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// ARC-04, CAN-30: wraps a router with a server-generated UUIDv7 request id.
pub fn apply(router: Router) -> Router {
    router.layer(middleware::from_fn(assign_request_id))
}

async fn assign_request_id(mut request: Request<Body>, next: Next) -> Response<Body> {
    let request_id = Uuid::now_v7();
    request.extensions_mut().insert(request_id);

    let mut response = REQUEST_ID.scope(request_id, next.run(request)).await;
    if let Ok(value) = HeaderValue::from_str(&request_id.to_string()) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }
    response
}
