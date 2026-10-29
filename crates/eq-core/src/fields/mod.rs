//! VAL-20: canonical request field kinds and shared error collection.

mod collector;
mod kinds;
mod schema;

pub use collector::Collector;
pub use kinds::*;
pub use schema::{SchemaBounds, array_schema, object_schema, string_schema};

use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use utoipa::{PartialSchema, ToSchema};

/// VAL-11 sanitisation profile carried into generated field schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// NFC line profile.
    Line,
    /// NFC multiline profile.
    Multiline,
    /// NFC identifier profile.
    Identifier,
    /// Lowercased email profile.
    Email,
    /// Phone-number parser profile.
    Phone,
    /// NFKC secret profile.
    Secret,
    /// No text normalisation.
    None,
}

impl Profile {
    /// Returns the stable wire name for an OpenAPI extension (VAL-11).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Multiline => "multiline",
            Self::Identifier => "identifier",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Secret => "secret",
            Self::None => "none",
        }
    }
}

/// VAL-03 error for one field value, with no user input in the parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldIssue {
    /// Closed VAL-03 code.
    pub code: &'static str,
    /// Safe, typed parameters for the translated error message.
    pub params: Map<String, Value>,
    /// Optional pointer relative to the containing field.
    pub pointer: Option<String>,
}

impl FieldIssue {
    /// Creates a parameter-free issue.
    pub fn new(code: &'static str) -> Self {
        Self {
            code,
            params: Map::new(),
            pointer: None,
        }
    }

    /// Creates an issue with a safe numeric bound.
    pub fn bound(code: &'static str, key: &str, value: impl Into<Value>) -> Self {
        let mut params = Map::new();
        params.insert(key.to_owned(), value.into());
        Self {
            code,
            params,
            pointer: None,
        }
    }

    /// Adds a relative JSON Pointer, used by object and list kinds.
    pub fn at(mut self, pointer: impl Into<String>) -> Self {
        self.pointer = Some(pointer.into());
        self
    }
}

/// VAL-20 field kind: one input representation, parser and OpenAPI schema.
pub trait FieldKind: Sized {
    /// Raw JSON/query value accepted at V2.
    type Input: DeserializeOwned + ToSchema;

    /// OpenAPI component name and validation-corpus key.
    const NAME: &'static str;
    /// Unique OpenAPI component name; const-generic kinds include their bounds.
    fn component_name() -> Cow<'static, str> {
        Cow::Borrowed(Self::NAME)
    }
    /// VAL-11 sanitising profile.
    const PROFILE: Profile;
    /// Whether Debug output for a raw value must redact it.
    const SENSITIVE: bool = false;
    /// Whether an optional line field treats an empty sanitized value as absent (CAN-22).
    const EMPTY_OPTIONAL_IS_NULL: bool = false;

    /// Sanitises, validates and returns a canonical typed value.
    fn parse(raw: Self::Input, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>>;
    /// Emits the JSON Schema rules used by the parser, including VAL-11 extensions.
    fn schema() -> utoipa::openapi::schema::Schema;
    /// Safe parity representation; secrets may deliberately return None.
    fn canonical_json(&self) -> Option<Value> {
        None
    }
}

/// V5 context values consumed by field kinds (VAL-20/21).
#[derive(Debug, Clone, Copy)]
pub struct FieldCtx {
    /// Optional tenant country required only for national phone parsing.
    pub country: Option<celes::Country>,
    /// Tenant currency used by money and currency fields.
    pub tenant_currency: crate::money::Currency,
    /// Tenant default locale used by internationalised text.
    pub tenant_locale: crate::policy::types::Locale,
    /// Tenant-local calendar date used by birth-date validation.
    pub today: chrono::NaiveDate,
    /// Policy supplied password minimum; None means no policy minimum was supplied here.
    pub password_min_length: Option<usize>,
}

/// Untyped V2 input wrapper; field validation runs only in Collector (VAL-20/41).
#[derive(serde::Deserialize)]
#[serde(transparent, bound(deserialize = "K::Input: DeserializeOwned"))]
pub struct Raw<K: FieldKind>(pub K::Input, #[serde(skip)] PhantomData<fn() -> K>);

impl<K: FieldKind> Raw<K> {
    /// Wraps a raw value for hand-written adapters and tests.
    pub fn new(input: K::Input) -> Self {
        Self(input, PhantomData)
    }
}

impl<K: FieldKind> std::fmt::Debug for Raw<K>
where
    K::Input: std::fmt::Debug,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Raw([REDACTED])")
    }
}

impl<K: FieldKind> PartialSchema for Raw<K> {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::Ref::from_schema_name(K::component_name().into_owned()).into()
    }
}

impl<K: FieldKind> ToSchema for Raw<K> {
    fn name() -> Cow<'static, str> {
        K::component_name()
    }

    fn schemas(schemas: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) {
        schemas.push((K::component_name().into_owned(), K::schema().into()));
    }
}

/// Escapes one RFC 6901 JSON Pointer path segment (CAN-24).
pub(crate) fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// Applies the selected text profile without exposing input through an error.
pub(crate) fn sanitize(input: &str, profile: Profile) -> Result<String, FieldIssue> {
    let result = match profile {
        Profile::Line => crate::text::sanitize_line(input),
        Profile::Multiline => crate::text::sanitize_multiline(input),
        Profile::Identifier => crate::text::sanitize_identifier(input),
        Profile::Email => crate::text::sanitize_email(input),
        Profile::Phone => crate::text::sanitize_phone(input),
        Profile::Secret => crate::text::sanitize_secret(input),
        Profile::None => Ok(input.to_owned()),
    };
    result.map_err(|issue| match issue {
        crate::text::TextIssue::InvalidCharacters => FieldIssue::new(crate::errors::INVALID_CHARACTERS),
        crate::text::TextIssue::InvalidFormat => FieldIssue::new(crate::errors::INVALID_FORMAT),
    })
}

#[cfg(test)]
mod tests;
