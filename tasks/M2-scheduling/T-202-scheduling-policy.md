---
id: T-202
title: Add SchedulingPolicy with validation, summary, x-ui hints, and the SchedulingModule wiring
milestone: M2
status: todo
depends_on: [T-201]
spec_refs: [POL-01, POL-06, POL-07, SCH-01, SCH-05, SCH-08, SCH-09, SCH-10]
estimated_size: S
---

## Goal
Every scheduling business value (hours, buffer, durations, step, holiday handling, recurrence cap, cancellation tiers, alternatives) is a validated, versioned policy field with a plain-language summary.

## Context
Normative struct (spec 04 §6.3). Defaults are the **generic** defaults (presets override them in M3).
```rust
pub struct SchedulingPolicy {
  pub opening_hours: WeeklyHours,          // E, widget weekday-hours; default Mon–Fri 09:00–21:00, Sat 09:00–17:00, Sun closed
  pub buffer_minutes: u16,                 // E 0..=120, default 10
  pub slot_step_minutes: u16,              // A one of 5,10,15,30,60; default 15
  pub default_durations_minutes: Vec<u16>, // E chips, each 10..=480, 1..=8 items, default [30,45,60]
  pub booking_window: BookingWindow,       // A
  pub holidays: HolidayRules,              // A
  pub recurrence: RecurrenceRules,         // A, x-ui requires_module = enrollment
  pub cancellation: CancellationRules,     // E
  pub alternatives: AlternativeSearch,     // A
}
pub struct BookingWindow { pub min_notice_hours: u16 /*0..=720, 0*/, pub max_days_ahead: u16 /*1..=365, 90*/ }
pub struct HolidayRules { pub on_closure: ClosureAction /*ShiftToNextWeek*/, pub when_no_slot: NoSlotAction /*MakeupCredit*/ }
pub enum ClosureAction { Skip, ShiftToNextWeek, MakeupCredit }
pub enum NoSlotAction { Skip, MakeupCredit, Substitute }
pub struct RecurrenceRules {
  pub materialize_weeks_ahead: u8,         // 1..=26, default 6
  pub sessions_per_cycle: Option<u8>,      // 1..=31 when Some, default None
  pub extra_occurrence: ExtraOccurrence,   // default Regular; x-ui visible_if sessions_per_cycle != null
}
pub enum ExtraOccurrence { Regular, Skip, Substitute { offering_key: String, duration_minutes: u16, consumes_credit: bool } }
pub struct CancellationRules {
  pub payer_can_cancel: bool,              // default true
  pub notice_tiers: Vec<NoticeTier>,       // E widget tiers; default [{min_hours_before: 24, makeup_credit_bps: 10000}]
  pub staff_cancel_credit_bps: Bps,        // default 10000
  pub instructor_absence: InstructorAbsence, // default MakeupCredit
  pub student_no_show: NoShow,             // E default Forfeit
  pub reason_required: bool,               // default true
}
pub struct NoticeTier { pub min_hours_before: u16, pub makeup_credit_bps: Bps }
pub enum InstructorAbsence { SubstituteThenReschedule, Reschedule, MakeupCredit }
pub enum NoShow { Forfeit, MakeupCredit }
pub struct AlternativeSearch { pub count: u8 /*1..=10, 3*/, pub radius_minutes: u16 /*15..=480, 60*/ }
```
- Serde (matches the preset YAML in spec 05 §1.3): **externally tagged, `rename_all = "snake_case"`**. Unit variants are plain strings (`"skip"`, `"substitute_then_reschedule"`). Data-carrying variants are single-key objects: `{"substitute": {"offering_key": "theory_class", "duration_minutes": 30, "consumes_credit": false}}`. Use the same convention in every policy struct.
- `validate()` (cross-field):
  1. Opening intervals are ordered and non-overlapping (already enforced by `DayHours`); at least one open day → otherwise `/opening_hours` `at_least_one_open_day`.
  2. Every default duration is a multiple of `slot_step_minutes` → `/default_durations_minutes/<i>` `multiple_of {step}`.
  3. Notice tiers have strictly descending `min_hours_before` → `/cancellation/notice_tiers/<i>/min_hours_before` `must_be_lt_previous`.
  4. `sessions_per_cycle` is `Some(n)` with n in 1..=31.
- The service-level check: implement `SchedulingModule::validate_policy_in_context` (trait method from T-105, already called by the T-113 PUT handler). `Substitute.offering_key` must exist in the catalog → `/recurrence/extra_occurrence/offering_key` `unknown_offering`.
- Summary lines (keys under `policy.scheduling.summary.*`):
  - `hours` `{days: [{weekday, intervals: ["14:30–17:00"]}]}`
  - `buffer {minutes}`
  - `durations {list}`
  - `cancellation_tiers {tiers}` or `cancellation_none`
  - `staff_cancel {bps}`
  - `no_show {mode}`
  - `holidays {on_closure, when_no_slot}`
  - `cycle_cap {n, extra}` when Some
  - `alternatives {count, radius}`
- x-ui: groups as marked (E/A); widgets: `weekday-hours`, `duration` (buffer), `chips` (durations), `tiers` (notice_tiers, columns `min_hours_before` unit hours and `makeup_credit_bps` widget percent), `percent` (staff_cancel_credit_bps), `select` (enums), `offering-ref` (offering_key).
- Permissions declared by scheduling (from `spec/modules/scheduling.md` §3):

| key | roles |
|---|---|
| scheduling.catalog.read | owner, admin, instructor, guardian, student |
| scheduling.catalog.write | admin |
| scheduling.series.write | admin |
| scheduling.session.read | owner, admin, instructor, guardian, student (scoped) |
| scheduling.session.write | admin |
| scheduling.session.cancel | admin, guardian, student (participant-only, enforced in service) |
| scheduling.session.override_hours | admin |
| scheduling.session.custom_duration | admin |
| scheduling.override.revert | owner |

- Nav items: `scheduling.calendar` (`/schedule`, icon `calendar`, main, order 10, perm `scheduling.session.read`), `scheduling.catalog` (`/catalog`, `book-open`, admin, 20, `scheduling.catalog.write`), `scheduling.rooms` (`/rooms`, `door-open`, admin, 21, `scheduling.catalog.write`).

## Scope
### In
- `eq_scheduling::policy`, the `SchedulingModule` manifest, policy wiring, permissions and nav, and `validate_policy_in_context`.

### Out
- Using the policy (T-205+).

## Acceptance criteria
- AC1. Default policy validates, and its JSON Schema snapshot matches (insta). → `policy::tests::default_valid`, `schema_snapshot`
- AC2. Durations `[45]` with step 30 → error pointer `/default_durations_minutes/0`. → `durations_multiple_of_step`
- AC3. Tiers `[{12,…},{24,…}]` → error at `/cancellation/notice_tiers/1/min_hours_before`. → `tiers_descending`
- AC4. An all-closed week → `at_least_one_open_day`. → `requires_open_day`
- AC5. Summary of the default policy matches an insta snapshot. → `summary_snapshot`
- AC6. Saving with `Substitute{offering_key:"nope"}` via the API → 422 with pointer `/recurrence/extra_occurrence/offering_key`. → `api::unknown_offering_key_rejected`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/src/policy.rs` | AC1–AC5 |
| `crates/eq-scheduling/tests/api_policy.rs` | `unknown_offering_key_rejected` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(scheduling): scheduling policy [T-202]`, signed off

## Report

## Questions
