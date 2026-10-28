---
id: T-117a
title: M1 runtime, seed and journey setup integration
milestone: M1
status: todo
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
- Implement only server wiring, seed-dev, readiness, lifecycle and journey setup and the acceptance criteria/tests assigned to T-117a.
### Out
- The other integration behaviors assigned to the sibling split part.

## Files
| Path | Action | Purpose |
|---|---|---|
| `CONTRIBUTING.md` | create or modify | Path named in this task's contract. |
| `crates/eq-server/tests/lifecycle.rs` | test | Task-listed verification file. |
| `crates/eq-server/tests/m1_journey.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T3, T4 (SBD-30).
- Data classes touched: S1/S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `m1_journey_seed_auth` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Seed via the `seed-dev` service. → `m1_journey_seed_auth`
- AC2. The owner logs in. → `m1_journey_seed_auth`
- AC3. The owner reads `/meta/modules` and sees all modules enabled. → `m1_journey_seed_auth`
- AC4. The owner PUTs the identity policy with `password_min_length: 12`. → `m1_journey_seed_auth`
- AC5. The admin logs in. → `m1_journey_seed_auth`
- AC6. The admin creates a guardian with an email and `invite: true`, channel `link`. → `m1_journey_seed_auth`
- AC7. `ritma serve` starts against a fresh DB and `/readyz` is 200 within 5 s. SIGTERM exits 0 within 10 s. → `serve_lifecycle`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/m1_journey.rs` | `m1_journey_seed_auth` |
| `crates/eq-server/tests/lifecycle.rs` | `serve_lifecycle` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks pass (`make check` if present, else the Rust checks + guard scripts)
- [ ] Commit `feat(server): M1 integration and dev seed [T-117]`, signed off

## Report
**Deviations** — RATIFY: TD-063, split the original integration journey into bounded dependent parts; existing test name `m1_journey` stays in T-117b.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original journey has 12 ordered assertions plus a lifecycle criterion and exceeds CQ-101 when counted individually → split the first six journey steps plus lifecycle into T-117a and the final six into T-117b, preserving `m1_journey` in T-117b and adding a focused `m1_journey_seed_auth` test in T-117a.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
