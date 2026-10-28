---
id: T-108
title: Add the Postgres job queue, job runner, and cron planner evaluated in tenant timezone
milestone: M1
status: review
depends_on: [T-104, T-105, T-107a]
spec_refs: [ARC-09, ARC-10, TST-15, STM-81, CAN-62, ERR-51, OPS-08]
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
- `JobRunner::run_due(&[TenantInfo], now)` processes every due job synchronously in bounded claim batches. It is used by TST-15 time-travel tests.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Job payloads carry a `v` integer and ids only (CAN-62). A `running` job whose `locked_until` passed is reclaimable (STM-81, D-52).

## Scope
### In
- `eq_core::jobs::{JobHandler, JobCtx, CronSpec, enqueue, JobRunner, CronPlanner}` and the migration `0005`.
- Gating: jobs whose module is disabled for the tenant are marked `done` with `last_error='skipped: module disabled'`.
- ERR-51/OPS-08: a tenant job that reaches dead-letter creates one high-severity `attention_items` row with an id-only payload and a dedupe key based on its job id; every dead-letter is logged with ids only.

### Out
- Concrete jobs (module tasks).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/jobs.rs` | modify | Public queue, runner, and planner contracts. |
| `crates/eq-core/src/jobs/repo.rs` | create | Parameterized SQL for the core-owned `jobs` and `cron_state` tables. |
| `crates/eq-core/tests/jobs.rs` | create | Task-listed PostgreSQL and cron verification tests. |
| `migrations/0005_core_cron_state.sql` | create | Exact §4.12 `cron_state` DDL mapped by ERD §7. |
| `crates/eq-core/Cargo.toml` | modify | Use the workspace-pinned `cron` crate. |
| `Cargo.lock` | modify | Record the resolved workspace dependency graph. |
| `.sqlx/` | modify | Compile-time metadata for the added `query!` statements. |
| `tasks/DECISIONS.md` | modify | Record traceable run 6 decisions TD-059–TD-065. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

The planner and runner receive trusted caller-supplied tenant metadata as permitted by TD-062: `CronPlanner::plan_due(&[TenantInfo], Timestamp)` and `JobRunner::run_due(&[TenantInfo], Timestamp)`. They do not enumerate `tenants` through a new database function or grant. Cron strings in `CronSpec` are five-field expressions; the approved `cron` crate's seconds/year fields are supplied by the adapter.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: cross-tenant background processing over system tables (SBD-30 T4/T8).
- Data class: all `jobs` and `cron_state` columns are S1 per the core data dictionary in `spec/29-erd.md`; this task reads no S2/S3 fields.
- C: the planner accepts only trusted `TenantInfo` supplied by its caller; it must not read `tenants` directly or use an owner pool. Tenant job handlers run only after the existing `EnabledModuleSet` gate, and handler transactions use `db::begin_system` so tenant context is pinned (SBD-40/42, SEC-05, MOD-05).
- I: job payloads contain ids only (CAN-62); job `Debug` output redacts payloads, and persisted `last_error` uses fixed PII-free status text rather than handler messages (SBD-10/21/23, ERR-51).
- Tenant dead letters use the existing core-owned `attention_items` table. Platform jobs have no tenant id, while `attention_items.tenant_id` is required; do not invent a platform tenant or schema. The platform path emits the fixed alert log and remains a documented ERR-51 gap.
- Abuse-case acceptance criterion: `disabled_module_job_is_skipped` proves a disabled module cannot execute its handler; `planner_dedupe_across_instances` proves duplicate planners cannot enqueue a second job (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. A cron `0 1 * * *` for a tenant in `Asia/Jakarta`, with the clock moving from 2026-01-05T17:59Z to 18:01Z (01:00 local on Jan 6), inserts exactly one job with dedupe `x:{tenant}:2026-01-06`. → `cron::tests::fires_in_tenant_tz`
- AC2. Two planners running the same tick insert one job. → `planner_dedupe_across_instances`
- AC3. Retryable failures follow the stated backoff and become `dead` after 5 attempts; a tenant dead letter creates one high-severity OPS-08 attention item. Permanent failures go directly to dead-letter per ERR-51. → `job_retry_and_dead`, `job_permanent_failure_is_dead_lettered`
- AC4. An expired lease is reclaimed by the reaper. → `lease_reaped`
- AC5. After a 3-day downtime, only one catch-up occurrence fires. → `catch_up_single_occurrence`
- AC6. DST: in `America/New_York`, a `30 2 * * *` job on the spring-forward day fires once at 03:00 local. → `dst_spring_forward`
- AC7. A due job for a disabled tenant module is completed without invoking its handler. → `disabled_module_job_is_skipped`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/jobs.rs` | `fires_in_tenant_tz`, `planner_dedupe_across_instances`, `job_retry_and_dead`, `lease_reaped`, `catch_up_single_occurrence`, `dst_spring_forward`, `disabled_module_job_is_skipped`, `dst_fall_back_first_occurrence`, `job_permanent_failure_is_dead_lettered`, `runner_processes_due_jobs_across_batches` |

## Definition of done
- [x] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass; no `Utc::now()` in `jobs/` except the `SystemClock` impl
- [x] Commit `feat(core): job queue and tenant-timezone cron [T-108]`, signed off

## Report
**Plan** — (1) Use only the mapped `cron_state` DDL from spec 06 §4.12 and verify it with ERD checks and PostgreSQL 16. (2) Adapt the approved `cron` parser to five-field schedules and resolve DST gaps and folds. (3) Implement transaction-bound enqueue, bounded claims drained until no due jobs remain, lease recovery, completion, ERR-51 retry/dead-letter handling, module gating and OPS-08 attention for tenant jobs. (4) Plan cron jobs for the trusted active `TenantInfo` slice and insert only the latest catch-up occurrence. (5) Run all listed tests, Rust/SQLx/dependency/ERD/guard/security checks, and document limits without widening database access.
**Assumptions** — `cron_state` matches spec 06 §4.12 and ERD §7. Retry delay, lease, max attempts and first-tick lookback use only values already in this task. The planner and runner consume trusted tenant metadata supplied by their caller; this task adds no tenant-table grant or resolver. Platform jobs cannot have an `attention_items` row because that existing table requires a tenant id.
**Test plan** — AC1 `fires_in_tenant_tz`; AC2 `planner_dedupe_across_instances`; AC3 retry/dead/backoff and tenant attention in `job_retry_and_dead`, plus immediate permanent failure in `job_permanent_failure_is_dead_lettered`; AC4 `lease_reaped`; AC5 `catch_up_single_occurrence`; AC6 `dst_spring_forward`; AC7 `disabled_module_job_is_skipped`. Also test fall-back's first occurrence with `dst_fall_back_first_occurrence`, ensure `run_due` drains more than one claim batch with `runner_processes_due_jobs_across_batches`, and verify PII-free persistence and `Debug` output in the retry test.
**Summary** — T-108 is implemented. The ten jobs tests and 203 workspace tests pass. Tenant dead letters create a deduped high-severity attention row; platform dead letters emit an ids-only alert log but cannot create the required tenant-scoped attention row under the mapped schema. The planner/runner accept a trusted `TenantInfo` slice and do not widen tenant-table access.
**Workflow trace** — Step 0: selected the lowest-ID eligible task T-108 on `milestone/m1-core`; T-104, T-105 and T-107a are review. Marked T-108 `in_progress` and regenerated the index. Step 1: read the full task and cited specs, including `cron_state` DDL and ERD §7 mapping. Step 2: read dependency Reports and shipped code for T-104, T-105 and T-107a, including resolver, transaction, module-gate and jobs-schema contracts. Step 3: n/a; backend task. Step 4: recorded the plan, assumptions and AC-to-test mapping above. Step 5: implemented the queue, runner, cron planner, migration and tests within Scope. Step 6: PostgreSQL-backed service tests exercise success, disabled modules, retry/dead letters, lease recovery, batching and cron paths; `make dev` is not applicable because T-108 does not wire the worker into the server runtime (T-117 owns that integration). Step 7: see Debug log. Step 8: reviewed queries, transitions, PII handling, tenant gating and DST behavior against specs 16–19 and 29. Step 9: no measured optimization was needed; claim batches remain bounded and are drained until empty. Step 10: reviewed SBD-30 T4/T8 and ran targeted secret scans on changed code, tests, migration, SQLx metadata and task file. Step 11: final Rust, SQLx, dependency, ERD, lint, guard and database checks below. Step 12: status set to `review`, task index regenerated, one signed-off task commit prepared.
**Spec read** — `spec/02-architecture.md` §§7–8; `spec/03-module-system.md` §§3–4; `spec/06-domain-model.md` §§4.1/4.12; `spec/12-testing.md` TST-15/20; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§3–5/8; `spec/19-error-handling.md` ERR-50/51; `spec/20-state-machines.md` STM-81; `spec/21-data-flows.md` FLW-04; `spec/23-contracts-and-canonical-forms.md` CAN-62; `spec/24-open-source-stack.md` §§3/5; `spec/29-erd.md` core data dictionary and §7. Also read T-104/T-105/T-107a Reports and inspected their shipped resolver, transaction, module, migration and job contracts.
**Files** — Updated `crates/eq-core/src/jobs.rs`, created `crates/eq-core/src/jobs/repo.rs` and `crates/eq-core/tests/jobs.rs`, added `migrations/0005_core_cron_state.sql`, added the workspace `cron` dependency and SQLx metadata, and updated this task, `tasks/DECISIONS.md` and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `cron::tests::fires_in_tenant_tz` | Pass |
| AC2 | `planner_dedupe_across_instances` | Pass |
| AC3 | `job_retry_and_dead`, `job_permanent_failure_is_dead_lettered` | Pass for retry/permanent paths and tenant OPS-08 attention; platform attention row not met (see TD-065) |
| AC4 | `lease_reaped` | Pass |
| AC5 | `catch_up_single_occurrence` | Pass |
| AC6 | `dst_spring_forward` | Pass |
| AC7 | `disabled_module_job_is_skipped` | Pass |
| Context | `dst_fall_back_first_occurrence`, `runner_processes_due_jobs_across_batches` | Pass |
**Checks run** — `cargo fmt --all --check` ✅. `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` ✅. `SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` ✅ (203 passed, 0 failed; includes T-108's 10/10). `SQLX_OFFLINE=true cargo test -p eq-core --test jobs -- --test-threads=1` ✅ (10/10 on fresh Testcontainers `postgres:16-alpine`; each `TestDb` gets an isolated database cloned from a newly migrated test template, with runtime queries through `eq_app`). `env DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:32770/ritma cargo sqlx prepare --workspace -- --all-targets` ✅; `env DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:32770/ritma cargo sqlx prepare --workspace --check -- --all-targets` ✅. `cargo deny check` ✅ (advisories, bans, licenses and sources; `deny.toml` unchanged). `python3 -I scripts/erd.py --migrations` and `python3 -I scripts/erd.py --check` ✅. `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` ✅. `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8). `scripts/tasks-index.sh --check` and `git diff --check` ✅. Targeted `gitleaks dir --redact --no-banner` scans of `crates/eq-core/src/jobs.rs`, `crates/eq-core/src/jobs/repo.rs`, `crates/eq-core/tests/jobs.rs`, `migrations/0005_core_cron_state.sql`, `.sqlx/` and this task ✅ (no leaks). The exact requested `python3 -I -m unittest scripts/tests/test_tasks_lint.py` was run and failed with `ModuleNotFoundError: No module named 'scripts'` because Python isolated mode removes the repository root from `sys.path`; the equivalent isolated discovery command above passed. Not run: hosted GitHub Actions, which requires a hosted runner. `make e2e` is not applicable to T-108, which is not the M1 integration task; T-117 owns the fresh-database M1 journey. ERR-51 platform dead-letter attention row: not met because `jobs.tenant_id` may be NULL while the mapped `attention_items.tenant_id` is NOT NULL; adding a platform tenant/table would violate the schema boundary (TD-065).
**Manual checks** — n/a; T-108 defines no HTTP route. Runtime behavior is exercised through the database-backed job and cron tests.
**Debug log** — The first nine-test run failed only in `job_retry_and_dead`: its assertion selected `attention_items` without a tenant transaction, so RLS hid the row. The assertion now reads through `db::begin` with the job tenant; the rerun passed 10/10. Clippy first rejected two test-fixture `expect()` calls; narrow `#[expect(clippy::expect_used, reason = ...)]` annotations document why those fixed fixtures must fail fast. An earlier planner test tick at exactly 18:01Z missed the 18:00 instant under the first one-minute lookback; it now ticks at 18:00:30Z within the required interval. No failure remains in final checks.
**Performance** — The claim query uses the technical 100-job bound and `FOR UPDATE SKIP LOCKED`; `run_due` drains successive bounded batches so all jobs due at the supplied instant are processed. No benchmark was required.
**Security** — Triggers: cross-tenant system-table processing and tenant-bound job effects (SBD-30 T4/T8). SBD-81 review: (1) no HTTP/auth chain is added; (2) no client tenant id is accepted, the planner uses caller-trusted `TenantInfo`, and handlers use the existing tenant-pinned system transaction; `jobs`/`cron_state` retain the mapped system-table RLS design; (3) every queue and cron column is S1, with no S2/S3 data; (4) schedules are static module registrations, payloads contain ids only, and SQL is parameterized; (5) no HTML, CSV or URL output is produced; (6) no crypto or token handling is added; (7) no external calls or webhook handling is added; (8) dedupe uses the mapped unique key and `ON CONFLICT`, while leases and retries follow STM-81/ERR-51; (9) no security event is emitted, so no new security audit event is required; and (10) `disabled_module_job_is_skipped`, `planner_dedupe_across_instances`, and the PII-redaction assertions cover the abuse cases and fail if their controls are removed. Jobs/cron columns are S1. The planner never reads `tenants` or uses an owner pool; claims are restricted to supplied tenant ids; module handlers run only after `EnabledModuleSet`; handler DB work uses `db::begin_system`. Payload and attention records contain ids only; `Debug`, logs and persisted error text redact handler details. The platform-job attention limitation is documented because `attention_items.tenant_id` is NOT NULL and no platform alert table is mapped. Test fixtures use an owner pool only for setup; code under test uses `TestDb::app_pool` and tenant context for the RLS-protected attention assertion.
**Deviations** — RATIFY TD-059–TD-062: use the approved `cron` adapter; use ERR-51's typed `JobError`; bound each claim with a technical batch constant; and accept trusted tenant metadata without a new grant/resolver. RATIFY TD-064–TD-065: reuse the T-107c OPS-08 attention pattern for tenant dead letters; platform dead letters cannot create a tenant-scoped row under the mapped schema, so the ids-only alert log is retained and the missing platform row is reported. RATIFY TD-066–TD-067: complete spec-backed Security sections and required task headings from the audited task contract; details are in `tasks/DECISIONS.md` and `tasks/HYGIENE.md`.
**Dependencies affected** — T-109 and later M1 tasks depend on T-108; the runtime integration task must supply trusted tenant metadata to `CronPlanner` and `JobRunner`.
**Rollback plan** — The migration is forward-only. To disable this feature, stop invoking the planner/runner; if schema removal is needed later, use a reviewed compensating migration after all readers and writers are removed.
**Follow-ups** — T-117 must provide the authorized source of the active `TenantInfo` set. The architect should decide how OPS-08 represents dead-lettered platform jobs without adding an unauthorized tenant or table.

## Questions
- **Self-resolved (implementer, run 6; TD-059):** the task names `croner` but spec 24 §3 lists the pinned `cron` crate → use `cron` and adapt its parser to the task's five-field schedule form, because the spec names the approved dependency.
- **Self-resolved (implementer, run 6; TD-060):** the task sketch returns `anyhow::Result`, while ERR-51 defines retryable and permanent job failures → use `Result<(), JobError>` with the typed variants.
- **Self-resolved (implementer, run 6; TD-061):** ARC-09 leaves claim batch size `n` unspecified → use `JOB_CLAIM_BATCH_SIZE = 100` as a `const` with a technical-bound comment, matching the shipped outbox claim size.
- **Self-resolved (implementer, run 6; TD-062):** the active-tenant planner/runner has no permitted database enumerator; Section 3.2 limits new functions and grants to those named in Section 3.3 → keep `eq_app` tenant-table access unchanged and accept trusted caller-supplied `TenantInfo` values in both components. Do not add a SECURITY DEFINER function, grant, table, column or policy.
- **Self-resolved (implementer, run 6; TD-064):** ERR-51/OPS-08 requires tenant dead-letter attention but T-108 does not specify its row shape → reuse T-107c's high-severity attention pattern, with `kind='job_dead_letter'`, a job-id reference/payload and dedupe by job id, because the existing table supports this without schema changes.
- **Self-resolved (implementer, run 6; TD-065):** ERR-51 covers platform jobs, but `attention_items.tenant_id` is NOT NULL while platform jobs have no tenant → do not invent a tenant or schema; emit the fixed ids-only alert log and report the missing platform attention row as a failing criterion under the delegated objective's §3.7.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
