---
id: T-406
title: Add leave (cuti): request, limits per year, notice, fee, over-limit handling and billing suspension
milestone: M4
status: draft
depends_on: [T-405]
spec_refs: [ENR-07, POL-05]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Families can pause lessons within the school's rules, and the schedule and the bill follow automatically.

## Context
- ENR-07 in `spec/modules/enrollment.md` §1, using the **snapshotted** `leave_terms` on the enrollment (POL-05), not the current policy.
- Module AC4: max 30 days/year with 20 used, request 15 with Block → 422 `remaining=10`; with `end_and_require_reregistration` → the enrollment ends at the limit.
- Emits `enrollment.enrollment_on_leave.v1` (scheduling cancels participations, T-212) and `enrollment.leave_fee_charged.v1` when `leave.fee > 0` (billing pending charge, T-412).

## Scope
### In
- Endpoints (request, approve if needed, cancel leave), the pure `domain::leave` calculator (days used per rolling or calendar year as the policy says), events

### Out
- UI (T-417)

## Acceptance criteria
- AC1. Module AC4 leave limit → `api_leave::limit_remaining`
- AC2. A leave in range cancels those participations (consumer from T-212) with credit 0 → `leave_cancels_sessions`
- AC3. A fee > 0 emits `leave_fee_charged` → `leave_fee_event`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/api_leave.rs` | AC names |
| `crates/eq-enrollment/src/domain/leave.rs` | `proptest days_used_never_negative` |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): leave [T-406]`, signed off

## Report

## Questions
