---
id: T-405
title: Add enrollment changes (level, series, price plan) effective from a date
milestone: M4
status: draft
depends_on: [T-404]
spec_refs: [ENR-06, POL-05]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Moving a student to a new level, time slot or price plan is one dated action that keeps history and billing correct.

## Context
- ENR-06 `POST /enrollments/{id}/change {effective_on, level_id?, series_id?, price_plan_id?, reason}` in `spec/modules/enrollment.md`.
- Series switch = end membership on effective_on − 1 and add the new one from effective_on, through the `SeriesMembership` port, atomically.
- `enrollment_changes` rows (DDL `spec/06-domain-model.md` §4.12, created by T-401) keep history; billing reads the plan in force per period through `BillableSource` (T-412).

## Scope
### In
- Endpoint, service, history, audit

### Out
- Billing proration effects (handled by T-411/T-412)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/tests/api_changes.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T4, T7 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `series_switch_from_date` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Series switch from the 15th moves future participations only → `api_changes::series_switch_from_date`
- AC2. A conflict in the new series → 409 and no change → `series_switch_conflict`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/api_changes.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): dated enrollment changes [T-405]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
