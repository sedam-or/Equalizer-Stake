//! JSON Schema, serde and policy-specific validation (POL-10/VAL-22).

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use jsonschema::error::ValidationErrorKind;
use jsonschema::{Draft, JSONSchema};
use serde_json::{Map, Value};

use crate::error::{AppError, FieldError};
use crate::errors;
use crate::policy::doc::PolicyDoc;

static VALIDATORS: OnceLock<RwLock<HashMap<TypeId, Arc<JSONSchema>>>> = OnceLock::new();

/// POL-10: validates a policy in schema, serde and cross-field order.
pub fn validate_document<P: PolicyDoc>(document: Value) -> Result<P, AppError> {
    let validator = validator::<P>()?;
    let errors = match validator.validate(&document) {
        Ok(()) => Vec::new(),
        Err(errors) => errors.map(schema_field_error).collect::<Vec<_>>(),
    };
    if !errors.is_empty() {
        return Err(AppError::Validation { errors });
    }

    let input = serde_json::to_string(&document).map_err(|error| AppError::Internal(error.into()))?;
    let mut deserializer = serde_json::Deserializer::from_str(&input);
    let typed: P =
        serde_path_to_error::deserialize(&mut deserializer).map_err(|error| AppError::Validation {
            errors: vec![FieldError {
                pointer: serde_pointer(&error.path().to_string()),
                code: errors::INVALID_TYPE.to_owned(),
                params: Map::new(),
            }],
        })?;

    typed
        .validate()
        .map_err(|errors| AppError::Validation { errors })?;
    Ok(typed)
}

fn validator<P: PolicyDoc>() -> Result<Arc<JSONSchema>, AppError> {
    let cache = VALIDATORS.get_or_init(|| RwLock::new(HashMap::new()));
    if let Some(validator) = cache
        .read()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("policy schema cache unavailable")))?
        .get(&TypeId::of::<P>())
    {
        return Ok(Arc::clone(validator));
    }

    let schema =
        serde_json::to_value(schemars::schema_for!(P)).map_err(|error| AppError::Internal(error.into()))?;
    let validator = Arc::new(
        JSONSchema::options()
            .with_draft(Draft::Draft202012)
            .compile(&schema)
            .map_err(|error| AppError::Internal(anyhow::anyhow!("{error}")))?,
    );
    let mut entries = cache
        .write()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("policy schema cache unavailable")))?;
    Ok(Arc::clone(entries.entry(TypeId::of::<P>()).or_insert(validator)))
}

fn schema_field_error(error: jsonschema::ValidationError<'_>) -> FieldError {
    let kind = format!("{:?}", error.kind);
    let code = if kind.contains("Required") {
        errors::REQUIRED
    } else if kind.contains("Type") {
        errors::INVALID_TYPE
    } else if kind.contains("MinLength") {
        errors::TOO_SHORT
    } else if kind.contains("MaxLength") {
        errors::TOO_LONG
    } else if kind.contains("MinItems") {
        errors::TOO_FEW_ITEMS
    } else if kind.contains("MaxItems") {
        errors::TOO_MANY_ITEMS
    } else if kind.contains("Minimum") || kind.contains("ExclusiveMinimum") {
        errors::MUST_BE_GTE
    } else if kind.contains("Maximum") || kind.contains("ExclusiveMaximum") {
        errors::MUST_BE_LTE
    } else if kind.contains("Pattern") || kind.contains("Format") {
        errors::INVALID_FORMAT
    } else if kind.contains("UniqueItems") {
        errors::NOT_UNIQUE
    } else {
        errors::NOT_ALLOWED
    };

    let mut pointer = error.instance_path.to_string();
    if let ValidationErrorKind::Required { property } = error.kind
        && let Some(property) = property.as_str()
    {
        pointer.push('/');
        pointer.push_str(&escape_pointer(property));
    }
    FieldError {
        pointer,
        code: code.to_owned(),
        params: Map::new(),
    }
}

fn serde_pointer(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let segments = path
        .replace('[', ".")
        .replace(']', "")
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(escape_pointer)
        .collect::<Vec<_>>();
    format!("/{}", segments.join("/"))
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_json::{Map, json};

    use crate::module::{ModuleId, SummaryLine};
    use crate::policy::validate::validate_document;
    use crate::{error::FieldError, errors};

    #[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
    #[serde(deny_unknown_fields)]
    struct ExamplePolicy {
        #[serde(default)]
        amount: Option<i32>,
        #[serde(default)]
        names: Vec<String>,
    }

    impl super::PolicyDoc for ExamplePolicy {
        const MODULE: ModuleId = ModuleId::Scheduling;
        const SCHEMA_VERSION: u32 = 1;

        fn validate(&self) -> Result<(), Vec<FieldError>> {
            if self.amount.is_some_and(|amount| amount < 0) {
                return Err(vec![FieldError {
                    pointer: "/amount".to_owned(),
                    code: errors::MUST_BE_GTE.to_owned(),
                    params: Map::new(),
                }]);
            }
            Ok(())
        }

        fn summarize(&self) -> Vec<SummaryLine> {
            Vec::new()
        }
    }

    #[test]
    fn schema_keywords_map_to_field_codes() {
        let result = validate_document::<ExamplePolicy>(json!({"amount":"bad"}));
        let Err(crate::error::AppError::Validation { errors }) = result else {
            panic!("schema type error must map to field validation");
        };
        assert!(errors.iter().any(|error| error.code == errors::INVALID_TYPE));
    }

    #[test]
    fn required_schema_error_has_field_pointer() {
        #[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
        struct RequiredPolicy {
            name: String,
        }
        impl super::PolicyDoc for RequiredPolicy {
            const MODULE: ModuleId = ModuleId::Identity;
            const SCHEMA_VERSION: u32 = 1;
            fn validate(&self) -> Result<(), Vec<FieldError>> {
                Ok(())
            }
            fn summarize(&self) -> Vec<SummaryLine> {
                Vec::new()
            }
        }
        let result = validate_document::<RequiredPolicy>(json!({}));
        let Err(crate::error::AppError::Validation { errors }) = result else {
            panic!("missing required field must map to validation");
        };
        assert!(
            errors
                .iter()
                .any(|error| error.pointer == "/name" && error.code == errors::REQUIRED),
            "schema errors: {errors:?}"
        );
    }

    #[test]
    fn cross_field_policy_validation_uses_its_pointer() {
        let result = validate_document::<ExamplePolicy>(json!({"amount":-1}));
        let Err(crate::error::AppError::Validation { errors }) = result else {
            panic!("cross-field policy rule must be validated");
        };
        assert_eq!(errors[0].pointer, "/amount");
        assert_eq!(errors[0].code, errors::MUST_BE_GTE);
    }
}
