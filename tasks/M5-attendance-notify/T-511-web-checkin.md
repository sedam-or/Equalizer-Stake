---
id: T-511
title: Add the instructor check-in page: today's sessions, camera with on-device face detection, GPS, and the offline queue
milestone: M5
status: draft
depends_on: [T-510, T-315]
spec_refs: [ATT-06, ATT-07, FE-35, SEC-20]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
An instructor checks in with one tap and a selfie, even with bad signal, and never wonders whether it worked.

## Context
- ATT-06 client face detection with `@mediapipe/tasks-vision` (lazy-loaded on `/checkin` only), computing the face/frame ratio sent as evidence; ATT-07 offline queue in IndexedDB with retry and visible status.
- Camera and geolocation require a secure context (DST-22); show a clear explanation when not available.
- Permissions-Policy allows camera and geolocation for self (SEC-20).

## Scope
### In
- `/checkin` route, camera component, GPS accuracy display, queue, locale strings

### Out
- Review UI (T-512)

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

## Questions
