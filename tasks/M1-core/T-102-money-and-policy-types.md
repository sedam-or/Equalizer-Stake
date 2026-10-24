---
id: T-102
title: Add Money, Bps and the policy value types (LocalTime, WeeklyHours, DayOfMonth, I18nText, GeoPoint, Locale, Currency)
milestone: M1
status: todo
depends_on: [T-001]
spec_refs: [ARC-11, POL-11, POL-12, ADR-0007, CAN-10, CAN-12, OSS-02]
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

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Canonical wire forms are CAN-10 (e.g. `Money` `|amount_minor| ≤ 10^15` = `MONEY_MAX_ABS_MINOR`, `Instant` output in tenant TZ with `SecondsFormat::AutoSi`). Use `iso_currency` for exponents instead of a hand-written table (OSS §2); keep the v1 supported-currency list as the allow-list.
  - Field kinds that wrap these types (`MoneyAmount`, `BpsField`, …) are added by T-102a, not here.

## Scope
### In
- `eq_core::money::{Money, Currency, MoneyError, MONEY_MAX_ABS_MINOR, apply_bps(amount: i64, bps: Bps) -> Result<i64, MoneyError>}`, plus `Currency::exponent()`, `Currency::from_code`, and `format_minor` (debug/tests only).
- `eq_core::policy::types::*` as above, plus the consts `Bps::ZERO` and `Bps::FULL` (10000), which later tasks use instead of literals. Each has `Serialize`/`Deserialize` with validation in `TryFrom` (serde `try_from = "…"`), and a `JsonSchema` impl with constraints.
- **Private fields, validating constructors (CQ-23, CAN-10; architect decision, spec 04 §5).** No type below exposes a field that could hold an invalid value. Each has `new(..) -> Result<Self, _>` (or `TryFrom`), read accessors, and deserialises only through its validating `TryFrom`. The numeric newtypes (`Bps`, `MinorAmount`, `DayOfMonth`) and `LocalTime` also have a `pub const fn from_const(..)` that `assert!`s the bound: it is for `const` items and `Default` literals, where a bad literal fails the build, and it is not for runtime input.

### Out
- PolicyStore (T-106).

## Interfaces
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, utoipa::ToSchema, sqlx::Type)]
#[sqlx(type_name = "bpchar")] pub enum Currency { IDR, USD, SGD, MYR, PHP, THB, VND, EUR, GBP, AUD, JPY }
pub const MONEY_MAX_ABS_MINOR: i64 = 1_000_000_000_000_000; // technical: CAN-10, D-04 (below JS MAX_SAFE_INTEGER)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema, utoipa::ToSchema)]
pub struct Money { amount_minor: i64, currency: Currency }   // private fields; Deserialize through `try_from = "MoneyRepr"` (a private repr struct); the schema keeps |amount_minor| <= 10^15
impl Money {
  pub fn new(amount_minor: i64, currency: Currency) -> Result<Money, MoneyError>; // OutOfRange if |amount_minor| > MONEY_MAX_ABS_MINOR
  pub fn amount_minor(&self) -> i64;
  pub fn currency(&self) -> Currency;
  pub fn zero(c: Currency) -> Self;
  pub fn checked_add(self, o: Money) -> Result<Money, MoneyError>;
  pub fn checked_sub(self, o: Money) -> Result<Money, MoneyError>;
  pub fn checked_mul(self, qty: i64) -> Result<Money, MoneyError>;
  pub fn apply_bps(self, bps: Bps) -> Result<Money, MoneyError>; // half-up
}
#[derive(Debug, thiserror::Error, PartialEq)] pub enum MoneyError { #[error("currency mismatch")] CurrencyMismatch, #[error("overflow")] Overflow, #[error("amount out of range")] OutOfRange }
pub fn apply_bps(amount_minor: i64, bps: Bps) -> Result<i64, MoneyError>; // (amount*bps + 5000) / 10000 for amount>=0; symmetric for negative; Overflow only if the result doesn't fit i64
```

## Business rules
None (primitives).

## Implementation notes
- Use `i128` intermediates in `apply_bps` to avoid overflow, and return `Overflow` if the result doesn't fit in i64. `Money::checked_add`, `checked_sub`, `checked_mul` and `Money::apply_bps` return `Overflow` when the result's magnitude exceeds `MONEY_MAX_ABS_MINOR` (which also covers i64 overflow), so a `Money` value is always in range. `Money::new` and deserialisation return `OutOfRange` for input beyond the cap.
- `Bps` is 0..=10000 and `Bps::new` rejects more. A value that must exceed 100% would be a different field type, not a widened `Bps`; v1 has no such case.
- `GeoPoint::new` rejects NaN and out-of-range values; `I18nText::new` rejects an empty map; `TimeInterval::new`, `DayHours::new` and `WeeklyHours::new` run the same checks as their deserialisation.
- `LocalTime` "24:00" is allowed only as `TimeInterval.end`. Represent it internally as minutes-of-day (0..=1440) rather than `NaiveTime`.

## Acceptance criteria
- AC1. `apply_bps(10_000_000, Bps::from_const(7000)) == Ok(7_000_000)`; `apply_bps(15, Bps::from_const(5000)) == Ok(8)` (half-up); `apply_bps(i64::MAX, Bps::FULL) == Ok(i64::MAX)`; a case whose result exceeds i64 returns `Err(Overflow)`. → `money::tests::bps_half_up`
- AC2. Property: for random a, b in ±(`MONEY_MAX_ABS_MINOR` / 2) of the same currency (so the sum stays within the cap), `(a+b)-b == a`. → `money::proptests::add_sub_roundtrip`
- AC3. Property: `apply_bps(x, Bps::FULL) == Ok(x)` and `apply_bps(x, Bps::ZERO) == Ok(0)`. → `bps_identity`
- AC4. `DayOfMonth` 29 fails to deserialize. → `types::tests::day_of_month_max_28`
- AC5. Overlapping intervals in a `DayHours` fail. → `day_hours_rejects_overlap`
- AC6. `WeeklyHours` schema JSON contains `minItems: 7, maxItems: 7` (insta snapshot). → `weekly_hours_schema_snapshot`
- AC7. `Money::new(MONEY_MAX_ABS_MINOR + 1, Currency::IDR)` is `Err(OutOfRange)`, and deserialising `{"amount_minor": 1000000000000001, "currency": "IDR"}` fails. → `money::tests::money_rejects_out_of_range`
- AC8. Every constrained type rejects invalid input through `new`/`TryFrom` as well as through serde: `Bps::new(10_001)`, `MinorAmount::new(-1)`, `DayOfMonth::new(29)`, `GeoPoint::new(f64::NAN, 0.0)`, `I18nText::new(empty)`, an overlapping `DayHours`, a `WeeklyHours` out of Mon..Sun order. → `types::tests::constrained_types_reject_invalid`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/money.rs` | `bps_half_up`, `currency_mismatch`, `overflow_detected`, `money_rejects_out_of_range`, proptests `add_sub_roundtrip`, `bps_identity`, `bps_monotonic` |
| `crates/eq-core/src/policy/types.rs` | `day_of_month_max_28`, `day_hours_rejects_overlap`, `local_time_format`, `interval_end_2400_ok`, `i18n_text_requires_entry`, `geo_bounds`, `constrained_types_reject_invalid`, `weekly_hours_schema_snapshot` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `scripts/check-literals.sh` passes (the ISO exponent table needs `// technical:` consts)
- [ ] Commit `feat(core): money and policy value types [T-102]`, signed off

## Report

**Plan** — Pending Q1 and Q2:
1. Add `money.rs` with `Currency`, `Money`, `MoneyError`, ISO exponent lookup, checked arithmetic, and basis-point rounding → verify the exact money unit/property tests.
2. Add `policy/types.rs` with the common value types and validated serde forms → verify the exact boundary and validation tests.
3. Add JSON Schema constraints and the weekly-hours snapshot → verify `weekly_hours_schema_snapshot`.
4. Wire the required workspace dependencies and public `eq-core` modules → verify Cargo metadata and dependency guards.
5. Run the Rust and repository checks required by the task, recording any unavailable checks without marking them green.

**Assumptions** — Use `iso_currency` for exponents and the task's supported-currency allow-list, as required by OSS-02/CAN-10. Use the stated canonical Money bound of `10^15`. No assumption is made about the contradictory `apply_bps` return type or how private-field validation is to coexist with the declared public fields; Q1 and Q2 block implementation.

**Test plan** —
- AC1: Given the specified positive amount and basis points, when `apply_bps` runs, then 70% of 10,000,000 is 7,000,000 and 50% of 15 rounds half-up to 8 → `money::tests::bps_half_up`.
- AC2: Given random `a` and `b` within ±10^15 in one currency, when adding then subtracting `b`, then the original amount is recovered → `money::proptests::add_sub_roundtrip`.
- AC3: Given a valid amount and Bps at 10000 or 0, when applied, then the result is respectively the input or zero → `bps_identity`.
- AC4: Given serialized `DayOfMonth(29)`, when deserialized, then validation fails → `types::tests::day_of_month_max_28`.
- AC5: Given overlapping intervals in one `DayHours`, when deserialized, then validation fails → `day_hours_rejects_overlap`.
- AC6: Given the `WeeklyHours` schema, when serialized to JSON, then the array has exactly seven items → `weekly_hours_schema_snapshot`.
- Additional exact tests: `money::tests::currency_mismatch`, `money::tests::overflow_detected`, `money::proptests::bps_identity`, `money::proptests::bps_monotonic`, `types::tests::local_time_format`, `types::tests::interval_end_2400_ok`, `types::tests::i18n_text_requires_entry`, and `types::tests::geo_bounds`.
- Edge cases: ±10^15 amount bounds and one beyond each bound; zero and maximum Bps; half-up ties for positive and negative values; checked arithmetic at i64 limits; unsupported/lowercase currency codes; all locale values; time minute boundaries and `24:00` only as interval end; interval overlap/touching; empty/open hours; weekly weekday order; day 28/29; empty and one-entry i18n maps; geographic coordinate endpoints and just-outside values.
- Error cases: currency mismatch, arithmetic overflow, invalid currency, invalid serde type/range/format, invalid time interval, invalid day schedule, empty i18n text, and out-of-bounds coordinates.
- Abuse cases: n/a; these value primitives do not trigger SBD-30.

**Step 0 — Start** — Clean tracked tree on `run/codex-m0-m3`; `scripts/tasks-index.sh --check` passed, no `changes_requested` task or other in-progress task was present, and T-102 was the lowest eligible todo under the run instruction that treats T-001 `review` as satisfied. Claimed on `task/T-102-money-and-policy-types`.
**Step 1 — Task/spec** — Read the full task. It has conflicting free-function signatures/expectations for `apply_bps` and a public-field/derived-serde contract that does not say how the CAN-10 Money bound and CQ-23 invalid-value rule are enforced.
**Step 2 — Dependencies** — Read T-001's Report and inspected its shipped workspace and `eq-core` scaffold. No dependency deviation blocks the work; T-001 is `review`, accepted for ordering under the run instruction.
**Step 3 — Design reference** — n/a; no UI work.
**Step 4 — Plan and test plan** — Recorded above. Blocked before implementation under `WORKFLOW.md` §4 because the money API/value invariants conflict and any implementation would have to choose a contract.
**Steps 5–11** — Not run because Step 4 requires blocking before implementation. No code, runtime, debug loop, self-review, optimization, or implementation security audit was performed. Step 11 verification is limited to the task index and diff hygiene checks recorded below.

**Summary** — No implementation or tests were added. Scope and the acceptance criteria treat the free `apply_bps` function as returning an `i64`, while the interface and implementation note require `Result<i64, MoneyError>` so overflow can be reported. The task also declares public fields/derived serde while CAN-10 requires a Money wire bound and CQ-23 requires invalid values to have private fields and validating constructors. These contracts need an explicit resolution before the primitives can be implemented.

**Spec read** — `spec/02-architecture.md` §9 (ARC-11); `spec/04-policy-engine.md` §5 (POL-11/12); `spec/16-code-quality.md` CQ-23 and §12; `spec/17-clean-architecture.md` §1 and §4; `spec/18-security-by-design.md` §4 and §5 (no SBD-30 trigger); `spec/19-error-handling.md` §10; `spec/22-validation-and-sanitization.md` Money/Bps/Locale rows; `spec/23-contracts-and-canonical-forms.md` §2 (CAN-10/12); `spec/24-open-source-stack.md` OSS-02 and the Money/`iso_currency` entry.

**Files** —
- Modified `tasks/M1-core/T-102-money-and-policy-types.md` with this report, Q1/Q2, and `status: blocked`.
- Regenerated `tasks/INDEX.md` with `scripts/tasks-index.sh`.

**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `money::tests::bps_half_up` | Not run: blocked before implementation. |
| AC2 | `money::proptests::add_sub_roundtrip` | Not run: blocked before implementation. |
| AC3 | `bps_identity` | Not run: blocked before implementation. |
| AC4 | `types::tests::day_of_month_max_28` | Not run: blocked before implementation. |
| AC5 | `day_hours_rejects_overlap` | Not run: blocked before implementation. |
| AC6 | `weekly_hours_schema_snapshot` | Not run: blocked before implementation. |

**Checks run** — `scripts/tasks-index.sh --check` passed before claiming and after the blocked status was generated. `git diff --check` passed. Rust tests, format, clippy, build, `make dev`, API generation, and SQLx preparation were not run because the type/API contract is unresolved. Web checks are n/a; no web files are in scope.

**Manual checks** — Not run; implementation is blocked.
**Debug log** — None; no implementation failure occurred.
**Performance** — n/a; no runtime path was added.
**Security** — No SBD-30 triggers; no S2/S3 data or endpoint is introduced.
**Deviations** — None; implementation has not started.
**Dependencies affected** — T-102a and later tasks depending on T-102 cannot start until the task is resolved and completed.
**Rollback plan** — Revert the task-block commit; it changes only the task report/status and generated index.
**Follow-ups** — Resolve Q1 and Q2, update the task interfaces/acceptance criteria, then restart T-102 from its task branch procedure.

## Questions
- **Resolved (architect, 2026-10-25):** Q1 — option (a). `apply_bps` returns `Result<i64, MoneyError>`; Scope, Interfaces, AC1 and AC3 now say so. `Bps::ZERO` and `Bps::FULL` were added because T-208 and T-210 use `Bps::FULL`.
- **Q1 (blocking):** Which free-function signature and acceptance-test contract should be authoritative for `apply_bps`?
  - Context: Scope line 40 and AC1/AC3 at lines 71/73 use a plain `i64` result, but the interface at line 60 returns `Result<i64, MoneyError>` and the implementation note at line 67 requires returning `Overflow` when the result does not fit.
  - Options: (a) make the fallible `Result<i64, MoneyError>` signature authoritative and update Scope/AC assertions to compare with `Ok(...)`; (b) retain `i64` and revise the overflow requirement to specify a non-error result for out-of-range values.
  - Recommendation: (a), because `MoneyError::Overflow` is explicitly part of the task and needs a fallible return path.
- **Resolved (architect, 2026-10-25):** Q2 — option (a). Private fields with validating constructors, a `const fn from_const` for literals, and `MONEY_MAX_ABS_MINOR` enforced by `Money::new` and deserialisation (new `MoneyError::OutOfRange`, AC7, AC8). `spec/04-policy-engine.md` §5 was updated to match, and AC2 now uses ±(cap/2) operands so sums stay in range.
- **Q2 (blocking):** May T-102 revise the Money and constrained policy-type interfaces to use private fields with validated constructors/`TryFrom` and serde/schema implementations that enforce the canonical bounds?
  - Context: T-102 and `spec/04-policy-engine.md` §5 show public fields/tuple constructors, including constructible invalid values. `spec/23-contracts-and-canonical-forms.md` CAN-10 caps the Money wire amount at 10^15, while `spec/16-code-quality.md` CQ-23 requires private fields and validating constructors when a value can be invalid. The task does not specify how these requirements fit together.
  - Options: (a) amend the task interface and tests for private validated types, with the Money canonical bound enforced; (b) retain the public shapes and have the maintainer explicitly resolve the conflicting invariant/validation contract in the task/spec before implementation.
  - Recommendation: (a), because it enforces CAN-10 and CQ-23 at the type boundary.
