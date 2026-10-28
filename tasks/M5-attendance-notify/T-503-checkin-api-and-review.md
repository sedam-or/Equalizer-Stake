---
id: T-503
title: Add the check-in API (photo upload, GPS, client face ratio), duplicate protection, and admin review
milestone: M5
status: draft
depends_on: [T-303, T-502]
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-attendance/tests/api_checkin.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T3, T5 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- Abuse-case acceptance criterion: `sbd_503_abuse` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
