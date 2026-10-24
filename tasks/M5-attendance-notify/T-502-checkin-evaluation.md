---
id: T-502
title: Add the pure check-in evaluation (window, GPS with accuracy and jitter, photo checks, face ratio) and outcome
milestone: M5
status: draft
depends_on: [T-501]
spec_refs: [ATT-02, ATT-03, TST-01]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Whether an instructor's check-in is accepted is decided by one pure, fully tested function that reads only policy and evidence.

## Context
- ATT-02/ATT-03 in `spec/modules/attendance.md` §1: haversine distance; pass if `distance - accuracy <= radius + jitter`; `not_configured` when the location has no geo (skip rule, SET-13); photo dimension/size checks; client face ratio verified against policy; outcome verified | pending_review per `on_failed_check`.
- Module ACs 1–3 and 5. **Use neutral test coordinates** (e.g. a location at (-6.2000, 106.8000)); never institution coordinates.

## Scope
### In
- `eq_attendance::domain::checkin::evaluate` with unit and property tests

### Out
- API (T-503)

## Acceptance criteria
- AC1. All pass at 120 m with accuracy 10 (module AC1 with neutral coords) → `checkin::tests::all_pass`
- AC2. 200 m → gps fail → pending_review (AC2) → `gps_fail`
- AC3. No geo → not_configured (AC3) → `not_configured`
- AC4. 320×240 photo → invalid (AC5) → `photo_too_small`
- AC5. Property: distance monotonic → verdict monotonic → `proptest gps_monotonic`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-attendance/src/domain/checkin.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): check-in evaluation [T-502]`, signed off

## Report

## Questions
