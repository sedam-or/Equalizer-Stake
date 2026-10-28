---
id: T-501
title: Add the attendance schema, AttendancePolicy and the AttendanceModule wiring
milestone: M5
status: todo
depends_on: [T-408, T-419]
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-attendance/src/policy.rs` | create or modify | Path named in this task's contract. |
| `migrations/0040_attendance.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `all_presets_valid` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Every preset's attendance patch validates → `presets::all_presets_valid`
- AC2. Summary snapshot (music preset) → `attendance::policy::tests::summary_snapshot`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-attendance/src/policy.rs` | AC names |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): schema and policy [T-501]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
