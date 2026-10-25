//! Embedded SPA asset delivery (FE-01, CACHE-20/21).

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode};
use axum::response::IntoResponse;
use rust_embed::{EmbeddedFile, RustEmbed};

const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";
const NO_CACHE: &str = "no-cache";

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/dist"]
struct Assets;

/// FE-01: serves embedded files, preserves API errors, and falls back to the SPA shell.
pub(crate) async fn fallback(request: Request<Body>) -> Response<Body> {
    let path = request.uri().path();

    if path == "/api" || path.starts_with("/api/") {
        return eq_core::error::AppError::NotFound { entity: "route" }.into_response();
    }

    if request.method() != Method::GET {
        return eq_core::error::AppError::NotFound { entity: "route" }.into_response();
    }

    let asset_path = path.trim_start_matches('/');
    if let Some(asset) = Assets::get(asset_path) {
        return asset_response(asset, asset_path, request.headers());
    }

    if path.starts_with("/assets/") {
        return eq_core::error::AppError::NotFound { entity: "asset" }.into_response();
    }

    let Some(index) = Assets::get("index.html") else {
        return eq_core::error::AppError::Internal(anyhow::anyhow!("embedded SPA index.html is missing"))
            .into_response();
    };

    asset_response(index, "index.html", request.headers())
}

fn asset_response(asset: EmbeddedFile, asset_path: &str, request_headers: &HeaderMap) -> Response<Body> {
    let is_asset = asset_path.starts_with("assets/");
    let cache_control = if is_asset { IMMUTABLE_CACHE } else { NO_CACHE };
    let has_validator = matches!(asset_path, "index.html" | "sw.js" | "manifest.webmanifest");
    let etag = format!("\"{}\"", sha256_hex(&asset));

    if has_validator && if_none_match_matches(request_headers, &etag) {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::NOT_MODIFIED;
        add_header(&mut response, CACHE_CONTROL, cache_control);
        add_header_string(&mut response, ETAG, &etag);
        return response;
    }

    let content_type = mime_guess::from_path(asset_path)
        .first_or_octet_stream()
        .essence_str()
        .to_owned();
    let mut response = Response::new(Body::from(asset.data.into_owned()));
    add_header(&mut response, CACHE_CONTROL, cache_control);
    add_header_string(&mut response, CONTENT_TYPE, &content_type);
    if has_validator {
        add_header_string(&mut response, ETAG, &etag);
    }
    response
}

fn if_none_match_matches(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value.split(',').any(|candidate| {
                let candidate = candidate.trim();
                candidate == "*" || candidate.strip_prefix("W/").unwrap_or(candidate) == etag
            })
        })
}

fn sha256_hex(asset: &EmbeddedFile) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut digest = String::with_capacity(64);
    for byte in asset.metadata.sha256_hash() {
        digest.push(char::from(HEX[usize::from(byte >> 4)]));
        digest.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    digest
}

fn add_header(response: &mut Response<Body>, name: axum::http::header::HeaderName, value: &'static str) {
    response
        .headers_mut()
        .insert(name, HeaderValue::from_static(value));
}

fn add_header_string(response: &mut Response<Body>, name: axum::http::header::HeaderName, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        response.headers_mut().insert(name, value);
    }
}
