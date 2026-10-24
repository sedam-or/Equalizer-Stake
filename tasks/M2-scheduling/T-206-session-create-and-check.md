---
id: T-206
title: Add one-off session creation and the dry-run check, with conflict-to-409 mapping
milestone: M2
status: todo
depends_on: [T-203, T-204, T-205]
spec_refs: [SCH-01, SCH-02, SCH-03, SCH-06, DOM-10, DOM-11, API-08, AUD-01]
estimated_size: M
---

## Goal
An admin books a session. Either it is stored with all holds in one transaction, or the response explains exactly what collided (with alternatives once T-207 lands) or which rule blocked it.

## Context
- `POST /api/v1/sessions` (perm `scheduling.session.write`, `Idempotency-Key` required):
  - Body `CreateSession {offering_id, resource_id, instructor_ids: [uuid] (1..=3), participant_ids: [uuid] (0..), starts_at, duration_minutes, kind: one_off|trial|makeup (default one_off), override_hours?: bool, notes?}`.
  - Responses: 201 `Session` + `ETag`; 409 `slot_taken {conflicts, alternatives: []}`; 422 `policy_violation {rule}` / `validation`.
  - The response may carry `warnings: [..]` (SCH-06/15).
- `POST /api/v1/sessions/check`: the same body (no idempotency key) → `{ok, violations: [{rule, detail}], conflicts, alternatives, warnings}` with **no writes**. Conflicts come from `find_overlaps` (advisory, DOM-11).
- Service steps for create, inside `db::begin`:
  1. Load the offering (not archived), the resource (not archived), its location, and the tz (location tz else tenant tz).
  2. Validate that instructors have an active `instructor` membership and participants an active `student` membership.
  3. `policy = PolicyStore::get::<SchedulingPolicy>(at = starts_at)`.
  4. Run `domain::slot::evaluate`. Violations → 422 (first violation's rule; all of them in `errors`).
  5. Insert the session (`buffer_minutes` = policy snapshot), instructors (role primary for the first, assistant for the rest) and participants.
  6. `calendar::claim(session_holds(...))`.
  7. Audit `scheduling.session.created` (after-snapshot; includes `override_hours: true` when used).
  8. Emit `scheduling.session_scheduled.v1 {session_id, offering_id, starts_at, instructor_ids, participant_ids}`.
  9. Commit.
- On `CalendarError::Conflict`: drop the tx. In a **new** tx, `find_overlaps` for all subjects and `calendar::describe`. Alternatives are an empty array until T-207 plugs in `AlternativesFinder`. Return 409. **The idempotency guard must not store a 409 from a conflict** (a retry may legitimately succeed later); store only 2xx.
- Session DTO (spec): `{id, offering: {id, key, name}, kind, status, starts_at, ends_at, local_date, local_start, duration_minutes, resource: {id, name}, location_id, instructors: [{id, display_name, role}], participants: [{id, display_name, status}], series_id?, consumes_credit, cancelled?: {at, by_role, reason}, version, warnings?}`.
- `GET /api/v1/sessions/{id}` (scoped: admin/owner all; instructor if teaching; guardian if a linked student participates; student if participating; else 404).
- Implement the `ScopeResolver` port from T-112 for the Instructor scope: the people visible to an instructor are participants of sessions they teach from 30 days ago (technical const) onwards.

## Scope
### In
- Create, check, get-by-id, the DTO mapping, the scope port, and the `AlternativesFinder` trait (stub returns `vec![]`).

### Out
- Alternatives (T-207). Reschedule/cancel (T-208). Lists (T-211).

## Acceptance criteria
- AC1. Buffer: session 15:00–15:45 in Room 1, buffer 10 → booking Room 1 at 15:50 → 409 with conflict subject `resource`. At 15:55 → 201. → `api_sessions::buffer_rule`
- AC2. Instructor clash: T teaches 15:00–15:45 in Room 1 → T in Room 2 at 15:30 → 409, subject = T with `subject_name` = T's display_name. → `instructor_clash`
- AC3. Student back-to-back: S in 15:00–15:45 → S in a 15:45 session → 201. → `student_no_buffer`
- AC4. Opening hours Mon 14:30–17:00 → 16:30+45 → 422 `scheduling.opening_hours`. With `override_hours: true` as admin → 201, and the audit `after.override_hours == true`. → `opening_hours_and_override`
- AC5. `/sessions/check` returns the same conflict as create but writes nothing (row counts unchanged). → `check_is_dry_run`
- AC6. The same Idempotency-Key twice → one session, identical 201. → `create_idempotent`
- AC7. A guardian GETs a session of an unrelated student → 404. → `get_scoped`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/api_sessions.rs` | AC names above |

## Definition of done
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(scheduling): create and check sessions [T-206]`, signed off

## Report

## Questions
