---
id: T-503
title: Add the check-in API (photo upload, GPS, client face ratio), duplicate protection, and admin review
milestone: M5
status: draft
depends_on: [T-502]
spec_refs: [ATT-01, ATT-03, ATT-04, SEC-13]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Instructors check in from their phone in seconds; anything doubtful goes to an admin queue instead of being silently accepted or rejected.

## Context
- ATT-01 eligibility (instructor of the session, window), ATT-04 immutability (no update/delete; review fields once).
- Photo through `FileService` purpose `checkin_photo`, max bytes from policy (T-303 supports caller-provided limit).
- Module AC4: second check-in → 409 `already_checked_in`. Events `attendance.checkin_recorded.v1`, `attendance.checkin_reviewed.v1`.

## Scope
### In
- `POST /sessions/{id}/checkin` (multipart), `GET /attendance/review`, `POST /checkins/{id}/review`

### Out
- Web (T-511/T-512)

## Acceptance criteria
- AC1. Duplicate → 409 (AC4) → `api_checkin::duplicate`
- AC2. Pending review appears in queue; approve once; second review → 409 → `review_once`
- AC3. Outside window → 403 with the window in the problem → `outside_window`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-attendance/tests/api_checkin.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): check-in API and review [T-503]`, signed off

## Report

## Questions
