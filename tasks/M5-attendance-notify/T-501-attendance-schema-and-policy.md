---
id: T-501
title: Add the attendance schema, AttendancePolicy and the AttendanceModule wiring
milestone: M5
status: draft
depends_on: [T-419]
spec_refs: [ATT-01, POL-01, MOD-03, SEC-07]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Attendance becomes a module with typed rules for instructor check-in (GPS radius, photo, face ratio, windows) and student marks.

## Context
- DDL `spec/06-domain-model.md` §4.6 (checkins, attendance_marks) verbatim into `migrations/0040_attendance.sql`; `checkins` review columns writable only via function (SEC-07).
- Policy `AttendancePolicy` exactly as `spec/04-policy-engine.md` §6.6 (instructor_checkin {windows, gps {enabled, radius_m, jitter_allowance_m}, photo, face {enabled, min_face_ratio_bps}, on_failed_check}, review {sla_minutes, remind_after_minutes, escalate_to_owner}, student_marks {...}).
- Permissions and nav from `spec/modules/attendance.md`; requires_all scheduling.
- Implements the `AttendanceStats` port (T-408) and the checklist item `location_pin` (GPS on, no geo).

## Scope
### In
- Migration, policy, module wiring, port, checklist, policy i18n

### Out
- Check-in logic (T-502+)

## Acceptance criteria
- AC1. Every preset's attendance patch validates → `presets::all_presets_valid`
- AC2. Summary snapshot (music preset) → `attendance::policy::tests::summary_snapshot`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-attendance/src/policy.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): schema and policy [T-501]`, signed off

## Report

## Questions
