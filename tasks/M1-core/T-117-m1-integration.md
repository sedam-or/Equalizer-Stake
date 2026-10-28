---
id: T-117
title: M1 integration — wire real audit and outbox sinks, start background workers, seed CLI, end-to-end API scenario
milestone: M1
status: draft
depends_on: [T-105, T-106, T-107c, T-108, T-109, T-113b, T-114, T-116, T-116a]
spec_refs: [ARC-04, ARC-05, ARC-07, ARC-09, AUD-04, MOD-04, TST-04, OPS-04, CAN-96]
estimated_size: M
---

## Goal
The `ritma serve` binary runs the full M1 stack: tenant resolution, auth, module gating, policies, audit sealing, outbox dispatch and the job runner. One integration test drives a realistic owner and admin journey through the HTTP API.

## Context
- Replace the test doubles from T-105/T-106 with the real `AuditSink` (eq-audit) and `EventSink` (core outbox) in `eq-server` wiring.
- Background tasks started by `serve` (each stops gracefully on SIGTERM via `CancellationToken`, and the shutdown waits ≤ 10 s):
  1. Outbox dispatcher (T-107)
  2. Job runner + cron planner (T-108)
  3. Audit sealer, a 10 s interval (T-109)
  4. Cache invalidation listeners for `eq_modules_changed` and `eq_policy_changed`. When the listener connection reconnects, clear every NOTIFY-backed cache (spec 26 CACHE-04, D-71)
- `ritma seed-dev`, **dev only** (refuses unless `RITMA_ENV=dev`): creates tenant `dev` (slug `dev`, timezone `Asia/Jakarta`, currency IDR, locale id), an owner `owner@dev.test` and an admin `admin@dev.test` with password `devpassword123`, all optional modules enabled, and policies at their defaults. **It must call the same services the API uses** (no raw inserts except the tenant row, because setup comes in M3). Document the dev credentials in `CONTRIBUTING.md` under "Dev login" (one line).
- `/readyz` checks DB connectivity, that migrations are applied (`_sqlx_migrations` count equals embedded), and that the dispatcher and runner tasks are alive.

## Scope
### In
- Track the M1 integration split and its dependency order only.
### Out
- Runtime implementation and integration tests, assigned to T-117a and T-117b.

## Split
| Part | Owns | Depends on | Status |
|---|---|---|---|
| T-117a | Runtime wiring, seed-dev, lifecycle and journey steps 1–6 | T-105, T-106, T-107c, T-108, T-109, T-113b, T-114, T-116, T-116a | draft |
| T-117b | Journey steps 7–12 and final audit/outbox assertions | T-117a | draft |

## Files
| Path | Action | Purpose |
|---|---|---|
| `CONTRIBUTING.md` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. The original M1 lifecycle and journey assertions are owned exactly once by T-117a and T-117b.

## Tests to write
| File | Test names |
|---|---|
| n/a | The child tasks own all integration tests. |

## Definition of done
- [ ] The original journey and lifecycle checks are owned exactly once by the split parts.
- [ ] The task index reflects the dependency chain.

## Report
**Deviations** — RATIFY: TD-063, split M1 integration into bounded runtime/setup and journey/audit parts.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original journey has 12 ordered assertions plus a lifecycle criterion and exceeds CQ-101 when counted individually → split the first six journey steps plus lifecycle into T-117a and the final six into T-117b, preserving `m1_journey` in T-117b and adding a focused `m1_journey_seed_auth` test in T-117a.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
