//! Typed policy documents and dynamic module validation (POL-01/06/07).

use schemars::JsonSchema;
use std::fmt;

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::error::FieldError;
use crate::module::{Module, ModuleId, SummaryLine};

/// POL-01: typed, versioned policy document implemented by each module's policy type.
pub trait PolicyDoc:
    Serialize + DeserializeOwned + JsonSchema + Default + Clone + Send + Sync + 'static
{
    const MODULE: ModuleId;
    const SCHEMA_VERSION: u32;

    /// POL-10: applies pure cross-field checks after schema and serde validation.
    fn validate(&self) -> Result<(), Vec<FieldError>>;

    /// POL-07: returns translated summary keys and interpolation arguments.
    fn summarize(&self) -> Vec<SummaryLine>;
}

/// POL-05: policy document selected for one effective interval.
#[derive(Clone, Debug, PartialEq)]
pub struct Versioned<P> {
    pub version: i32,
    pub effective_from: chrono::DateTime<chrono::Utc>,
    pub doc: P,
}

/// POL-10: object-safe validation and summary adapter for settings APIs.
pub struct DynPolicy<'a> {
    module: &'a dyn Module,
}

impl fmt::Debug for DynPolicy<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DynPolicy")
            .field("module", &self.module.manifest().id)
            .finish()
    }
}

impl<'a> DynPolicy<'a> {
    /// Builds a dynamic policy adapter around the compiled module implementation.
    pub fn new(module: &'a dyn Module) -> Self {
        Self { module }
    }

    /// Validates an untyped policy document through its module contract.
    pub fn validate(&self, document: &Value) -> Result<(), Vec<FieldError>> {
        self.module.validate_policy(document)
    }

    /// Summarizes an untyped policy document through its module contract.
    pub fn summarize(&self, document: &Value) -> Vec<SummaryLine> {
        self.module.summarize_policy(document)
    }
}

/// SET-02/POL-09: resolves a tenant's stored preset version to its raw module patch.
pub trait PresetPatchSource: Send + Sync {
    fn policy_patch(
        &self,
        preset_id: &str,
        preset_version: i32,
        module: ModuleId,
    ) -> Result<Option<Value>, PresetSourceError>;
}

/// SET-02: static preset identity/version could not be resolved by the embedded loader.
#[derive(Debug, thiserror::Error)]
#[error("preset identity or version unavailable")]
pub struct PresetSourceError;
