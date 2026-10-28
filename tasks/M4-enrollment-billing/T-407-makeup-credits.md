---
id: T-407
title: Add make-up credits from cancellations, skipped occurrences and no-shows, with expiry and booking
milestone: M4
status: todo
depends_on: [T-208, T-406]
spec_refs: [ENR-08, SCH-09, SCH-10, SCH-11]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
When a lesson is lost for a reason the school compensates, the family gets a credit they can see and book, exactly once.

## Context
- ENR-08 in `spec/modules/enrollment.md` §1 (`makeup_credits` policy: enabled, expiry, booking rules).
- Consumes `scheduling.session_cancelled.v1` (per-participant `credit_bps`), `scheduling.occurrence_skipped.v1`, `attendance.no_show_recorded.v1` / `no_show_reverted.v1` (apply `student_no_show`), and `scheduling.session_reverted.v1` (void credits).
- Module AC5: a credit of 10000 → 1 available credit expiring per policy; duplicate event delivery → still 1 (receipts + a unique key per source event and person).
- Booking a credit = adding the student as a `makeup` participant to a session with room (T-208 participants endpoint) and marking the credit used, in one tx.

## Scope
### In
- Event handlers, credits API (list for staff and guardian, book, expire job), events `makeup_credit_granted`

### Out
- UI (T-417)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/tests/credits.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `expiry_job` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Module AC5 → `credits::from_cancellation_once`
- AC2. Revert voids the credit → `revert_voids`
- AC3. Expiry job expires unused credits (time travel) → `expiry_job`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/credits.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): make-up credits [T-407]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
