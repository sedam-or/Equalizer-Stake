---
id: T-008
title: M0 integration — release build with embedded SPA passes a Playwright smoke test in CI
milestone: M0
status: review
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
  - Wait for `/readyz` (60 s timeout), then `pnpm -C web exec playwright test --config=e2e/playwright.config.ts`.
  - Upload the report as an artifact on failure.
- `make e2e` does the same locally.

### Out
- Any feature tests.

## Files
| Path | Action |
|---|---|
| `web/e2e/playwright.config.ts`, `web/e2e/smoke.spec.ts` | create |
| `web/package.json`, `web/pnpm-lock.yaml` | add the pinned Playwright test runner |
| `web/biome.json` | exclude generated Playwright result and report directories from source lint |
| `.github/workflows/ci.yml` | add PostgreSQL-backed release E2E steps and failure artifact upload |
| `Makefile` | implement the matching local `make e2e` workflow |

## Acceptance criteria
- AC1. The CI `e2e` job is green on the PR (link it).

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/smoke.spec.ts` | `loads home`, `health ok`, `spa fallback` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] CI green including e2e (hosted Actions not run; the local equivalent passed)
- [x] Commit `test(e2e): smoke test against release build [T-008]`, signed off

## Report

**Plan** — (1) Add the exact `@playwright/test` dependency and browser project configuration (`chromium`, `webkit-mobile`) with `E2E_BASE_URL` → verify config/type checks and browser installation. (2) Add the three named smoke tests for the Ritma home heading, `/healthz`, and SPA fallback → run both browser projects against the release binary. (3) Exclude generated Playwright output from source lint while preserving reports for CI upload → rerun web lint after E2E. (4) Implement `make e2e` with local PostgreSQL startup, web/release builds, server lifecycle cleanup, and a bounded `/readyz` wait → run `make e2e` end to end. (5) Wire the CI service container, fake test environment, role setup, release startup, readiness wait, and failure-only Playwright artifact → validate YAML/action pins and compare its command sequence with the local target. (6) Run all task and workspace checks, report hosted Actions as not run, then commit and push this task.

**Assumptions** — Playwright Test is the mapped OSS dependency from spec 24; pin its current stable release exactly and install browser binaries separately in CI. The compose file's `eq` owner and `eq_app` test credentials are fake local-only values. T-004's `serve` command applies migrations through `RITMA_DATABASE_OWNER_URL`, so the E2E target starts that server directly rather than running a separate migration command. Hosted GitHub Actions cannot be invoked from this local run.

**Test plan** —
- Given the release server is ready, when Chromium and WebKit-mobile visit `/`, then the canonical Ritma heading is visible → `loads home`.
- Given the server is ready, when the browser request fixture fetches `/healthz`, then the response is successful → `health ok`.
- Given the SPA server is ready, when either project opens `/does/not/exist`, then the SPA shell returns 200 and the browser title is `Ritma` → `spa fallback`.
- Project configuration: Playwright runs `chromium` and a 390×844 WebKit mobile context, uses `E2E_BASE_URL` or the specified default, and does not start a separate web server.
- Error/edge cases: server exit or DB/migration startup failure must fail the readiness gate within the task's 60-second bound; Playwright failure must preserve the HTML report as a CI artifact.
- Abuse/security cases: all credentials are obvious fakes; no auth, PII or external provider is exercised; server process cleanup must run on test success and failure.

**Step 0 — Start** — Clean tree and current `scripts/tasks-index.sh --check` passed on `milestone/m0-foundation`; T-008 is the lowest eligible task and T-005/T-006/T-007 are all `review`. Set T-008 to `in_progress` and regenerated the index before implementation.

**Step 1 — Task/spec** — Read the complete T-008 task and FE-01/OPS-06/TST-07 excerpts. Read `spec/08-frontend.md` FE-01, `spec/12-testing.md` TST-07 and CI §3, `spec/16-code-quality.md` §12, `spec/17-clean-architecture.md` CA-01, `spec/18-security-by-design.md` §8/SBD-81, `spec/19-error-handling.md`, and `spec/24-open-source-stack.md` Playwright/CI entries.

**Step 2 — Dependencies** — Read the T-005, T-006 and T-007 Reports, including deviations and follow-ups. Confirmed shipped code: T-005 exports `BRAND.name` as `Ritma` and embeds `web/dist`; T-006 has a SHA-pinned CI e2e job shell and Makefile placeholder plus PostgreSQL compose service; T-007 gives the current dev DB/app-role configuration. T-004's CLI starts migrations from the owner URL before serving, and its readiness endpoint checks database connectivity and migration state.

**Step 3 — Design reference** — n/a; this task adds browser smoke coverage for the existing static home screen and changes no UI behavior or presentation.

**Step 4 — Plan and test plan** — Recorded above before implementation. Three mechanical details are self-resolved: the task omitted a Files section while Scope explicitly names CI/Makefile/Playwright work; the config is nested under `web/e2e` while the test command runs from `web`, so Playwright needs the explicit `--config` path; and Playwright's generated report/trace contains bundled HTML/JS that the source linter otherwise scans, so those output directories are excluded from lint inputs while retained as CI artifacts. These preserve the named config/tests/artifact and add only required paths/flags.

**Summary** — Added a PostgreSQL-backed CI E2E path and matching `make e2e` workflow. The release binary served the embedded SPA, `/healthz`, and an unknown route; Playwright passed `loads home`, `health ok`, and `spa fallback` in Chromium and 390×844 WebKit-mobile.

**Spec read** — `spec/08-frontend.md` FE-01; `spec/12-testing.md` TST-07 and CI §3; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` CA-01; `spec/18-security-by-design.md` §8/SBD-81; `spec/19-error-handling.md`; `spec/24-open-source-stack.md` Playwright and CI entries.

**Files** — Created the two `web/e2e` files; pinned `@playwright/test` in `web/package.json`/`web/pnpm-lock.yaml`; excluded generated `test-results` and `playwright-report` outputs from `web/biome.json`; implemented `make e2e`; added the Postgres service, fake test environment, `eq_app` setup and failure artifact upload in `.github/workflows/ci.yml`; updated this task and regenerated `tasks/INDEX.md`.

**Acceptance criteria → tests**

| AC | Exact test/check | Result |
|---|---|---|
| AC1 | CI `e2e` job on the PR | Not run: hosted GitHub Actions cannot be launched from this local run. The local equivalent `make e2e` passed all six browser/project cases; PR #17 remains draft pending hosted results. |
| smoke | `loads home` | Passed in Chromium and WebKit-mobile. |
| smoke | `health ok` | Passed in Chromium and WebKit-mobile. |
| smoke | `spa fallback` | Passed in Chromium and WebKit-mobile; unknown path returned the SPA shell with HTTP 200 and the Ritma title. |

**Checks run** —
- ✅ `make e2e` — built web assets and the release server, started the local Postgres-backed server, waited for `/readyz`, and passed all six Playwright cases. The local default uses the pinned Playwright image because this host lacks WebKit's Ubuntu libraries; CI uses its installed browser binaries (`E2E_PLAYWRIGHT_CONTAINER=0`).
- ✅ `pnpm -C web install --frozen-lockfile`.
- ✅ `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` (including database and testkit tests).
- ✅ `pnpm -C web lint`; `pnpm -C web typecheck`; `pnpm -C web test`; `pnpm -C web build`; `pnpm -C web size` (98.77 KB gzip); `pnpm -C web audit --prod`.
- ✅ `scripts/check-deps.sh`; `scripts/check-literals.sh`; `scripts/check-web-boundaries.sh`; `scripts/check-layers.sh`; `scripts/tasks-index.sh --check`; `python3 scripts/erd.py --check`; `git diff --check`.
- ✅ `cargo audit`; `cargo deny check advisories`; `cargo machete`; `gitleaks detect --redact --no-banner --config=.gitleaks.toml`; Docker actionlint 1.7.12; `/home/kepolu/.cargo/bin/zizmor .`.
- Not run: hosted GitHub Actions / PR `e2e` status; this unattended local run cannot launch the hosted CI result. The PR body records this and stays draft.
- Not run: `pnpm -C web run i18n:check`; the script is not present yet and is owned by T-306.
- Not run: `cargo sqlx prepare --workspace` and `python3 scripts/erd.py --migrations`; no query or migration changed.

**Manual checks** — `make e2e` started the release binary with the compose Postgres database, and its `/readyz` wait succeeded before browser tests. Playwright issued real requests for `/`, `/healthz`, embedded assets, and `/does/not/exist`; the unknown path returned 200 with the SPA document title. After completion the server process exited and port 8080 was released. Local Docker mode runs the browser container as the current user so generated artifacts remain writable.

**Debug log** — The nested config is not auto-discovered from `web`, so both invocations use `--config=e2e/playwright.config.ts`. This package is CommonJS for Playwright config loading, so the config uses `__dirname` and `module.exports`; importing `import.meta` failed in the first attempt. Playwright output initially entered Biome's recursive source list, so only its generated output paths are excluded. The SPA fallback correctly delivered `index.html` but TanStack Router rendered its `Not Found` route for the unknown URL, so the test checks the Ritma document title and HTTP 200 rather than expecting the home heading. Native browser launch on Athena OS lacks WebKit `libicu74`/`libflite1`; the local target defaults to the official pinned browser image, which passed. The first container run wrote root-owned artifacts; running it as the current UID/GID fixed repeatability. `runner.temp` was rejected in a job-level Actions environment by actionlint, so CI uses `/tmp/ritma-e2e`.

**Performance** — `pnpm -C web size` measured 98.77 KB gzip against the 200 KB limit. The browser smoke suite completed in about three seconds after server readiness; no application query or hot path changed.

**Security** — No feature endpoint, auth flow, PII or third-party service was added. CI uses the fake database credentials and test key only; migrations use the owner URL while the server's request pool connects as `eq_app` (`NOSUPERUSER NOBYPASSRLS`). Browser tests only call the local server. The `make e2e` cleanup terminates the server on success, failure, interrupt or termination; gitleaks passed.

**Deviations** — RATIFY the three self-resolutions recorded under Questions: add the task's omitted Files list for directly required paths; pass an explicit config path for the nested Playwright config; and exclude generated reports/traces from source lint while retaining CI artifacts. Local browser execution defaults to the pinned official Playwright container on unsupported developer hosts; CI stays on the runner's installed browser dependencies. Decision register references: TD-018, TD-019, TD-020 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).

**Dependencies affected** — Uses T-005's canonical `BRAND.name` and SPA fallback, T-004's `ritma serve`/`/readyz`, and T-006's Postgres compose service and CI workflow. No API or schema contract changed.

**Rollback plan** — Revert the T-008 commit to remove the Playwright dependency, tests, Makefile target and CI job setup. No production schema or tenant data changed; only ignored local E2E outputs and the existing dev database were used.

**Follow-ups** — Human reviewer: confirm the hosted `e2e` status on PR #17; the local suite is green but AC1's hosted result remains unverified. T-306 owns the web `i18n:check` script.

**Step 5 — Implement** — Added the two browser projects, three named smoke tests, local build/start/readiness/cleanup flow, CI service role, and failure-only HTML report upload. Playwright's local container runs as the invoking UID/GID.

**Step 6 — Run locally** — Ran `make e2e` against the local PostgreSQL 16 compose service and release binary. The browser tests exercised home, health and SPA fallback on both projects; no hosted service or real credential was used.

**Step 7 — Debug loop** — Fixed config loading, generated-output linting, the SPA fallback assertion, unsupported host browser libraries, container artifact ownership, and the CI env-context lint finding. Final `make e2e` passed all six cases; failures and fixes are recorded above.

**Step 8 — Self code review** — Reviewed changes against FE-01, TST-07, CQ-12 and CA-01. Confirmed no feature behavior or database schema changed, webServer remains disabled, the app name comes from `BRAND`, actions remain SHA-pinned, artifacts upload only on failure, and no lint/test/security gate was weakened.

**Step 9 — Optimize** — Measured the existing bundle with `pnpm -C web size` at 98.77 KB gzip; added no runtime code or speculative optimization.

**Step 10 — Security audit** — Confirmed CI/test credentials are fake, request traffic stays local, the server app connection uses `eq_app`, cleanup covers the child process, and gitleaks/zizmor/actionlint passed. No SBD-81 feature-trigger controls were introduced.

**Step 11 — Revise and verify** — Reran the final E2E suite and completed workspace Rust checks, web checks, guard scripts, dependency/advisory audits, secret scan, actionlint and zizmor. Hosted Actions and the absent i18n checker are explicitly Not run above.

**Step 12 — Report, commit** — Report and status are ready for the specified signed-off task commit; the regenerated index is current. The commit and push are the final task operations and are recorded in `RUN_LOG.md`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** T-008 Scope requires Playwright files, Makefile and CI workflow changes but the task has no Files section; adding Playwright also requires its web manifest and lockfile → add only those directly required paths because they are explicitly needed to satisfy Scope and the mapped Playwright tool. [TD-018]
- **Self-resolved (implementer, run 6, needs architect ratification):** The Playwright config is required at `web/e2e/playwright.config.ts`, but a bare `pnpm -C web exec playwright test` searches from `web` and will not load that nested config → pass `--config=e2e/playwright.config.ts` in CI and `make e2e`, keeping the required config location and tests while making execution deterministic. [TD-019]
- **Self-resolved (implementer, run 6, needs architect ratification):** Playwright writes generated JS/HTML reports and traces under `web/`, which Biome's recursive `includes` otherwise treats as source after E2E → exclude only `test-results` and `playwright-report` from Biome inputs, preserving the task-required failure artifact and existing lint rules. [TD-020]
