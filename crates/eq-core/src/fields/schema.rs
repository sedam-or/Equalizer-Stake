//! VAL-20: OpenAPI schemas generated from field parser bounds.

use utoipa::openapi::{
    extensions::Extensions,
    schema::{AdditionalProperties, ArrayBuilder, ArrayItems, ObjectBuilder, Schema, SchemaType, Type},
};

use super::Profile;

/// Shared schema bounds that keep parser and OpenAPI metadata aligned.
#[derive(Debug, Clone, Copy)]
pub struct SchemaBounds {
    /// Minimum string code-point length, when applicable.
    pub min_length: Option<usize>,
    /// Maximum string code-point length, when applicable.
    pub max_length: Option<usize>,
    /// Inclusive numeric lower bound, when applicable.
    pub minimum: Option<i64>,
    /// Inclusive numeric upper bound, when applicable.
    pub maximum: Option<i64>,
}

/// Builds a string schema from the same limits used by a text parser (VAL-20).
pub fn string_schema(
    name: &'static str,
    profile: Profile,
    bounds: SchemaBounds,
    pattern: Option<&'static str>,
) -> Schema {
    let extensions = Extensions::from_iter([
        ("x-sanitize", serde_json::json!(profile.as_str())),
        ("x-field-kind", serde_json::json!(name)),
    ]);
    ObjectBuilder::new()
        .schema_type(Type::String)
        .title(Some(name))
        .min_length(bounds.min_length)
        .max_length(bounds.max_length)
        .pattern(pattern)
        .extensions(Some(extensions))
        .build()
        .into()
}

/// Builds a numeric schema from the same limits used by a numeric parser (VAL-20).
pub fn number_schema(
    name: &'static str,
    profile: Profile,
    minimum: Option<i64>,
    maximum: Option<i64>,
) -> Schema {
    let extensions = Extensions::from_iter([
        ("x-sanitize", serde_json::json!(profile.as_str())),
        ("x-field-kind", serde_json::json!(name)),
    ]);
    ObjectBuilder::new()
        .schema_type(Type::Integer)
        .title(Some(name))
        .minimum(minimum)
        .maximum(maximum)
        .extensions(Some(extensions))
        .build()
        .into()
}

/// Builds an object schema with explicit properties and field metadata (VAL-20).
pub fn object_schema(
    name: &'static str,
    profile: Profile,
    properties: impl IntoIterator<Item = (&'static str, Schema)>,
) -> Schema {
    let extensions = Extensions::from_iter([
        ("x-sanitize", serde_json::json!(profile.as_str())),
        ("x-field-kind", serde_json::json!(name)),
    ]);
    let mut schema = ObjectBuilder::new()
        .schema_type(SchemaType::Type(Type::Object))
        .title(Some(name))
        .extensions(Some(extensions));
    for (property, field_schema) in properties {
        schema = schema.property(property, field_schema).required(property);
    }
    schema = schema.additional_properties(Some(AdditionalProperties::FreeForm(false)));
    schema.build().into()
}

/// Builds an array schema with an item schema and inclusive count limits (VAL-20).
pub fn array_schema(
    name: &'static str,
    profile: Profile,
    items: Schema,
    min_items: usize,
    max_items: usize,
    unique: bool,
) -> Schema {
    let extensions = Extensions::from_iter([
        ("x-sanitize", serde_json::json!(profile.as_str())),
        ("x-field-kind", serde_json::json!(name)),
    ]);
    let array = ArrayBuilder::new()
        .items(ArrayItems::RefOrSchema(Box::new(items.into())))
        .min_items(Some(min_items))
        .max_items(Some(max_items))
        .unique_items(unique)
        .extensions(Some(extensions))
        .build();
    array.into()
}
