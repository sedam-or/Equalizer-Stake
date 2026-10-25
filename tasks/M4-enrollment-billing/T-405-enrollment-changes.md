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

## Questions
