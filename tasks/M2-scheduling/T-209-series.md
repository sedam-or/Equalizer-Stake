---
id: T-209
title: Add recurring series (create with atomic multi-week validation, edit from a date, end, manual members)
milestone: M2
status: todo
depends_on: [T-208]
spec_refs: [SCH-07, SCH-08, DOM-10, API-07]
estimated_size: M
---

## Goal
An admin creates a weekly slot ("Voice group, Tue 15:00, Room 2, Ayu"). Its first weeks are booked atomically, either all of them or none, with the first conflict explained.

## Context
- Table `series` (T-201): `offering_id, location_id, resource_id, instructor_id, weekday, start_local (time), duration_minutes, capacity, valid_from, valid_to?, materialized_until?, status active|ended, version`. Members: `series_members(series_id, person_id, enrollment_id?, active_from, active_to?)`.
- API (perm `scheduling.series.write`, reads for staff):
  - `GET /series?location_id&instructor_id&status&cursor`, `GET /series/{id}` (includes members and next 5 occurrences).
  - `POST /series {offering_id, resource_id, instructor_id, weekday, start_local "HH:MM", duration_minutes, capacity, valid_from, valid_to?, member_ids?: [uuid]}` + Idempotency-Key → 201 Series | 409 `slot_taken {first_conflict_date, conflicts, alternatives}` | 422.
  - `PATCH /series/{id} {effective_date, start_local?, weekday?, duration_minutes?, resource_id?, instructor_id?, capacity?}` + If-Match. **Changes apply from `effective_date`:** future sessions (local date ≥ effective_date, status scheduled, not individually rescheduled, i.e. `rescheduled_from IS NULL` and the range still equals the series pattern) are cancelled with reason `series_changed` and credit 0, **and then regenerated** with the new pattern in the same tx. Any conflict → 409, and nothing changes.
  - `POST /series/{id}/end {end_date}`: sets `valid_to = end_date`, `status = ended` if end_date ≤ today. Future sessions after end_date are cancelled (reason `series_ended`, credit 0), and holds are released.
  - `POST /series/{id}/members {person_id, active_from}` and `DELETE /series/{id}/members/{person_id}?active_to=` (manual membership, used when enrollment is disabled; enrollment calls the same service in M4). Adding a member first locks the series row (`SELECT … FOR UPDATE`) and checks the active member count < `capacity` (else 422 `policy_violation` rule `scheduling.capacity`), then adds a participant + hold to already-materialised future sessions from `active_from` (409 if the student is busy on any → nothing changes; the response lists the dates). Removing sets `active_to` and cancels their future participations after that date (credit 0, reason `member_removed`).
- SCH-07 creation:
  1. Validate the slot rules for the pattern (`domain::slot::evaluate` with the first occurrence; capacity ≥ members).
  2. Insert the series.
  3. **Materialise the first `recurrence.materialize_weeks_ahead` weeks inside the same tx**, using the materialiser core defined by T-209, called with `mode = Strict`. Strict mode fails on the first conflict instead of skipping. Closure handling still applies (skip/shift per policy).
  4. On conflict → 409 with `first_conflict_date` (local) and the alternatives for that occurrence.
- T-209 owns the materialisation **core**; T-210 extends it with daily jobs, cycle caps and closure handling. Because T-210 comes after this task, put the materialisation **core** (`fn plan_occurrences(series, from, to, policy, closures, cycle_counts) -> Vec<PlannedOccurrence>` and `async fn apply(tx, plan, mode)`) here in `eq_scheduling::materialize`. T-210 adds the job, cycle cap and closure strategies on top. For this task, `plan_occurrences` handles: weekly dates in `[max(valid_from, from), min(valid_to, to)]` for the weekday, local start → UTC via the location tz, and **no** closure or cycle logic yet (T-210 extends it).
- Audit `scheduling.series.created|updated|ended`, `scheduling.series_member.added|removed`. Events: each created session emits `session_scheduled` (via the shared session-creation helper).

## Scope
### In
- The endpoints above, `eq_scheduling::materialize::{plan_occurrences, apply, Mode}` (basic), and the members service (public within the crate as `SeriesMembers` for enrollment to call through a port in M4: define the port `SeriesMembership` in `eq_core::ports` with `add_member` and `end_member`).

### Out
- The daily materialisation job, closures and the cycle cap (T-210).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-scheduling/tests/api_series.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `sbd_209_abuse` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Create a series with 6 free weeks → 6 sessions with holds. → `api_series::create_materializes_weeks_ahead`
- AC2. Week 4 occupied → 409 with `first_conflict_date` = that date, and zero sessions and series rows. → `create_atomic_conflict`
- AC3. PATCH start 15:00 → 16:00 from week 3 → weeks 1–2 unchanged; weeks 3–6 at 16:00; old holds released. → `patch_from_effective_date`
- AC4. End the series at week 2 → sessions after it are cancelled and holds freed. → `end_series`
- AC5. Adding a member whose student is busy on week 5 → 409 listing that date; no participants added. → `add_member_atomic`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/api_series.rs` | AC names above |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(scheduling): recurring series [T-209]`, signed off

## Report

**Deviations** — RATIFY: TD-069, T-209 owns the base occurrence planner; T-210 extends it.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-069):** the task text assigns the materialisation core to T-210 and later assigns it to T-209 → T-209 defines the reusable core and T-210 extends it, because T-209 precedes its consumer and its own implementation notes name that ownership.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
