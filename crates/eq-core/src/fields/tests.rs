//! VAL-31: shared Rust-side validation corpus and schema parity tests.

use serde_json::Value;

use super::*;

#[test]
fn raw_field_schema_is_a_reference() {
    let schema = serde_json::to_value(<Raw<DisplayName> as PartialSchema>::schema())
        .expect("field schema reference serializes");
    assert_eq!(schema["$ref"], "#/components/schemas/DisplayName");

    assert_eq!(<Raw<Minutes<10, 480>> as ToSchema>::name(), "Minutes10_480");
    assert_eq!(<Raw<Minutes<30, 90>> as ToSchema>::name(), "Minutes30_90");
    assert_eq!(<Raw<I18nLine<80>> as ToSchema>::name(), "I18nLine80");
    assert_eq!(<Raw<IdList<25>> as ToSchema>::name(), "IdList25");
}

#[test]
fn schema_matches_parse_bounds() {
    use kinds::*;

    assert_schema::<DisplayName>();
    assert_schema::<LegalName>();
    assert_schema::<TenantName>();
    assert_schema::<PlaceName>();
    assert_schema::<ShortText>();
    assert_schema::<Address>();
    assert_schema::<Note>();
    assert_schema::<Comment>();
    assert_schema::<I18nLine<200>>();
    assert_schema::<Email>();
    assert_schema::<Phone>();
    assert_schema::<Password>();
    assert_schema::<Key>();
    assert_schema::<Slug>();
    assert_schema::<SearchQuery>();
    assert_schema::<IdField>();
    assert_schema::<LocalDateField>();
    assert_schema::<BirthDate>();
    assert_schema::<LocalTimeField>();
    assert_schema::<InstantField>();
    assert_schema::<Minutes<10, 480>>();
    assert_schema::<MoneyAmount>();
    assert_schema::<CurrencyField>();
    assert_schema::<BpsField>();
    assert_schema::<CountryField>();
    assert_schema::<TimeZoneField>();
    assert_schema::<LocaleField>();
    assert_schema::<GeoField>();
    assert_schema::<Hue>();
    assert_schema::<Stars>();
    assert_schema::<IdList<100>>();

    let display_name = serde_json::to_value(<DisplayName as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(display_name["minLength"], 1);
    assert_eq!(display_name["maxLength"], 80);
    assert_eq!(display_name["x-sanitize"], "line");
    assert_eq!(display_name["x-field-kind"], "DisplayName");

    let minutes = serde_json::to_value(<Minutes<10, 480> as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(minutes["minimum"], 10);
    assert_eq!(minutes["maximum"], 480);

    let money = serde_json::to_value(<MoneyAmount as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(money["minimum"], 0);
    assert_eq!(money["maximum"], crate::money::MONEY_MAX_ABS_MINOR);

    let key = serde_json::to_value(<Key as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(key["pattern"], r"^[a-z0-9_]{2,40}$");
    assert_eq!(key["minLength"], 2);
    assert_eq!(key["maxLength"], 40);

    let slug = serde_json::to_value(<Slug as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(slug["pattern"], r"^[a-z0-9](-?[a-z0-9])*$");

    let email = serde_json::to_value(<Email as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(
        email["pattern"],
        r"^\s*[^@\s]{1,64}@[^@\s.]+(?:\.[^@\s.]+)+\.?\s*$"
    );

    let ids = serde_json::to_value(<IdList<100> as FieldKind>::schema()).expect("schema serializes");
    let id = serde_json::to_value(<IdField as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(ids["items"]["pattern"], id["pattern"]);

    let geo = serde_json::to_value(<GeoField as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(geo["required"], serde_json::json!(["lat", "lng"]));
    assert_eq!(geo["additionalProperties"], false);

    let i18n = serde_json::to_value(<I18nLine<200> as FieldKind>::schema()).expect("schema serializes");
    assert_eq!(i18n["additionalProperties"], false);
    assert_eq!(i18n["minProperties"], 1);
    assert_eq!(i18n["maxProperties"], 2);
}

fn assert_schema<K: FieldKind>() {
    let schema = serde_json::to_value(K::schema()).expect("schema serializes");
    assert_eq!(schema["x-field-kind"], K::NAME);
    assert_eq!(schema["x-sanitize"], K::PROFILE.as_str());
}

#[test]
fn cases_match() {
    use kinds::*;

    assert_cases::<DisplayName>();
    assert_cases::<LegalName>();
    assert_cases::<TenantName>();
    assert_cases::<PlaceName>();
    assert_cases::<ShortText>();
    assert_cases::<Address>();
    assert_cases::<Note>();
    assert_cases::<Comment>();
    assert_cases::<I18nLine<200>>();
    assert_cases::<Email>();
    assert_cases::<Phone>();
    assert_cases::<Password>();
    assert_cases::<Key>();
    assert_cases::<Slug>();
    assert_cases::<SearchQuery>();
    assert_cases::<IdField>();
    assert_cases::<LocalDateField>();
    assert_cases::<BirthDate>();
    assert_cases::<LocalTimeField>();
    assert_cases::<InstantField>();
    assert_cases::<Minutes<10, 480>>();
    assert_cases::<MoneyAmount>();
    assert_cases::<CurrencyField>();
    assert_cases::<BpsField>();
    assert_cases::<CountryField>();
    assert_cases::<TimeZoneField>();
    assert_cases::<LocaleField>();
    assert_cases::<GeoField>();
    assert_cases::<Hue>();
    assert_cases::<Stars>();
    assert_cases::<IdList<100>>();
}

fn assert_cases<K: FieldKind>() {
    let path = format!(
        "{}/../../contracts/validation/cases/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        to_snake_case(K::NAME)
    );
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} case file {path} is unavailable: {error}", K::NAME));
    let cases: Vec<Value> = serde_json::from_str(&source).expect("case file is valid JSON");
    assert!(!cases.is_empty(), "{} has cases", K::NAME);
    for case in cases {
        let raw: K::Input = serde_json::from_value(case["input"].clone())
            .unwrap_or_else(|error| panic!("{} input is invalid: {error}", K::NAME));
        let expected_ok = case["ok"].as_bool().expect("case has boolean ok");
        match K::parse(raw, &test_context()) {
            Ok(value) => {
                assert!(expected_ok, "{} unexpectedly accepted {case}", K::NAME);
                if let Some(expected) = case.get("canonical") {
                    assert_eq!(
                        value.canonical_json().as_ref(),
                        Some(expected),
                        "{} canonical mismatch for {case}",
                        K::NAME
                    );
                }
            }
            Err(issues) => {
                assert!(!expected_ok, "{} unexpectedly rejected {case}", K::NAME);
                if let Some(expected) = case.get("codes") {
                    let actual = issues.iter().map(|issue| issue.code).collect::<Vec<_>>();
                    assert_eq!(
                        serde_json::to_value(actual).expect("codes serialize"),
                        *expected,
                        "{} issue codes mismatch for {case}",
                        K::NAME
                    );
                }
            }
        }
    }
}

fn to_snake_case(value: &str) -> String {
    let mut result = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() && index > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

fn test_context() -> FieldCtx {
    FieldCtx {
        country: celes::Country::from_alpha2("ID").expect("known test country"),
        tenant_currency: crate::money::Currency::IDR,
        tenant_locale: crate::policy::types::Locale::Id,
        today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
        password_min_length: Some(8),
    }
}
