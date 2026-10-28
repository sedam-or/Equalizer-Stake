//! Audit retention and administrator category policy (AUD-07, POL-01/06/07/10).

use eq_core::error::{FieldError, MUST_BE_GTE, MUST_BE_LTE};
use eq_core::module::{ModuleId, SummaryLine};
use eq_core::policy::{Group, PolicyDoc, XUi};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value, json};

const AUDIT_POLICY_SCHEMA_FIELDS: [(&str, u32); 3] = [
    ("retention_years", 1),
    ("anonymize_pii_after_years", 2),
    ("admin_categories", 3),
]; // technical: keep display order aligned with the policy field declaration order

/// AUD-07: audit entry category used by records and administrator policy.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// AUD-07: routine, non-security audit activity.
    Operational,
    /// AUD-07: security-relevant activity.
    Security,
    /// AUD-07: compliance-related activity.
    Compliance,
}

impl Category {
    /// AUD-07: returns the canonical database representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Security => "security",
            Self::Compliance => "compliance",
        }
    }
}

/// AUD-07: category alias used by the audit policy contract.
pub type AuditCategory = Category;

/// AUD-07: versioned audit retention and administrator visibility settings.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditPolicy {
    #[schemars(range(min = 1, max = 10))]
    retention_years: u8,
    #[schemars(range(min = 1, max = 10))]
    anonymize_pii_after_years: u8,
    admin_categories: Vec<AuditCategory>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditPolicyWire {
    retention_years: u8,
    anonymize_pii_after_years: u8,
    admin_categories: Vec<AuditCategory>,
}

impl<'de> Deserialize<'de> for AuditPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AuditPolicyWire::deserialize(deserializer)?;
        Self::new(
            wire.retention_years,
            wire.anonymize_pii_after_years,
            wire.admin_categories,
        )
        .map_err(|_| serde::de::Error::custom("invalid audit policy values"))
    }
}

impl AuditPolicy {
    /// AUD-07/CQ-23/CAN-10: validates the individual field ranges before construction.
    ///
    /// # Errors
    /// Returns field errors when either numeric value is outside its inclusive 1..=10 range.
    pub fn new(
        retention_years: u8,
        anonymize_pii_after_years: u8,
        admin_categories: Vec<AuditCategory>,
    ) -> Result<Self, Vec<FieldError>> {
        let mut errors = Vec::new();
        if !(1..=10).contains(&retention_years) {
            errors.push(field_error(
                "/retention_years",
                if retention_years < 1 {
                    MUST_BE_GTE
                } else {
                    MUST_BE_LTE
                },
            ));
        }
        if !(1..=10).contains(&anonymize_pii_after_years) {
            errors.push(field_error(
                "/anonymize_pii_after_years",
                if anonymize_pii_after_years < 1 {
                    MUST_BE_GTE
                } else {
                    MUST_BE_LTE
                },
            ));
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            retention_years,
            anonymize_pii_after_years,
            admin_categories,
        })
    }

    /// AUD-07: returns the retention interval in calendar years.
    pub const fn retention_years(&self) -> u8 {
        self.retention_years
    }

    /// AUD-07: returns the PII anonymisation interval in calendar years.
    pub const fn anonymize_pii_after_years(&self) -> u8 {
        self.anonymize_pii_after_years
    }

    /// AUD-07: returns the categories visible to administrators.
    pub fn admin_categories(&self) -> &[AuditCategory] {
        &self.admin_categories
    }

    /// Adds the policy renderer metadata required by POL-06.
    pub fn json_schema() -> Value {
        let Ok(mut schema) = serde_json::to_value(schemars::schema_for!(Self)) else {
            // Fail closed: `{}` would accept any policy if static schema serialization regressed.
            return json!({"not": {}});
        };
        let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) else {
            return schema;
        };
        for (field, order) in AUDIT_POLICY_SCHEMA_FIELDS {
            if let Some(property) = properties.get_mut(field).and_then(Value::as_object_mut) {
                property.insert("x-ui".to_owned(), XUi::new(Group::Advanced, order).into_value());
            }
        }
        schema
    }
}

impl Default for AuditPolicy {
    fn default() -> Self {
        Self {
            retention_years: 7,
            anonymize_pii_after_years: 2,
            admin_categories: vec![AuditCategory::Operational],
        }
    }
}

impl PolicyDoc for AuditPolicy {
    const MODULE: ModuleId = ModuleId::Audit;
    const SCHEMA_VERSION: u32 = 1;

    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.anonymize_pii_after_years > self.retention_years {
            return Err(vec![field_error("/anonymize_pii_after_years", MUST_BE_LTE)]);
        }
        Ok(())
    }

    fn summarize(&self) -> Vec<SummaryLine> {
        vec![
            summary("policy.audit.summary.retention", self.retention_years),
            summary("policy.audit.summary.anonymize", self.anonymize_pii_after_years),
        ]
    }
}

fn summary(key: &str, years: u8) -> SummaryLine {
    let mut args = Map::new();
    args.insert("years".to_owned(), json!(years));
    SummaryLine {
        key: key.to_owned(),
        args,
    }
}

fn field_error(pointer: &str, code: &str) -> FieldError {
    FieldError {
        pointer: pointer.to_owned(),
        code: code.to_owned(),
        params: Map::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{AUDIT_POLICY_SCHEMA_FIELDS, AuditCategory, AuditPolicy};
    use eq_core::error::MUST_BE_LTE;
    use eq_core::policy::PolicyDoc;
    use serde_json::json;

    #[test]
    fn defaults_schema_and_summary_match_the_policy_contract() {
        let policy = AuditPolicy::default();
        assert_eq!(policy.retention_years(), 7);
        assert_eq!(policy.anonymize_pii_after_years(), 2);
        assert_eq!(policy.admin_categories(), &[AuditCategory::Operational]);
        assert!(policy.validate().is_ok());

        let schema = AuditPolicy::json_schema();
        for (field, order) in AUDIT_POLICY_SCHEMA_FIELDS {
            assert_eq!(schema["properties"][field]["x-ui"]["group"], "advanced");
            assert_eq!(schema["properties"][field]["x-ui"]["order"], order);
        }
        assert_eq!(schema["properties"]["retention_years"]["minimum"], 1);
        assert_eq!(schema["properties"]["retention_years"]["maximum"], 10);
        assert_eq!(schema["properties"]["anonymize_pii_after_years"]["minimum"], 1);
        assert_eq!(schema["properties"]["anonymize_pii_after_years"]["maximum"], 10);
        let summaries = policy.summarize();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].key, "policy.audit.summary.retention");
        assert_eq!(summaries[0].args["years"], 7);
        assert_eq!(summaries[1].key, "policy.audit.summary.anonymize");
        assert_eq!(summaries[1].args["years"], 2);
    }

    #[test]
    fn cross_field_limit_reports_the_contract_pointer_and_code() {
        let policy = serde_json::from_value::<AuditPolicy>(json!({
            "retention_years": 2,
            "anonymize_pii_after_years": 3,
            "admin_categories": ["operational"]
        }))
        .expect("field values and category are individually valid");
        let errors = policy.validate().err().unwrap_or_default();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].pointer, "/anonymize_pii_after_years");
        assert_eq!(errors[0].code, MUST_BE_LTE);
    }

    #[test]
    fn deserialization_validates_value_ranges_and_rejects_unknown_fields() {
        let invalid_range = serde_json::from_value::<AuditPolicy>(json!({
            "retention_years": 0,
            "anonymize_pii_after_years": 2,
            "admin_categories": ["operational"]
        }));
        assert!(invalid_range.is_err());

        let invalid_anonymize_range = serde_json::from_value::<AuditPolicy>(json!({
            "retention_years": 7,
            "anonymize_pii_after_years": 11,
            "admin_categories": ["operational"]
        }));
        assert!(invalid_anonymize_range.is_err());

        let unknown_field = serde_json::from_value::<AuditPolicy>(json!({
            "retention_years": 7,
            "anonymize_pii_after_years": 2,
            "admin_categories": ["operational"],
            "unexpected": true
        }));
        assert!(unknown_field.is_err());
    }
}
