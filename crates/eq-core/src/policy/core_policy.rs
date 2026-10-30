//! Core tenant policy values and defaults (POL-06, POL-07, POL-12).

use std::collections::BTreeMap;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{FieldError, INVALID_TYPE};
use crate::module::{ModuleId, SummaryLine};
use crate::money::Currency;
use crate::policy::PolicyDoc;
use crate::policy::types::{I18nText, Locale, PolicyTypeError};

/// A validated hue in the design-system range (DS-10, CQ-23).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(try_from = "u16")]
#[schemars(with = "BrandHueSchema")]
#[schema(value_type = u16, minimum = 0, maximum = 359)]
pub struct BrandHue(u16);

#[derive(JsonSchema)]
#[schemars(rename = "BrandHue", transparent)]
#[expect(dead_code, reason = "Schemars reads the validated BrandHue range")]
struct BrandHueSchema(#[schemars(range(min = 0, max = 359))] u16);

impl BrandHue {
    /// Validates a hue from 0 through 359 (DS-10).
    pub fn new(value: u16) -> Result<Self, PolicyTypeError> {
        if value > 359 {
            return Err(PolicyTypeError::Invalid("brand hue"));
        }
        Ok(Self(value))
    }

    /// Returns the canonical hue value.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for BrandHue {
    type Error = PolicyTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Validated statutory deletion service-level days (POL-06, CQ-23).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(try_from = "u16")]
#[schemars(with = "DeletionSlaBusinessDaysSchema")]
#[schema(value_type = u16, minimum = 1, maximum = 30)]
pub struct DeletionSlaBusinessDays(u16);

#[derive(JsonSchema)]
#[schemars(rename = "DeletionSlaBusinessDays", transparent)]
#[expect(dead_code, reason = "Schemars reads the validated deletion SLA range")]
struct DeletionSlaBusinessDaysSchema(#[schemars(range(min = 1, max = 30))] u16);

impl DeletionSlaBusinessDays {
    /// Validates the policy's 1 through 30 business-day range (POL-06).
    pub fn new(value: u16) -> Result<Self, PolicyTypeError> {
        if !(1..=30).contains(&value) {
            return Err(PolicyTypeError::Invalid("deletion SLA business days"));
        }
        Ok(Self(value))
    }

    /// Returns the canonical number of business days.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for DeletionSlaBusinessDays {
    type Error = PolicyTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Validated attendance-photo retention days (POL-06, CQ-23).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(try_from = "u16")]
#[schemars(with = "PhotoRetentionDaysSchema")]
#[schema(value_type = u16, minimum = 7, maximum = 3650)]
pub struct PhotoRetentionDays(u16);

#[derive(JsonSchema)]
#[schemars(rename = "PhotoRetentionDays", transparent)]
#[expect(dead_code, reason = "Schemars reads the validated photo retention range")]
struct PhotoRetentionDaysSchema(#[schemars(range(min = 7, max = 3650))] u16);

impl PhotoRetentionDays {
    /// Validates a retention period from 7 through 3650 days (POL-06).
    pub fn new(value: u16) -> Result<Self, PolicyTypeError> {
        if !(7..=3650).contains(&value) {
            return Err(PolicyTypeError::Invalid("photo retention days"));
        }
        Ok(Self(value))
    }

    /// Returns the canonical retention period.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PhotoRetentionDays {
    type Error = PolicyTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// IANA timezone name serialized as its canonical identifier (CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
#[schema(value_type = String)]
pub struct TzName(chrono_tz::Tz);

impl TzName {
    /// Parses a canonical IANA timezone name.
    pub fn new(value: &str) -> Result<Self, PolicyTypeError> {
        chrono_tz::Tz::from_str(value)
            .map(Self)
            .map_err(|_| PolicyTypeError::Invalid("timezone"))
    }

    /// Returns the canonical IANA name.
    pub fn as_str(self) -> &'static str {
        self.0.name()
    }
}

impl TryFrom<String> for TzName {
    type Error = PolicyTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

/// The supported Core week-start choices (POL-06).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WeekStartsOn {
    /// Monday.
    #[default]
    Mon,
    /// Sunday.
    Sun,
}

/// Tenant branding corner token (POL-06).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CornerStyle {
    /// Square corners.
    Sharp,
    /// Product default corners.
    #[default]
    Default,
    /// Rounded corners.
    Round,
}

/// Localized tenant vocabulary replacements (POL-06).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Terminology {
    /// Per-term localized replacements.
    pub overrides: BTreeMap<TermKey, I18nText>,
}

/// Closed set of vocabulary keys (POL-06).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TermKey {
    /// Instructor role term.
    Instructor,
    /// Guardian role term.
    Guardian,
    /// Student role term.
    Student,
    /// Offering term.
    Offering,
    /// Session term.
    Session,
    /// Resource term.
    Resource,
    /// Location term.
    Location,
    /// Enrollment term.
    Enrollment,
}

/// Core branding settings (POL-06).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Branding {
    /// DS-10 hue from 0 through 359.
    pub brand_hue: BrandHue,
    /// Semantic corner token.
    pub corner_style: CornerStyle,
    /// Tenant-local logo file id; until file delivery is implemented, APIs return no public URL.
    pub logo_file_id: Option<Uuid>,
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            brand_hue: BrandHue::new(255).unwrap_or(BrandHue(255)),
            corner_style: CornerStyle::Default,
            logo_file_id: None,
        }
    }
}

/// Privacy defaults and retention values (POL-06).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrivacyPolicy {
    /// Deletion SLA expressed in business days.
    pub deletion_sla_business_days: DeletionSlaBusinessDays,
    /// Whether invoices prevent account deletion.
    pub block_deletion_with_open_invoices: bool,
    /// Attendance-photo retention in days.
    pub photo_retention_days: PhotoRetentionDays,
}

impl Default for PrivacyPolicy {
    fn default() -> Self {
        Self {
            deletion_sla_business_days: DeletionSlaBusinessDays::new(14)
                .unwrap_or(DeletionSlaBusinessDays(14)),
            block_deletion_with_open_invoices: true,
            photo_retention_days: PhotoRetentionDays::new(365).unwrap_or(PhotoRetentionDays(365)),
        }
    }
}

/// Core tenant-wide policy document (POL-06, POL-07, POL-12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CorePolicy {
    /// Default tenant language.
    pub locale: Locale,
    /// ISO 4217 currency used by policy amounts.
    pub currency: Currency,
    /// Tenant's IANA timezone.
    pub timezone: TzName,
    /// First weekday shown by scheduling UI.
    pub week_starts_on: WeekStartsOn,
    /// Tenant-specific terminology.
    pub terminology: Terminology,
    /// Privacy and retention policy.
    pub privacy: PrivacyPolicy,
    /// Branding rendered by the tenant UI.
    pub branding: Branding,
}

impl Default for CorePolicy {
    fn default() -> Self {
        Self {
            locale: Locale::En,
            currency: Currency::USD,
            timezone: TzName(chrono_tz::UTC),
            week_starts_on: WeekStartsOn::Mon,
            terminology: Terminology::default(),
            privacy: PrivacyPolicy::default(),
            branding: Branding::default(),
        }
    }
}

impl CorePolicy {
    /// Returns a schema with canonical timezone values and renderer hints (POL-06).
    pub fn json_schema() -> Value {
        let Ok(mut schema) = serde_json::to_value(schemars::schema_for!(Self)) else {
            return json!({"not": {}});
        };
        let timezones = chrono_tz::TZ_VARIANTS
            .iter()
            .map(|timezone| json!(timezone.name()))
            .collect::<Vec<_>>();
        schema["properties"]["timezone"] = json!({"type": "string", "enum": timezones});
        let fields = [
            ("locale", 1),
            ("currency", 2),
            ("timezone", 3),
            ("week_starts_on", 4),
            ("terminology", 5),
            ("privacy", 6),
            ("branding", 7),
        ];
        for (field, order) in fields {
            if let Some(property) = schema["properties"][field].as_object_mut() {
                property.insert(
                    "x-ui".to_owned(),
                    crate::policy::XUi::new(crate::policy::Group::Advanced, order).into_value(),
                );
            }
        }
        if let Some(branding) = schema["properties"]["branding"].as_object_mut() {
            branding.insert(
                "x-ui".to_owned(),
                crate::policy::XUi::new(crate::policy::Group::Advanced, 7)
                    .widget(crate::policy::Widget::Hidden)
                    .into_value(),
            );
        }
        schema
    }
}

impl PolicyDoc for CorePolicy {
    const MODULE: ModuleId = ModuleId::Core;
    const SCHEMA_VERSION: u32 = 1;

    fn validate(&self) -> Result<(), Vec<FieldError>> {
        Ok(())
    }

    fn summarize(&self) -> Vec<SummaryLine> {
        let mut locale_currency = Map::new();
        locale_currency.insert("locale".to_owned(), json!(self.locale));
        locale_currency.insert("currency".to_owned(), json!(self.currency.code()));
        let mut timezone = Map::new();
        timezone.insert("timezone".to_owned(), json!(self.timezone.as_str()));
        vec![
            SummaryLine {
                key: "policy.core.summary.locale_currency".to_owned(),
                args: locale_currency,
            },
            SummaryLine {
                key: "policy.core.summary.timezone".to_owned(),
                args: timezone,
            },
        ]
    }
}

/// POL-10: validates a dynamic Core document for the Core module manifest.
pub fn validate_core_document(document: &Value) -> Result<(), Vec<FieldError>> {
    match crate::policy::validate_document::<CorePolicy>(document.clone()) {
        Ok(_) => Ok(()),
        Err(crate::error::AppError::Validation { errors }) => Err(errors),
        Err(_) => Err(vec![FieldError {
            pointer: String::new(),
            code: INVALID_TYPE.to_owned(),
            params: Map::new(),
        }]),
    }
}

#[cfg(test)]
mod tests {
    use super::{BrandHue, CorePolicy, DeletionSlaBusinessDays, PhotoRetentionDays, TzName, WeekStartsOn};
    use crate::policy::PolicyDoc;
    use serde_json::json;

    #[test]
    fn defaults_schema_and_summary_match_contract() {
        let policy = CorePolicy::default();
        assert_eq!(policy.locale, crate::policy::types::Locale::En);
        assert_eq!(policy.currency.code(), "USD");
        assert_eq!(policy.timezone.as_str(), "UTC");
        assert_eq!(policy.week_starts_on, WeekStartsOn::Mon);
        assert_eq!(policy.privacy.deletion_sla_business_days.get(), 14);
        assert!(policy.privacy.block_deletion_with_open_invoices);
        assert_eq!(policy.privacy.photo_retention_days.get(), 365);
        assert_eq!(policy.branding.brand_hue.get(), 255);
        assert_eq!(policy.summarize().len(), 2);
        let schema = CorePolicy::json_schema();
        assert!(schema["properties"]["timezone"]["enum"].as_array().is_some());
        assert_eq!(schema["properties"]["branding"]["x-ui"]["widget"], "hidden");
        assert!(crate::policy::validate_document::<CorePolicy>(json!({
            "locale": "en", "currency": "USD", "timezone": "UTC", "week_starts_on": "mon",
            "terminology": {"overrides": {}},
            "privacy": {"deletion_sla_business_days": 14, "block_deletion_with_open_invoices": true, "photo_retention_days": 365},
            "branding": {"brand_hue": 255, "corner_style": "default", "logo_file_id": null}
        }))
        .is_ok());
    }

    #[test]
    fn range_values_validate_at_construction_and_deserialization() {
        assert!(BrandHue::new(360).is_err());
        assert!(DeletionSlaBusinessDays::new(0).is_err());
        assert!(PhotoRetentionDays::new(6).is_err());
        assert!(TzName::new("Not/A_Timezone").is_err());
        assert!(serde_json::from_value::<BrandHue>(json!(360)).is_err());
        assert!(serde_json::from_value::<DeletionSlaBusinessDays>(json!(31)).is_err());
        assert!(serde_json::from_value::<PhotoRetentionDays>(json!(3651)).is_err());
        assert!(serde_json::from_value::<TzName>(json!("Not/A_Timezone")).is_err());
    }
}
