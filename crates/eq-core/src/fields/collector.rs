//! VAL-04: deterministic collection of all field errors.

use super::{FieldCtx, FieldIssue, FieldKind, Raw, escape_pointer_segment};
use crate::{
    error::{AppError, FieldError},
    errors::{TOO_FEW_ITEMS, TOO_MANY_ITEMS},
};

/// Collects field issues in the order adapters invoke field parsing (VAL-04).
#[derive(Debug)]
pub struct Collector<'c> {
    ctx: &'c FieldCtx,
    errors: Vec<FieldError>,
}

impl<'c> Collector<'c> {
    /// Creates a collector for one request context.
    pub fn new(ctx: &'c FieldCtx) -> Self {
        Self {
            ctx,
            errors: Vec::new(),
        }
    }

    /// Parses one required field and appends every issue at its JSON Pointer.
    pub fn field<K: FieldKind>(&mut self, pointer: &str, raw: Raw<K>) -> Option<K> {
        match K::parse(raw.0, self.ctx) {
            Ok(value) => Some(value),
            Err(issues) => {
                self.push_all(pointer, issues);
                None
            }
        }
    }

    /// Parses an optional field while preserving absence separately from invalid input.
    pub fn optional<K: FieldKind>(&mut self, pointer: &str, raw: Option<Raw<K>>) -> Option<Option<K>> {
        match raw {
            None => Some(None),
            Some(value) => match K::parse(value.0, self.ctx) {
                Ok(parsed) => Some(Some(parsed)),
                Err(issues)
                    if K::EMPTY_OPTIONAL_IS_NULL
                        && issues.iter().all(|issue| issue.code == crate::errors::REQUIRED) =>
                {
                    Some(None)
                }
                Err(issues) => {
                    self.push_all(pointer, issues);
                    None
                }
            },
        }
    }

    /// Parses every list member and reports bound and member errors in encounter order.
    pub fn list<K: FieldKind>(
        &mut self,
        pointer: &str,
        raw: Vec<Raw<K>>,
        min: usize,
        max: usize,
    ) -> Option<Vec<K>> {
        let length = raw.len();
        if length < min {
            self.push(pointer, FieldIssue::bound(TOO_FEW_ITEMS, "min", min));
        }
        if length > max {
            self.push(pointer, FieldIssue::bound(TOO_MANY_ITEMS, "max", max));
        }

        let mut values = Vec::with_capacity(length.min(max));
        let mut valid = true;
        for (index, item) in raw.into_iter().enumerate() {
            let item_pointer = format!("{pointer}/{index}");
            match K::parse(item.0, self.ctx) {
                Ok(value) => values.push(value),
                Err(issues) => {
                    valid = false;
                    self.push_all(&item_pointer, issues);
                }
            }
        }
        (valid && length >= min && length <= max).then_some(values)
    }

    /// Adds one request-rule issue at the supplied body pointer (VAL-41).
    pub fn push(&mut self, pointer: &str, issue: FieldIssue) {
        self.push_one(pointer, issue);
    }

    /// Returns all collected field errors as one 422 application error.
    pub fn finish(self) -> Result<(), AppError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation { errors: self.errors })
        }
    }

    fn push_all(&mut self, pointer: &str, issues: Vec<FieldIssue>) {
        for issue in issues {
            self.push_one(pointer, issue);
        }
    }

    fn push_one(&mut self, pointer: &str, issue: FieldIssue) {
        let full_pointer = match issue.pointer {
            Some(relative) if relative.starts_with('/') => {
                format!("{pointer}{relative}")
            }
            Some(relative) if relative.is_empty() => pointer.to_owned(),
            Some(relative) => format!("{pointer}/{}", escape_pointer_segment(&relative)),
            None => pointer.to_owned(),
        };
        self.errors.push(FieldError {
            pointer: full_pointer,
            code: issue.code.to_owned(),
            params: issue.params,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::AppError,
        fields::{DisplayName, Raw},
        money::Currency,
        policy::types::Locale,
    };

    #[test]
    fn optional_empty_line_is_null_but_invalid_characters_are_reported() {
        let context = FieldCtx {
            country: celes::Country::from_alpha2("ID").expect("known test country"),
            tenant_currency: Currency::IDR,
            tenant_locale: Locale::Id,
            today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
            password_min_length: Some(8),
        };
        let mut collector = Collector::new(&context);
        let empty = collector.optional::<DisplayName>("/name", Some(Raw::new("  ".to_owned())));
        assert!(matches!(empty, Some(None)));
        assert!(collector.finish().is_ok());

        let mut collector = Collector::new(&context);
        let invalid = collector.optional::<DisplayName>("/name", Some(Raw::new("\u{202e}".to_owned())));
        assert!(invalid.is_none());
        assert!(matches!(collector.finish(), Err(AppError::Validation { .. })));
    }
}
