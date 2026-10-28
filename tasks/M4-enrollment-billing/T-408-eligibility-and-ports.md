---
id: T-408
title: Add eligibility (exam readiness) and the BillableSource port for billing
milestone: M4
status: todo
depends_on: [T-407]
spec_refs: [ENR-09, ARC-08]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Schools that run exams see who is eligible by their own rule, and billing can ask enrollment what to bill without importing it.

## Context
- ENR-09 in `spec/modules/enrollment.md` §1; module AC6 (3 months tenure, 90% attendance, 1 no-show → eligible).
- Attendance figures come from an `AttendanceStats` port (default: no data → eligibility shows `unknown` rather than false).
- `BillableSource::billable_lines(tx, period)` (ARC-08 example) implemented by enrollment: active enrollments in the period with their price plan in force, sessions remaining for proration, payer, leave suspension.

## Scope
### In
- Eligibility calculator + endpoint, `AttendanceStats` port definition with default, `BillableSource` impl

### Out
- Billing usage (T-412)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/tests/eligibility.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Module AC6 → `eligibility::rule_example`
- AC2. `billable_lines` for a payer with 2 students returns 2 lines with the right plans → `billable_source_lines`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/eligibility.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): eligibility and billable source [T-408]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
