//! VAL-18: shared structural limits for request validation.

/// Maximum nesting depth accepted by the JSON request parser.
pub const JSON_MAX_DEPTH: usize = 32; // technical: prevents recursive parser exhaustion (VAL-18)
/// Maximum number of items accepted when a field kind has no narrower bound.
pub const DEFAULT_ARRAY_MAX: usize = 100; // technical: default bounded collection size (VAL-18)
/// Inclusive minimum search query length in Unicode code points.
pub const SEARCH_QUERY_MIN: usize = 1; // technical: VAL-21 SearchQuery bound
/// Inclusive maximum search query length in Unicode code points.
pub const SEARCH_QUERY_MAX: usize = 100; // technical: VAL-21 SearchQuery bound
/// Inclusive minimum requested page size.
pub const PAGE_SIZE_MIN: u16 = 1; // technical: CAN-32 page-size bound
/// Inclusive maximum requested page size.
pub const PAGE_SIZE_MAX: u16 = 200; // technical: CAN-32 page-size bound
/// Maximum encoded cursor size in bytes.
pub const CURSOR_MAX_BYTES: usize = 512; // technical: VAL-18 cursor bound
/// Maximum JSON body size accepted by the extractor.
pub const JSON_BODY_MAX_BYTES: usize = 1_048_576; // technical: VAL-18 JSON body bound

/// Shared canonical money limit from CAN-10.
pub use crate::money::MONEY_MAX_ABS_MINOR;
