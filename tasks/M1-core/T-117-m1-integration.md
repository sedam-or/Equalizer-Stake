---
id: T-117
title: M1 integration — wire real audit and outbox sinks, start background workers, seed CLI, end-to-end API scenario
milestone: M1
status: todo
depends_on: [T-116, T-114]
spec_refs: [ARC-04, ARC-05, ARC-07, ARC-09, AUD-04, MOD-04, TST-04, OPS-04]
estimated_size: S
---

## Goal
The `ritma serve` binary runs the full M1 stack: tenant resolution, auth, module gating, policies, audit sealing, outbox dispatch and the job runner. One integration test drives a realistic owner and admin journey through the HTTP API.

## Context
- Replace the test doubles from T-105/T-106 with the real `AuditSink` (eq-audit) and `EventSink` (core outbox) in `eq-server` wiring.
- Background tasks started by `serve` (each stops gracefully on SIGTERM via `CancellationToken`, and the shutdown waits ≤ 10 s):
  1. Outbox dispatcher (T-107)
  2. Job runner + cron planner (T-108)
  3. Audit sealer, a 10 s interval (T-109)
  4. Cache invalidation listeners for `eq_modules_changed` and `eq_policy_changed`
- `ritma seed-dev`, **dev only** (refuses unless `RITMA_ENV=dev`): creates tenant `dev` (slug `dev`, timezone `Asia/Jakarta`, currency IDR, locale id), an owner `owner@dev.test` and an admin `admin@dev.test` with password `devpassword123`, all optional modules enabled, and policies at their defaults. **It must call the same services the API uses** (no raw inserts except the tenant row, because setup comes in M3). Document the dev credentials in `CONTRIBUTING.md` under "Dev login" (one line).
- `/readyz` checks DB connectivity, that migrations are applied (`_sqlx_migrations` count equals embedded), and that the dispatcher and runner tasks are alive.

## Scope
### In
- Wiring in `eq-server`, the `seed-dev` CLI command, the readiness checks, and the integration test below.

### Out
- The SPA (M3).

## Acceptance criteria
- AC1. Scenario `m1_journey` (axum-test against the full router + real DB + the background tasks driven by `drain_once`/`run_due`):
  1. Seed via the `seed-dev` service.
  2. The owner logs in.
  3. The owner reads `/meta/modules` and sees all modules enabled.
  4. The owner PUTs the identity policy with `password_min_length: 12`.
  5. The admin logs in.
  6. The admin creates a guardian with an email and `invite: true`, channel `link`.
  7. The invite is accepted with an 11-char password → 422 `password_weak`, and with a 12-char one → 200.
  8. The guardian calls `GET /people` and sees only themselves.
  9. The owner disables `feedback`.
  10. The owner reads `/audit/entries` and sees `core.policy.changed`, `identity.person.created`, `identity.invite.accepted` and `core.module.disabled`.
  11. After `Sealer::run_once`, `/audit/verify` is OK.
  12. The outbox has `identity.user_invited.v1` dispatched (`dispatched_at` set). → `crates/eq-server/tests/m1_journey.rs::m1_journey`
- AC2. `ritma serve` starts against a fresh DB and `/readyz` is 200 within 5 s. SIGTERM exits 0 within 10 s. → `serve_lifecycle` (spawns the binary)

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/m1_journey.rs` | `m1_journey` |
| `crates/eq-server/tests/lifecycle.rs` | `serve_lifecycle` |

## Definition of done
- [ ] All checks pass (`make check` if present, else the Rust checks + guard scripts)
- [ ] Commit `feat(server): M1 integration and dev seed [T-117]`, signed off

## Report

## Questions
