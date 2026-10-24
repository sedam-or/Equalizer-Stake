---
id: T-603
title: Add reservations: create with holds at creation, approval, booker cancellation with refund terms, and billing events
milestone: M6
status: draft
depends_on: [T-602]
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
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): reservations [T-603]`, signed off

## Report

## Questions
