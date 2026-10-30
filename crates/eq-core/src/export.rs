//! Safe CSV cell handling (SBD-46).

/// SBD-46: prefixes formula-capable CSV values so spreadsheet software treats them as text.
pub fn guard_cell(value: &str) -> String {
    if value
        .chars()
        .next()
        .is_some_and(|first| matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r'))
    {
        format!("'{value}")
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::guard_cell;

    #[test]
    fn formula_leading_csv_cells_are_prefixed() {
        for value in ["=1+1", "+cmd", "-cmd", "@SUM(A1)", "\tvalue", "\rvalue"] {
            assert_eq!(guard_cell(value), format!("'{value}"));
        }
        assert_eq!(guard_cell("plain"), "plain");
        assert_eq!(guard_cell(""), "");
    }
}
