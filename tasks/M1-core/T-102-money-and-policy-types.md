---
id: T-102
title: Add Money, Bps and the policy value types (LocalTime, WeeklyHours, DayOfMonth, I18nText, GeoPoint, Locale, Currency)
milestone: M1
status: todo
depends_on: [T-001]
spec_refs: [ARC-11, POL-11, POL-12, ADR-0007]
estimated_size: M
---

## Goal
Correct, property-tested primitives for money and for every common policy field type. They serialize as specified and derive JSON Schema with the right constraints.

## Context
- ARC-11:
  - `Money { amount_minor: i64, currency: Currency }`. Mixed-currency arithmetic returns `Err(CurrencyMismatch)`.
  - No floats. Percentages are basis points (`Bps(u32)`, 10000 = 100%).
  - Rounding is half-up to the minor unit, **only** in `apply_bps`.
- ISO 4217 minor exponents: IDR 2, USD 2, SGD 2, MYR 2, EUR 2, JPY 0. Supported currencies in v1: IDR, USD, SGD, MYR, PHP, THB, VND (exponent 0 per ISO), EUR, GBP, AUD, JPY.
- 04 §5 types (serde formats):
  - `LocalTime` "HH:MM", minute precision.
  - `TimeInterval {start, end}` with start < end, same day. `end` may be "24:00" meaning midnight.
  - `Weekday` lowercase `"mon".."sun"`.
  - `DayHours {weekday, open, intervals}`: intervals non-overlapping and sorted, empty if `open=false`.
  - `WeeklyHours([DayHours; 7])`, serialized as an array of 7 in mon..sun order.
  - `Bps` 0..=10000.
  - `MinorAmount(i64)` ≥ 0.
  - `DayOfMonth(u8)` 1..=28.
  - `I18nText(BTreeMap<Locale, String>)` with ≥ 1 entry.
  - `GeoPoint {lat ∈ [-90,90], lng ∈ [-180,180]}`.
  - `Locale` enum `en | id`.
- JSON Schema: constraints (min/max, pattern) must appear in the schemars output, because the SPA validates with ajv.

## Scope
### In
- `eq_core::money::{Money, Currency, MoneyError, apply_bps(amount: i64, bps: Bps) -> i64}`, plus `Currency::exponent()`, `Currency::from_code`, and `format_minor` (debug/tests only).
- `eq_core::policy::types::*` as above. Each has `Serialize`/`Deserialize` with validation in `TryFrom` (serde `try_from = "…"`), and a `JsonSchema` impl with constraints.

### Out
- PolicyStore (T-106).

## Interfaces
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, utoipa::ToSchema, sqlx::Type)]
#[sqlx(type_name = "bpchar")] pub enum Currency { IDR, USD, SGD, MYR, PHP, THB, VND, EUR, GBP, AUD, JPY }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, utoipa::ToSchema)]
pub struct Money { pub amount_minor: i64, pub currency: Currency }
impl Money {
  pub fn zero(c: Currency) -> Self;
  pub fn checked_add(self, o: Money) -> Result<Money, MoneyError>;
  pub fn checked_sub(self, o: Money) -> Result<Money, MoneyError>;
  pub fn checked_mul(self, qty: i64) -> Result<Money, MoneyError>;
  pub fn apply_bps(self, bps: Bps) -> Result<Money, MoneyError>; // half-up
}
#[derive(Debug, thiserror::Error, PartialEq)] pub enum MoneyError { #[error("currency mismatch")] CurrencyMismatch, #[error("overflow")] Overflow }
pub fn apply_bps(amount_minor: i64, bps: Bps) -> Result<i64, MoneyError>; // (amount*bps + 5000) / 10000 for amount>=0; symmetric for negative
```

## Business rules
None (primitives).

## Implementation notes
- Use `i128` intermediates in `apply_bps` to avoid overflow, and return `Overflow` if the result doesn't fit in i64.
- `LocalTime` "24:00" is allowed only as `TimeInterval.end`. Represent it internally as minutes-of-day (0..=1440) rather than `NaiveTime`.

## Acceptance criteria
- AC1. `apply_bps(10_000_000, Bps(7000)) == 7_000_000`; `apply_bps(15, Bps(5000)) == 8` (half-up). → `money::tests::bps_half_up`
- AC2. Property: for random a, b in ±10^15 of the same currency, `(a+b)-b == a`. → `money::proptests::add_sub_roundtrip`
- AC3. Property: `apply_bps(x, 10000) == x` and `apply_bps(x, 0) == 0`. → `bps_identity`
- AC4. `DayOfMonth` 29 fails to deserialize. → `types::tests::day_of_month_max_28`
- AC5. Overlapping intervals in a `DayHours` fail. → `day_hours_rejects_overlap`
- AC6. `WeeklyHours` schema JSON contains `minItems: 7, maxItems: 7` (insta snapshot). → `weekly_hours_schema_snapshot`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/money.rs` | `bps_half_up`, `currency_mismatch`, `overflow_detected`, proptests `add_sub_roundtrip`, `bps_identity`, `bps_monotonic` |
| `crates/eq-core/src/policy/types.rs` | `day_of_month_max_28`, `day_hours_rejects_overlap`, `local_time_format`, `interval_end_2400_ok`, `i18n_text_requires_entry`, `geo_bounds`, `weekly_hours_schema_snapshot` |

## Definition of done
- [ ] Rust checks pass; `scripts/check-literals.sh` passes (the ISO exponent table needs `// technical:` consts)
- [ ] Commit `feat(core): money and policy value types [T-102]`, signed off

## Report

## Questions
