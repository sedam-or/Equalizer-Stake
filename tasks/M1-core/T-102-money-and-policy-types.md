---
id: T-102
title: Add Money, Bps and the policy value types (LocalTime, WeeklyHours, DayOfMonth, I18nText, GeoPoint, Locale, Currency)
milestone: M1
status: review
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
#[serde(try_from = "String")]
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
pub fn apply_bps(amount_minor: i64, bps: Bps) -> Result<i64, MoneyError>; // (amount*bps + 5000) / 10000 for amount>=0; symmetric for negative; the `Err(Overflow)` arm is unreachable for a valid `Bps` and is not tested
```

## Business rules
None (primitives).

## Implementation notes
- Use `i128` intermediates in `apply_bps` to avoid overflow, and convert back with `i64::try_from(..).map_err(|_| MoneyError::Overflow)` (unreachable for a valid `Bps`, so no `unwrap`/`expect`). `Money::checked_add`, `checked_sub`, `checked_mul` and `Money::apply_bps` return `Overflow` when the result's magnitude exceeds `MONEY_MAX_ABS_MINOR` (which also covers i64 overflow), so a `Money` value is always in range. `Money::new` and deserialisation return `OutOfRange` for input beyond the cap.
- `Bps` is 0..=10000 and `Bps::new` rejects more. A value that must exceed 100% would be a different field type, not a widened `Bps`; v1 has no such case.
- `GeoPoint::new` rejects NaN and out-of-range values; `I18nText::new` rejects an empty map; `TimeInterval::new`, `DayHours::new` and `WeeklyHours::new` run the same checks as their deserialisation.
- `LocalTime` "24:00" is allowed only as `TimeInterval.end`. A standalone `LocalTime` is 0..=1439 minutes; the interval stores its end as 0..=1440 minutes so the 1440 sentinel cannot become a standalone LocalTime.

## Acceptance criteria
- AC1. `apply_bps(10_000_000, Bps::from_const(7000)) == Ok(7_000_000)`; `apply_bps(15, Bps::from_const(5000)) == Ok(8)` (half-up); `apply_bps(i64::MAX, Bps::FULL) == Ok(i64::MAX)`; `apply_bps(i64::MIN, Bps::FULL) == Ok(i64::MIN)`. A valid `Bps` (0..=10000) never scales a magnitude up, so `apply_bps` cannot overflow for a valid input: it keeps the `Result` for the `i64::try_from` of its `i128` intermediate and has no overflow test of its own. → `money::tests::bps_half_up`
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
- [x] Code quality and architecture follow `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` (no tables, no new `#[allow]`)
- [ ] `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them; both are absent and skipped per WORKFLOW Step 11
- [x] Rust checks pass
- [ ] `scripts/check-literals.sh` passes; it is absent and skipped per WORKFLOW Step 11 before T-006
- [x] Commit `feat(core): money and policy value types [T-102]`, signed off

## Report
**Plan** — Implement the architect-resolved Money, Currency and policy value type interfaces. Add the exact named unit and property tests first, then implement validated constructors and serde boundaries, canonical schemas, ISO currency exponent lookup, and run all available Rust and guard checks.

**Assumptions** — No business rules are inferred. Architect resolutions Q1-Q3 are authoritative. Currency input follows CAN-10, and iso_currency is the exponent source under OSS-02. Two mechanical interface ambiguities were self-resolved and are listed under Deviations and Questions for architect ratification.

**Test plan** —
- AC1: Given signed minor-unit values and valid Bps, when apply_bps runs, then it returns the expected result, handles both i64 extremes at FULL and rounds half-up for both signs: money::tests::bps_half_up.
- AC2: Given random same-currency operands within plus/minus MONEY_MAX_ABS_MINOR / 2, when adding then subtracting, then the original Money is recovered: money::proptests::add_sub_roundtrip.
- AC3: Given any i64 amount, when applying FULL or ZERO, then the input or zero is returned: money::proptests::bps_identity.
- AC4: Given day 29, when constructed or deserialized as DayOfMonth, then validation fails: policy::types::tests::day_of_month_max_28.
- AC5: Given overlapping intervals, when constructing or deserializing DayHours, then validation fails: policy::types::tests::day_hours_rejects_overlap.
- AC6: Given WeeklyHours, when its JSON Schema is generated, then the array has minItems and maxItems of 7: policy::types::tests::weekly_hours_schema_snapshot.
- AC7: Given a Money amount one above the canonical cap, when constructed or deserialized, then it is rejected: money::tests::money_rejects_out_of_range.
- AC8: Given invalid bounded or structured policy values, when passed to constructors or serde, then both reject them: policy::types::tests::constrained_types_reject_invalid.
- Additional exact tests: money::tests::currency_mismatch, money::tests::overflow_detected, money::proptests::bps_monotonic, policy::types::tests::local_time_format, policy::types::tests::interval_end_2400_ok, policy::types::tests::i18n_text_requires_entry and policy::types::tests::geo_bounds.
- Schema bounds are checked for Bps, MinorAmount, DayOfMonth, Money, GeoPoint, I18nText, LocalTime and TimeInterval by policy::types::tests::constrained_schema_bounds.
- Edge cases: positive and negative half-up ties, i64 extrema, Money cap boundaries, zero and full Bps, unsupported and lowercase currency codes, 24:00 only at interval end, malformed times, touching and overlapping intervals, closed hours, weekday order, day 28/29, empty translations, coordinate endpoints, non-finite and out-of-range coordinates.
- Error cases: currency mismatch, Money overflow and out-of-range values, rejected Bps and policy values, invalid intervals and malformed serde input.
- Abuse cases: n/a; pure value types introduce no SBD-30 trigger.

**Summary** — Added canonical Money and Currency primitives, checked arithmetic, half-up Bps calculations and shared validated policy types. Constructors and serde enforce the documented bounds; generated schemas retain those constraints, including weekly length, nonempty localized text and the interval-only 24:00 end.

**Spec read** — spec/02-architecture.md section 9 (ARC-11); spec/04-policy-engine.md section 5 (POL-11/12); spec/adr/0007-money-minor-units.md; spec/16-code-quality.md CQ-23 and section 12; spec/17-clean-architecture.md sections 1 and 4 (CA-01/31); spec/18-security-by-design.md section 5; spec/19-error-handling.md section 10; spec/23-contracts-and-canonical-forms.md section 2 (CAN-10/12); spec/24-open-source-stack.md sections 1 and 2 (OSS-02).

**Files** —
- Created crates/eq-core/src/money.rs with Money, Currency, checked arithmetic, apply_bps, formatting helper and required tests.
- Created crates/eq-core/src/policy/mod.rs and crates/eq-core/src/policy/types.rs with validated policy value types, schemas and tests.
- Modified crates/eq-core/src/lib.rs to expose money and policy modules.
- Modified crates/eq-core/Cargo.toml and Cargo.toml to use the mapped iso_currency dependency and schema, serde, SQLx, error, OpenAPI and test dependencies.
- Modified Cargo.lock for iso_currency and the resolved workspace dependency graph.
- Modified tasks/M1-core/T-102-money-and-policy-types.md with the resolved plan, Run 3 Report and self-resolution entries.
- Regenerated tasks/INDEX.md through scripts/tasks-index.sh.

**Acceptance criteria -> tests**

| AC | Test | Result |
|---|---|---|
| AC1 | money::tests::bps_half_up | Passed |
| AC2 | money::proptests::add_sub_roundtrip | Passed |
| AC3 | money::proptests::bps_identity | Passed |
| AC4 | policy::types::tests::day_of_month_max_28 | Passed |
| AC5 | policy::types::tests::day_hours_rejects_overlap | Passed |
| AC6 | policy::types::tests::weekly_hours_schema_snapshot | Passed |
| AC7 | money::tests::money_rejects_out_of_range | Passed |
| AC8 | policy::types::tests::constrained_types_reject_invalid | Passed |
| Additional required names | currency_mismatch, overflow_detected, bps_monotonic, local_time_format, interval_end_2400_ok, i18n_text_requires_entry, geo_bounds | Passed |
| Schema assertions | policy::types::tests::constrained_schema_bounds | Passed |

**Checks run** —
- Passed: cargo fmt --all --check.
- Passed: cargo clippy --workspace --all-targets -- -D warnings.
- Passed: cargo test --workspace (17 eq-core unit tests, the existing CLI test, all workspace targets and doc-tests).
- Passed: cargo test -p eq-core --lib (17 tests).
- Passed: scripts/check-deps.sh.
- Passed: scripts/tasks-index.sh --check.
- Passed: git diff --check.
- Passed: gitleaks dir --redact --no-banner --no-color on each changed source, manifest, task and index path; no leaks found.
- Skipped per WORKFLOW Step 11 before T-006: scripts/check-literals.sh, scripts/check-web-boundaries.sh and scripts/check-layers.sh are absent from scripts/. No replacement was run or marked green.
- n/a: web checks; web/ was not changed by this task.
- n/a: API generation and cargo sqlx prepare; no API or SQL query changed.
- n/a: make dev and database tests; this task adds pure value types with no server runtime or DB behavior.

**Manual checks** — n/a; this task adds no runtime endpoint or standalone process. Serialization, deserialization and schema behavior were exercised by unit tests.

**Debug log** —
- Initial test compilation found a missing Money import in the schema test and an invalid equality assertion on Result<Currency, serde_json::Error>; imported Money and asserted the deserialized value with expect.
- Schema assertions exposed that Schemars length validation does not add minProperties for BTreeMap. Added an explicit minProperties extension to the I18nText schema helper; the schema test then passed.
- The first full Clippy run found the required uppercase ISO currency variants, unfulfilled panic lint expectations and a needless test borrow. Removed lint expectations that Clippy did not emit and the unnecessary borrow. A second attempt showed an enum-level upper_case_acronyms expectation did not cover enum variants; moved the expectation to the money module with the ISO-code reason. Targeted and final workspace Clippy checks passed.
- No unresolved failures remain.

**Performance** — n/a; no runtime path or query was added.

**Security** — No SBD-30 triggers. No secrets, PII, endpoint, SQL query or external-system behavior was introduced. Constructors and serde validate inputs at the type boundary. Gitleaks scanned all changed source, manifest, task and index paths with no findings. No abuse-case test applies.

**Deviations** — Decision register references: TD-024, TD-025 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).
- RATIFY: Currency's derive-only deserialization sketch and unspecified helper signatures omitted CAN-10 lowercase-input normalization and the iso_currency lookup shape. Deserialization uses TryFrom<String>, output uses uppercase canonical codes, lookups expose Option, and formatting uses the ISO exponent. This follows CAN-10 and OSS-02 without adding a business value.
- RATIFY: The public LocalTime from_const sketch alongside CAN-10's rule that 24:00 is only a TimeInterval end could create a standalone invalid LocalTime. Standalone LocalTime is capped at 23:59; the interval stores 1440 privately and exposes until_midnight for this boundary. This is the stricter canonical reading.
- Cargo.lock was updated for the explicitly mapped iso_currency dependency and resolved workspace graph (OSS-04).

**Dependencies affected** — T-102a, T-103 and T-106 directly depend on T-102. Their implementations can use the validated public types after this task is review.

**Rollback plan** — Revert the single T-102 task commit. No migration or persistent data was added.

**Follow-ups** — Architect ratification is needed for the two mechanical resolutions listed under Deviations. The three guard scripts are expected from T-006; run them once present and before the milestone PR is made ready. The implementation is approximately 895 non-test source lines, above the task README's approximate 400-line M-size guide; review whether the broad task was intentionally scoped as one task.

**Step 0 - Start** — Started from Run 3 commit f15dc08fc1880ab82af623238f32d90427f2a5e6. T-001 was review and the only dependency of T-102. Checked out existing milestone/m1-core and merged milestone/m0-foundation. T-102 was the lowest eligible task. The tracked tree was clean before claiming; scripts/tasks-index.sh --check passed. Set this task in_progress and regenerated the index.

**Step 1 - Read task and specs** — Read the entire T-102 task, including architect Resolved lines Q1-Q3, historical Questions and prior Report. Read the cited and mandatory spec sections listed above. The resolved contract makes apply_bps fallible, requires private validated fields and the Money CAN-10 cap, and removes the impossible apply_bps overflow clause.

**Step 2 - Read dependencies** — Read T-001's Report, especially Deviations and Follow-ups, and its shipped eq-core scaffold. Cargo.lock's prior inclusion is a recorded OSS-04 deviation; the PostgreSQL follow-up does not affect this database-independent task. T-001 is review under Run 3 ordering.

**Step 3 - Design reference** — n/a; no UI.

**Step 4 - Plan and test plan** — Recorded above before implementation. Added tests for the exact AC and listed names first, then implemented only the validated value surfaces needed. Self-resolutions and reasons are recorded in Questions and Deviations.

**Step 5 - Implement** — Added bounded Money and Currency, checked same-currency arithmetic, half-up apply_bps, ISO exponent lookup, canonical currency deserialization, validated policy types, private interval end-of-day storage and JSON Schemas. Added the required unit, property and schema tests. No database, web, API or unrelated task code changed.

**Step 6 - Run locally** — n/a; pure value types introduce no runtime endpoint or process behavior. Constructor, serialization and deserialization paths ran through unit tests.

**Step 7 - Debug loop** — Fixed the compile and schema issues recorded in Debug log. For the repeated acronym lint failure, the enum-level expectation did not cover variants; the module-level expect was the alternative approach and passed. Final workspace lint and tests are green.

**Step 8 - Self-review** — Reviewed the diff against spec/16 section 9 and spec/17 anti-patterns. Invalid values have private fields and validated constructors/TryFrom deserialization; money has no floating-point arithmetic; schema bounds are tested. No SQL, endpoint, cross-module dependency, production unwrap, dbg, todo, allow attribute, PII or secret was introduced. The approximate size and absent pre-T-006 guards are recorded in Follow-ups and Checks.

**Step 9 - Optimize** — n/a; no measured runtime or query exists.

**Step 10 - Security audit** — No SBD-30 trigger. Reviewed validation at constructors and serde boundaries and ran Gitleaks against each changed source, manifest, task and index path; it found no leaks.

**Step 11 - Revise and verify** — Applied code-review findings and reran the exact workspace format, Clippy and test command; all passed. Ran check-deps, tasks-index, diff hygiene and Gitleaks successfully. WORKFLOW Step 11 permits scripts not yet created before T-006 to be skipped; the three absent guards are not claimed as passed.

**Step 12 - Report** — Wrote this Report, set T-102 to review and regenerated tasks/INDEX.md. The single signed-off task commit contains implementation, tests, dependency lock, task report and generated index.

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
- **Resolved (architect, 2026-10-25):** Q3 — option (a). The impossible `apply_bps` overflow clause is removed from AC1. `MoneyError::Overflow` is covered by `Money::checked_add`/`checked_sub`/`checked_mul` beyond `MONEY_MAX_ABS_MINOR`, in the existing test `money::tests::overflow_detected` (`Money::new(MONEY_MAX_ABS_MINOR, IDR)` plus one minor unit, and `checked_mul`).
- **Q3 (historical; resolved):** How should T-102 resolve AC1's demand for an `apply_bps` result outside i64 when the task restricts `Bps` to 0..=10000 and the amount to i64?
  - Context: The resolved interface returns `Result<i64, MoneyError>`, but `Bps` is valid only from 0 through 10000 (spec/04 §5; T-102 Scope and implementation notes). For any i64 amount, applying at most 100% and rounding half-up cannot produce a result whose magnitude exceeds the input, so the AC1 overflow example has no valid input. `MoneyError::Overflow` can still be exercised by checked Money arithmetic that exceeds `MONEY_MAX_ABS_MINOR` (CAN-10).
  - Options: (a) remove the impossible `apply_bps` overflow clause and cover `MoneyError::Overflow` through checked Money arithmetic; (b) define a different rate/input type and update the API and canonical contract to specify a valid overflow case.
  - Recommendation: (a), because it keeps `Bps` within the explicit 0..=10000 contract and tests overflow where it is reachable.
- **Self-resolved (implementer, run 3, needs architect ratification):** Currency's derive-only deserialization sketch and unspecified helper signatures do not state CAN-10 lowercase-input normalization or the `iso_currency` lookup shape -> deserialize through `TryFrom<String>`, serialize uppercase canonical codes, expose `Option` for code/exponent lookup, and format minor units with the ISO exponent because CAN-10 and OSS-02 define the wire form and metadata source. [TD-024]
- **Self-resolved (implementer, run 3, needs architect ratification):** `LocalTime` requires `from_const` while CAN-10 permits `24:00` only as a `TimeInterval.end` -> cap standalone `LocalTime` at 23:59 and store minute 1440 only in the private interval-end representation, with `TimeInterval::until_midnight` for construction, because canonical local times must remain valid outside interval context. [TD-025]
