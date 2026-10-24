---
id: T-207
title: Add the alternative-slot finder (pure search plus DB loader) and plug it into 409 responses and /sessions/check
milestone: M2
status: todo
depends_on: [T-206]
spec_refs: [SCH-05, DOM-11, TST-02]
estimated_size: S
---

## Goal
When a slot is taken, the user immediately sees up to N bookable options, ordered the way a front-desk admin would try them.

## Context
- SCH-05. Up to `alternatives.count` options, in priority order:
  1. **Same time, other suitable resource:** same location, not archived, capacity ≥ participants, and if the original resource is rentable, any kind. Resources are ordered by `sort`, then name.
  2. **Same resource, nearest start within ±`radius_minutes`**, stepping by `slot_step_minutes`. Order by |Δ| ascending; on a tie, the earlier start first.
  3. **Other resource, nearest start within the radius** (same ordering, then resource order).
- Each candidate must satisfy:
  - `domain::slot::evaluate` with **no** override (opening hours, closures, alignment, capacity).
  - No overlap with the holds of **all** subjects (resource with buffer; instructors with buffer; participants without buffer), computed in memory from holds loaded for the window `[day_start_local - radius, day_end_local + radius]`.
- Duplicates are removed. Results are stable (deterministic).
- The pure function:
```rust
pub struct AltInput<'a> { pub req: &'a SlotRequest<'a>, pub ctx: &'a SlotContext<'a>, pub original_resource: Uuid,
  pub resources: &'a [ResourceInfo], pub instructors: &'a [Uuid], pub participants: &'a [Uuid],
  pub holds: &'a [HoldRow], pub buffer_min: u16, pub exclude_owner: Option<(OwnerKind, Uuid)> }
pub struct Alternative { pub resource_id: Uuid, pub resource_name: String, pub starts_at: Timestamp, pub ends_at: Timestamp, pub reason: AltReason } // SameTimeOtherResource | SameResourceNearby | OtherResourceNearby
pub fn find(input: &AltInput, count: u8, radius_min: u16, step_min: u16) -> Vec<Alternative>;
```
- `AlternativesFinder` (DB loader) implementation: load resources at the location, holds in the window for those resources plus instructors and participants, closures, and instructor availability, then call `find`. It runs in the read-only tx used for conflict description.
- Reschedule (T-208) passes `exclude_owner = Some(session)` so the session's own holds don't block its alternatives.

## Scope
### In
- `eq_scheduling::domain::alternatives::find` + the loader + wiring into the 409 body and `/sessions/check`.

### Out
- UI.

## Acceptance criteria
- AC1. Every returned alternative is bookable: in the test, book each via `POST /sessions` in a separate tx and expect 201, then roll back. → `api_alternatives::every_alternative_bookable`
- AC2. Priority order: with Room 2 free at the same time, the first alternative is Room 2 at the same time. → `same_time_other_room_first`
- AC3. `count = 3` → ≤ 3 results. Radius 60 → no start more than 60 min away. → `respects_count_and_radius`
- AC4. Property: for random hold layouts, `find` never returns a slot overlapping any hold of the relevant subjects (with buffer rules). → `proptest alternatives_never_overlap`
- AC5. Opening hours are respected (no alternative outside hours). → `alternatives_within_hours`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/src/domain/alternatives.rs` | `same_time_other_room_first` (pure), `respects_count_and_radius`, `alternatives_never_overlap`, `alternatives_within_hours` |
| `crates/eq-scheduling/tests/api_alternatives.rs` | `every_alternative_bookable` |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `feat(scheduling): alternative slot suggestions [T-207]`, signed off

## Report

## Questions
