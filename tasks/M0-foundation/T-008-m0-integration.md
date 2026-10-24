---
id: T-008
title: M0 integration — release build with embedded SPA passes a Playwright smoke test in CI
milestone: M0
status: todo
depends_on: [T-005, T-006, T-007]
spec_refs: [FE-01, OPS-06]
estimated_size: S
---

## Goal
Prove the stack end to end: the CI builds the web, embeds it into the release binary, starts it against Postgres, and a Playwright smoke test loads the app.

## Context
- FE-01: the SPA is embedded with `rust-embed` (T-005). Hashed assets are immutable, `index.html` is `no-cache`, and any non-API GET falls back to `index.html`.
- OPS-06: `/healthz` (liveness) and `/readyz` (DB reachable, migrations applied) exist since T-004.
- TST-07 (later milestones) reuses this Playwright setup. Keep the config generic: projects, `baseURL`, artifacts.

## Scope
### In
- `web/e2e/playwright.config.ts`:
  - Projects `chromium` and `webkit-mobile` (viewport 390×844).
  - `baseURL` from `E2E_BASE_URL` (default `http://localhost:8080`).
  - `webServer` disabled (CI starts the binary).
- `web/e2e/smoke.spec.ts`: visit `/`, expect the product name visible. Fetch `/healthz` → ok. Visit `/does/not/exist` → still the app (SPA fallback).
- CI `e2e` job:
  - Postgres service container.
  - Build web, then `cargo build --release -p eq-server`.
  - Start `ritma serve` in the background with test env.
  - Wait for `/readyz` (60 s timeout), then `pnpm -C web exec playwright test`.
  - Upload the report as an artifact on failure.
- `make e2e` does the same locally.

### Out
- Any feature tests.

## Acceptance criteria
- AC1. The CI `e2e` job is green on the PR (link it).

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/smoke.spec.ts` | `loads home`, `health ok`, `spa fallback` |

## Definition of done
- [ ] CI green including e2e
- [ ] Commit `test(e2e): smoke test against release build [T-008]`, signed off

## Report

## Questions
