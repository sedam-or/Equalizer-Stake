//! VAL-40: shared HTTP request extractors.

pub mod validated;

pub use validated::{ValidatedJson, ValidatedQuery, parse_path_id};
