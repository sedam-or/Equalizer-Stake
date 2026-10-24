---
id: T-006
title: Add the CI workflow, Makefile, dev database compose and guard scripts
milestone: M0
status: todo
depends_on: [T-005]
spec_refs: [TST-14, ARC-02, SEC-22]
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
  5. `scripts`: `check-deps.sh`, `check-literals.sh`
- TST-14 `scripts/check-literals.sh`:
  - Scans `crates/*/src/**/{domain,service}*.rs` and `crates/*/src/domain/**/*.rs`.
  - Fails on IANA timezone names (`Asia/`, `America/`, `Europe/`), `WIB`, 3-letter currency codes in string literals (`"IDR"`, `"USD"`), integer literals ≥ 1000 that aren't inside a `const` item whose preceding line has `// technical:`, `.day() == <n>`, and float literals with ≥ 3 decimals (coordinates).
  - Prints file:line and the reason.
  - Allow-list file: `scripts/check-literals.allow`, one `path:line` or `path:*` per line, each with a justification comment.
- SEC-22: `deny.toml` license allow-list MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, MPL-2.0, Unicode-3.0, CC0-1.0, and AGPL-3.0-only for workspace crates. Advisories deny, unmaintained warn, yanked deny.

## Scope
### In
- `.github/workflows/ci.yml` with the jobs above, caching (`Swatinem/rust-cache`, pnpm store cache) and `concurrency` cancel-in-progress per ref.
- `Makefile` targets:
  - `dev-db` (`docker compose -f deploy/dev/compose.yaml up -d`, Postgres 16 on 5433, user/pass `eq`/`eq`, db `ritma`)
  - `dev` (runs `cargo watch -x 'run -p eq-server'` and `pnpm -C web dev` concurrently; fall back to instructions if cargo-watch is missing)
  - `test`, `lint`, `api` (placeholder until T-114), `e2e` (placeholder)
  - `build` (web then release binary), `clean`
- `deploy/dev/compose.yaml`, `.env.example` (all OPS-01 vars with comments, dev values).
- `deny.toml`, `scripts/check-literals.sh` (+ `.allow`), `.github/dependabot.yml` or `renovate.json` (weekly, grouped).

### Out
- Release workflow (T-706). E2E content (T-008).

## Files
| Path | Action |
|---|---|
| `.github/workflows/ci.yml`, `Makefile`, `deploy/dev/compose.yaml`, `.env.example`, `deny.toml`, `scripts/check-literals.sh`, `scripts/check-literals.allow`, `renovate.json` | create |

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

## Definition of done
- [ ] CI green on the PR
- [ ] Commit `ci: workflow, makefile, dev db and guard scripts [T-006]`, signed off

## Report

## Questions
