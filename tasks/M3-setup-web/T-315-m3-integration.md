---
id: T-315
title: M3 integration — the time-to-first-session E2E, the accessibility sweep, bundle budget and the hero-GIF recording script
milestone: M3
status: todo
depends_on: [T-314]
spec_refs: [TST-07, TST-11, TST-12, FE-02, PRD-07, PRD-14, JRN-01]
estimated_size: S
---

## Goal
Prove the core promise end to end on a fresh install: from the first page load to the first scheduled lesson in under 10 minutes of human time, accessibly, on desktop and phone. Produce the asset the launch depends on.

## Context
- TST-07 scenario `first-session.spec.ts`, against `ritma serve` built with the embedded web, Postgres service, fresh DB, Chromium desktop 1280×800 **and** WebKit mobile 390×844:
  1. Open `/setup?token=` (the token is read from `RITMA_DATA_DIR/setup-token.txt` by the test).
  2. Step 1: music school, Indonesia, 3 rooms, owner.
  3. Step 2: unchanged.
  4. Step 3: unchanged → Finish.
  5. Dashboard → the checklist → "Add instructors": create an instructor (invite link).
  6. Add a student.
  7. `/schedule` → click a free cell Monday 15:00 → create a 45-min session for that student and instructor.
  8. The session block is visible.
  - Assert: ≤ 10 user actions per wizard step; the total test time < 60 s (CI).
  - Log `setup_seconds` and `first_session_seconds` into the job summary (PRD-07 tracking).
- TST-11 `a11y.spec.ts`: visit every route in the route map that exists in M3 (setup steps, auth pages, dashboard per role, schedule, series, catalog, resources, closures, people, person detail, settings pages, `/me` tabs) as the right role, run `@axe-core/playwright`, and require 0 serious or critical violations at 390 and 1280 px.
- TST-12: visual snapshots of `/__ds` (from T-305) are part of CI. Update the baselines in this task if the components changed since.
- FE-02: `size-limit` with initial JS ≤ 200 KB gzip, enforced in the `web` CI job. Record the actual size in the Report.
- **Hero GIF recording** (brand asset, BRAND §8): `web/e2e/record-hero.spec.ts` (`test.skip` unless `RECORD=1`):
  - It runs the 3-step setup with human-like pacing (`slowMo` 250 ms, deliberate typing) at 1280×720, with `video: 'on'`.
  - Then `scripts/make-hero-gif.sh` converts the webm to a ≤ 8 MB GIF and MP4/WebM with ffmpeg (palettegen/paletteuse, 12 fps, 45–60 s).
  - The output goes to `brand/assets/hero-setup.{gif,mp4,webm}`. **Do not commit the outputs in this task**; document the command in the Report.
  - **Environment-limited:** if `ffmpeg` or the Playwright browsers cannot be installed, ship the spec and the script, run the three E2E specs, and list the recording under "Not run: <reason>"; this does not block the task.
- The scenario follows the owner journey JRN-01 (spec 28): install → setup in 3 steps → first lesson booked; its metric is PRD-07 (< 10 min to the first session).

## Scope
### In
- The three E2E specs, the CI wiring in the `e2e` job (both browsers), the recording spec and script.

### Out
- Marketing copy.

## Files
| Path | Action | Purpose |
|---|---|---|
| `scripts/make-hero-gif.sh` | create or modify | Path named in this task's contract. |
| `web/e2e/a11y.spec.ts` | test | Task-listed verification file. |
| `web/e2e/first-session.spec.ts` | test | Task-listed verification file. |
| `web/e2e/record-hero.spec.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T3, T4 (SBD-30).
- Data classes touched: S1/S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `every route has no serious violations` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. `first-session.spec.ts` passes on both projects in CI, 3 consecutive runs.
- AC2. `a11y.spec.ts` passes.
- AC3. The size budget passes.
- AC4. `RECORD=1 pnpm -C web exec playwright test record-hero` + the script produce a GIF ≤ 8 MB locally (stated in the Report with the file size).

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/first-session.spec.ts` | `fresh install to first session` |
| `web/e2e/a11y.spec.ts` | `every route has no serious violations` |
| `web/e2e/record-hero.spec.ts` | `record hero` (skipped by default) |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All CI jobs green
- [ ] Commit `test(web): M3 end-to-end, accessibility and hero recording [T-315]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
