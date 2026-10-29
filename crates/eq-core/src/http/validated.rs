//! VAL-40: bounded request parsing and stable rejection mapping.

use std::collections::BTreeMap;

use axum::{
    body::to_bytes,
    extract::{FromRequest, FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
};
use serde::de::{DeserializeOwned, DeserializeSeed, Error as DeError, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use serde_path_to_error::{Segment as SerdePathSegment, deserialize as deserialize_with_path};

use crate::{
    error::{AppError, FieldError, codes},
    errors::{INVALID_TYPE, NOT_ALLOWED, REQUIRED},
    limits::{JSON_BODY_MAX_BYTES, JSON_MAX_DEPTH},
};

/// JSON request body after syntax, duplicate-key, depth and shape validation (VAL-40).
pub struct ValidatedJson<T>(pub T);

impl<T> std::fmt::Debug for ValidatedJson<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ValidatedJson([REDACTED])")
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request(request: Request, _state: &S) -> Result<Self, Self::Rejection> {
        if !is_json_content_type(request.headers()) {
            return Err(bad_request("Content-Type must be application/json with UTF-8"));
        }
        let body = to_bytes(request.into_body(), JSON_BODY_MAX_BYTES)
            .await
            .map_err(|_| AppError::PayloadTooLarge)?;
        let mut deserializer = serde_json::Deserializer::from_slice(&body);
        let value = JsonValueSeed { depth: 0 }
            .deserialize(&mut deserializer)
            .map_err(|_| bad_request("Invalid JSON request body"))?;
        deserializer
            .end()
            .map_err(|_| bad_request("Invalid JSON request body"))?;
        let value_deserializer = value.into_deserializer();
        match deserialize_with_path(value_deserializer) {
            Ok(value) => Ok(Self(value)),
            Err(error) => Err(shape_error(error.path(), error.inner().to_string())),
        }
    }
}

/// Typed query parameters; `T` must use `#[serde(deny_unknown_fields)]` (CAN-32).
/// Multi-value query fields use repeated parameters.
pub struct ValidatedQuery<T>(pub T);

impl<T> std::fmt::Debug for ValidatedQuery<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ValidatedQuery([REDACTED])")
    }
}

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        axum_extra::extract::Query::<T>::from_request_parts(parts, state)
            .await
            .map(|query| Self(query.0))
            .map_err(|_| bad_request("Invalid query parameters"))
    }
}

/// Parses a canonical path UUID; malformed identifiers map to 404 (CAN-33).
pub fn parse_path_id(raw: &str) -> Result<uuid::Uuid, AppError> {
    let bytes = raw.as_bytes();
    let canonical_shape = bytes.len() == 36
        && [8, 13, 18, 23].iter().all(|position| bytes[*position] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit());
    if !canonical_shape {
        return Err(AppError::NotFound { entity: "record" });
    }
    uuid::Uuid::parse_str(raw).map_err(|_| AppError::NotFound { entity: "record" })
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    let Some(value) = headers.get(axum::http::header::CONTENT_TYPE) else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };
    let mut parts = value.split(';');
    if !parts
        .next()
        .is_some_and(|media_type| media_type.trim().eq_ignore_ascii_case("application/json"))
    {
        return false;
    }
    parts.all(|part| {
        let Some((name, value)) = part.trim().split_once('=') else {
            return false;
        };
        name.trim().eq_ignore_ascii_case("charset")
            && value.trim().trim_matches('"').eq_ignore_ascii_case("utf-8")
    })
}

fn bad_request(detail: &str) -> AppError {
    AppError::BadRequest {
        code: &codes::BAD_REQUEST,
        detail: detail.to_owned(),
    }
}

fn shape_error(path: &serde_path_to_error::Path, message: String) -> AppError {
    let (code, pointer, params) = if message.contains("unknown variant") {
        let mut params = serde_json::Map::new();
        if let Some(allowed) = allowed_variants(&message) {
            params.insert(
                "allowed".to_owned(),
                Value::Array(allowed.into_iter().map(Value::String).collect()),
            );
        }
        (NOT_ALLOWED, pointer_from_path(path), params)
    } else if message.contains("unknown field") {
        (
            NOT_ALLOWED,
            pointer_for_shape(path, &message),
            serde_json::Map::new(),
        )
    } else if let Some(field) = quoted_field(&message, "missing field") {
        (REQUIRED, pointer_with_field(path, &field), serde_json::Map::new())
    } else {
        let mut params = serde_json::Map::new();
        params.insert(
            "expected".to_owned(),
            Value::String("valid request shape".to_owned()),
        );
        (INVALID_TYPE, pointer_from_path(path), params)
    };
    AppError::Validation {
        errors: vec![FieldError {
            pointer,
            code: code.to_owned(),
            params,
        }],
    }
}

fn allowed_variants(message: &str) -> Option<Vec<String>> {
    let mut remaining = message.split_once("expected")?.1;
    let mut allowed = Vec::new();
    while let Some((_, tail)) = remaining.split_once('`') {
        let (variant, rest) = tail.split_once('`')?;
        allowed.push(variant.to_owned());
        remaining = rest;
    }
    (!allowed.is_empty()).then_some(allowed)
}

fn pointer_for_shape(path: &serde_path_to_error::Path, message: &str) -> String {
    quoted_field(message, "unknown field")
        .map(|field| {
            let segments = pointer_segments(path);
            if segments.last().is_some_and(|segment| segment == &field) {
                pointer_from_segments(segments)
            } else {
                pointer_with_field(path, &field)
            }
        })
        .unwrap_or_else(|| pointer_from_path(path))
}

fn quoted_field(message: &str, marker: &str) -> Option<String> {
    let remainder = message.split_once(marker)?.1.trim_start();
    let quote = remainder.chars().next()?;
    if quote != '\x60' && quote != '\'' {
        return None;
    }
    let value = remainder[quote.len_utf8()..].split_once(quote)?.0;
    Some(value.to_owned())
}

fn pointer_from_path(path: &serde_path_to_error::Path) -> String {
    pointer_from_segments(pointer_segments(path))
}

fn pointer_with_field(path: &serde_path_to_error::Path, field: &str) -> String {
    pointer_from_segments(pointer_segments(path).into_iter().chain([field.to_owned()]))
}

fn pointer_segments(path: &serde_path_to_error::Path) -> Vec<String> {
    path.iter()
        .filter_map(|segment| match segment {
            SerdePathSegment::Seq { index } => Some(index.to_string()),
            SerdePathSegment::Map { key } => Some(key.clone()),
            SerdePathSegment::Enum { variant } => Some(variant.clone()),
            SerdePathSegment::Unknown => None,
        })
        .collect()
}

fn pointer_from_segments(segments: impl IntoIterator<Item = String>) -> String {
    let segments = segments
        .into_iter()
        .map(|segment| escape_pointer(&segment))
        .collect::<Vec<_>>();
    if segments.is_empty() {
        String::new()
    } else {
        format!("/{}", segments.join("/"))
    }
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

struct JsonValueSeed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for JsonValueSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonValueVisitor { depth: self.depth })
    }
}

struct JsonValueVisitor {
    depth: usize,
}

impl<'de> Visitor<'de> for JsonValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate keys or excessive depth")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let depth = self.depth + 1;
        if depth > JSON_MAX_DEPTH {
            return Err(A::Error::custom("JSON nesting exceeds configured limit"));
        }
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(JsonValueSeed { depth })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let depth = self.depth + 1;
        if depth > JSON_MAX_DEPTH {
            return Err(A::Error::custom("JSON nesting exceeds configured limit"));
        }
        let mut values = BTreeMap::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(A::Error::custom("duplicate JSON object key"));
            }
            let value = object.next_value_seed(JsonValueSeed { depth })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values.into_iter().collect()))
    }
}

use serde::de::IntoDeserializer;

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        extract::Path,
        http::{Request, StatusCode, header::CONTENT_TYPE},
        routing::{get, post},
    };
    use serde::Deserialize;
    use tower::ServiceExt;

    use super::*;
    use crate::{
        fields::{Collector, DisplayName, FieldCtx, Raw},
        money::Currency,
        policy::types::Locale,
    };

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BodyDto {
        name: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum RequestStatus {
        Active,
        Inactive,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct EnumBodyDto {
        status: RequestStatus,
    }

    async fn enum_body_route(ValidatedJson(body): ValidatedJson<EnumBodyDto>) -> &'static str {
        match body.status {
            RequestStatus::Active | RequestStatus::Inactive => "ok",
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct NestedBodyDto {
        profile: BodyDto,
        items: Vec<BodyDto>,
    }

    async fn body_route(ValidatedJson(body): ValidatedJson<BodyDto>) -> &'static str {
        let _ = body.name;
        "ok"
    }

    async fn nested_body_route(ValidatedJson(body): ValidatedJson<NestedBodyDto>) -> &'static str {
        let _ = (body.profile.name, body.items.len());
        "ok"
    }

    async fn value_route(ValidatedJson(_body): ValidatedJson<Value>) -> &'static str {
        "ok"
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct InvalidFieldsDto {
        first: Raw<DisplayName>,
        second: Raw<DisplayName>,
        third: Raw<DisplayName>,
    }

    async fn collect_route(
        ValidatedJson(dto): ValidatedJson<InvalidFieldsDto>,
    ) -> Result<&'static str, AppError> {
        let context = test_context();
        let mut collector = Collector::new(&context);
        let _first = collector.field("/first", dto.first);
        let _second = collector.field("/second", dto.second);
        let _third = collector.field("/third", dto.third);
        collector.finish()?;
        Ok("ok")
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct QueryDto {
        limit: Option<String>,
        status: Vec<String>,
    }

    async fn query_route(ValidatedQuery(query): ValidatedQuery<QueryDto>) -> &'static str {
        let _ = (query.limit, query.status);
        "ok"
    }

    async fn path_route(Path(id): Path<String>) -> Result<&'static str, AppError> {
        parse_path_id(&id)?;
        Ok("ok")
    }

    fn test_context() -> FieldCtx {
        FieldCtx {
            country: Some(celes::Country::from_alpha2("ID").expect("known test country")),
            tenant_currency: Currency::IDR,
            tenant_locale: Locale::Id,
            today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
            password_min_length: Some(8),
        }
    }

    fn json_request(uri: &str, body: &'static str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .expect("valid request")
    }

    #[tokio::test]
    async fn rejects_unknown_field() {
        let app = Router::new().route("/", post(body_route));
        let response = app
            .oneshot(json_request("/", r#"{"name":"test","tenant_id":"fake"}"#))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        assert_eq!(value["errors"][0]["pointer"], "/tenant_id");
        assert_eq!(value["errors"][0]["code"], "not_allowed");
    }

    #[tokio::test]
    async fn unknown_enum_variant_is_not_allowed_with_allowed_values() {
        let app = Router::new().route("/", post(enum_body_route));
        let response = app
            .oneshot(json_request("/", r#"{"status":"pending"}"#))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        assert_eq!(value["errors"][0]["pointer"], "/status");
        assert_eq!(value["errors"][0]["code"], "not_allowed");
        assert_eq!(
            value["errors"][0]["params"]["allowed"],
            serde_json::json!(["active", "inactive"])
        );
    }

    #[tokio::test]
    async fn shape_errors_use_nested_object_and_array_pointers() {
        let app = Router::new().route("/", post(nested_body_route));
        let response = app
            .clone()
            .oneshot(json_request(
                "/",
                r#"{"profile":{"name":"test","bad/~key":true},"items":[]}"#,
            ))
            .await
            .expect("response");
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        assert_eq!(value["errors"][0]["pointer"], "/profile/bad~1~0key");

        let response = app
            .clone()
            .oneshot(json_request(
                "/",
                r#"{"profile":{"name":"test"},"items":[{"name":"test","extra":true}]}"#,
            ))
            .await
            .expect("response");
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        assert_eq!(value["errors"][0]["pointer"], "/items/0/extra");

        let response = app
            .clone()
            .oneshot(json_request("/", r#"{"profile":{},"items":[]}"#))
            .await
            .expect("response");
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        assert_eq!(value["errors"][0]["pointer"], "/profile/name");

        let response = app
            .oneshot(json_request(
                "/",
                r#"{"profile":{"name":"test"},"items":[{"name":1}]}"#,
            ))
            .await
            .expect("response");
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        assert_eq!(value["errors"][0]["pointer"], "/items/0/name");
    }

    #[tokio::test]
    async fn rejects_duplicate_keys() {
        let app = Router::new().route("/", post(value_route));
        let response = app
            .oneshot(json_request("/", r#"{"a":1,"a":2}"#))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_deep_json() {
        let nested = format!("{}0{}", "[".repeat(33), "]".repeat(33));
        let app = Router::new().route("/", post(value_route));
        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(nested))
            .expect("valid request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn collects_all_field_errors() {
        let app = Router::new().route("/", post(collect_route));
        let response = app
            .oneshot(json_request(
                "/",
                r#"{"first":"","second":"bad\u202evalue","third":"   "}"#,
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let value: Value = serde_json::from_slice(&body).expect("problem JSON");
        let pointers = value["errors"]
            .as_array()
            .expect("field errors")
            .iter()
            .map(|error| error["pointer"].as_str().expect("pointer"))
            .collect::<Vec<_>>();
        assert_eq!(pointers, ["/first", "/second", "/third"]);
    }

    #[tokio::test]
    async fn wrong_content_type_is_400() {
        let app = Router::new().route("/", post(body_route));
        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from("{}"))
            .expect("valid request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn bad_path_id_is_404() {
        let app = Router::new().route("/items/{id}", get(path_route));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/not-an-id")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_query_param_is_400() {
        let app = Router::new().route("/", get(query_route));
        let request = Request::builder()
            .uri("/?limit=10&status=open&unexpected=true")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
