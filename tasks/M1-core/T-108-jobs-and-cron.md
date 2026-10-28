---
id: T-108
title: Add the Postgres job queue, job runner, and cron planner evaluated in tenant timezone
milestone: M1
status: blocked
depends_on: [T-107a]
spec_refs: [ARC-09, ARC-10, TST-15, STM-81, CAN-62]
estimated_size: M
---

## Goal
Modules register job handlers and cron specs. Scheduled work runs exactly once per dedupe key across any number of instances, in the tenant's local time, and is fully testable by time travel.

## Context
- The `jobs` table was created by T-107's migration (spec 06 §4.1): `id, tenant_id (NULL = platform), kind, payload, run_at, status queued|running|done|failed|dead, attempts, max_attempts (5), locked_until, last_error, dedupe_key UNIQUE, created_at, finished_at`, index `jobs_ready ON (run_at) WHERE status='queued'`.
- ARC-09:
  - **Claim:** `UPDATE jobs SET status='running', locked_until=now()+lease, attempts=attempts+1 WHERE id IN (SELECT id FROM jobs WHERE status='queued' AND run_at <= $now ORDER BY run_at FOR UPDATE SKIP LOCKED LIMIT $n) RETURNING …`. The lease is 5 min (technical const).
  - **Reaper:** `running` with `locked_until < now` → back to `queued`.
  - **Failure:** back off `run_at = now + 30s * 2^(attempts-1)` (technical const), and set `dead` when `attempts >= max_attempts`.
  - **Cron planner:** runs every minute. For each **active** tenant × each enabled module's `CronSpec`, it computes due fire times in `(last_tick, now]` evaluated in the tenant TZ, and inserts `jobs` with `ON CONFLICT (dedupe_key) DO NOTHING`.
  - Policy-driven days (e.g. "issue on day N") are **not** cron expressions. Modules register a daily cron (e.g. `0 1 * * *` local) whose handler checks the policy. Never hard-code a day.
- Cron spec:
```rust
pub struct CronSpec {
    pub kind: &'static str,                     // e.g. "billing.daily"
    pub module: ModuleId,
    pub schedule: &'static str,                 // 5-field cron, evaluated in tenant TZ (spec 24 `cron`)
    pub dedupe: fn(TenantId, chrono::DateTime<chrono_tz::Tz>) -> String, // e.g. "billing.daily:{tenant}:{2026-11-05}"
}
pub enum JobError { Retryable(anyhow::Error), Permanent(anyhow::Error) }
#[async_trait] pub trait JobHandler: Send + Sync {
    fn kind(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    async fn run(&self, ctx: &JobCtx, payload: &serde_json::Value) -> Result<(), JobError>;
}
pub struct JobCtx { pub tenant: Option<Arc<TenantInfo>>, pub pool: PgPool, pub clock: Arc<dyn Clock>, pub now: Timestamp }
```
- `enqueue(tx, tenant, kind, payload, run_at, dedupe_key: Option<String>)` for ad-hoc jobs.
- DST handling: the spec 24 `cron` parser is evaluated on `DateTime<Tz>`. Nonexistent local times (spring-forward) fire at the next valid instant. Ambiguous times (fall-back) fire once (the first occurrence).
- The planner's `last_tick` is stored in a `cron_state(tenant_id, kind, last_fire_at)` table (add it in `migrations/0005_core_cron_state.sql`, DDL verbatim from `spec/06-domain-model.md` §4.12: PK `(tenant_id, kind)`, a system table **without RLS** like `jobs`, used only by the planner). On first run, use `now - 1 minute`, so there's no backfill storm. **Catch-up:** if the server was down, fire at most **one** missed occurrence per spec (the latest).
- `JobRunner::run_due(now)` processes every due job synchronously. It is used by TST-15 time-travel tests.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Job payloads carry a `v` integer and ids only (CAN-62). A `running` job whose `locked_until` passed is reclaimable (STM-81, D-52).

## Scope
### In
- `eq_core::jobs::{JobHandler, JobCtx, CronSpec, enqueue, JobRunner, CronPlanner}` and the migration `0005`.
- Gating: jobs whose module is disabled for the tenant are marked `done` with `last_error='skipped: module disabled'`.

### Out
- Concrete jobs (module tasks).

## Acceptance criteria
- AC1. A cron `0 1 * * *` for a tenant in `Asia/Jakarta`, with the clock moving from 2026-01-05T17:59Z to 18:01Z (01:00 local on Jan 6), inserts exactly one job with dedupe `x:{tenant}:2026-01-06`. → `cron::tests::fires_in_tenant_tz`
- AC2. Two planners running the same tick insert one job. → `planner_dedupe_across_instances`
- AC3. A failed job is retried with backoff and becomes `dead` after 5 attempts. → `job_retry_and_dead`
- AC4. An expired lease is reclaimed by the reaper. → `lease_reaped`
- AC5. After a 3-day downtime, only one catch-up occurrence fires. → `catch_up_single_occurrence`
- AC6. DST: in `America/New_York`, a `30 2 * * *` job on the spring-forward day fires once at 03:00 local. → `dst_spring_forward`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/jobs.rs` | the AC names above |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; no `Utc::now()` in `jobs/` except the `SystemClock` impl
- [ ] Commit `feat(core): job queue and tenant-timezone cron [T-108]`, signed off

## Report
**Plan** — (1) Add only the `cron_state` DDL from §4.12 under the mapped `0005` filename → verify `scripts/erd.py --migrations` and migration on fresh PostgreSQL 16. (2) Parse module cron specs with the stack's `cron` crate and evaluate tenant-local fire times using `chrono-tz` → verify Jakarta boundary, catch-up, and New York DST cases. (3) Add queue enqueue/claim/reap/finish transitions, ERR-51 typed errors, and tenant module gating → verify retry/dead/lease tests and two-runner dedupe. (4) Add the cron planner using an authorized active-tenant source, module specs and `cron_state` → verify per-tick idempotency and latest-only catch-up. (5) Run Rust, SQLx, deny, ERD, guard, security and fresh-database E2E checks. Work is blocked before implementation until the active-tenant source is defined.
**Assumptions** — The exact `cron_state` schema and migration mapping are `spec/06` §4.12 and `spec/29` §7. The task's retry delays, lease, tick frequency, and max attempts are technical values already specified. No new role grant, table, permission, business value or error code is inferred.
**Test plan** — AC1 Given a registered 01:00 daily schedule and an active Jakarta tenant, when the clock advances across 18:00Z on Jan 5, 2026, then exactly one local Jan 6 job with the canonical dedupe key is inserted → `cron::tests::fires_in_tenant_tz`. AC2 Given two planners process the same tick, when both insert the same key, then one job remains → `planner_dedupe_across_instances`. AC3 Given retryable failures, when the task backoff is applied five times, then attempts are retried and end `dead` at the max → `job_retry_and_dead`. AC4 Given an expired running lease, when the runner reaps it, then it returns to `queued` → `lease_reaped`. AC5 Given three days of downtime, when the planner catches up, then only the latest missed occurrence is inserted → `catch_up_single_occurrence`. AC6 Given the New York spring-forward gap, when a 02:30 schedule is evaluated, then it fires once at 03:00 local → `dst_spring_forward`. Abuse cases include module-disabled jobs not running, duplicate planners, expired-lease recovery and payload/error redaction. These tests were not run because the planner has no authorized way to enumerate active tenants.
**Summary** — Blocked before implementation. The required planner must enumerate active tenants and their timezones, but the runtime `eq_app` role has no least-privilege tenant-list interface; only per-slug and single-tenant resolvers exist. Adding a grant or cross-tenant reader without an architect-defined contract would violate the security and migration rules.
**Workflow trace** — Step 0: selected the lowest-ID eligible task T-108 on `milestone/m1-core`; T-107a is `review`, the only dependency, and the task index initially had no in-progress work. Claimed T-108 and regenerated the index. Step 1: read the full task, task constraints, specs, DDL and migration map. Step 2: read T-107a's Report, deviations/follow-ups and shipped migration/outbox/transaction code. Step 3: n/a; backend-only task. Step 4: recorded the implementation plan, assumptions and AC-to-test plan above; classified three mechanical mismatches using cited specs. Step 5: n/a; implementation blocked before code because no authorized active-tenant enumeration interface exists. Step 6: n/a; no runtime code was changed. Step 7: n/a; no implementation failures occurred. Step 8: checked the current resolver functions and `eq_app` privileges; no new code diff to review. Step 9: n/a; no query or index was implemented. Step 10: reviewed the cross-tenant planner trigger and documented the missing least-privilege control below. Step 11: ran task-index, ERD-document and diff checks; implementation, database and browser suites were not run because the blocker prevents a safe implementation. Step 12: set `blocked`, regenerated the index, and prepared one signed-off block commit.
**Spec read** — `spec/02-architecture.md` §§7–8 (ARC-09/10); `spec/03-module-system.md` §§3–4 (MOD-01/04/05); `spec/06-domain-model.md` §§4.1/4.12; `spec/12-testing.md` TST-15/20; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§3–5/8 (SBD-20/30/31/42); `spec/19-error-handling.md` ERR-50/51; `spec/20-state-machines.md` STM-81; `spec/21-data-flows.md` FLW-04; `spec/23-contracts-and-canonical-forms.md` CAN-62; `spec/24-open-source-stack.md` §§3/5; `spec/29-erd.md` core data dictionary and §7. Also read T-107a's Report and inspected its shipped migration, resolver functions, `Tx`, module gate and `jobs.rs` contracts.
**Files** — Updated `tasks/M1-core/T-108-jobs-and-cron.md` with the three self-resolutions, blocking question and report; regenerated `tasks/INDEX.md`. No source, dependency, migration or schema file was changed.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `cron::tests::fires_in_tenant_tz` | Not run: blocked before implementation. |
| AC2 | `planner_dedupe_across_instances` | Not run: blocked before implementation. |
| AC3 | `job_retry_and_dead` | Not run: blocked before implementation. |
| AC4 | `lease_reaped` | Not run: blocked before implementation. |
| AC5 | `catch_up_single_occurrence` | Not run: blocked before implementation. |
| AC6 | `dst_spring_forward` | Not run: blocked before implementation. |
**Checks run** — `scripts/tasks-index.sh --check` ✅. `python3 -I scripts/erd.py --check` ✅. `git diff --check` ✅. `scripts/erd.py --migrations` was not run because no migration was created; the task stopped before implementation. Rust checks, SQLx preparation, dependency gate, guard scripts, database tests and E2E were not run because the cross-tenant read contract is unresolved. Hosted Actions was not run (hosted runner required).
**Manual checks** — n/a; no executable code was changed.
**Debug log** — No code/test failure. Blocker verified by inspecting `migrations/0002_core_tenancy.sql`, `migrations/0009_core_tenant_resolver.sql`, and `TenantResolver`: `eq_app` can resolve one known slug or the single tenant, but cannot enumerate active tenants for multi-tenant planning. The module gate accepts a known tenant id; it does not provide tenant enumeration.
**Performance** — n/a; no query or implementation was created.
**Security** — Triggers: T4 (database roles and tenant-scoped system tables) and T8 (a planner operating across tenants). The intended controls are the mapped no-RLS job tables, `eq_app` as the runtime role, `EnabledModuleSet` checks per tenant/module, `db::begin_system` for tenant-bound effects, CAN-62 id-only job payloads, and PII-free `last_error` capped at 500 characters under ERR-51. Existing resolver functions do not enumerate active tenants; a broad `SELECT` grant or owner/superuser pool would widen access and is not authorized. No abuse-case test was run because there is no safe planner input source yet.
**Deviations** — RATIFY: (1) The task's cron sketch names `croner`, while spec 24 §3 lists `cron`; use the already pinned workspace `cron` crate and adapt the five-field input to its parser without changing the public schedule form. (2) The task's `JobHandler` sketch returns `anyhow::Result`, while ERR-51 requires `JobError::{Retryable, Permanent}`; the typed spec contract takes precedence. (3) ARC-09 uses an unspecified claim `LIMIT n`; use `JOB_CLAIM_BATCH_SIZE = 100` as a technical bound aligned with the existing outbox claim size. These choices are recorded under Questions and need architect ratification. Blocking Q1 remains unresolved.
**Dependencies affected** — T-109 directly depends on T-108 and is ineligible until this decision is resolved and the job runner reaches `review`; downstream tasks depending on T-109 remain ineligible.
**Rollback plan** — Revert the block commit; no code, migration, database state or schema was changed.
**Follow-ups** — Architect to define an authorized active-tenant/timezone enumeration contract and assign its grant/function or port ownership, then return T-108 to `todo` with its allowed files and tests updated.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** the task names the `croner` crate but spec 24 §3 lists `cron` → use the already pinned workspace `cron` crate and adapt parser input to its accepted syntax while preserving the task's five-field `CronSpec.schedule`, because spec 24 is authoritative.
- **Self-resolved (implementer, run 6, needs architect ratification):** the `JobHandler` sketch returns `anyhow::Result`, but ERR-51 defines retryable/permanent job failures → use `Result<(), JobError>` with `JobError::Retryable` and `JobError::Permanent`, because ERR-51 defines the failure classification.
- **Self-resolved (implementer, run 6, needs architect ratification):** ARC-09 leaves the claim batch size `n` unspecified → use `JOB_CLAIM_BATCH_SIZE = 100` with a technical-bound comment, matching the existing outbox claim size and bounding one worker transaction.
- **Q1 (blocking):** What least-privilege interface should the runtime use to enumerate active tenant IDs and validated timezones for cron planning?
  - Context: T-108 requires the planner to visit every active tenant. `eq_app` has no `SELECT` on `tenants`; migrations 0002 and 0009 grant it only `resolve_tenant_by_slug(text)`, `resolve_single_tenant()`, and `resolve_tenant_info_by_slug(text)`. The first two require a known slug or single-tenant mode; none enumerates tenants. `EnabledModuleSet` checks a supplied tenant id, but cannot supply the ids. The planner cannot meet ARC-09 in multi-tenant mode without widening access or an architect-defined port/function.
  - Options: (a) add a narrow `SECURITY DEFINER` resolver returning only active tenant id/timezone and grant `EXECUTE` to `eq_app`, with its migration/task ownership explicitly mapped; (b) provide an authorized upstream tenant-enumerator port and assign its implementation/owner; (c) run planning with a database owner or superuser connection.
  - Recommendation: (a), because a narrow resolver can preserve least privilege while enabling the specified cross-tenant planner; the architect must define its allowed migration/function and grant before implementation.
