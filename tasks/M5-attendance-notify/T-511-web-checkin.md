---
id: T-511
title: Add the instructor check-in page: today's sessions, camera with on-device face detection, GPS, and the offline queue
milestone: M5
status: todo
depends_on: [T-306, T-315, T-510]
spec_refs: [ATT-06, ATT-07, FE-35, SEC-20]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
An instructor checks in with one tap and a selfie, even with bad signal, and never wonders whether it worked.

## Context
- ATT-06 client face detection with `@mediapipe/tasks-vision` (lazy-loaded on `/checkin` only), computing the face/frame ratio sent as evidence; ATT-07 offline queue in IndexedDB with retry and visible status.
- Camera and geolocation require a secure context (DST-22); show a clear explanation when not available.
- Permissions-Policy allows camera and geolocation for self (SEC-20). **Security (SBD-58):** the offline check-in queue registers a `registerLogoutCleanup` hook (T-306) that wipes its IndexedDB store and queued photos on logout.

## Scope
### In
- `/checkin` route, camera component, GPS accuracy display, queue, locale strings

### Out
- Review UI (T-512)

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/e2e/checkin.spec.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3, T5 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- Abuse-case acceptance criterion: `size-limit` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. E2E with fake media stream + mocked geolocation: check-in verified → `web/e2e/checkin.spec.ts › verified`
- AC2. Offline: queued, then sent when online → `offline queue`
- AC3. Bundle: MediaPipe not in the initial chunk → `size-limit`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/checkin.spec.ts` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): instructor check-in [T-511]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
