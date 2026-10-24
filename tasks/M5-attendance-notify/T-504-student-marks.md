---
id: T-504
title: Add student attendance marks with lock window, no-show events and completion checks
milestone: M5
status: draft
depends_on: [T-503]
spec_refs: [ATT-10, ATT-11, ATT-12]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Instructors (or admins, per policy) mark who came; no-shows trigger the school's rule automatically and marks lock after the window.

## Context
- ATT-10/11/12 in `spec/modules/attendance.md`. Module AC7: marking 49 h after end with `lock_after_hours=48` → 403 `marks_locked`.
- Transition to `absent` emits `attendance.no_show_recorded.v1`; changing away emits `no_show_reverted.v1` (enrollment applies `student_no_show`, T-407).
- Optimistic UI is allowed for marks (FE-33), so the endpoint is idempotent per (session, person).

## Scope
### In
- `PUT /sessions/{id}/marks` (batch), reads, events, completion attention item when check-in missing

### Out
- UI (T-512)

## Acceptance criteria
- AC1. AC7 lock window → `api_marks::locked_after_window`
- AC2. absent → event → credit/forfeit per policy end to end → `no_show_flows_to_enrollment`
- AC3. Idempotent batch → `marks_idempotent`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-attendance/tests/api_marks.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): student marks [T-504]`, signed off

## Report

## Questions
