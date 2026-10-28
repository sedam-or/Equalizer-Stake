---
id: T-406
title: Add leave (cuti): request, limits per year, notice, fee, over-limit handling and billing suspension
milestone: M4
status: draft
depends_on: [T-212, T-405]
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/src/domain/leave.rs` | create or modify | Path named in this task's contract. |
| `crates/eq-enrollment/tests/api_leave.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `limit_remaining` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): leave [T-406]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
