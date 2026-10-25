//! Shared field validation codes from VAL-03.

pub const REQUIRED: &str = "required";
pub const INVALID_TYPE: &str = "invalid_type";
pub const TOO_SHORT: &str = "too_short";
pub const TOO_LONG: &str = "too_long";
pub const TOO_FEW_ITEMS: &str = "too_few_items";
pub const TOO_MANY_ITEMS: &str = "too_many_items";
pub const MUST_BE_GTE: &str = "must_be_gte";
pub const MUST_BE_LTE: &str = "must_be_lte";
pub const INVALID_FORMAT: &str = "invalid_format";
pub const INVALID_CHARACTERS: &str = "invalid_characters";
pub const NOT_ALLOWED: &str = "not_allowed";
pub const NOT_UNIQUE: &str = "not_unique";
pub const UNKNOWN_REFERENCE: &str = "unknown_reference";

pub const FIELD_ERROR_CODES: &[&str] = &[
    REQUIRED,
    INVALID_TYPE,
    TOO_SHORT,
    TOO_LONG,
    TOO_FEW_ITEMS,
    TOO_MANY_ITEMS,
    MUST_BE_GTE,
    MUST_BE_LTE,
    INVALID_FORMAT,
    INVALID_CHARACTERS,
    NOT_ALLOWED,
    NOT_UNIQUE,
    UNKNOWN_REFERENCE,
];

#[cfg(test)]
mod tests {
    use super::FIELD_ERROR_CODES;

    #[test]
    fn field_error_codes_match_val_03() {
        assert_eq!(
            FIELD_ERROR_CODES,
            &[
                "required",
                "invalid_type",
                "too_short",
                "too_long",
                "too_few_items",
                "too_many_items",
                "must_be_gte",
                "must_be_lte",
                "invalid_format",
                "invalid_characters",
                "not_allowed",
                "not_unique",
                "unknown_reference",
            ]
        );
    }
}
