---
id: T-212
title: Add the scheduling event consumers (payer frozen/unfrozen, enrollment on-leave/ended) and the series-membership port impl
milestone: M2
status: todo
depends_on: [T-211]
spec_refs: [SCH-13, ARC-07, ARC-08, MOD-08]
estimated_size: S
---

## Goal
Scheduling reacts correctly to billing and enrollment, which don't exist yet, through events and ports only. When those modules arrive (M4), nothing in scheduling changes.

## Context
- **Event payload types are needed before their producers exist.** Define the payload structs in `eq_core::events` (shared contracts, `DomainEvent` impls with the exact `TYPE` strings) for:
  - `billing.payer_frozen.v1 {payer_id, reason, invoice_ids}`
  - `billing.payer_unfrozen.v1 {payer_id, by}`
  - `enrollment.enrollment_on_leave.v1 {enrollment_id, from, until}`
  - `enrollment.enrollment_ended.v1 {enrollment_id, reason}`

  Producers in M4 must use these same structs. **Also move the scheduling event payloads (`session_scheduled`, `session_rescheduled`, `session_cancelled`, `session_reverted`, `occurrence_skipped`, `session_completed`) into `eq_core::events`**, so consumers don't import `eq-scheduling`.
- Enrollment lookups the consumers need (`enrollment_id → student_id, offering_id, series_ids`) come from a port `EnrollmentLookup` in `eq_core::ports` (default impl: `None`). Payer → students comes from `person_links` (core table): the payer is a guardian, whose students are the linked ones; a payer who is the student themselves also counts.
- Handlers (`EventHandler`, module Scheduling):
  1. `scheduling.on_payer_frozen` (SCH-13): all `expected` participations of the payer's students in **future** scheduled sessions → `blocked`. Holds are kept. Audit `scheduling.participant.blocked` (system actor, one entry per session batch).
  2. `scheduling.on_payer_unfrozen`: `blocked` → `expected` for future sessions.
  3. `scheduling.on_enrollment_on_leave`: cancel the student's participations in future sessions of that enrollment's offering whose local date is in `[from, until]`, with credit 0 and reason `on_leave`, and release the participant holds. (`session_participants.enrollment_id` links them when set; otherwise match on series membership for that offering.)
  4. `scheduling.on_enrollment_ended`: set `series_members.active_to = ended_on` (from the lookup port), and cancel participations after that date (credit 0, reason `enrollment_ended`).
- `SeriesMembership` port impl (declared in T-209): `add_member(tx, series_id, person_id, enrollment_id, active_from)` and `end_member(tx, series_id, person_id, active_to)`, registered by `SchedulingModule::register_ports`.
- Blocked participants appear in DTOs with `status: "blocked"`. Instructors see the status but no reason.

## Scope
### In
- The shared event structs, the 4 handlers, the port impl, and `EnrollmentLookup` + its default.

### Out
- Producing these events (M4).

## Acceptance criteria
- AC1. A synthetic `payer_frozen` event in the outbox → after `drain_once`, the future participations of the payer's 2 children are `blocked`, past ones are unchanged, and holds still exist. → `consumers::frozen_blocks_future`
- AC2. Unfrozen → back to `expected`. → `unfrozen_restores`
- AC3. On-leave from the 10th to the 20th (with a test `EnrollmentLookup`) → participations inside the range are cancelled with credit 0, outside unchanged, and those holds released. → `on_leave_cancels_range`
- AC4. The same event delivered twice → the same final state (idempotent). → `consumers_idempotent`
- AC5. `scripts/check-deps.sh` passes: `eq-scheduling` has no dependency on `eq-billing` or `eq-enrollment`. → CI

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/consumers.rs` | AC1–AC4 |

## Definition of done
- [ ] Rust checks pass; guard scripts pass
- [ ] Commit `feat(scheduling): cross-module consumers and ports [T-212]`, signed off

## Report

## Questions
