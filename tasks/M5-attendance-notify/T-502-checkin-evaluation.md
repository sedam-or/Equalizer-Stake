---
id: T-502
title: Add the pure check-in evaluation (window, GPS with accuracy and jitter, photo checks, face ratio) and outcome
milestone: M5
status: todo
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-attendance/src/domain/checkin.rs` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `all_pass` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): check-in evaluation [T-502]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
