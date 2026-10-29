//! Versioned identity policy values (IDN-01..06/12, POL-01/06/07/10, CQ-23, CAN-10).

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value, json};

use eq_core::error::{FieldError, MUST_BE_GTE, MUST_BE_LTE};
use eq_core::module::{ModuleId, SummaryLine};
use eq_core::policy::PolicyDoc;
use eq_core::policy::types::RoleMap;

/// IDN-06: lockout policy thresholds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Lockout {
    #[schemars(range(min = 3, max = 20))]
    max_failed: u8,
    #[schemars(range(min = 1, max = 120))]
    window_minutes: u16,
    #[schemars(range(min = 1, max = 1440))]
    lock_minutes: u16,
    notify_admins: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LockoutWire {
    max_failed: u8,
    window_minutes: u16,
    lock_minutes: u16,
    notify_admins: bool,
}

impl<'de> Deserialize<'de> for Lockout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = LockoutWire::deserialize(deserializer)?;
        Self::new(
            wire.max_failed,
            wire.window_minutes,
            wire.lock_minutes,
            wire.notify_admins,
        )
        .map_err(|_| serde::de::Error::custom("invalid identity lockout values"))
    }
}

impl Lockout {
    /// Validates lockout limits from IDN-06 and spec 04 §6.2.
    pub fn new(
        max_failed: u8,
        window_minutes: u16,
        lock_minutes: u16,
        notify_admins: bool,
    ) -> Result<Self, Vec<FieldError>> {
        let mut errors = Vec::new();
        push_range_error(&mut errors, "/lockout/max_failed", max_failed, 3, 20);
        push_range_error(&mut errors, "/lockout/window_minutes", window_minutes, 1, 120);
        push_range_error(&mut errors, "/lockout/lock_minutes", lock_minutes, 1, 1440);
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            max_failed,
            window_minutes,
            lock_minutes,
            notify_admins,
        })
    }

    /// IDN-06: Returns the failed-attempt threshold.
    pub const fn max_failed(&self) -> u8 {
        self.max_failed
    }

    /// IDN-06: Returns the failed-attempt window in minutes.
    pub const fn window_minutes(&self) -> u16 {
        self.window_minutes
    }

    /// IDN-06: Returns the lock interval in minutes.
    pub const fn lock_minutes(&self) -> u16 {
        self.lock_minutes
    }

    /// IDN-06: Returns whether identity should notify tenant administrators after lockout.
    pub const fn notify_admins(&self) -> bool {
        self.notify_admins
    }
}

/// IDN-12: accepted TOTP enforcement modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TotpPolicy {
    /// IDN-12: TOTP is not required by policy.
    Off,
    /// IDN-12: Staff may enable TOTP for their account.
    #[default]
    OptionalForStaff,
    /// IDN-12: Owners and administrators must enroll in TOTP.
    RequiredForOwnerAdmin,
}

/// IDN-01..06/12: validated identity settings consumed by authentication flows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct IdentityPolicy {
    idle_timeout_minutes: RoleMap<u16>,
    #[schemars(range(min = 1, max = 90))]
    absolute_session_days: u16,
    lockout: Lockout,
    #[schemars(range(min = 8, max = 64))]
    password_min_length: u8,
    #[schemars(range(min = 1, max = 30))]
    invite_expires_days: u16,
    #[schemars(range(min = 10, max = 1440))]
    reset_token_ttl_minutes: u16,
    #[schemars(range(min = 1, max = 10))]
    reset_max_per_hour: u8,
    totp: TotpPolicy,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityPolicyWire {
    idle_timeout_minutes: RoleMap<u16>,
    absolute_session_days: u16,
    lockout: Lockout,
    password_min_length: u8,
    invite_expires_days: u16,
    reset_token_ttl_minutes: u16,
    reset_max_per_hour: u8,
    totp: TotpPolicy,
}

impl<'de> Deserialize<'de> for IdentityPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = IdentityPolicyWire::deserialize(deserializer)?;
        Self::new(
            wire.idle_timeout_minutes,
            wire.absolute_session_days,
            wire.lockout,
            wire.password_min_length,
            wire.invite_expires_days,
            wire.reset_token_ttl_minutes,
            wire.reset_max_per_hour,
            wire.totp,
        )
        .map_err(|_| serde::de::Error::custom("invalid identity policy values"))
    }
}

impl IdentityPolicy {
    /// Validates all identity policy field bounds from spec 04 §6.2.
    #[expect(
        clippy::too_many_arguments,
        reason = "the constructor validates the eight fields in the normative identity policy"
    )]
    pub fn new(
        idle_timeout_minutes: RoleMap<u16>,
        absolute_session_days: u16,
        lockout: Lockout,
        password_min_length: u8,
        invite_expires_days: u16,
        reset_token_ttl_minutes: u16,
        reset_max_per_hour: u8,
        totp: TotpPolicy,
    ) -> Result<Self, Vec<FieldError>> {
        let mut errors = Vec::new();
        for (role, timeout) in [
            ("owner", idle_timeout_minutes.owner),
            ("admin", idle_timeout_minutes.admin),
            ("instructor", idle_timeout_minutes.instructor),
            ("guardian", idle_timeout_minutes.guardian),
            ("student", idle_timeout_minutes.student),
        ] {
            let pointer = format!("/idle_timeout_minutes/{role}");
            push_range_error(&mut errors, &pointer, timeout, 5, 1440);
        }
        push_range_error(
            &mut errors,
            "/absolute_session_days",
            absolute_session_days,
            1,
            90,
        );
        push_range_error(&mut errors, "/password_min_length", password_min_length, 8, 64);
        push_range_error(&mut errors, "/invite_expires_days", invite_expires_days, 1, 30);
        push_range_error(
            &mut errors,
            "/reset_token_ttl_minutes",
            reset_token_ttl_minutes,
            10,
            1440,
        );
        push_range_error(&mut errors, "/reset_max_per_hour", reset_max_per_hour, 1, 10);
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            idle_timeout_minutes,
            absolute_session_days,
            lockout,
            password_min_length,
            invite_expires_days,
            reset_token_ttl_minutes,
            reset_max_per_hour,
            totp,
        })
    }

    /// IDN-01: Returns the per-role idle session timeout values.
    pub const fn idle_timeout_minutes(&self) -> &RoleMap<u16> {
        &self.idle_timeout_minutes
    }

    /// IDN-03: Returns the absolute session lifetime in days.
    pub const fn absolute_session_days(&self) -> u16 {
        self.absolute_session_days
    }

    /// IDN-06: Returns the validated lockout thresholds.
    pub const fn lockout(&self) -> &Lockout {
        &self.lockout
    }

    /// IDN-02: Returns the minimum accepted new-password length.
    pub const fn password_min_length(&self) -> u8 {
        self.password_min_length
    }

    /// IDN-05: Returns the invite expiry in days.
    pub const fn invite_expires_days(&self) -> u16 {
        self.invite_expires_days
    }

    /// IDN-04: Returns the password-reset token TTL in minutes.
    pub const fn reset_token_ttl_minutes(&self) -> u16 {
        self.reset_token_ttl_minutes
    }

    /// IDN-04: Returns the maximum number of reset requests allowed each hour.
    pub const fn reset_max_per_hour(&self) -> u8 {
        self.reset_max_per_hour
    }

    /// IDN-12: Returns the TOTP enforcement policy.
    pub const fn totp(&self) -> TotpPolicy {
        self.totp
    }
}

impl Default for IdentityPolicy {
    fn default() -> Self {
        Self {
            idle_timeout_minutes: RoleMap {
                owner: 30,
                admin: 30,
                instructor: 60,
                guardian: 120,
                student: 120,
            },
            absolute_session_days: 30,
            lockout: Lockout {
                max_failed: 5,
                window_minutes: 15,
                lock_minutes: 15,
                notify_admins: true,
            },
            password_min_length: 10,
            invite_expires_days: 7,
            reset_token_ttl_minutes: 60,
            reset_max_per_hour: 3,
            totp: TotpPolicy::OptionalForStaff,
        }
    }
}

impl PolicyDoc for IdentityPolicy {
    const MODULE: ModuleId = ModuleId::Identity;
    const SCHEMA_VERSION: u32 = 1;

    fn validate(&self) -> Result<(), Vec<FieldError>> {
        Ok(())
    }

    fn summarize(&self) -> Vec<SummaryLine> {
        vec![
            summary(
                "policy.identity.summary.sessions",
                json!({
                    "idle_admin": self.idle_timeout_minutes.admin,
                    "idle_guardian": self.idle_timeout_minutes.guardian,
                    "absolute_days": self.absolute_session_days,
                }),
            ),
            summary(
                "policy.identity.summary.lockout",
                json!({
                    "max_failed": self.lockout.max_failed,
                    "window": self.lockout.window_minutes,
                    "lock": self.lockout.lock_minutes,
                }),
            ),
        ]
    }
}

fn push_range_error<T: Ord>(errors: &mut Vec<FieldError>, pointer: &str, value: T, minimum: T, maximum: T) {
    let code = if value < minimum {
        MUST_BE_GTE
    } else if value > maximum {
        MUST_BE_LTE
    } else {
        return;
    };
    errors.push(field_error(pointer, code));
}

fn summary(key: &str, args: Value) -> SummaryLine {
    SummaryLine {
        key: key.to_owned(),
        args: args.as_object().cloned().unwrap_or_default(),
    }
}

fn field_error(pointer: &str, code: &str) -> FieldError {
    FieldError {
        pointer: pointer.to_owned(),
        code: code.to_owned(),
        params: Map::new(),
    }
}

impl Default for Lockout {
    fn default() -> Self {
        Self {
            max_failed: 5,
            window_minutes: 15,
            lock_minutes: 15,
            notify_admins: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IdentityPolicy, Lockout, RoleMap, TotpPolicy};
    use serde_json::{Value, json};

    #[test]
    fn invalid_values_rejected_on_deserialize() {
        let ranges: [(&[&str], u64, u64); 13] = [
            (&["idle_timeout_minutes", "owner"], 5, 1440),
            (&["idle_timeout_minutes", "admin"], 5, 1440),
            (&["idle_timeout_minutes", "instructor"], 5, 1440),
            (&["idle_timeout_minutes", "guardian"], 5, 1440),
            (&["idle_timeout_minutes", "student"], 5, 1440),
            (&["absolute_session_days"], 1, 90),
            (&["lockout", "max_failed"], 3, 20),
            (&["lockout", "window_minutes"], 1, 120),
            (&["lockout", "lock_minutes"], 1, 1440),
            (&["password_min_length"], 8, 64),
            (&["invite_expires_days"], 1, 30),
            (&["reset_token_ttl_minutes"], 10, 1440),
            (&["reset_max_per_hour"], 1, 10),
        ];
        for (path, minimum, maximum) in ranges {
            for value in [minimum, maximum] {
                let mut document = default_document();
                set_value(&mut document, path, json!(value));
                assert!(
                    serde_json::from_value::<IdentityPolicy>(document).is_ok(),
                    "the inclusive boundary {path:?}={value} must be accepted"
                );
            }
            for value in [minimum - 1, maximum + 1] {
                let mut document = default_document();
                set_value(&mut document, path, json!(value));
                assert!(
                    serde_json::from_value::<IdentityPolicy>(document).is_err(),
                    "the out-of-range value {path:?}={value} must be rejected"
                );
            }
        }

        let mut document = default_document();
        document["totp"] = json!("unknown_policy");
        assert!(serde_json::from_value::<IdentityPolicy>(document).is_err());

        let mut document = default_document();
        document["unknown"] = json!(true);
        assert!(serde_json::from_value::<IdentityPolicy>(document).is_err());

        let mut document = default_document();
        document["lockout"]["unknown"] = json!(true);
        assert!(serde_json::from_value::<IdentityPolicy>(document).is_err());
    }

    #[test]
    fn out_of_range_timeout_rejected() {
        for role in ["owner", "admin", "instructor", "guardian", "student"] {
            for timeout in [4, 5, 1440, 1441] {
                let mut timeouts = RoleMap {
                    owner: 30,
                    admin: 30,
                    instructor: 60,
                    guardian: 120,
                    student: 120,
                };
                match role {
                    "owner" => timeouts.owner = timeout,
                    "admin" => timeouts.admin = timeout,
                    "instructor" => timeouts.instructor = timeout,
                    "guardian" => timeouts.guardian = timeout,
                    "student" => timeouts.student = timeout,
                    _ => unreachable!(),
                }
                let result = IdentityPolicy::new(
                    timeouts,
                    30,
                    Lockout::default(),
                    10,
                    7,
                    60,
                    3,
                    TotpPolicy::OptionalForStaff,
                );
                assert_eq!(
                    result.is_ok(),
                    [5, 1440].contains(&timeout),
                    "{role} timeout {timeout} has the wrong boundary result"
                );
            }
        }
    }

    #[test]
    fn constructors_reject_out_of_range_values() {
        for (field, minimum, maximum) in [
            ("absolute_session_days", 1, 90),
            ("password_min_length", 8, 64),
            ("invite_expires_days", 1, 30),
            ("reset_token_ttl_minutes", 10, 1440),
            ("reset_max_per_hour", 1, 10),
        ] {
            for value in [minimum, maximum] {
                assert!(
                    reconstruct_policy(policy_with_field(field, value)).is_ok(),
                    "{field} boundary {value} must be accepted"
                );
            }
            for value in [minimum - 1, maximum + 1] {
                assert!(
                    reconstruct_policy(policy_with_field(field, value)).is_err(),
                    "{field} value {value} must be rejected"
                );
            }
        }

        assert!(Lockout::new(3, 1, 1, true).is_ok());
        assert!(Lockout::new(20, 120, 1440, true).is_ok());
        for (max_failed, window_minutes, lock_minutes) in [
            (2, 15, 15),
            (21, 15, 15),
            (5, 0, 15),
            (5, 121, 15),
            (5, 15, 0),
            (5, 15, 1441),
        ] {
            assert!(
                Lockout::new(max_failed, window_minutes, lock_minutes, true).is_err(),
                "lockout values {max_failed}/{window_minutes}/{lock_minutes} must be rejected"
            );
        }
    }

    fn policy_with_field(field: &str, value: u16) -> IdentityPolicy {
        let mut policy = IdentityPolicy::default();
        match field {
            "absolute_session_days" => policy.absolute_session_days = value,
            "password_min_length" => policy.password_min_length = u8::try_from(value).unwrap(),
            "invite_expires_days" => policy.invite_expires_days = value,
            "reset_token_ttl_minutes" => policy.reset_token_ttl_minutes = value,
            "reset_max_per_hour" => policy.reset_max_per_hour = u8::try_from(value).unwrap(),
            _ => unreachable!(),
        }
        policy
    }

    fn reconstruct_policy(policy: IdentityPolicy) -> Result<IdentityPolicy, Vec<eq_core::error::FieldError>> {
        IdentityPolicy::new(
            policy.idle_timeout_minutes,
            policy.absolute_session_days,
            policy.lockout,
            policy.password_min_length,
            policy.invite_expires_days,
            policy.reset_token_ttl_minutes,
            policy.reset_max_per_hour,
            policy.totp,
        )
    }

    fn default_document() -> Value {
        serde_json::to_value(IdentityPolicy::default()).unwrap_or_else(|_| json!({}))
    }

    fn set_value(document: &mut Value, path: &[&str], value: Value) {
        let Some((last, parent_path)) = path.split_last() else {
            return;
        };
        let mut parent = document;
        for segment in parent_path {
            parent = &mut parent[*segment];
        }
        parent[*last] = value;
    }
}
