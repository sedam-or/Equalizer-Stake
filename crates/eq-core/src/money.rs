//! Exact money values and arithmetic (spec/02 §9, ARC-11; spec/23 CAN-10).
#![expect(
    clippy::upper_case_acronyms,
    reason = "ISO 4217 currency identifiers use their canonical uppercase codes (CAN-10)"
)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use crate::policy::types::Bps;

/// Supported ISO 4217 currencies and their canonical wire codes (ARC-11, CAN-10).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, ToSchema, sqlx::Type,
)]
#[serde(try_from = "String")]
#[schemars(with = "CurrencySchema")]
#[sqlx(type_name = "bpchar")]
pub enum Currency {
    IDR,
    USD,
    SGD,
    MYR,
    PHP,
    THB,
    VND,
    EUR,
    GBP,
    AUD,
    JPY,
}

#[derive(JsonSchema)]
#[schemars(rename = "Currency")]
#[expect(
    dead_code,
    reason = "Schemars reads this enum as the canonical Currency wire schema (CAN-10)"
)]
enum CurrencySchema {
    IDR,
    USD,
    SGD,
    MYR,
    PHP,
    THB,
    VND,
    EUR,
    GBP,
    AUD,
    JPY,
}

impl Currency {
    /// Returns the canonical uppercase ISO 4217 code for this supported currency (CAN-10).
    pub const fn code(self) -> &'static str {
        match self {
            Self::IDR => "IDR",
            Self::USD => "USD",
            Self::SGD => "SGD",
            Self::MYR => "MYR",
            Self::PHP => "PHP",
            Self::THB => "THB",
            Self::VND => "VND",
            Self::EUR => "EUR",
            Self::GBP => "GBP",
            Self::AUD => "AUD",
            Self::JPY => "JPY",
        }
    }

    /// Returns the ISO 4217 minor-unit exponent from `iso_currency` (OSS-02).
    pub fn exponent(self) -> Option<u16> {
        iso_currency::Currency::from_code(self.code()).and_then(iso_currency::Currency::exponent)
    }

    /// Parses a supported ISO 4217 code, accepting lowercase and returning the canonical value (CAN-10).
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_ascii_uppercase().as_str() {
            "IDR" => Some(Self::IDR),
            "USD" => Some(Self::USD),
            "SGD" => Some(Self::SGD),
            "MYR" => Some(Self::MYR),
            "PHP" => Some(Self::PHP),
            "THB" => Some(Self::THB),
            "VND" => Some(Self::VND),
            "EUR" => Some(Self::EUR),
            "GBP" => Some(Self::GBP),
            "AUD" => Some(Self::AUD),
            "JPY" => Some(Self::JPY),
            _ => None,
        }
    }
}

impl TryFrom<String> for Currency {
    type Error = &'static str;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        Self::from_code(&code).ok_or("unsupported currency")
    }
}

/// Maximum absolute minor-unit amount accepted by the canonical wire contract (CAN-10).
// technical: keeps integer money below JavaScript's MAX_SAFE_INTEGER (CAN-10, D-04).
pub const MONEY_MAX_ABS_MINOR: i64 = 1_000_000_000_000_000;

/// Exact money in minor units and one ISO 4217 currency (ARC-11, CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(try_from = "MoneyRepr")]
pub struct Money {
    #[schemars(range(min = -1_000_000_000_000_000_i64, max = 1_000_000_000_000_000_i64))]
    #[schema(minimum = -1_000_000_000_000_000f64, maximum = 1_000_000_000_000_000f64)]
    amount_minor: i64,
    currency: Currency,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[derive(JsonSchema)]
struct MoneyRepr {
    #[schemars(range(min = -1_000_000_000_000_000_i64, max = 1_000_000_000_000_000_i64))]
    amount_minor: i64,
    currency: Currency,
}

impl TryFrom<MoneyRepr> for Money {
    type Error = MoneyError;

    fn try_from(value: MoneyRepr) -> Result<Self, Self::Error> {
        Self::new(value.amount_minor, value.currency)
    }
}

impl Money {
    /// Validates and constructs a money value within the canonical magnitude cap (CAN-10).
    ///
    /// # Errors
    /// Returns [`MoneyError::OutOfRange`] when the amount exceeds `MONEY_MAX_ABS_MINOR`.
    pub fn new(amount_minor: i64, currency: Currency) -> Result<Self, MoneyError> {
        if i128::from(amount_minor).abs() > i128::from(MONEY_MAX_ABS_MINOR) {
            return Err(MoneyError::OutOfRange);
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    /// Returns the signed number of minor currency units (ARC-11).
    pub const fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    /// Returns this value's currency (ARC-11).
    pub const fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns zero minor units in the requested currency (ARC-11).
    pub const fn zero(currency: Currency) -> Self {
        Self {
            amount_minor: 0,
            currency,
        }
    }

    /// Adds same-currency money and keeps the result within the canonical cap (ARC-11, CAN-10).
    ///
    /// # Errors
    /// Returns [`MoneyError::CurrencyMismatch`] for different currencies and [`MoneyError::Overflow`]
    /// when the sum exceeds the canonical cap.
    pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;
        let amount_minor = self
            .amount_minor
            .checked_add(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;
        Self::from_arithmetic(amount_minor, self.currency)
    }

    /// Subtracts same-currency money and keeps the result within the canonical cap (ARC-11, CAN-10).
    ///
    /// # Errors
    /// Returns [`MoneyError::CurrencyMismatch`] for different currencies and [`MoneyError::Overflow`]
    /// when the difference exceeds the canonical cap.
    pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;
        let amount_minor = self
            .amount_minor
            .checked_sub(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;
        Self::from_arithmetic(amount_minor, self.currency)
    }

    /// Multiplies by an integer quantity and keeps the result within the canonical cap (ARC-11, CAN-10).
    ///
    /// # Errors
    /// Returns [`MoneyError::Overflow`] when the product exceeds the canonical cap.
    pub fn checked_mul(self, quantity: i64) -> Result<Self, MoneyError> {
        let amount_minor = self
            .amount_minor
            .checked_mul(quantity)
            .ok_or(MoneyError::Overflow)?;
        Self::from_arithmetic(amount_minor, self.currency)
    }

    /// Applies basis points with half-up rounding to this amount (ARC-11).
    ///
    /// # Errors
    /// Returns [`MoneyError::Overflow`] if the rounded result is outside the canonical cap.
    pub fn apply_bps(self, bps: Bps) -> Result<Self, MoneyError> {
        let amount_minor = apply_bps(self.amount_minor, bps)?;
        Self::from_arithmetic(amount_minor, self.currency)
    }

    fn ensure_same_currency(self, other: Self) -> Result<(), MoneyError> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(MoneyError::CurrencyMismatch)
        }
    }

    fn from_arithmetic(amount_minor: i64, currency: Currency) -> Result<Self, MoneyError> {
        match Self::new(amount_minor, currency) {
            Ok(value) => Ok(value),
            Err(MoneyError::OutOfRange) => Err(MoneyError::Overflow),
            Err(MoneyError::CurrencyMismatch | MoneyError::Overflow) => Err(MoneyError::Overflow),
        }
    }
}

/// Money arithmetic failures (ARC-11, CAN-10).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MoneyError {
    /// The two operands use different currencies.
    #[error("currency mismatch")]
    CurrencyMismatch,
    /// The arithmetic result exceeds the canonical money cap.
    #[error("overflow")]
    Overflow,
    /// An external amount exceeds the canonical money cap.
    #[error("amount out of range")]
    OutOfRange,
}

/// Applies bounded basis points to signed minor units with symmetric half-up rounding (ARC-11).
///
/// # Errors
/// Returns [`MoneyError::Overflow`] only if the checked conversion to `i64` fails.
pub fn apply_bps(amount_minor: i64, bps: Bps) -> Result<i64, MoneyError> {
    let product = i128::from(amount_minor).abs() * i128::from(bps.get());
    let rounded = (product + 5_000) / 10_000;
    let signed = if amount_minor < 0 { -rounded } else { rounded };
    i64::try_from(signed).map_err(|_| MoneyError::Overflow)
}

/// Formats minor units as a decimal amount followed by its code for diagnostics/tests (ARC-11).
pub fn format_minor(amount_minor: i64, currency: Currency) -> String {
    let Some(exponent) = currency.exponent() else {
        return format!("{amount_minor} {}", currency.code());
    };
    let precision = usize::from(exponent);
    let mut divisor = 1_i128;
    for _ in 0..exponent {
        let Some(next) = divisor.checked_mul(10) else {
            return format!("{amount_minor} {}", currency.code());
        };
        divisor = next;
    }
    let amount = i128::from(amount_minor);
    let absolute = amount.abs();
    let whole = absolute / divisor;
    let sign = if amount < 0 { "-" } else { "" };
    if precision == 0 {
        return format!("{sign}{whole} {}", currency.code());
    }

    let fraction = (absolute % divisor).to_string();
    let mut fractional = String::with_capacity(precision);
    for _ in fraction.len()..precision {
        fractional.push('0');
    }
    fractional.push_str(&fraction);
    format!("{sign}{whole}.{fractional} {}", currency.code())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::{Currency, MONEY_MAX_ABS_MINOR, Money, MoneyError, apply_bps};
    use crate::policy::types::Bps;

    #[test]
    fn bps_half_up() {
        assert_eq!(apply_bps(10_000_000, Bps::from_const(7000)), Ok(7_000_000));
        assert_eq!(apply_bps(15, Bps::from_const(5000)), Ok(8));
        assert_eq!(apply_bps(-15, Bps::from_const(5000)), Ok(-8));
        assert_eq!(apply_bps(i64::MAX, Bps::FULL), Ok(i64::MAX));
        assert_eq!(apply_bps(i64::MIN, Bps::FULL), Ok(i64::MIN));
    }

    #[test]
    fn currency_mismatch() {
        let idr = Money::new(1, Currency::IDR).expect("valid money");
        let usd = Money::new(1, Currency::USD).expect("valid money");

        assert_eq!(idr.checked_add(usd), Err(MoneyError::CurrencyMismatch));
        assert_eq!(idr.checked_sub(usd), Err(MoneyError::CurrencyMismatch));
    }

    #[test]
    fn overflow_detected() {
        let maximum = Money::new(MONEY_MAX_ABS_MINOR, Currency::IDR).expect("cap is valid");
        let minimum = Money::new(-MONEY_MAX_ABS_MINOR, Currency::IDR).expect("negative cap is valid");
        let one = Money::new(1, Currency::IDR).expect("one is valid");

        assert_eq!(maximum.checked_add(one), Err(MoneyError::Overflow));
        assert_eq!(minimum.checked_sub(one), Err(MoneyError::Overflow));
        assert_eq!(maximum.checked_mul(2), Err(MoneyError::Overflow));
    }

    #[test]
    fn money_rejects_out_of_range() {
        assert_eq!(
            Money::new(MONEY_MAX_ABS_MINOR + 1, Currency::IDR),
            Err(MoneyError::OutOfRange)
        );
        let json = json!({"amount_minor": 1_000_000_000_000_001_i64, "currency": "IDR"});
        assert!(serde_json::from_value::<Money>(json).is_err());
    }

    #[test]
    fn currency_exponents_match_iso() {
        assert_eq!(Currency::IDR.exponent(), Some(2));
        assert_eq!(Currency::USD.exponent(), Some(2));
        assert_eq!(Currency::VND.exponent(), Some(0));
        assert_eq!(Currency::JPY.exponent(), Some(0));
        assert_eq!(Currency::from_code("idr"), Some(Currency::IDR));
        assert_eq!(Currency::from_code("XXX"), None);
        assert_eq!(
            serde_json::from_str::<Currency>("\"idr\"").expect("lowercase code is accepted"),
            Currency::IDR
        );
        assert_eq!(
            serde_json::to_string(&Currency::IDR).expect("currency serializes"),
            "\"IDR\""
        );
    }

    proptest! {
        #[test]
        fn add_sub_roundtrip(
            a in (-MONEY_MAX_ABS_MINOR / 2)..=(MONEY_MAX_ABS_MINOR / 2),
            b in (-MONEY_MAX_ABS_MINOR / 2)..=(MONEY_MAX_ABS_MINOR / 2),
        ) {
            let a = Money::new(a, Currency::IDR).expect("generated amount is in range");
            let b = Money::new(b, Currency::IDR).expect("generated amount is in range");

            prop_assert_eq!(a.checked_add(b).and_then(|sum| sum.checked_sub(b)), Ok(a));
        }

        #[test]
        fn bps_identity(amount in any::<i64>()) {
            prop_assert_eq!(apply_bps(amount, Bps::FULL), Ok(amount));
            prop_assert_eq!(apply_bps(amount, Bps::ZERO), Ok(0));
        }

        #[test]
        fn bps_monotonic(
            amount in 0_i64..=i64::MAX,
            first in 0_u32..=10_000,
            second in 0_u32..=10_000,
        ) {
            let (lower, upper) = (first.min(second), first.max(second));
            let lower = Bps::new(lower).expect("generated basis points are in range");
            let upper = Bps::new(upper).expect("generated basis points are in range");

            prop_assert!(apply_bps(amount, lower)? <= apply_bps(amount, upper)?);
        }
    }
}
