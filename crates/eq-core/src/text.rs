//! VAL-11: shared, deterministic text sanitising profiles.

use unicode_normalization::UnicodeNormalization;

/// A forbidden character or an input that cannot be sanitised safely (VAL-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextIssue {
    /// The text contains a character forbidden by VAL-12.
    InvalidCharacters,
    /// The text cannot be canonicalised without changing its meaning (VAL-11).
    InvalidFormat,
}

/// VAL-11 line profile: NFC, safe characters, invisible-format removal and single spaces.
pub fn sanitize_line(input: &str) -> Result<String, TextIssue> {
    let normalized: String = input.nfc().collect();
    check_forbidden(&normalized, false)?;
    let visible = remove_line_format_chars(&normalized);
    Ok(collapse_whitespace(&visible))
}

/// VAL-11 multiline profile: preserve line breaks while normalising each line.
pub fn sanitize_multiline(input: &str) -> Result<String, TextIssue> {
    let line_endings = input.replace("\r\n", "\n").replace('\r', "\n").replace('\t', " ");
    let normalized: String = line_endings.nfc().collect();
    check_forbidden(&normalized, true)?;

    let visible = remove_line_format_chars(&normalized);
    let mut lines = Vec::new();
    let mut blank_run = 0usize;
    for line in visible.split('\n') {
        let line = collapse_whitespace(line);
        if line.is_empty() {
            if !lines.is_empty() && blank_run < 2 {
                lines.push(String::new());
                blank_run += 1;
            }
        } else {
            lines.push(line);
            blank_run = 0;
        }
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    Ok(lines.join("\n"))
}

/// VAL-11 identifier profile: NFC, trim and reject internal whitespace.
pub fn sanitize_identifier(input: &str) -> Result<String, TextIssue> {
    let normalized: String = input.nfc().collect();
    check_forbidden(&normalized, false)?;
    let without_marks: String = normalized
        .chars()
        .filter(|ch| !matches!(*ch, '\u{200e}' | '\u{200f}'))
        .collect();
    let trimmed = without_marks.trim();
    if trimmed.chars().any(char::is_whitespace) {
        return Err(TextIssue::InvalidFormat);
    }
    Ok(trimmed.to_owned())
}

/// VAL-11 email profile: NFC, trim, remove one trailing domain dot and lowercase.
pub fn sanitize_email(input: &str) -> Result<String, TextIssue> {
    let normalized: String = input.nfc().collect();
    check_forbidden(&normalized, false)?;
    let without_marks: String = normalized
        .chars()
        .filter(|ch| !matches!(*ch, '\u{200e}' | '\u{200f}'))
        .collect();
    let trimmed = without_marks.trim();
    let Some((local, domain)) = trimmed.rsplit_once('@') else {
        return Ok(trimmed.to_lowercase());
    };
    let domain = domain.strip_suffix('.').unwrap_or(domain);
    if domain.ends_with('.') {
        return Err(TextIssue::InvalidFormat);
    }
    Ok(format!("{}@{}", local.to_lowercase(), domain.to_lowercase()))
}

/// VAL-11 phone profile: trim before parsing and reject forbidden controls.
pub fn sanitize_phone(input: &str) -> Result<String, TextIssue> {
    check_forbidden(input, false)?;
    Ok(input.trim().to_owned())
}

/// VAL-11 secret profile: NFKC only, without trimming or whitespace folding.
pub fn sanitize_secret(input: &str) -> Result<String, TextIssue> {
    let normalized: String = input.nfkc().collect();
    if normalized.chars().any(|ch| ch <= '\u{001f}') {
        return Err(TextIssue::InvalidCharacters);
    }
    Ok(normalized)
}

fn remove_line_format_chars(input: &str) -> String {
    input
        .chars()
        .filter(|ch| !matches!(*ch, '\u{200b}' | '\u{feff}' | '\u{2060}' | '\u{00ad}'))
        .collect()
}

fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn check_forbidden(input: &str, allow_newline: bool) -> Result<(), TextIssue> {
    let mut combining_run = 0usize;
    for ch in input.chars() {
        let code = u32::from(ch);
        if unicode_normalization::char::is_combining_mark(ch) {
            combining_run += 1;
        } else {
            combining_run = 0;
        }
        let control = (code <= 0x1f && !(allow_newline && ch == '\n')) || (0x7f..=0x9f).contains(&code);
        let bidi = (0x202a..=0x202e).contains(&code) || (0x2066..=0x2069).contains(&code);
        let noncharacter = (0xfdd0..=0xfdef).contains(&code) || (code & 0xffff) >= 0xfffe;
        let private_use = (0xe000..=0xf8ff).contains(&code)
            || (0xf0000..=0xffffd).contains(&code)
            || (0x100000..=0x10fffd).contains(&code);
        if control || bidi || noncharacter || private_use || combining_run > 8 {
            return Err(TextIssue::InvalidCharacters);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        TextIssue, sanitize_email, sanitize_identifier, sanitize_line, sanitize_multiline, sanitize_phone,
        sanitize_secret,
    };
    use proptest::prelude::*;

    #[test]
    fn line_profile_collapses_whitespace_and_removes_invisible_format_chars() {
        assert_eq!(
            sanitize_line("  Ayu\u{200b}   Sari\u{feff}  ").unwrap(),
            "Ayu Sari"
        );
        assert_eq!(
            sanitize_line("Ayu\u{202e}Sari"),
            Err(TextIssue::InvalidCharacters)
        );
    }

    #[test]
    fn multiline_profile_preserves_lines_and_caps_blank_runs() {
        assert_eq!(
            sanitize_multiline("  first\r\n\tsecond  \r\n\r\n\r\n\r\nthird  ").unwrap(),
            "first\nsecond\n\n\nthird"
        );
    }

    #[test]
    fn identifier_profile_normalizes_nfc_and_rejects_internal_space() {
        assert_eq!(sanitize_identifier("  key_name  ").unwrap(), "key_name");
        assert_eq!(sanitize_identifier("two words"), Err(TextIssue::InvalidFormat));
        assert_eq!(sanitize_identifier("a\u{200e}b").unwrap(), "ab");
    }

    #[test]
    fn email_profile_normalizes_domain_and_strips_direction_marks() {
        assert_eq!(sanitize_email(" Budi@Example.COM. ").unwrap(), "budi@example.com");
        assert_eq!(
            sanitize_email("bu\u{200e}di@example.com").unwrap(),
            "budi@example.com"
        );
        assert_eq!(sanitize_email("a@b.."), Err(TextIssue::InvalidFormat));
    }

    #[test]
    fn email_profile_rejects_repeated_trailing_domain_dots() {
        assert_eq!(
            sanitize_email("a@example.invalid."),
            Ok("a@example.invalid".to_owned())
        );
        assert_eq!(
            sanitize_email("a@example.invalid.."),
            Err(TextIssue::InvalidFormat)
        );
    }

    #[test]
    fn phone_profile_trims_and_rejects_controls() {
        assert_eq!(sanitize_phone(" 0812-3456-7890 ").unwrap(), "0812-3456-7890");
        assert_eq!(
            sanitize_phone("0812\u{0007}3456"),
            Err(TextIssue::InvalidCharacters)
        );
    }

    #[test]
    fn secret_profile_nfkc_no_trim() {
        assert_eq!(sanitize_secret("  Ｐass  ").unwrap(), "  Pass  ");
        assert_eq!(
            sanitize_secret("secret\u{0007}"),
            Err(TextIssue::InvalidCharacters)
        );
    }

    proptest! {
        #[test]
        fn prop_sanitize_idempotent(input in ".{0,128}") {
            if let Ok(once) = sanitize_line(&input) {
                prop_assert_eq!(sanitize_line(&once), Ok(once.clone()));
            }
            if let Ok(once) = sanitize_multiline(&input) {
                prop_assert_eq!(sanitize_multiline(&once), Ok(once.clone()));
            }
            if let Ok(once) = sanitize_identifier(&input) {
                prop_assert_eq!(sanitize_identifier(&once), Ok(once.clone()));
            }
            if let Ok(once) = sanitize_email(&input) {
                prop_assert_eq!(sanitize_email(&once), Ok(once.clone()));
            }
            if let Ok(once) = sanitize_phone(&input) {
                prop_assert_eq!(sanitize_phone(&once), Ok(once.clone()));
            }
            if let Ok(once) = sanitize_secret(&input) {
                prop_assert_eq!(sanitize_secret(&once), Ok(once.clone()));
            }
        }
    }
}
