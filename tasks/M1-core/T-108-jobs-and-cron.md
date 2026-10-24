---
id: T-108
title: Add the Postgres job queue, job runner, and cron planner evaluated in tenant timezone
milestone: M1
status: todo
depends_on: [T-107]
spec_refs: [ARC-09, ARC-10, TST-15]
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
    pub schedule: &'static str,                 // 5-field cron, evaluated in tenant TZ (crate `croner`)
    pub dedupe: fn(TenantId, chrono::DateTime<chrono_tz::Tz>) -> String, // e.g. "billing.daily:{tenant}:{2026-11-05}"
}
#[async_trait] pub trait JobHandler: Send + Sync {
    fn kind(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    async fn run(&self, ctx: &JobCtx, payload: &serde_json::Value) -> anyhow::Result<()>;
}
pub struct JobCtx { pub tenant: Option<Arc<TenantInfo>>, pub pool: PgPool, pub clock: Arc<dyn Clock>, pub now: Timestamp }
```
- `enqueue(tx, tenant, kind, payload, run_at, dedupe_key: Option<String>)` for ad-hoc jobs.
- DST handling: `croner` evaluated on `DateTime<Tz>`. Nonexistent local times (spring-forward) fire at the next valid instant. Ambiguous times (fall-back) fire once (the first occurrence).
- The planner's `last_tick` is stored in a `cron_state(tenant_id, kind, last_fire_at)` table (add it in `migrations/0005_core_cron_state.sql`). On first run, use `now - 1 minute`, so there's no backfill storm. **Catch-up:** if the server was down, fire at most **one** missed occurrence per spec (the latest).
- `JobRunner::run_due(now)` processes every due job synchronously. It is used by TST-15 time-travel tests.

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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; no `Utc::now()` in `jobs/` except the `SystemClock` impl
- [ ] Commit `feat(core): job queue and tenant-timezone cron [T-108]`, signed off

## Report

## Questions
