---
id: T-006
title: Add the CI workflow, Makefile, dev database compose and guard scripts
milestone: M0
status: todo
depends_on: [T-005]
spec_refs: [TST-14, ARC-02, SEC-22, TST-01, CQ-64, CQ-80, CQ-100, CQ-101, CA-01, SBD-51, SBD-74, SBD-75, OSS-20, OSS-21, OSS-22]
estimated_size: M
---

## Goal
Every PR runs the full check suite on GitHub Actions, and contributors get `make` targets for the common workflows (CONTRIBUTING.md references them).

## Context
- CI jobs (12-testing §3):
  1. `rust`: fmt → clippy -D warnings → `cargo deny check` → `cargo test --workspace` (needs Docker for testcontainers; ubuntu-latest has it)
  2. `web`: pnpm install --frozen-lockfile → lint → typecheck → test → build → size
  3. `contract` (placeholder until T-114: `echo skip`)
  4. `e2e` (placeholder until T-008)
  5. `scripts`: `check-deps.sh`, `check-literals.sh`, `check-web-boundaries.sh`, `check-layers.sh`
  6. `pr-title` (pull_request only): the title must match `COMMIT_CONVENTION.md` §9: `^(feat|fix|perf|refactor|test|docs|build|ci|chore|revert)(\([a-z0-9-]+\))?!?: [A-Za-z0-9].{0,70}$`, and be at most 72 characters (CQ-100/101). A small inline script is fine; no third-party action needed.
- Optional local hook `.githooks/commit-msg` that applies the same regex and length check to the commit header (skipping `fixup!`/`squash!` and merge commits). Document `git config core.hooksPath .githooks` in CONTRIBUTING.md.
- CQ-64 `scripts/check-web-boundaries.sh`: for each `web/src/modules/<a>/`, fail if any file imports from `@/modules/<b>/` or `../<b>/` with `b ≠ a` (`registry.ts` at `web/src/modules/` is exempt). Print file:line.
  7. `security`: `gitleaks detect --redact` over the PR diff (SBD-51/74), with a `.gitleaks.toml` that allow-lists only `.env.example` placeholders and test fixtures named `test-secret-*`.
- `.github/workflows/security.yml` (scheduled daily + manual): `cargo audit` and `pnpm -C web audit --prod` (SEC-22, SBD-75). Failures open or update an issue labelled `security`, or at minimum fail visibly.
- CA-01 `scripts/check-layers.sh`: fail with file:line if (a) any file under `crates/*/src/domain/` mentions `sqlx`, `axum`, `reqwest`, `tokio`, `tracing`, `std::env`, `std::fs`, or the identifiers `Clock` / `Tx` (word match, ignoring comments and `#[cfg(test)]` modules is optional; keep it a simple grep with an allow-list file `scripts/check-layers.allow` if needed); (b) any `crates/*/src/http.rs` or `http/**` mentions `sqlx::`; (c) any `repo.rs` or `repo/**` mentions `axum`.
- TST-01 coverage: the `rust` job runs `cargo llvm-cov --workspace` and fails if line coverage of files under `crates/*/src/domain/` is < 90%. While no `domain/` files exist, the gate passes.
- TST-14 `scripts/check-literals.sh`:
  - Scans `crates/*/src/**/{domain,service}*.rs` and `crates/*/src/domain/**/*.rs`.
  - Fails on IANA timezone names (`Asia/`, `America/`, `Europe/`), `WIB`, 3-letter currency codes in string literals (`"IDR"`, `"USD"`), integer literals ≥ 1000 that aren't inside a `const` item whose preceding line has `// technical:`, `.day() == <n>`, and float literals with ≥ 3 decimals (coordinates).
  - Prints file:line and the reason.
  - Allow-list file: `scripts/check-literals.allow`, one `path:line` or `path:*` per line, each with a justification comment.
- SEC-22: `deny.toml` license allow-list MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, MPL-2.0, Unicode-3.0, CC0-1.0, and AGPL-3.0-only for workspace crates. Advisories deny, unmaintained warn, yanked deny.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - The GitHub workflows already exist (`ci.yml`, `codeql.yml`, `security.yml`, `scorecard.yml`, `release-please.yml`, `release.yml`, `.github/dependabot.yml`), with every action pinned by SHA and jobs that activate when their inputs exist (OSS-20/21). This task **modifies** them instead of creating them: keep the job names, keep `ci-ok` as the single required check, and keep actions SHA-pinned.
  - Add `scripts/check-coverage.sh <lcov>` (the `rust` job calls it) implementing the TST-01 90% gate for `crates/*/src/domain/**`.
  - Add the `cargo` (`/`) and `npm` (`/web`) ecosystems to `.github/dependabot.yml` (weekly, grouped, `cooldown.default-days: 7`, prefix `build`, scope `deps`). Use Dependabot rather than Renovate (OSS-04).
  - The `docs` job already runs `scripts/tasks-index.sh --check`, `typos` (`_typos.toml`) and `lychee`; the `secrets` job runs gitleaks and will pick up `.gitleaks.toml` when this task adds it.

## Scope
### In
- `.github/workflows/ci.yml` with the jobs above, caching (`Swatinem/rust-cache`, pnpm store cache) and `concurrency` cancel-in-progress per ref.
- `Makefile` targets:
  - `dev-db` (`docker compose -f deploy/dev/compose.yaml up -d`, Postgres 16 on 5433, user/pass `eq`/`eq`, db `ritma`)
  - `dev` (runs `cargo watch -x 'run -p eq-server'` and `pnpm -C web dev` concurrently; fall back to instructions if cargo-watch is missing)
  - `test`, `lint`, `api` (placeholder until T-114), `e2e` (placeholder)
  - `build` (web then release binary), `clean`
- `deploy/dev/compose.yaml`, `.env.example` (all OPS-01 vars with comments, dev values).
  - **Dev database roles (isolation must hold in dev):** the compose user `eq` is a superuser and ignores RLS, so the server must not run as `eq`. Mount `deploy/dev/initdb.sql` into `/docker-entrypoint-initdb.d/` containing `CREATE ROLE eq_app LOGIN PASSWORD 'eq_app' NOSUPERUSER NOBYPASSRLS;` (migration 0001 skips creating a role that exists). `.env.example` sets `DATABASE_URL=postgres://eq_app:eq_app@localhost:5433/ritma` and `RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma`, with a comment that the owner URL is only for migrations. `make dev` loads `.env`.
- `deny.toml`, `scripts/check-literals.sh` (+ `.allow`), `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`, `.github/dependabot.yml` cargo + npm ecosystems (weekly, grouped), `scripts/check-coverage.sh`.

### Out
- Release workflow (T-706). E2E content (T-008).

## Files
| Path | Action |
|---|---|
| `.github/workflows/ci.yml` (modify), `Makefile`, `deploy/dev/compose.yaml`, `deploy/dev/initdb.sql`, `.env.example`, `deny.toml`, `scripts/check-literals.sh`, `scripts/check-literals.allow`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`, `scripts/tests/*.sh`, `.github/dependabot.yml` (modify), `.github/workflows/security.yml` (modify), `.gitleaks.toml`, `scripts/check-coverage.sh`, `.githooks/commit-msg` | create |

## Business rules
None.

## Implementation notes
- `check-literals.sh` can be a small Rust bin in `crates/eq-testkit/src/bin/check_literals.rs` if regexes in sh get unwieldy, called from the script. Using `syn` to find `const` items is acceptable.
- Make the scripts executable (`chmod +x`), with a shebang `#!/usr/bin/env bash` and `set -euo pipefail`.

## Acceptance criteria
- AC1. Pushing the branch runs all CI jobs green (link the run in the Report).
- AC2. Adding `let due = 10_000_000;` in a file `crates/eq-billing/src/domain/x.rs` makes `check-literals.sh` fail with path:line. Removing it passes. (Describe in the Report; add a shell test `scripts/tests/check_literals_test.sh` that creates a temp file in a temp crate copy and asserts the exit code.)
- AC3. `make dev-db` then `make test` works locally.

## Tests to write
| File | Test names |
|---|---|
| `scripts/tests/check_literals_test.sh` | detects money literal; ignores technical const; detects timezone string |
| `scripts/tests/check_layers_test.sh` | detects sqlx in domain; detects axum in repo; passes clean crate |
| `scripts/tests/check_web_boundaries_test.sh` | detects cross-module import; allows registry; allows same-module import |

## Definition of done
- [ ] CI green on the PR
- [ ] Commit `ci: workflow, makefile, dev db and guard scripts [T-006]`, signed off

## Report

## Questions
