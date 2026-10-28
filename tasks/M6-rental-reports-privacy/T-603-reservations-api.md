---
id: T-603
title: Add reservations: create with holds at creation, approval, booker cancellation with refund terms, and billing events
milestone: M6
status: todo
depends_on: [T-415, T-602]
spec_refs: [RNT-04, RNT-05, RNT-08, DOM-10, API-08]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Members book a room themselves; the room is held immediately, approvals and payments follow the school's rule, and cancellation refunds are computed from the terms they agreed to.

## Context
- RNT-04/05/08 in `spec/modules/rental.md`; holds via `calendar::reservation_holds` at creation (pending requests block the slot). Module AC1 shared ledger, AC4 cancellation window, AC5 frozen booker → 422 `rule=billing.overdue.action` (payer state read through a `PayerState` port implemented by billing, default normal).
- Snapshots `cancel_terms` on the reservation (POL-05). Events `rental.reservation_requested|confirmed|cancelled.v1` → billing one-off invoice / credit note (T-415 consumers).

## Scope
### In
- Endpoints, services, `PayerState` port + billing impl, events

### Out
- Jobs (T-604)
- Public booking (T-605)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-rental/tests/api_reservations.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T7 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `cancellation_window` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. AC1 shared ledger with lessons → `api_reservations::shared_ledger`
- AC2. AC4 window refund → `cancellation_window`
- AC3. AC5 frozen booker → `frozen_booker`
- AC4. Concurrency: 20 concurrent reservations of one slot → 1 → `reservation_race`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-rental/tests/api_reservations.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): reservations [T-603]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
