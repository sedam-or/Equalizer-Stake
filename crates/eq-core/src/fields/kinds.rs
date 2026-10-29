//! VAL-21: field kind parsers and canonical value wrappers.

use std::{borrow::Cow, collections::BTreeMap, fmt, str::FromStr};

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use email_address::{EmailAddress, Options as EmailOptions};
use nutype::nutype;
use secrecy::{ExposeSecret, SecretString};
use serde::de::Error as SerdeDeError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::RefOr,
    openapi::schema::{AdditionalProperties, ObjectBuilder, Schema, SchemaType, Type},
};
use uuid::Uuid;

use super::{
    FieldCtx, FieldIssue, FieldKind, Profile, sanitize,
    schema::{SchemaBounds, array_schema, number_schema, object_schema, string_schema},
};
use crate::{
    errors::{
        INVALID_FORMAT, MUST_BE_GTE, MUST_BE_LTE, NOT_ALLOWED, REQUIRED, TOO_LONG, TOO_MANY_ITEMS, TOO_SHORT,
    },
    limits::{MONEY_MAX_ABS_MINOR, SEARCH_QUERY_MAX, SEARCH_QUERY_MIN},
    money::{Currency, Money},
    policy::types::{Bps, GeoPoint, LocalTime, Locale},
};

const DISPLAY_NAME_MIN: usize = 1;
const DISPLAY_NAME_MAX: usize = 80;
const LEGAL_NAME_MIN: usize = 1;
const LEGAL_NAME_MAX: usize = 120;
const TENANT_NAME_MIN: usize = 2;
const TENANT_NAME_MAX: usize = 80;
const PLACE_NAME_MIN: usize = 1;
const PLACE_NAME_MAX: usize = 80;
const SHORT_TEXT_MIN: usize = 1;
const SHORT_TEXT_MAX: usize = 200;
const ADDRESS_MIN: usize = 1;
const ADDRESS_MAX: usize = 300;
const NOTE_MIN: usize = 1;
const NOTE_MAX: usize = 1_000;
const COMMENT_MIN: usize = 1;
const COMMENT_MAX: usize = 2_000;
const EMAIL_MAX: usize = 254;
const EMAIL_LOCAL_MAX: usize = 64;
const PASSWORD_MAX: usize = 128;
const KEY_MIN: usize = 2;
const KEY_MAX: usize = 40;
const SLUG_MIN: usize = 3;
const SLUG_MAX: usize = 40;
const ID_MIN: usize = 36;
const ID_MAX: usize = 36;
const LOCAL_DATE_MIN: &str = "1900-01-01";
const LOCAL_DATE_MAX: &str = "2100-12-31";
const INSTANT_MIN: &str = "2000-01-01T00:00:00Z";
const INSTANT_MAX: &str = "2100-01-01T00:00:00Z";
const BPS_MAX: i64 = 10_000;
const GEO_SCALE: f64 = 1_000_000.0;
const HUE_MIN: i64 = 0;
const HUE_MAX: i64 = 359;
const STARS_MIN: i64 = 1;
const STARS_MAX: i64 = 5;

fn one(issue: FieldIssue) -> Vec<FieldIssue> {
    vec![issue]
}

fn phone_region_for_country(country: celes::Country) -> Option<rlibphonenumber::enums::Region> {
    // The generated alpha-2 lookup is the single conversion boundary to the parser's Region enum.
    rlibphonenumber::enums::Region::from_code(country.alpha2).ok()
}

fn parse_text(
    input: String,
    profile: Profile,
    min: usize,
    max: usize,
    kind: &'static str,
) -> Result<String, Vec<FieldIssue>> {
    let value = sanitize(&input, profile).map_err(one)?;
    if kind == "Key" && value != input {
        return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", kind)));
    }
    let length = value.chars().count();
    if length == 0 {
        return Err(one(FieldIssue::new(REQUIRED)));
    }
    if length < min {
        return Err(one(FieldIssue::bound(TOO_SHORT, "min", min)));
    }
    if length > max {
        return Err(one(FieldIssue::bound(TOO_LONG, "max", max)));
    }
    if !matches_pattern(kind, &value) {
        return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", kind)));
    }
    Ok(value)
}

fn sanitize_newtype(input: String, profile: Profile, strict: bool) -> String {
    if strict {
        return input;
    }
    match sanitize(&input, profile) {
        Ok(canonical) => canonical,
        Err(_) => input,
    }
}

fn valid_text_newtype(value: &str, profile: Profile, min: usize, max: usize, kind: &str) -> bool {
    let Ok(canonical) = sanitize(value, profile) else {
        return false;
    };
    if canonical != value {
        return false;
    }
    let length = value.chars().count();
    (min..=max).contains(&length) && matches_pattern(kind, value)
}

fn matches_pattern(kind: &str, value: &str) -> bool {
    match kind {
        "Key" => {
            (KEY_MIN..=KEY_MAX).contains(&value.chars().count())
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        }
        "Slug" => {
            let bytes = value.as_bytes();
            (SLUG_MIN..=SLUG_MAX).contains(&bytes.len())
                && bytes.first().is_some_and(u8::is_ascii_lowercase_or_digit)
                && bytes.last().is_some_and(u8::is_ascii_lowercase_or_digit)
                && bytes.iter().enumerate().all(|(index, byte)| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || (*byte == b'-' && index > 0 && index + 1 < bytes.len() && bytes[index - 1] != b'-')
                })
        }
        _ => true,
    }
}

trait AsciiLowerOrDigit {
    fn is_ascii_lowercase_or_digit(&self) -> bool;
}

impl AsciiLowerOrDigit for u8 {
    fn is_ascii_lowercase_or_digit(&self) -> bool {
        self.is_ascii_lowercase() || self.is_ascii_digit()
    }
}

macro_rules! text_kind {
    ($name:ident, $profile:expr, $min:ident, $max:ident, $pattern:expr, $strict:expr, $empty_optional:expr) => {
        #[nutype(
                            sanitize(with = |input: String| sanitize_newtype(input, $profile, $strict)),
                            validate(
                                not_empty,
                                len_char_min = $min,
                                len_char_max = $max,
                                predicate = |value: &str| valid_text_newtype(
                                    value,
                                    $profile,
                                    $min,
                                    $max,
                                    stringify!($name),
                                ),
                            ),
                            derive(AsRef, Serialize, Deserialize)
                        )]
        pub struct $name(String);

        impl FieldKind for $name {
            type Input = String;
            const NAME: &'static str = stringify!($name);
            const PROFILE: Profile = $profile;
            const EMPTY_OPTIONAL_IS_NULL: bool = $empty_optional;

            fn parse(raw: Self::Input, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
                let canonical = parse_text(raw, Self::PROFILE, $min, $max, Self::NAME)?;
                Self::try_new(canonical).map_err(|_| one(FieldIssue::bound(TOO_LONG, "max", $max)))
            }

            fn schema() -> Schema {
                string_schema(
                    Self::NAME,
                    Self::PROFILE,
                    SchemaBounds {
                        min_length: Some($min),
                        max_length: Some($max),
                        minimum: None,
                        maximum: None,
                    },
                    $pattern,
                )
            }

            fn canonical_json(&self) -> Option<Value> {
                Some(json!(self.as_ref()))
            }
        }
    };
}

text_kind!(
    DisplayName,
    Profile::Line,
    DISPLAY_NAME_MIN,
    DISPLAY_NAME_MAX,
    None,
    false,
    true
);
text_kind!(
    LegalName,
    Profile::Line,
    LEGAL_NAME_MIN,
    LEGAL_NAME_MAX,
    None,
    false,
    true
);
text_kind!(
    TenantName,
    Profile::Line,
    TENANT_NAME_MIN,
    TENANT_NAME_MAX,
    None,
    false,
    true
);
text_kind!(
    PlaceName,
    Profile::Line,
    PLACE_NAME_MIN,
    PLACE_NAME_MAX,
    None,
    false,
    true
);
text_kind!(
    ShortText,
    Profile::Line,
    SHORT_TEXT_MIN,
    SHORT_TEXT_MAX,
    None,
    false,
    true
);
text_kind!(
    Address,
    Profile::Multiline,
    ADDRESS_MIN,
    ADDRESS_MAX,
    None,
    false,
    true
);
text_kind!(Note, Profile::Multiline, NOTE_MIN, NOTE_MAX, None, false, true);
text_kind!(
    Comment,
    Profile::Multiline,
    COMMENT_MIN,
    COMMENT_MAX,
    None,
    false,
    true
);
text_kind!(
    SearchQuery,
    Profile::Line,
    SEARCH_QUERY_MIN,
    SEARCH_QUERY_MAX,
    None,
    false,
    true
);

fn sanitize_email_newtype(input: String) -> String {
    match crate::text::sanitize_email(&input) {
        Ok(canonical) => canonical,
        Err(_) => input,
    }
}

fn valid_email_newtype(value: &str) -> bool {
    let Ok(canonical) = crate::text::sanitize_email(value) else {
        return false;
    };
    if canonical != value || value.chars().count() > EMAIL_MAX || value.contains('"') {
        return false;
    }
    if let Some((local, domain)) = value.split_once('@')
        && !domain.contains('@')
        && local.chars().count() > EMAIL_LOCAL_MAX
    {
        return false;
    }
    let options = EmailOptions::default()
        .with_minimum_sub_domains(2)
        .without_domain_literal()
        .without_display_text();
    EmailAddress::parse_with_options(value, options).is_ok()
}

#[nutype(
    sanitize(with = sanitize_email_newtype),
    validate(
        not_empty,
        len_char_max = EMAIL_MAX,
        predicate = valid_email_newtype,
    ),
    derive(AsRef, Serialize, Deserialize)
)]
pub struct Email(String);

impl FieldKind for Email {
    type Input = String;
    const NAME: &'static str = "Email";
    const PROFILE: Profile = Profile::Email;

    fn parse(raw: Self::Input, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let canonical = sanitize(&raw, Self::PROFILE).map_err(one)?;
        if canonical.chars().count() > EMAIL_MAX {
            return Err(one(FieldIssue::bound(TOO_LONG, "max", EMAIL_MAX)));
        }
        if let Some((local, domain)) = canonical.split_once('@')
            && !domain.contains('@')
            && local.chars().count() > EMAIL_LOCAL_MAX
        {
            return Err(one(FieldIssue::bound(TOO_LONG, "max", EMAIL_LOCAL_MAX)));
        }
        if canonical.contains('"') {
            return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", "email")));
        }
        let options = EmailOptions::default()
            .with_minimum_sub_domains(2)
            .without_domain_literal()
            .without_display_text();
        let parsed = EmailAddress::parse_with_options(&canonical, options)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "email")))?;
        if parsed.local_part().chars().count() > EMAIL_LOCAL_MAX {
            return Err(one(FieldIssue::bound(TOO_LONG, "max", EMAIL_LOCAL_MAX)));
        }
        Self::try_new(canonical).map_err(|_| one(FieldIssue::bound(TOO_LONG, "max", EMAIL_MAX)))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(5),
                max_length: Some(EMAIL_MAX),
                minimum: None,
                maximum: None,
            },
            Some(r"^\s*[^@\s]{1,64}@[^@\s.]+(?:\.[^@\s.]+)+\.?\s*$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.as_ref()))
    }
}

/// Canonical E.164 value with redacted diagnostics (SBD-10).
#[derive(Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Phone(String);

impl Phone {
    /// Returns the canonical E.164 number for adapters that need it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Phone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Phone([REDACTED])")
    }
}

impl TryFrom<String> for Phone {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let digits = value.strip_prefix('+').ok_or("invalid E.164 number")?;
        if !(8..=15).contains(&digits.len()) || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("invalid E.164 number");
        }
        let parsed = rlibphonenumber::PHONE_NUMBER_UTIL
            .parse(&value, None)
            .map_err(|_| "invalid E.164 number")?;
        if !parsed.is_valid()
            || parsed
                .format_as(rlibphonenumber::enums::PhoneNumberFormat::E164)
                .as_ref()
                != value
        {
            return Err("invalid E.164 number");
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for Phone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl FieldKind for Phone {
    type Input = String;
    const NAME: &'static str = "Phone";
    const PROFILE: Profile = Profile::Phone;

    fn parse(raw: Self::Input, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let input = sanitize(&raw, Self::PROFILE).map_err(one)?;
        if input.chars().any(|ch| ch.is_ascii_alphabetic()) {
            return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", "phone")));
        }
        let region = ctx
            .country
            .and_then(phone_region_for_country)
            .ok_or_else(|| one(FieldIssue::bound(INVALID_FORMAT, "format", "phone")))?;
        let parsed = rlibphonenumber::PHONE_NUMBER_UTIL
            .parse(&input, Some(region))
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "phone")))?;
        if !parsed.is_valid() {
            return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", "phone")));
        }
        let canonical = parsed
            .format_as(rlibphonenumber::enums::PhoneNumberFormat::E164)
            .into_owned();
        Self::try_from(canonical).map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "phone")))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
            },
            None,
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.as_str()))
    }
}

/// V2 password input stored in zeroizing memory and redacted on Debug (SBD-50).
pub struct PasswordInput(SecretString);

impl<'de> Deserialize<'de> for PasswordInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self(SecretString::from(value)))
    }
}

impl fmt::Debug for PasswordInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PasswordInput([REDACTED])")
    }
}

impl PartialSchema for PasswordInput {
    fn schema() -> RefOr<Schema> {
        Password::schema().into()
    }
}

impl ToSchema for PasswordInput {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("PasswordInput")
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        schemas.push((Self::name().into_owned(), Self::schema()));
    }
}

/// NFKC password input stored in a secrecy wrapper and redacted on Debug (SBD-50).
pub struct Password(SecretString);

impl Password {
    /// Exposes the password only to the authentication adapter.
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password([REDACTED])")
    }
}

impl FieldKind for Password {
    type Input = PasswordInput;
    const NAME: &'static str = "Password";
    const PROFILE: Profile = Profile::Secret;
    const SENSITIVE: bool = true;

    fn parse(raw: Self::Input, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let canonical = sanitize(raw.0.expose_secret(), Self::PROFILE).map_err(one)?;
        let length = canonical.chars().count();
        if length > PASSWORD_MAX {
            return Err(one(FieldIssue::bound(TOO_LONG, "max", PASSWORD_MAX)));
        }
        if let Some(minimum) = ctx.password_min_length
            && length < minimum
        {
            return Err(one(FieldIssue::bound(TOO_SHORT, "min", minimum)));
        }
        Ok(Self(SecretString::from(canonical)))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: None,
                max_length: Some(PASSWORD_MAX),
                minimum: None,
                maximum: None,
            },
            None,
        )
    }
}

text_kind!(
    Key,
    Profile::Identifier,
    KEY_MIN,
    KEY_MAX,
    Some(r"^[a-z0-9_]{2,40}$"),
    true,
    false
);
text_kind!(
    Slug,
    Profile::Identifier,
    SLUG_MIN,
    SLUG_MAX,
    Some(r"^[a-z0-9](-?[a-z0-9])*$"),
    false,
    false
);

/// ISO 3166-1 alpha-2 country value, parsed through celes (CAN-10).
#[derive(Debug)]
pub struct CountryField(celes::Country);

impl CountryField {
    /// Returns the canonical alpha-2 code.
    pub const fn alpha2(&self) -> &'static str {
        self.0.alpha2
    }
}

impl FieldKind for CountryField {
    type Input = String;
    const NAME: &'static str = "Country";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let value = raw.to_ascii_uppercase();
        celes::Country::from_alpha2(&value)
            .map(Self)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "country")))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(2),
                max_length: Some(2),
                minimum: None,
                maximum: None,
            },
            Some("^[A-Za-z]{2}$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.alpha2))
    }
}

/// ISO 4217 currency constrained to the tenant currency outside setup (CAN-10).
#[derive(Debug)]
pub struct CurrencyField(Currency);

impl FieldKind for CurrencyField {
    type Input = String;
    const NAME: &'static str = "Currency";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let Some(currency) = Currency::from_code(&raw) else {
            return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", "currency")));
        };
        if currency != ctx.tenant_currency {
            return Err(one(FieldIssue {
                code: NOT_ALLOWED,
                params: [("allowed".to_owned(), json!([ctx.tenant_currency.code()]))]
                    .into_iter()
                    .collect(),
                pointer: None,
            }));
        }
        Ok(Self(currency))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(3),
                max_length: Some(3),
                minimum: None,
                maximum: None,
            },
            Some("^[A-Za-z]{3}$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.code()))
    }
}

/// Validated locale from the supported closed set (CAN-10).
#[derive(Debug)]
pub struct LocaleField(Locale);

impl FieldKind for LocaleField {
    type Input = String;
    const NAME: &'static str = "Locale";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let canonical = raw.to_ascii_lowercase();
        let locale = match canonical.as_str() {
            "en" | "en-us" => Locale::En,
            "id" | "id-id" => Locale::Id,
            _ => {
                return Err(one(FieldIssue {
                    code: NOT_ALLOWED,
                    params: [("allowed".to_owned(), json!(["en", "id"]))]
                        .into_iter()
                        .collect(),
                    pointer: None,
                }));
            }
        };
        Ok(Self(locale))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(2),
                max_length: Some(5),
                minimum: None,
                maximum: None,
            },
            Some(r"^([eE][nN](-[uU][sS])?|[iI][dD](-[iI][dD])?)$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(match self.0 {
            Locale::En => "en",
            Locale::Id => "id",
        }))
    }
}

/// Validated IANA timezone with aliases canonicalised by chrono-tz (CAN-10).
#[derive(Debug)]
pub struct TimeZoneField(chrono_tz::Tz);

impl TimeZoneField {
    /// Returns the canonical IANA name.
    pub fn name(&self) -> &'static str {
        self.0.name()
    }
}

impl FieldKind for TimeZoneField {
    type Input = String;
    const NAME: &'static str = "TimeZone";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        chrono_tz::Tz::from_str(&raw)
            .map(Self)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "time_zone")))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(1),
                max_length: Some(64),
                minimum: None,
                maximum: None,
            },
            None,
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.name()))
    }
}

/// Typed canonical UUID field (CAN-10).
#[derive(Debug)]
pub struct IdField(Uuid);

impl IdField {
    /// Returns the validated UUID.
    pub const fn value(&self) -> Uuid {
        self.0
    }
}

impl FieldKind for IdField {
    type Input = String;
    const NAME: &'static str = "Id";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        parse_canonical_uuid(&raw)
            .map(Self)
            .ok_or_else(|| one(FieldIssue::bound(INVALID_FORMAT, "format", "uuid")))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(ID_MIN),
                max_length: Some(ID_MAX),
                minimum: None,
                maximum: None,
            },
            Some("^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.hyphenated().to_string().to_ascii_lowercase()))
    }
}

fn parse_canonical_uuid(raw: &str) -> Option<Uuid> {
    let bytes = raw.as_bytes();
    let canonical_shape = bytes.len() == ID_MIN
        && [8, 13, 18, 23].iter().all(|position| bytes[*position] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit());
    canonical_shape.then(|| Uuid::parse_str(raw).ok()).flatten()
}

/// Shared typed local date with strict canonical input (CAN-10).
#[derive(Debug)]
pub struct LocalDateField(NaiveDate);

impl FieldKind for LocalDateField {
    type Input = String;
    const NAME: &'static str = "LocalDate";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let value = NaiveDate::parse_from_str(&raw, "%Y-%m-%d")
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "date")))?;
        if value.format("%Y-%m-%d").to_string() != raw {
            return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", "date")));
        }
        let minimum = NaiveDate::parse_from_str(LOCAL_DATE_MIN, "%Y-%m-%d")
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "date")))?;
        let maximum = NaiveDate::parse_from_str(LOCAL_DATE_MAX, "%Y-%m-%d")
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "date")))?;
        if value < minimum {
            return Err(one(FieldIssue::bound(MUST_BE_GTE, "min", LOCAL_DATE_MIN)));
        }
        if value > maximum {
            return Err(one(FieldIssue::bound(MUST_BE_LTE, "max", LOCAL_DATE_MAX)));
        }
        Ok(Self(value))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(10),
                max_length: Some(10),
                minimum: None,
                maximum: None,
            },
            Some(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.format("%Y-%m-%d").to_string()))
    }
}

/// Tenant-local birth date bounded above by the request context date (VAL-21).
#[derive(Debug)]
pub struct BirthDate(NaiveDate);

impl FieldKind for BirthDate {
    type Input = String;
    const NAME: &'static str = "BirthDate";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let date = LocalDateField::parse(raw, ctx)
            .map_err(|mut issues| {
                for issue in &mut issues {
                    issue.pointer = None;
                }
                issues
            })?
            .0;
        if date > ctx.today {
            return Err(one(FieldIssue::bound(
                MUST_BE_LTE,
                "max",
                ctx.today.format("%Y-%m-%d").to_string(),
            )));
        }
        Ok(Self(date))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(10),
                max_length: Some(10),
                minimum: None,
                maximum: None,
            },
            Some(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.format("%Y-%m-%d").to_string()))
    }
}

/// Canonical local time; 24:00 is handled by interval adapters (CAN-10).
#[derive(Debug)]
pub struct LocalTimeField(LocalTime);

impl FieldKind for LocalTimeField {
    type Input = String;
    const NAME: &'static str = "LocalTime";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        LocalTime::new(&raw)
            .map(Self)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "time")))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(5),
                max_length: Some(8),
                minimum: None,
                maximum: None,
            },
            Some(r"^(?:[01][0-9]|2[0-3]):[0-5][0-9](?::00)?$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.as_hh_mm()))
    }
}

/// Canonical UTC instant parsed from an RFC 3339 value with an explicit offset (CAN-10).
#[derive(Debug)]
pub struct InstantField(DateTime<Utc>);

impl FieldKind for InstantField {
    type Input = String;
    const NAME: &'static str = "Instant";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: String, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        if !has_explicit_offset(&raw) || fractional_digits(&raw).is_some_and(|digits| digits > 6) {
            return Err(one(FieldIssue::bound(INVALID_FORMAT, "format", "date-time")));
        }
        let parsed = DateTime::<FixedOffset>::parse_from_rfc3339(&raw)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "date-time")))?;
        let value = parsed.with_timezone(&Utc);
        let minimum = DateTime::parse_from_rfc3339(INSTANT_MIN)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "date-time")))?
            .with_timezone(&Utc);
        let maximum = DateTime::parse_from_rfc3339(INSTANT_MAX)
            .map_err(|_| one(FieldIssue::bound(INVALID_FORMAT, "format", "date-time")))?
            .with_timezone(&Utc);
        if value < minimum {
            return Err(one(FieldIssue::bound(MUST_BE_GTE, "min", INSTANT_MIN)));
        }
        if value > maximum {
            return Err(one(FieldIssue::bound(MUST_BE_LTE, "max", INSTANT_MAX)));
        }
        Ok(Self(value))
    }

    fn schema() -> Schema {
        string_schema(
            Self::NAME,
            Self::PROFILE,
            SchemaBounds {
                min_length: Some(20),
                max_length: Some(35),
                minimum: None,
                maximum: None,
            },
            Some(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,6})?(?:Z|[+-]\d{2}:\d{2})$"),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)))
    }
}

fn has_explicit_offset(value: &str) -> bool {
    value.ends_with('Z')
        || value
            .rfind(['+', '-'])
            .is_some_and(|index| index > value.find('T').unwrap_or(0))
}

fn fractional_digits(value: &str) -> Option<usize> {
    let time = value.split_once('T')?.1;
    let dot = time.find('.')?;
    Some(time[dot + 1..].chars().take_while(char::is_ascii_digit).count())
}

/// Bounded integer duration with a compile-time field range (VAL-21).
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "i64")]
#[derive(Debug)]
pub struct Minutes<const MIN: u32, const MAX: u32>(u32);

impl<const MIN: u32, const MAX: u32> Minutes<MIN, MAX> {
    /// Validates and constructs a bounded minute value.
    pub fn new(value: u32) -> Result<Self, &'static str> {
        if MIN > MAX || value < MIN || value > MAX {
            return Err("minutes out of range");
        }
        Ok(Self(value))
    }

    /// Returns the validated minute count.
    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl<const MIN: u32, const MAX: u32> TryFrom<i64> for Minutes<MIN, MAX> {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = u32::try_from(value).map_err(|_| "minutes out of range")?;
        Self::new(value)
    }
}

impl<const MIN: u32, const MAX: u32> FieldKind for Minutes<MIN, MAX> {
    type Input = i64;
    const NAME: &'static str = "Minutes";
    const PROFILE: Profile = Profile::None;

    fn component_name() -> Cow<'static, str> {
        Cow::Owned(format!("Minutes{MIN}_{MAX}"))
    }

    fn parse(raw: i64, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        if raw < i64::from(MIN) {
            return Err(one(FieldIssue::bound(MUST_BE_GTE, "min", MIN)));
        }
        if raw > i64::from(MAX) {
            return Err(one(FieldIssue::bound(MUST_BE_LTE, "max", MAX)));
        }
        let value = u32::try_from(raw).map_err(|_| one(FieldIssue::bound(MUST_BE_GTE, "min", MIN)))?;
        Self::new(value).map_err(|_| one(FieldIssue::bound(MUST_BE_LTE, "max", MAX)))
    }

    fn schema() -> Schema {
        number_schema(
            Self::NAME,
            Self::PROFILE,
            Some(i64::from(MIN)),
            Some(i64::from(MAX)),
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0))
    }
}

/// Non-negative tenant-currency minor amount (CAN-10).
#[derive(Debug)]
pub struct MoneyAmount(Money);

impl FieldKind for MoneyAmount {
    type Input = i64;
    const NAME: &'static str = "MoneyAmount";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: i64, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        if raw < 0 {
            return Err(one(FieldIssue::bound(MUST_BE_GTE, "min", 0)));
        }
        if raw > MONEY_MAX_ABS_MINOR {
            return Err(one(FieldIssue::bound(MUST_BE_LTE, "max", MONEY_MAX_ABS_MINOR)));
        }
        Money::new(raw, ctx.tenant_currency)
            .map(Self)
            .map_err(|_| one(FieldIssue::bound(MUST_BE_LTE, "max", MONEY_MAX_ABS_MINOR)))
    }

    fn schema() -> Schema {
        number_schema(Self::NAME, Self::PROFILE, Some(0), Some(MONEY_MAX_ABS_MINOR))
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!({
            "amount_minor": self.0.amount_minor(),
            "currency": self.0.currency().code()
        }))
    }
}

/// Basis points in the canonical range 0 through 10000 (CAN-10).
#[derive(Debug)]
pub struct BpsField(Bps);

impl FieldKind for BpsField {
    type Input = i64;
    const NAME: &'static str = "Bps";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: i64, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        if raw < 0 {
            return Err(one(FieldIssue::bound(MUST_BE_GTE, "min", 0)));
        }
        if raw > BPS_MAX {
            return Err(one(FieldIssue::bound(MUST_BE_LTE, "max", BPS_MAX)));
        }
        let value = u32::try_from(raw).map_err(|_| one(FieldIssue::bound(MUST_BE_LTE, "max", BPS_MAX)))?;
        Bps::new(value)
            .map(Self)
            .map_err(|_| one(FieldIssue::bound(MUST_BE_LTE, "max", BPS_MAX)))
    }

    fn schema() -> Schema {
        number_schema(Self::NAME, Self::PROFILE, Some(0), Some(BPS_MAX))
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0.get()))
    }
}

/// Geo JSON input matching CAN-10.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GeoInput {
    /// Latitude in WGS 84 degrees.
    pub lat: f64,
    /// Longitude in WGS 84 degrees.
    pub lng: f64,
}

/// Validated and six-decimal rounded geographic point (CAN-10).
#[derive(Debug)]
pub struct GeoField(GeoPoint);

impl FieldKind for GeoField {
    type Input = GeoInput;
    const NAME: &'static str = "Geo";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: GeoInput, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        if !(-90.0..=90.0).contains(&raw.lat) {
            return Err(one(FieldIssue::bound(
                if raw.lat < -90.0 { MUST_BE_GTE } else { MUST_BE_LTE },
                if raw.lat < -90.0 { "min" } else { "max" },
                if raw.lat < -90.0 { -90 } else { 90 },
            )));
        }
        if !(-180.0..=180.0).contains(&raw.lng) {
            return Err(one(FieldIssue::bound(
                if raw.lng < -180.0 {
                    MUST_BE_GTE
                } else {
                    MUST_BE_LTE
                },
                if raw.lng < -180.0 { "min" } else { "max" },
                if raw.lng < -180.0 { -180 } else { 180 },
            )));
        }
        let lat = round_geo(raw.lat);
        let lng = round_geo(raw.lng);
        GeoPoint::new(lat, lng)
            .map(Self)
            .map_err(|_| one(FieldIssue::new(INVALID_FORMAT)))
    }

    fn schema() -> Schema {
        object_schema(
            Self::NAME,
            Self::PROFILE,
            [
                (
                    "lat",
                    number_schema("Latitude", Profile::None, Some(-90), Some(90)),
                ),
                (
                    "lng",
                    number_schema("Longitude", Profile::None, Some(-180), Some(180)),
                ),
            ],
        )
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!({"lat": self.0.lat(), "lng": self.0.lng()}))
    }
}

fn round_geo(value: f64) -> f64 {
    let rounded = (value * GEO_SCALE).round() / GEO_SCALE;
    if rounded == 0.0 { 0.0 } else { rounded }
}

/// Hue integer in the inclusive range 0 through 359 (VAL-21).
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "i64")]
#[derive(Debug)]
pub struct Hue(u16);

impl TryFrom<i64> for Hue {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if !(HUE_MIN..=HUE_MAX).contains(&value) {
            return Err("hue out of range");
        }
        Ok(Self(u16::try_from(value).map_err(|_| "hue out of range")?))
    }
}

impl FieldKind for Hue {
    type Input = i64;
    const NAME: &'static str = "Hue";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: i64, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        bounded_integer(raw, HUE_MIN, HUE_MAX)?;
        Self::try_from(raw).map_err(|_| one(FieldIssue::bound(MUST_BE_LTE, "max", HUE_MAX)))
    }

    fn schema() -> Schema {
        number_schema(Self::NAME, Self::PROFILE, Some(HUE_MIN), Some(HUE_MAX))
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0))
    }
}

/// Star rating integer in the inclusive range 1 through 5 (VAL-21).
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "i64")]
#[derive(Debug)]
pub struct Stars(u8);

impl TryFrom<i64> for Stars {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if !(STARS_MIN..=STARS_MAX).contains(&value) {
            return Err("stars out of range");
        }
        Ok(Self(u8::try_from(value).map_err(|_| "stars out of range")?))
    }
}

impl FieldKind for Stars {
    type Input = i64;
    const NAME: &'static str = "Stars";
    const PROFILE: Profile = Profile::None;

    fn parse(raw: i64, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        bounded_integer(raw, STARS_MIN, STARS_MAX)?;
        Self::try_from(raw).map_err(|_| one(FieldIssue::bound(MUST_BE_LTE, "max", STARS_MAX)))
    }

    fn schema() -> Schema {
        number_schema(Self::NAME, Self::PROFILE, Some(STARS_MIN), Some(STARS_MAX))
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(self.0))
    }
}

fn bounded_integer(value: i64, min: i64, max: i64) -> Result<(), Vec<FieldIssue>> {
    if value < min {
        Err(one(FieldIssue::bound(MUST_BE_GTE, "min", min)))
    } else if value > max {
        Err(one(FieldIssue::bound(MUST_BE_LTE, "max", max)))
    } else {
        Ok(())
    }
}

/// Internationalised line map, checked against supported locales and tenant defaults (VAL-21).
pub struct I18nLine<const MAX: usize>(BTreeMap<Locale, String>);

impl<const MAX: usize> fmt::Debug for I18nLine<MAX> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("I18nLine([REDACTED])")
    }
}

impl<const MAX: usize> I18nLine<MAX> {
    /// Returns validated locale strings.
    pub fn values(&self) -> &BTreeMap<Locale, String> {
        &self.0
    }
}

impl<const MAX: usize> FieldKind for I18nLine<MAX> {
    type Input = BTreeMap<String, String>;
    const NAME: &'static str = "I18nLine";
    const PROFILE: Profile = Profile::Line;

    fn component_name() -> Cow<'static, str> {
        Cow::Owned(format!("I18nLine{MAX}"))
    }

    fn parse(raw: Self::Input, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        let mut values = BTreeMap::new();
        let mut issues = Vec::new();
        let mut default_locale_present = false;
        let mut default_locale_empty = false;
        for (key, value) in raw {
            let locale = match key.as_str() {
                "en" => Locale::En,
                "id" => Locale::Id,
                _ => {
                    issues.push(
                        FieldIssue {
                            code: NOT_ALLOWED,
                            params: [("allowed".to_owned(), json!(["en", "id"]))]
                                .into_iter()
                                .collect(),
                            pointer: None,
                        }
                        .at(format!("/{}", super::escape_pointer_segment(&key))),
                    );
                    continue;
                }
            };
            if locale == ctx.tenant_locale {
                default_locale_present = true;
            }
            match sanitize(&value, Self::PROFILE) {
                Ok(canonical) if canonical.is_empty() => {
                    if locale == ctx.tenant_locale {
                        default_locale_empty = true;
                    }
                }
                Ok(canonical) => {
                    let length = canonical.chars().count();
                    if length > MAX {
                        issues.push(FieldIssue::bound(TOO_LONG, "max", MAX).at(format!("/{}", key)));
                    } else {
                        values.insert(locale, canonical);
                    }
                }
                Err(issue) => issues.push(issue.at(format!("/{}", key))),
            }
        }
        if !values.contains_key(&ctx.tenant_locale) && (!default_locale_present || default_locale_empty) {
            let locale = match ctx.tenant_locale {
                Locale::En => "en",
                Locale::Id => "id",
            };
            issues.push(FieldIssue::new(REQUIRED).at(format!("/{locale}")));
        }
        if issues.is_empty() {
            Ok(Self(values))
        } else {
            Err(issues)
        }
    }

    fn schema() -> Schema {
        let extensions = utoipa::openapi::extensions::Extensions::from_iter([
            ("x-sanitize", json!(Self::PROFILE.as_str())),
            ("x-field-kind", json!(Self::NAME)),
        ]);
        ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::Object))
            .title(Some(Self::NAME))
            .property(
                "en",
                string_schema(
                    "I18nLineValue",
                    Self::PROFILE,
                    SchemaBounds {
                        min_length: Some(1),
                        max_length: Some(MAX),
                        minimum: None,
                        maximum: None,
                    },
                    None,
                ),
            )
            .property(
                "id",
                string_schema(
                    "I18nLineValue",
                    Self::PROFILE,
                    SchemaBounds {
                        min_length: Some(1),
                        max_length: Some(MAX),
                        minimum: None,
                        maximum: None,
                    },
                    None,
                ),
            )
            .min_properties(Some(1))
            .max_properties(Some(2))
            .additional_properties(Some(AdditionalProperties::FreeForm(false)))
            .extensions(Some(extensions))
            .build()
            .into()
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(
            self.0
                .iter()
                .map(|(locale, value)| {
                    (
                        match locale {
                            Locale::En => "en",
                            Locale::Id => "id",
                        }
                        .to_owned(),
                        json!(value),
                    )
                })
                .collect::<serde_json::Map<String, Value>>()
                .into(),
        )
    }
}

/// Bounded, canonical UUID list with duplicate rejection (VAL-21).
#[derive(Serialize, PartialEq, Eq, Debug)]
pub struct IdList<const MAX: usize>(Vec<Uuid>);

impl<const MAX: usize> IdList<MAX> {
    /// Returns the validated UUID values.
    pub fn values(&self) -> &[Uuid] {
        &self.0
    }
}

impl<const MAX: usize> TryFrom<Vec<Uuid>> for IdList<MAX> {
    type Error = &'static str;

    fn try_from(values: Vec<Uuid>) -> Result<Self, Self::Error> {
        if values.len() > MAX {
            return Err("too many identifiers");
        }
        let unique: std::collections::HashSet<_> = values.iter().collect();
        if unique.len() != values.len() {
            return Err("duplicate identifiers");
        }
        Ok(Self(values))
    }
}

impl<'de, const MAX: usize> Deserialize<'de> for IdList<MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = Vec::<String>::deserialize(deserializer)?;
        let mut values = Vec::with_capacity(raw.len());
        for value in raw {
            let id = parse_canonical_uuid(&value)
                .ok_or_else(|| D::Error::custom("invalid canonical identifier"))?;
            values.push(id);
        }
        Self::try_from(values).map_err(D::Error::custom)
    }
}

impl<const MAX: usize> FieldKind for IdList<MAX> {
    type Input = Vec<String>;
    const NAME: &'static str = "IdList";
    const PROFILE: Profile = Profile::None;

    fn component_name() -> Cow<'static, str> {
        Cow::Owned(format!("IdList{MAX}"))
    }

    fn parse(raw: Self::Input, _ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>> {
        if raw.len() > MAX {
            return Err(one(FieldIssue::bound(TOO_MANY_ITEMS, "max", MAX)));
        }
        let mut values = Vec::with_capacity(raw.len());
        let mut issues = Vec::new();
        for (index, raw_id) in raw.into_iter().enumerate() {
            match IdField::parse(raw_id, _ctx) {
                Ok(id) => values.push(id.0),
                Err(mut errors) => {
                    for error in &mut errors {
                        error.pointer = Some(format!("/{index}"));
                    }
                    issues.extend(errors);
                }
            }
        }
        if !issues.is_empty() {
            return Err(issues);
        }
        let unique: std::collections::HashSet<_> = values.iter().collect();
        if unique.len() != values.len() {
            return Err(one(FieldIssue::new(crate::errors::NOT_UNIQUE)));
        }
        Ok(Self(values))
    }

    fn schema() -> Schema {
        array_schema(Self::NAME, Self::PROFILE, IdField::schema(), 0, MAX, true)
    }

    fn canonical_json(&self) -> Option<Value> {
        Some(json!(
            self.0
                .iter()
                .map(|id| id.hyphenated())
                .map(|id| id.to_string().to_ascii_lowercase())
                .collect::<Vec<_>>()
        ))
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;
    use crate::errors::INVALID_CHARACTERS;

    #[test]
    fn display_name_collapses_whitespace() {
        let context = test_context();
        let value = DisplayName::parse("  Ayu   Sari  ".to_owned(), &context).expect("valid name");
        assert_eq!(value.as_ref(), "Ayu Sari");
    }

    #[test]
    fn rejects_bidi_override() {
        let context = test_context();
        let issues = DisplayName::parse("Ayu\u{202e}Sari".to_owned(), &context)
            .err()
            .expect("bidi override is forbidden");
        assert_eq!(issues[0].code, INVALID_CHARACTERS);
    }

    #[test]
    fn email_canonical() {
        let context = test_context();
        let email = Email::parse("Budi@Example.COM ".to_owned(), &context).expect("valid email");
        assert_eq!(email.as_ref(), "budi@example.com");
        let issue = Email::parse("budi@[127.0.0.1]".to_owned(), &context)
            .err()
            .expect("IP literal rejected");
        assert_eq!(issue[0].code, INVALID_FORMAT);
        let repeated_dot = Email::parse("a@b..".to_owned(), &context)
            .err()
            .expect("only one trailing domain dot may be removed");
        assert_eq!(repeated_dot[0].code, INVALID_FORMAT);
    }

    #[test]
    fn phone_national_to_e164() {
        let context = test_context();
        let phone = Phone::parse("0812-3456-7890".to_owned(), &context).expect("valid fake phone");
        assert_eq!(phone.as_str(), "+6281234567890");
        let invalid = "+000000000000".to_owned();
        assert!(Phone::try_from(invalid.clone()).is_err());
        assert!(serde_json::from_value::<Phone>(json!(invalid)).is_err());
    }

    #[test]
    fn phone_requires_country_context() {
        let context = FieldCtx {
            country: None,
            ..test_context()
        };
        let issues = Phone::parse("0812-3456-7890".to_owned(), &context)
            .expect_err("national phone parsing requires a country");
        assert_eq!(issues[0].code, INVALID_FORMAT);

        let email =
            Email::parse("Budi@Example.COM".to_owned(), &context).expect("email ignores country context");
        assert_eq!(email.as_ref(), "budi@example.com");
    }

    #[test]
    fn identifier_whitespace_is_invalid_format() {
        let context = test_context();
        let key = Key::parse("a b".to_owned(), &context)
            .err()
            .expect("keys cannot contain whitespace");
        let slug = Slug::parse("a b".to_owned(), &context)
            .err()
            .expect("slugs cannot contain whitespace");
        assert_eq!(key[0].code, INVALID_FORMAT);
        assert_eq!(slug[0].code, INVALID_FORMAT);
    }

    #[test]
    fn instant_requires_offset() {
        let context = test_context();
        assert!(InstantField::parse("2026-11-05T15:00:00".to_owned(), &context).is_err());
        assert!(InstantField::parse("2026-11-05T15:00:00+07:00".to_owned(), &context).is_ok());
    }

    #[test]
    fn password_debug_is_redacted() {
        let context = test_context();
        let raw = serde_json::from_str::<crate::fields::Raw<Password>>(r#""test-secret-password""#)
            .expect("fake test password deserializes into a secrecy wrapper");
        let raw_debug = format!("{raw:?}");
        assert!(!raw_debug.contains("test-secret-password"));
        let input_debug = format!("{:?}", raw.0);
        assert!(!input_debug.contains("test-secret-password"));
        let password = Password::parse(raw.0, &context).expect("fake test password valid");
        let rendered = format!("{password:?}");
        assert!(!rendered.contains(password.0.expose_secret()));
    }

    #[test]
    fn password_policy_minimum_rejects_short_value() {
        let mut context = test_context();
        context.password_min_length = Some(8);
        let raw = serde_json::from_str::<crate::fields::Raw<Password>>(r#""x""#)
            .expect("fake password deserializes into a secrecy wrapper");
        let issue = Password::parse(raw.0, &context).expect_err("policy minimum is enforced");
        assert_eq!(issue[0].code, TOO_SHORT);
        assert_eq!(issue[0].params["min"], 8);
    }

    #[test]
    fn validated_newtypes_deserialize_through_their_invariants() {
        let name: DisplayName = serde_json::from_str(r#""  Ayu   Sari  ""#)
            .expect("line profile normalizes text during deserialization");
        assert_eq!(name.as_ref(), "Ayu Sari");
        assert!(serde_json::from_str::<Key>(r#"" ab ""#).is_err());
        assert!(serde_json::from_str::<Key>(r#""a-b""#).is_err());
        assert!(serde_json::from_str::<Slug>(r#""a--b""#).is_err());

        let ids = serde_json::from_str::<IdList<4>>(r#"["0192f3a4-1234-5678-9abc-def012345678"]"#)
            .expect("canonical ids deserialize");
        assert_eq!(ids.values().len(), 1);
        assert!(serde_json::from_str::<IdList<4>>(r#"["{0192f3a4-1234-5678-9abc-def012345678}"]"#).is_err());
    }

    fn test_context() -> FieldCtx {
        FieldCtx {
            country: Some(celes::Country::from_alpha2("ID").expect("known test country")),
            tenant_currency: Currency::IDR,
            tenant_locale: Locale::Id,
            today: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
            password_min_length: Some(8),
        }
    }
}
