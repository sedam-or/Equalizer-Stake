---
id: T-205
title: Add the pure scheduling domain — slot alignment, opening-hours fit, closures, booking window, capacity checks
milestone: M2
status: todo
depends_on: [T-202]
spec_refs: [SCH-01, SCH-02, SCH-04, SCH-06, SCH-15, DOM-12, ARC-10, TST-02, POL-02]
estimated_size: S
---

## Goal
All "is this slot allowed?" rules live in pure, property-tested functions that take policy values and data as arguments. Services never re-implement them.

## Context
- SCH-01: `duration ∈ offering.durations_minutes`, unless the actor has `custom_duration`. Then any multiple of `slot_step_minutes` in 10..=480 is allowed. `starts_at` in **location-local** time must have `minute_of_day % slot_step_minutes == 0`.
- SCH-02:
  - `[start, end)` (without buffer) must fit inside **one** interval of `opening_hours` for the local weekday.
  - `end` exactly at the interval end is OK. A session crossing local midnight never fits.
  - The violation rule key is `scheduling.opening_hours`.
- SCH-04: the local date (of `start`) inside any closure (tenant-wide or same location) → rule `scheduling.closure`.
- Booking window (policy `booking_window`): `start >= now + min_notice_hours` and `local_date(start) <= local_date(now) + max_days_ahead` → rules `scheduling.booking_window.min_notice` / `.max_days_ahead`. **Staff (`scheduling.session.write`) bypass the booking window.** It applies to portal and self-service flows (rental in M6).
- SCH-06: participants ≤ min(offering.capacity_max, resource.capacity) → 422 `policy_violation` rule `scheduling.capacity`. Below `capacity_min` → warning `below_min_capacity`.
- SCH-15: outside instructor availability (when the instructor has any rows) → warning `outside_instructor_availability`.
- DST: local ↔ UTC uses `chrono_tz`. A local start inside a DST gap → rule `scheduling.nonexistent_local_time`. In ambiguous times, take the earliest.

## Interfaces
```rust
pub struct SlotRequest<'a> { pub start: Timestamp, pub duration_min: u16, pub tz: Tz, pub participants: u32,
  pub offering: &'a OfferingRules, pub resource_capacity: u32, pub actor_caps: ActorCaps, pub now: Timestamp }
pub struct OfferingRules { pub durations: Vec<u16>, pub capacity_min: u32, pub capacity_max: u32 }
pub struct ActorCaps { pub custom_duration: bool, pub override_hours: bool, pub bypass_booking_window: bool }
pub struct SlotContext<'a> { pub policy: &'a SchedulingPolicy, pub closures: &'a [ClosureSpan], pub instructor_availability: &'a [(Uuid, Vec<DayHours>)] }
pub struct SlotVerdict { pub violations: Vec<Violation>, pub warnings: Vec<Warning> }
pub struct Violation { pub rule: &'static str, pub detail: String }
pub fn evaluate(req: &SlotRequest, ctx: &SlotContext, override_hours: bool) -> SlotVerdict;
pub fn fits_opening_hours(start_local: NaiveDateTime, end_local: NaiveDateTime, hours: &WeeklyHours) -> bool;
pub fn aligned(start_local: NaiveDateTime, step: u16) -> bool;
pub fn to_local(ts: Timestamp, tz: Tz) -> NaiveDateTime; pub fn from_local(l: NaiveDateTime, tz: Tz) -> Result<Timestamp, LocalTimeError>;
```
- `override_hours = true` (with the cap) suppresses `scheduling.opening_hours` and `scheduling.closure` only. Never capacity, alignment or conflicts.

## Scope
### In
- `eq_scheduling::domain::slot` as above. No DB access.

### Out
- Conflict checks (DB) and alternatives (T-207).

## Acceptance criteria
- AC1. Mon 14:30–17:00: 16:15+45 fits; 16:30+45 doesn't; 14:30+30 fits. → `slot::tests::opening_hours_examples`
- AC2. Property: if `fits_opening_hours(s, e)`, then for every minute m in [s, e) the local time is inside an open interval. → `proptest fits_implies_inside`
- AC3. Step 15: 15:10 is not aligned, 15:15 is. → `alignment`
- AC4. A closure on that date + override → no `scheduling.closure` violation. Without override → violation. → `closure_override`
- AC5. 4 participants, offering max 6, room capacity 3 → `scheduling.capacity`. 0 participants with min 2 → warning only. → `capacity_rules`
- AC6. `America/New_York` 2026-03-08 02:30 → `scheduling.nonexistent_local_time`. → `dst_gap`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/src/domain/slot.rs` | AC names above |

## Definition of done
- [ ] Rust checks pass; `scripts/check-literals.sh` passes (no hours or minutes literals)
- [ ] Commit `feat(scheduling): slot rules domain [T-205]`, signed off

## Report

## Questions
