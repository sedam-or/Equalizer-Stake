//! Common policy value types (spec/04 §5, POL-11/12).

mod doc;
pub mod merge;
mod repo;
mod store;
pub mod types;
mod ui;
mod validate;

pub use doc::{DynPolicy, PolicyDoc, PresetPatchSource, PresetSourceError, Versioned};
pub use merge::merge_patch;
pub use store::PolicyStore;
pub use ui::{Group, Unit, Widget, XUi};
pub use validate::validate_document;
