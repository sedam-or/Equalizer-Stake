//! API-02/ERR-21: compiled OpenAPI contract for the API and its error registry.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use eq_core::error::ErrorCodeDef;
use eq_core::fields::{
    Address, BirthDate, BpsField, Comment, CountryField, CurrencyField, DisplayName, Email, FieldKind,
    GeoField, Hue, I18nLine, IdField, IdList, InstantField, Key, LegalName, LocalDateField, LocalTimeField,
    LocaleField, Minutes, MoneyAmount, Note, Password, Phone, PlaceName, SearchQuery, ShortText, Slug, Stars,
    TenantName, TimeZoneField,
};
use serde_json::{Value, json};
use utoipa::openapi::extensions::Extensions;
use utoipa::openapi::path::{Operation, PathItem};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::{Content, Info, License, OpenApi, OpenApiVersion, Ref, Response, Server};
use utoipa::{PartialSchema, ToSchema};

const API_PREFIX: &str = "/api/v1";

/// ERR-30: RFC 9457 response with the registered Ritma extensions.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Problem {
    /// Problem type URI derived from the stable code (ERR-20).
    #[serde(rename = "type")]
    pub type_uri: String,
    /// Fixed English title from the error registry.
    pub title: String,
    /// HTTP response status.
    pub status: u16,
    /// Stable registered error code.
    pub code: String,
    /// Safe human-readable or neutral developer detail.
    pub detail: String,
    /// Request identifier when the request-id layer supplied one.
    pub request_id: Option<uuid::Uuid>,
    /// ERR-31 validation details.
    pub errors: Option<Vec<FieldProblem>>,
    /// Scheduling conflicts, with the per-code shape owned by its module contract.
    pub conflicts: Option<Vec<Value>>,
    /// Scheduling alternatives, with the per-code shape owned by its module contract.
    pub alternatives: Option<Vec<Value>>,
    /// Required permission key for a forbidden response.
    pub perm: Option<String>,
    /// Module identifier for a module-state response.
    pub module: Option<String>,
    /// Policy key path for a policy response.
    pub rule: Option<String>,
    /// UTC retry or lock boundary.
    pub until: Option<DateTime<Utc>>,
    /// Missing module requirements, keyed by module id (MOD-03).
    pub missing: Option<BTreeMap<String, Vec<String>>>,
}

/// ERR-31: one validation entry in a Problem response.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct FieldProblem {
    /// JSON Pointer into the request body.
    pub pointer: String,
    /// Registered field error code.
    pub code: String,
    /// Safe typed translation parameters.
    pub params: BTreeMap<String, Value>,
}

/// Builds the OpenAPI 3.1 document from routes registered by compiled modules.
pub fn build() -> OpenApi {
    let mut api = route_document();
    api.openapi = OpenApiVersion::Version31;
    api.info = Info::new(
        format!("{} API", eq_core::brand::NAME),
        env!("CARGO_PKG_VERSION").to_owned(),
    );
    api.info.license = Some(License::new("AGPL-3.0-only"));
    api.servers = Some(vec![Server::new(API_PREFIX)]);

    normalize_paths(&mut api);
    set_root_server_for_health_routes(&mut api);
    add_error_responses(&mut api);
    add_components(&mut api);
    add_error_code_extension(&mut api);
    api
}

/// Serializes the contract with recursively sorted object keys for stable diffs.
pub fn pretty_json(api: &OpenApi) -> Result<String, serde_json::Error> {
    let value = sort_json(serde_json::to_value(api)?);
    let ordered: BTreeMap<String, Value> = serde_json::from_value(value)?;
    serde_json::to_string_pretty(&ordered)
}

/// Returns the operation set built by the shared runtime route declarations.
pub fn registered_route_operations() -> BTreeSet<(String, String)> {
    operation_set(&route_document())
}

fn route_document() -> OpenApi {
    let (_, mut api) = crate::modules::openapi_router().split_for_parts();
    let (_, health_api) = crate::health::openapi_routes().split_for_parts();
    api.merge(health_api);
    api
}

fn normalize_paths(api: &mut OpenApi) {
    let paths = std::mem::take(&mut api.paths.paths);
    api.paths.paths = paths
        .into_iter()
        .map(|(path, item)| (normalize_path(&path), item))
        .collect();
}

fn normalize_path(path: &str) -> String {
    match path.strip_prefix(API_PREFIX) {
        Some("") => String::from("/"),
        Some(suffix) => suffix.to_owned(),
        None => path.to_owned(),
    }
}

fn set_root_server_for_health_routes(api: &mut OpenApi) {
    for path in ["/healthz", "/readyz"] {
        if let Some(item) = api.paths.paths.get_mut(path) {
            for operation in operations_mut(item) {
                operation.servers = Some(vec![Server::new("/")]);
            }
        }
    }
}

fn add_error_responses(api: &mut OpenApi) {
    for item in api.paths.paths.values_mut() {
        for operation in operations_mut(item) {
            let mut response = Response::new("RFC 9457 application problem response");
            response.content.insert(
                String::from("application/problem+json"),
                Content::new(Some(Ref::from_schema_name("Problem"))).into(),
            );
            operation
                .responses
                .responses
                .entry(String::from("default"))
                .or_insert(response.into());
        }
    }
}

fn add_components(api: &mut OpenApi) {
    let components = api.components.get_or_insert_with(Default::default);
    components.add_security_scheme(
        "bearer",
        SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        ),
    );
    components.add_security_scheme(
        "refreshCookie",
        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("eq_rt"))),
    );
    components.schemas.insert("Problem".to_owned(), Problem::schema());
    components
        .schemas
        .insert("FieldProblem".to_owned(), FieldProblem::schema());

    macro_rules! register_field_kinds {
        ($($kind:ty),+ $(,)?) => {
            $(
                components.schemas.insert(
                    <$kind as FieldKind>::component_name().into_owned(),
                    <$kind as FieldKind>::schema().into(),
                );
            )+
        };
    }

    register_field_kinds!(
        DisplayName,
        LegalName,
        TenantName,
        PlaceName,
        ShortText,
        Address,
        Note,
        Comment,
        I18nLine<200>,
        Email,
        Phone,
        Password,
        Key,
        Slug,
        SearchQuery,
        IdField,
        LocalDateField,
        BirthDate,
        LocalTimeField,
        InstantField,
        Minutes<10, 480>,
        MoneyAmount,
        CurrencyField,
        BpsField,
        CountryField,
        TimeZoneField,
        LocaleField,
        GeoField,
        Hue,
        Stars,
        IdList<100>,
    );
}

fn add_error_code_extension(api: &mut OpenApi) {
    let codes = registered_error_codes()
        .into_iter()
        .map(|definition| {
            json!({
                "code": definition.code,
                "status": definition.status,
                "title": definition.title,
            })
        })
        .collect::<Vec<_>>();
    api.extensions = Some(Extensions::from_iter([("x-error-codes", json!(codes))]));
}

fn registered_error_codes() -> Vec<&'static ErrorCodeDef> {
    let mut codes = Vec::new();
    codes.extend_from_slice(eq_core::errors::CODES);
    codes.extend_from_slice(eq_identity::errors::CODES);
    codes.extend_from_slice(eq_audit::errors::CODES);
    codes.extend_from_slice(eq_scheduling::errors::CODES);

    #[cfg(feature = "enrollment")]
    codes.extend_from_slice(eq_enrollment::errors::CODES);
    #[cfg(feature = "billing")]
    codes.extend_from_slice(eq_billing::errors::CODES);
    #[cfg(feature = "attendance")]
    codes.extend_from_slice(eq_attendance::errors::CODES);
    #[cfg(feature = "rental")]
    codes.extend_from_slice(eq_rental::errors::CODES);
    #[cfg(feature = "notify")]
    codes.extend_from_slice(eq_notify::errors::CODES);
    #[cfg(feature = "reports")]
    codes.extend_from_slice(eq_reports::errors::CODES);
    #[cfg(feature = "feedback")]
    codes.extend_from_slice(eq_feedback::errors::CODES);

    codes.sort_by_key(|definition| definition.code);
    codes
}

fn operation_set(api: &OpenApi) -> BTreeSet<(String, String)> {
    api.paths
        .paths
        .iter()
        .flat_map(|(path, item)| {
            [
                ("get", item.get.as_ref()),
                ("put", item.put.as_ref()),
                ("post", item.post.as_ref()),
                ("delete", item.delete.as_ref()),
                ("options", item.options.as_ref()),
                ("head", item.head.as_ref()),
                ("patch", item.patch.as_ref()),
                ("trace", item.trace.as_ref()),
            ]
            .into_iter()
            .filter_map(move |(method, operation)| {
                operation.map(|_| (normalize_path(path), method.to_owned()))
            })
        })
        .collect()
}

fn operations_mut(item: &mut PathItem) -> Vec<&mut Operation> {
    [
        item.get.as_mut(),
        item.put.as_mut(),
        item.post.as_mut(),
        item.delete.as_mut(),
        item.options.as_mut(),
        item.head.as_mut(),
        item.patch.as_mut(),
        item.trace.as_mut(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn sort_json(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted = object
                .into_iter()
                .map(|(key, value)| (key, sort_json(value)))
                .collect::<BTreeMap<_, _>>();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_json).collect()),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let first = pretty_json(&build()).expect("OpenAPI serializes");
        let second = pretty_json(&build()).expect("OpenAPI serializes");
        assert_eq!(first, second);
    }
}
