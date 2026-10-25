---
id: T-208
title: Add reschedule, cancel with credit computation, participant/instructor changes, and owner revert
milestone: M2
status: todo
depends_on: [T-207]
spec_refs: [SCH-10, SCH-11, SCH-12, DOM-10, API-07]
estimated_size: M
---

## Goal
Sessions change safely after booking. Every change rewrites holds atomically, every cancellation records the credit the policy grants, and the owner can undo a cancellation when the slot is still free.

## Context
- **Reschedule** (SCH-12): `POST /sessions/{id}/reschedule {starts_at, duration_minutes?, resource_id?, instructor_ids?, reason}` + `If-Match`.
  - Status must be `scheduled` → else 409 `invalid_transition`.
  - Re-run slot evaluation with the policy at the **new** start. The buffer snapshot is updated to that policy.
  - `calendar::rewrite`, `version += 1`, audit `scheduling.session.rescheduled` (before/after ranges), emit `scheduling.session_rescheduled.v1 {session_id, old_range, new_range, reason}`.
  - On conflict → 409 with alternatives computed with `exclude_owner = this session`.
- **Cancel** (SCH-10): `POST /sessions/{id}/cancel {reason?, scope: "session"|"participant", participant_id?, cause?: "instructor_absence"}`.
  - `reason` is required when `cancellation.reason_required` → else 422 `validation` `/reason` `required`.
  - **Who:**
    - scope `session`: needs `scheduling.session.cancel` **and** a staff role (admin).
    - scope `participant`: staff, or a guardian whose linked student is that participant, or a student who is the participant themselves, and only if `payer_can_cancel`. Otherwise 403 `forbidden`.
    - Payers can never cancel scope `session`.
  - The session must be `scheduled` and `now < starts_at`. Otherwise 403 `cancel_window_passed`; staff may cancel a started session only via scope `session`.
  - **Credit** (pure `domain::cancellation::credit_bps(policy, actor_kind, scope, cause, hours_before) -> CreditDecision`):
    - Staff whole-session cancel, no cause → `staff_cancel_credit_bps`.
    - Staff with cause `instructor_absence` → by `instructor_absence`:
      - `MakeupCredit` → 10000.
      - `Reschedule` → the decision `TryReschedule`: the service tries the same local time +7 days in the same resource with the same instructors. On success it creates a `makeup` session for the same participants, and credit 0. On failure, credit 10000.
      - `SubstituteThenReschedule` → the response first returns `409 needs_decision {substitutes: [free instructors]}`, unless the body includes `substitute_instructor_id` (→ swap the instructor via the instructors endpoint logic, no cancel) or `skip_substitute: true` (→ behave as `Reschedule`).
    - Payer participant cancel: `h = floor((starts_at - now) / 1h)`. The result is the bps of the first tier (descending) with `min_hours_before <= h`; none → 0. Empty tiers → 0.
    - The value `10000` is `Bps::FULL` (a const on the type, not a literal in services).
  - Effects:
    - scope `session`: status `cancelled`, `cancelled_*` fields, every participant `cancelled` with `credit_bps`, `calendar::release(session)`.
    - scope `participant`: that participant `cancelled` + `credit_bps`, `calendar::release_subject`.
  - Audit `scheduling.session.cancelled` / `scheduling.participant.cancelled`, and emit `scheduling.session_cancelled.v1 {session_id, cancelled_by_role, reason, hours_before, participants: [{person_id, credit_bps}]}` (catalog in spec 03 §6).
- **Participants:** `POST /sessions/{id}/participants {person_id, kind?: makeup|trial}` claims a participant hold (409 `slot_taken` if the student is busy) and checks capacity. `DELETE /sessions/{id}/participants/{pid}` (staff) removes the hold, with no credit (an administrative correction).
- **Instructors:** `POST /sessions/{id}/instructors {person_id, role: substitute|assistant, replace?: uuid}` claims or rewrites the instructor holds.
- **Revert** (owner, `scheduling.override.revert`): `POST /sessions/{id}/revert`.
  - From `cancelled` (start still in the future) back to `scheduled`. Re-claims holds for the session and every participant who was cancelled **with** the session (`cancelled_at = session.cancelled_at`); 409 `slot_taken` if any can't be re-claimed.
  - Credits: sets `credit_bps = NULL` for those participants and emits `scheduling.session_reverted.v1 {session_id, participant_ids}` so enrollment can void credits (catalog in spec 03 §6).
- State machine (spec 06 §5.1): only `scheduled → scheduled|cancelled|completed`. Anything else → 409 `invalid_transition`.

## Scope
### In
- Endpoints, services, the `domain::cancellation` pure module, and the new event.

### Out
- Enrollment credit creation (M4 consumes the events).

## Acceptance criteria
- AC1. Tiers `[{24 → 10000}]`: a guardian cancels 30 h before → credit 10000; 3 h before → 0. Tiers `[]` → 0 in both. → `cancellation::tests::tiers` (pure) + `api_cancel::guardian_tiers`
- AC2. A guardian trying scope `session` → 403. A guardian cancelling another family's participant → 403. → `guardian_scope_limits`
- AC3. A staff whole-session cancel releases all holds, and the slot can be booked again. → `cancel_releases_holds`
- AC4. Reschedule to an occupied slot → 409 with alternatives that don't include conflicts with its own old holds. Reschedule to a free slot → version +1, and the old range is free. → `reschedule_flow`
- AC5. Reschedule without If-Match → 428. → `reschedule_requires_if_match`
- AC6. Instructor absence with `Reschedule`: next week is free → a makeup session exists, credit 0. Not free → credit 10000. → `instructor_absence_reschedule`
- AC7. Revert succeeds when free. After someone books the room, revert → 409. → `owner_revert`
- AC8. Cancelling a completed session → 409 `invalid_transition`. → `cancel_completed_rejected`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/src/domain/cancellation.rs` | `tiers`, `staff_full`, `instructor_absence_modes` |
| `crates/eq-scheduling/tests/api_cancel.rs` | AC2–AC8 |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(scheduling): reschedule, cancel and revert [T-208]`, signed off

## Report

## Questions
