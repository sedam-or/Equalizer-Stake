---
id: T-408
title: Add eligibility (exam readiness) and the BillableSource port for billing
milestone: M4
status: draft
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

## Questions
