---
id: T-102
title: Add Money, Bps and the policy value types (LocalTime, WeeklyHours, DayOfMonth, I18nText, GeoPoint, Locale, Currency)
milestone: M1
status: blocked
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
**Plan** — After Q1 and Q2 were resolved, use the task's updated interfaces:
1. Add the exact money tests first, then implement `Currency`, `Money`, bounded `Bps`, ISO currency exponents via `iso_currency`, and checked arithmetic → verify each named money unit/property test.
2. Add policy value-type tests first, then implement private fields, validating constructors/`TryFrom`, canonical serde and `from_const` functions → verify exact validation and boundary tests.
3. Add schemars constraints and the `WeeklyHours` schema snapshot → verify the seven-item bounds in the snapshot.
4. Wire `eq_core::money` and `eq_core::policy::types` plus required dependencies → verify workspace metadata and dependency guards.
5. Run the Rust checks, relevant guards, and any applicable local behavior checks → record unavailable checks honestly.

**Assumptions** — None about the contracts. Use the resolved task interfaces: fallible `apply_bps`, bounded `Bps`, private validated fields, and the `MONEY_MAX_ABS_MINOR` cap. Use `iso_currency` for exponents as required by OSS-02. No business rule or schema assumption is made.

**Test plan** —
- AC1: Given a valid amount and Bps, when applying 7000 or 5000 bps, then 10,000,000 becomes 7,000,000 and 15 rounds half-up to 8; at `Bps::FULL`, `i64::MAX` is unchanged → `money::tests::bps_half_up`. The additional demand for an `apply_bps` result outside i64 is unsatisfiable with `Bps` in 0..=10000 and an i64 amount; this blocks implementation pending Q3.
- AC2: Given random same-currency amounts within ±(`MONEY_MAX_ABS_MINOR` / 2), when adding then subtracting, then the original amount is recovered → `money::proptests::add_sub_roundtrip`.
- AC3: Given any valid amount, when applying `Bps::FULL` or `Bps::ZERO`, then the amount or zero is returned → `bps_identity`.
- AC4: Given serialized day 29, when deserialized, then validation fails → `types::tests::day_of_month_max_28`.
- AC5: Given overlapping intervals, when `DayHours` is built/deserialized, then validation fails → `day_hours_rejects_overlap`.
- AC6: Given the `WeeklyHours` schema, when serialized, then it has `minItems: 7` and `maxItems: 7` → `weekly_hours_schema_snapshot`.
- AC7: Given an amount one over `MONEY_MAX_ABS_MINOR`, when constructed or deserialized, then it is rejected → `money::tests::money_rejects_out_of_range`.
- AC8: Given invalid bounded or structured policy values, when passed to constructors and serde, then both paths reject them → `types::tests::constrained_types_reject_invalid`.
- Other exact tests: `money::tests::currency_mismatch`, `money::tests::overflow_detected`, `money::proptests::bps_identity`, `money::proptests::bps_monotonic`, `types::tests::local_time_format`, `types::tests::interval_end_2400_ok`, `types::tests::i18n_text_requires_entry`, and `types::tests::geo_bounds`.
- Edge cases: both Money cap boundaries and one beyond; Bps 0 and 10000 and one beyond; half-up ties for both signs; checked add/subtract/multiply at and beyond the Money cap; supported and unsupported currency codes; time minute boundaries and `24:00` only at interval end; touching/overlapping intervals; closed and open day hours; weekly weekday order; day 28/29; empty and populated i18n text; GeoPoint endpoints, out-of-range values and NaN.
- Error cases: currency mismatch, checked arithmetic overflow, out-of-range Money, invalid constructor/serde inputs, malformed time, invalid interval/schedule, empty I18nText, and invalid coordinates.
- Abuse cases: n/a; pure value types do not introduce an SBD-30 trigger.

**Step 0 — Start** — Preflight began clean at `20f0e09b3a8d0bbd02c02ae72b2a8c75870332b3`. T-001 is `review`; T-002 is blocked, making later M0 tasks ineligible. T-102 is the lowest-ID eligible task because its only dependency, T-001, is `review` under this run's ordering rule. Created `milestone/m1-core` from `milestone/m0-foundation` at `75a9bc018b38b86eba11ff3a190e2c778750500f`; set T-102 `in_progress` and regenerated the index. The clean-tree and index checks passed before claiming.
**Step 1 — Task/spec** — Read the full task, its Q1/Q2 resolution lines, and the cited spec sections. Q1 now makes `apply_bps` fallible; Q2 makes constrained fields private and validated, with the Money cap enforced on construction and deserialization. The remaining AC1 overflow example contradicts the task's valid input bounds: `Bps` is 0..=10000 and `amount_minor` is i64, so the rounded result magnitude cannot exceed the input magnitude and always fits i64. `MoneyError::Overflow` remains reachable for checked Money addition, subtraction, or multiplication beyond the canonical cap.
**Step 2 — Dependencies** — Read T-001's Report and its shipped `eq-core` scaffold. Its Cargo.lock deviation and PostgreSQL follow-up do not block this pure primitive task; no money or policy module exists yet. T-001's `review` status satisfies this run's ordering rule. The workspace does not yet include `iso_currency`; add it at the required `eq_core::money` seam if implementation resumes.
**Step 3 — Design reference** — n/a; no UI work.
**Step 4 — Plan and test plan** — Recorded above. Blocked before implementation under `WORKFLOW.md` §4 because AC1 demands an impossible overflow result for the explicitly bounded `Bps` and `i64` inputs.
**Step 5 — Implement** — Not run: blocked at Step 4. No source or test code was changed.
**Step 6 — Run locally** — n/a: no implementation was made; these are pure value types with no standalone server behavior.
**Step 7 — Debug loop** — n/a: no implementation failure occurred. The blocker is a contract contradiction, not a failing test.
**Step 8 — Self-review** — No implementation diff to review. Reviewed the task/report changes against the resolved interfaces, CAN-10, CQ-23 and the Bps bounds; the Q3 blocker is documented below.
**Step 9 — Optimize** — n/a: no implementation or runtime path.
**Step 10 — Security audit** — No SBD-30 triggers: no secrets, PII, external systems, or endpoints are introduced. A secret scan of the changed task file and index is recorded under Checks run.
**Step 11 — Revise and full verification** — Task-index and diff hygiene checks were run. Rust and runtime checks were not run because Step 4 blocked all implementation. Guard scripts not yet present before T-006 are identified below.
**Step 12 — Report** — This report records the blocker. T-102 is `blocked`, `tasks/INDEX.md` was regenerated, and both are included in the required signed-off blocked-task commit on the M1 milestone branch.

**Summary** — No implementation or tests were added. The resolved Q1/Q2 interfaces are clear, but AC1 requires `apply_bps` to overflow for a valid `Bps` and i64 amount; the stated bounds make that impossible. The architect should remove that clause and keep overflow coverage on checked Money arithmetic, or explicitly revise the type/API and canonical contract.

**Spec read** — `spec/02-architecture.md` §9 (ARC-11); `spec/04-policy-engine.md` §5 (POL-11/12); `spec/adr/0007-money-minor-units.md`; `spec/16-code-quality.md` CQ-23 and §12; `spec/17-clean-architecture.md` §§1/4 (CA-01/31); `spec/18-security-by-design.md` §5; `spec/19-error-handling.md` §10; `spec/23-contracts-and-canonical-forms.md` §2 (CAN-10/12); `spec/24-open-source-stack.md` §§1/2 (OSS-02).

**Files** —
- Modified `tasks/M1-core/T-102-money-and-policy-types.md` with the revised Plan, test plan, Step 0–12 Report and Q3.
- Regenerated `tasks/INDEX.md` with `scripts/tasks-index.sh`.
- No implementation files changed.

**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `money::tests::bps_half_up`; overflow clause tracked by `money::tests::overflow_detected` | Not run: blocked at Step 4. The clause requiring `apply_bps` to overflow has no valid input under the task's Bps/i64 bounds. |
| AC2 | `money::proptests::add_sub_roundtrip` | Not run: blocked before implementation. |
| AC3 | `bps_identity` | Not run: blocked before implementation. |
| AC4 | `types::tests::day_of_month_max_28` | Not run: blocked before implementation. |
| AC5 | `day_hours_rejects_overlap` | Not run: blocked before implementation. |
| AC6 | `weekly_hours_schema_snapshot` | Not run: blocked before implementation. |
| AC7 | `money::tests::money_rejects_out_of_range` | Not run: blocked before implementation. |
| AC8 | `types::tests::constrained_types_reject_invalid` | Not run: blocked before implementation. |
| Additional listed tests | `currency_mismatch`, `overflow_detected`, `bps_identity`, `bps_monotonic`, `local_time_format`, `interval_end_2400_ok`, `i18n_text_requires_entry`, `geo_bounds` | Not run: blocked before implementation. |

**Checks run** — `scripts/tasks-index.sh` regenerated the index; `scripts/tasks-index.sh --check`, `git diff --check`, and `scripts/check-deps.sh` passed. `/home/kepolu/.local/bin/gitleaks dir --redact --no-banner --no-color` found no leaks in the changed task file or `tasks/INDEX.md`. Rust format, clippy, tests, build, `make dev`, API generation and SQLx preparation were not run because implementation is blocked. `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh` and `scripts/check-layers.sh` do not exist yet; they are introduced by T-006.

**Manual checks** — n/a; no runtime behavior was implemented.
**Debug log** — None; no code failure occurred.
**Performance** — n/a; no runtime path was added.
**Security** — No SBD-30 triggers. No secret-bearing fields, PII, or endpoint are added. Gitleaks scanned the changed task file and index and found no leaks.
**Deviations** — None. The architect-resolved interfaces are followed in the revised plan. No implementation was started.
**Dependencies affected** — T-102a and all later tasks depending on T-102 remain ineligible until resolution and completion. No dependency behavior changed.
**Rollback plan** — Revert the blocked-task commit; it changes only this task's report/status and the generated task index.
**Follow-ups** — Resolve Q3 by removing the impossible `apply_bps` overflow case from AC1 and assigning `MoneyError::Overflow` coverage to checked Money arithmetic, or provide an explicit alternative contract consistent with CAN-10 and the 0..=10000 Bps bound.
## Questions
- **Resolved (architect, 2026-10-25):** Q1 — option (a). `apply_bps` returns `Result<i64, MoneyError>`; Scope, Interfaces, AC1 and AC3 now say so. `Bps::ZERO` and `Bps::FULL` were added because T-208 and T-210 use `Bps::FULL`.
- **Q1 (historical; resolved):** Which free-function signature and acceptance-test contract should be authoritative for `apply_bps`?
  - Context: Scope line 40 and AC1/AC3 at lines 71/73 use a plain `i64` result, but the interface at line 60 returns `Result<i64, MoneyError>` and the implementation note at line 67 requires returning `Overflow` when the result does not fit.
  - Options: (a) make the fallible `Result<i64, MoneyError>` signature authoritative and update Scope/AC assertions to compare with `Ok(...)`; (b) retain `i64` and revise the overflow requirement to specify a non-error result for out-of-range values.
  - Recommendation: (a), because `MoneyError::Overflow` is explicitly part of the task and needs a fallible return path.
- **Resolved (architect, 2026-10-25):** Q2 — option (a). Private fields with validating constructors, a `const fn from_const` for literals, and `MONEY_MAX_ABS_MINOR` enforced by `Money::new` and deserialisation (new `MoneyError::OutOfRange`, AC7, AC8). `spec/04-policy-engine.md` §5 was updated to match, and AC2 now uses ±(cap/2) operands so sums stay in range.
- **Q2 (historical; resolved):** May T-102 revise the Money and constrained policy-type interfaces to use private fields with validated constructors/`TryFrom` and serde/schema implementations that enforce the canonical bounds?
  - Context: T-102 and `spec/04-policy-engine.md` §5 show public fields/tuple constructors, including constructible invalid values. `spec/23-contracts-and-canonical-forms.md` CAN-10 caps the Money wire amount at 10^15, while `spec/16-code-quality.md` CQ-23 requires private fields and validating constructors when a value can be invalid. The task does not specify how these requirements fit together.
  - Options: (a) amend the task interface and tests for private validated types, with the Money canonical bound enforced; (b) retain the public shapes and have the maintainer explicitly resolve the conflicting invariant/validation contract in the task/spec before implementation.
  - Recommendation: (a), because it enforces CAN-10 and CQ-23 at the type boundary.
- **Q3 (blocking):** How should T-102 resolve AC1's demand for an `apply_bps` result outside i64 when the task restricts `Bps` to 0..=10000 and the amount to i64?
  - Context: The resolved interface returns `Result<i64, MoneyError>`, but `Bps` is valid only from 0 through 10000 (spec/04 §5; T-102 Scope and implementation notes). For any i64 amount, applying at most 100% and rounding half-up cannot produce a result whose magnitude exceeds the input, so the AC1 overflow example has no valid input. `MoneyError::Overflow` can still be exercised by checked Money arithmetic that exceeds `MONEY_MAX_ABS_MINOR` (CAN-10).
  - Options: (a) remove the impossible `apply_bps` overflow clause and cover `MoneyError::Overflow` through checked Money arithmetic; (b) define a different rate/input type and update the API and canonical contract to specify a valid overflow case.
  - Recommendation: (a), because it keeps `Bps` within the explicit 0..=10000 contract and tests overflow where it is reachable.
