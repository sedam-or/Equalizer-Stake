---
id: T-003
title: Add the DB pool, tenant-scoped transaction helper and bootstrap migration (extensions, roles, RLS helper)
milestone: M0
status: review
depends_on: [T-002]
spec_refs: [ARC-04, ARC-06, SEC-05, DOM-01, ADR-0006, ADR-0012]
estimated_size: M
---

## Goal
Services can open a transaction that has `app.tenant_id` and `app.actor_id` set (so RLS applies), and pooled connections never leak those settings. The first migration installs the extensions and DB roles that every later migration relies on.

## Context
- ARC-06 / SEC-05:
  - Every tenant table has RLS **enabled and forced**, with policy `tenant_isolation USING (tenant_id = current_setting('app.tenant_id', true)::uuid) WITH CHECK (same)`.
  - DB roles: `eq_owner` (runs migrations, owns objects); `eq_app` (used by the server, no BYPASSRLS, DML only); `eq_platform` (cloud jobs, BYPASSRLS, never used by handlers).
  - A connection returned to the pool must be `RESET ALL`.
- ARC-04 step 8: `core::db::begin(&pool, &tenant, &actor)` runs `SET LOCAL app.tenant_id = …, app.actor_id = …`.
- ADR-0012: one migration set in `/migrations`, files `NNNN_<module>_<desc>.sql`, forward-only.

- `Config` URLs are `SecretString` (T-002, SBD-50): `connect` and `run_migrations` callers read them with `.expose_secret()` at the single call into sqlx, and never log or format the URL. `run_migrations` keeps taking `&str`.

## Scope
### In
- `migrations/0001_core_bootstrap.sql`:
  - `CREATE EXTENSION IF NOT EXISTS btree_gist; pgcrypto; pg_trgm;`
  - Create roles `eq_app` and `eq_platform` if they don't exist (NOLOGIN; operators grant LOGIN/passwords in deployment). Wrap in `DO $$ … $$` so it's idempotent and tolerates a missing CREATEROLE (raise NOTICE and continue).
  - A helper function `eq_enable_rls(tbl regclass)` that runs the ENABLE/FORCE/CREATE POLICY boilerplate. Later migrations call `SELECT eq_enable_rls('locations');`.
  - Default privileges: `ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO eq_app; … GRANT USAGE, SELECT ON SEQUENCES TO eq_app;`. Wrap in an exception block if the roles are missing.
- `eq_core::db`:
  - `connect(config) -> PgPool`: uses `PgPoolOptions` with `max_connections = config.db_pool` and an `after_release` hook running `RESET ALL`.
  - `run_migrations(owner_url)`: uses `sqlx::migrate!("../../migrations")` with a Postgres advisory lock (7_000_001) around it.
  - `begin(pool, TenantId, &ActorCtx) -> Tx` and `begin_system(pool, TenantId) -> Tx` (actor = system).
  - `Tx` newtype over `sqlx::Transaction<'static, Postgres>` exposing `conn()` and `commit()` (see Interfaces; no `Deref`).
- `eq_core::ids::{TenantId, PersonId}` newtypes (uuid, serde transparent, sqlx transparent).
- `eq_core::actor::{ActorCtx, ActorRole}`: a minimal struct `{ person_id: Option<PersonId>, role: ActorRole, ip: Option<IpAddr>, request_id: Option<Uuid> }`, with `ActorRole::{Owner, Admin, Instructor, Guardian, Student, System, Platform}` and `ActorCtx::system(kind: &'static str)`.

### Out
- Domain tables (T-101). `Actor` with permissions (T-112).

## Files
| Path | Action | Purpose |
|---|---|---|
| `migrations/0001_core_bootstrap.sql` | create | extensions, roles, RLS helper |
| `crates/eq-core/src/db.rs`, `ids.rs`, `actor.rs` | create | |
| `crates/eq-core/src/lib.rs` | edit | modules |
| `crates/eq-core/Cargo.toml` | edit | direct Testcontainers test-only dependencies. |
| `crates/eq-core/tests/db.rs` | create | PostgreSQL role/RLS/pool integration tests. |
| `Cargo.toml` | edit | Use the current Testcontainers release with an audit-clean Docker client graph. |

## Interfaces
```rust
pub async fn connect(cfg: &Config) -> anyhow::Result<sqlx::PgPool>;
pub async fn run_migrations(owner_url: &str) -> anyhow::Result<()>;
pub struct Tx(sqlx::Transaction<'static, sqlx::Postgres>);
impl Tx { pub async fn commit(self) -> Result<(), sqlx::Error>; pub fn conn(&mut self) -> &mut sqlx::PgConnection; }
pub async fn begin(pool: &sqlx::PgPool, tenant: TenantId, actor: &ActorCtx) -> Result<Tx, AppError>;
pub async fn begin_system(pool: &sqlx::PgPool, tenant: TenantId) -> Result<Tx, AppError>;
```
`begin` runs, in one round trip: `SELECT set_config('app.tenant_id', $1, true), set_config('app.actor_id', $2, true)`. `is_local = true` is equivalent to SET LOCAL. The actor id is `''` for system.

`migrations/0001_core_bootstrap.sql` (normative core of it):
```sql
CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'eq_app') THEN CREATE ROLE eq_app NOLOGIN; END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'eq_platform') THEN CREATE ROLE eq_platform NOLOGIN BYPASSRLS; END IF;
EXCEPTION WHEN insufficient_privilege THEN RAISE NOTICE 'skipping role creation (no CREATEROLE)'; END $$;
CREATE OR REPLACE FUNCTION eq_enable_rls(tbl regclass) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', tbl);
  EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', tbl);
  EXECUTE format($p$CREATE POLICY tenant_isolation ON %s
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::uuid)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::uuid)$p$, tbl);
END $$;
```

## Business rules
None.

## Implementation notes
- FORCE RLS applies to the table owner too. Migrations run as the owner and don't touch tenant rows, so this is fine.
- **`FORCE ROW LEVEL SECURITY` binds the table owner but never a superuser or a `BYPASSRLS` role.** A connection made as the container's default user (`postgres`, or the dev compose user `eq`) sees every tenant's rows. Isolation therefore only holds when the server connects as `eq_app` (`NOSUPERUSER NOBYPASSRLS`); the owner URL is used for migrations only. T-006 makes the dev database do this. Say so in the Report.
- **Tests must exercise RLS as `eq_app`, never as the superuser** (a test on the superuser pool passes or fails vacuously). In the test: migrate with the superuser pool, run `ALTER ROLE eq_app LOGIN PASSWORD 'test-secret-app'` (a fake value), build a second `PgPool` that logs in as `eq_app`, and call `begin`/`connect` on that pool. Create the temp table with the superuser pool and `GRANT SELECT, INSERT ON <tbl> TO eq_app`. T-007 later moves this into `TestDb::app_pool`.
- `after_release`: `Box::pin(async move { sqlx::query("RESET ALL").execute(&mut *conn).await.map(|_| true).or(Ok(false)) })`. Returning `false` discards the connection.
- Integration tests need Postgres. Use `testcontainers` directly here (eq-testkit arrives in T-007).

## Acceptance criteria
- AC1. Migrations apply twice without error (idempotent bootstrap). → `db::tests::bootstrap_idempotent`
- AC2. (on the `eq_app` pool, see Notes) Given a temp table with RLS enabled through `eq_enable_rls` and rows for tenants A and B, When `begin(A)` selects, Then only A's rows are visible. When it selects without `begin` (plain pool connection), Then 0 rows. → `db::tests::rls_scopes_rows`
- AC3. After a transaction is committed and the connection reused from the pool, `current_setting('app.tenant_id', true)` is empty. → `db::tests::settings_do_not_leak`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/db.rs` | `bootstrap_idempotent`, `rls_scopes_rows`, `settings_do_not_leak` |

## Definition of done
- [x] Rust checks pass (tests need Docker running)
- [x] `cargo sqlx prepare --workspace` run (if any `query!` used)
- [x] Commit `feat(core): db pool, tenant tx and bootstrap migration [T-003]`, signed off

## Report

**Plan** — (1) Add the bootstrap migration exactly as mapped in ERD-06 → verify with `python3 scripts/erd.py --migrations`. (2) Add transparent ID newtypes and the specified actor context → verify the workspace compiles and IDs retain UUID serde/SQLx behavior. (3) Implement pool creation, reset-on-release, advisory-locked migrations and tenant/system transactions with compile-checked parameterized SQL → verify `cargo sqlx prepare --workspace`. (4) Add the exact three PostgreSQL integration tests and make them authenticate as `eq_app` → verify against PostgreSQL 16 using the Testcontainers service. (5) Run full Rust, audit, guard, ERD and E2E checks → record every unavailable check and rerun after fixes.

**Assumptions** — Use the inherited SQLx 0.9 baseline and the task's `Config` secret URLs. The task's `ActorCtx::system(kind)` argument has no storage field in the specified `ActorCtx` or schema; the constructor preserves the requested signature and sets only the specified `System` role/personless context, while `begin_system` passes an empty actor id per the task contract. Testcontainers is explicitly required by the integration-test notes, so its workspace-listed crate is a direct dev-dependency. Use the required `postgres:16-alpine` image explicitly. The initial Testcontainers 0.23 workspace version pulled audited advisories through Bollard; upgrade only that test dependency to 0.28 so audit remains enabled and clean.

**Test plan** —
- Given the bootstrap migration is available, when `run_migrations` is called twice, then both calls succeed → `db::tests::bootstrap_idempotent`.
- Given an RLS-enabled probe table with rows for tenants A and B, when the non-superuser `eq_app` connection begins for tenant A, then it sees only A; without a tenant transaction it sees no rows → `db::tests::rls_scopes_rows`.
- Given a pool connection has completed a tenant transaction and is returned, when the same one-connection pool is reused, then `app.tenant_id` is empty → `db::tests::settings_do_not_leak`.
- Edge/security cases: verify `eq_app` is NOSUPERUSER/NOBYPASSRLS; make the RLS assertions on its own pool (never the migration superuser); use a fake test-only password; ensure rollback/commit returns connections safely; apply migration lock/unlock around success and failure paths.

**Step 0 — Start** — On `milestone/m0-foundation`, the tree was clean after T-002 commit `5c9e1ea`; `scripts/tasks-index.sh --check` passed. T-002 is `review` and T-003 is `todo` with only T-002 as dependency, so T-003 is eligible. Set status to `in_progress` and regenerated the index.

**Step 1 — Task/spec** — Read the entire T-003 task and its interfaces/AC. The task is technical infrastructure only; no business values or tables are added. Read ARC-04/06, SEC-05, DOM-01, ADR-0006, ADR-0012, CQ-21/27/42 and §9/§12, CA-01/30, SBD-01/02/04/42/81, and ERD-06's `0001_core_bootstrap.sql` row. The migration creates no table, so its exact schema contract is the ERD row.

**Step 2 — Dependencies** — Read T-002's Report Deviations and Follow-ups and its shipped `Config`, `AppError`, and core module code. T-002 resolved the SQLx audit contract by moving to 0.9; `Config.database_url` and `database_owner_url` are `SecretString`, and only the connect call / caller into `run_migrations` may expose them. The two pending T-002 ratifications concern public-URL credentials and incomplete CAPTCHA settings; neither changes DB APIs. T-001's workspace manifest and lockfile provide direct Testcontainers workspace dependency declarations.

**Step 3 — Design reference** — n/a; this is a backend/database task.

**Step 4 — Plan and test plan** — Recorded above before implementation. Three task-shape self-resolutions are listed under Questions for architect ratification.

**Summary** — Added the core PostgreSQL pool, tenant transaction helpers, UUID ID newtypes, actor context, and the forward-only bootstrap migration. Integration tests verify repeatable migration execution, tenant isolation through a non-superuser `eq_app` pool, and transaction-local settings cleanup.

**Spec read** — `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4; `spec/18-security-by-design.md` §§4, 5, 8; `spec/19-error-handling.md` §10; `spec/29-erd.md` §§1, 4, 5, 7. Also read the task-inlined ARC-04/06, SEC-05, DOM-01, ADR-0006 and ADR-0012 excerpts.

**Files** —
- Created `migrations/0001_core_bootstrap.sql` with extensions, guarded roles, RLS helper and default privileges.
- Created `crates/eq-core/src/db.rs`, `ids.rs`, `actor.rs`, and `crates/eq-core/tests/db.rs`.
- Modified `crates/eq-core/src/lib.rs` to expose the task's new modules.
- Modified `Cargo.toml`, `crates/eq-core/Cargo.toml`, `Cargo.lock` to use the audit-clean Testcontainers release as a test-only dependency.
- Added SQLx offline metadata under `.sqlx/` for the compile-checked queries.
- Updated this task's Files, Plan and Report; regenerated `tasks/INDEX.md` through `scripts/tasks-index.sh`.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `bootstrap_idempotent` | Passed; `run_migrations` applied twice to disposable PostgreSQL 16. |
| AC2 | `rls_scopes_rows` | Passed; assertions ran on an `eq_app` pool with `NOSUPERUSER NOBYPASSRLS`. Tenant A saw only A; an unscoped connection saw no rows. |
| AC3 | `settings_do_not_leak` | Passed; after commit and pool reuse, both tenant and actor settings were empty. |

**Checks run** —
- ✅ `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` — full workspace checks passed after the Testcontainers 0.28 update.
- ✅ `cargo test -p eq-core --test db` — all three named PostgreSQL 16 integration tests passed after the Testcontainers update.
- ✅ `cargo sqlx prepare --workspace` — passed with the local PostgreSQL 16 dev database and generated the committed `.sqlx/` metadata.
- ✅ `cargo audit` and `cargo deny check advisories` — passed after updating the test-only Testcontainers dependency; no audit findings remained.
- ✅ `scripts/check-deps.sh`.
- ✅ `scripts/tasks-index.sh --check` and `git diff --check`.
- ✅ `python3 scripts/erd.py --migrations` — passed; the bootstrap adds no tables, so 0 migrated tables were checked and 65 remain unmigrated.
- ✅ `python3 scripts/erd.py --check`.
- ✅ `gitleaks detect --redact --no-banner` — scanned 387 commits and the working tree; no leaks found.
- Not run: `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh` are not present yet; T-006 introduces these guards.
- Not run: `make dev` and `make e2e`; the Makefile/runtime harness is not present yet (T-006/T-008). PostgreSQL behavior was exercised by the integration tests above.
- Not run: `cargo run -p eq-server -- migrate`; the `migrate` CLI subcommand is added by T-004. The migration itself was run through `run_migrations` in the integration tests.
- Not run: web checks; T-003 changes no `web/` files and the web app is not present yet.
- Not run: hosted GitHub Actions; it cannot be run locally.

**Manual checks** — Ran the exact three integration scenarios against disposable `postgres:16-alpine` containers. The migration owner pool applies migrations; RLS visibility checks use a separately authenticated app pool with the required non-superuser role. An interactive `make dev`/HTTP smoke test is unavailable until the server CLI and dev Makefile tasks land.

**Debug log** —
- Initial test compilation could not resolve `sqlx::query!` with the repository's `SQLX_OFFLINE=true` setting and no cache. Generated metadata with `SQLX_OFFLINE=false` against the local PostgreSQL 16 database, then reran the full suite in offline mode successfully.
- Initial formatting and Clippy runs found formatting differences and test-only `expect`/needless `Ok(...?)`; formatted the task-owned code and changed tests to propagate errors, then reran the full Rust checks successfully.
- Testcontainers 0.23 pulled dependencies with RustSec advisories `RUSTSEC-2025-0111` and `RUSTSEC-2025-0134`. Updated the task-listed workspace pin to 0.28, which retains Testcontainers while making `cargo audit` and `cargo deny check advisories` pass; full Rust checks and all three integration tests passed afterward.

**Performance** — n/a; this task adds infrastructure and no list or lookup queries.

**Security** — Triggers T2 and T4 (tenant context, RLS and database roles). The migration creates `eq_app` without login, superuser or RLS bypass, and `eq_platform` without login; the application test role is explicitly `NOSUPERUSER NOBYPASSRLS`. RLS is forced, the helper binds tenant predicates to transaction-local context, SQL is parameterized/compile-checked, pool release runs `RESET ALL`, and the fake test password is not logged. Abuse coverage: `rls_scopes_rows` proves cross-tenant rows and unscoped reads are denied using the app role; `settings_do_not_leak` proves context does not survive transaction reuse. SBD-81 items not applicable to this infrastructure change: request auth/permissions, DTO bounds, HTML/CSV output and external calls. `gitleaks` found no secrets.

**Deviations** — RATIFY (see the matching entries under Questions): (1) `ActorCtx::system(kind)` keeps its signature but does not persist `kind`, because no field or schema column is defined; the system transaction uses the required empty actor id. (2) Added the task-needed manifest paths and moved the test-only Testcontainers pin from 0.23 to 0.28 to retain its required test framework while keeping security audits green. (3) The RLS probe uses a regular table in the disposable database, because PostgreSQL temporary tables are session-local and cannot be read by the separate app-role pool required by the task. Decision register references: TD-004, TD-005, TD-006 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).

**Dependencies affected** — T-007 can move the direct integration-test setup into `TestDb::app_pool`; T-101 and later core consumers can use `TenantId`, `ActorCtx`, `Tx`, `begin` and `begin_system`.

**Rollback plan** — Reverting this task removes the Rust helpers and migration file. The bootstrap is forward-only and creates no application tables or rows; extensions, roles, function and default privileges already applied to a persistent database are not destructively removed. Any future correction should use a forward migration.

**Follow-ups** — T-006 should stop the pre-existing manual `ritma-dev-db` container before enabling its `make dev-db` target on port 5433. T-007 should centralize the app-role test-pool setup. Deployment must grant login credentials to `eq_app`; the migration intentionally leaves it `NOLOGIN`.

**Step 5 — Implement** — Implemented only the task's pool, transaction, ID, actor and bootstrap migration scope, with the three named integration tests.

**Step 6 — Run locally** — Ran the three PostgreSQL integration scenarios using Testcontainers. `make dev`/`make e2e` are unavailable until the later dev and integration tasks add their harness.

**Step 7 — Debug loop** — Fixed the offline SQLx metadata setup, formatting, and test lint findings; resolved the dependency audit finding with the task-listed Testcontainers workspace pin. Final reruns passed (details above).

**Step 8 — Self code review** — Reviewed the task diff against CQ-16 §9, CA-01/30, ERR-10 and SBD-81. The RLS test uses the least-privileged app role, queries are parameterized, system actor context does not add an undefined schema field, and no unrelated application tables were introduced.

**Step 9 — Optimize** — n/a; no measured performance issue or query workload was introduced.

**Step 10 — Security audit** — Completed the applicable SBD-81 review and abuse checks listed above; secret scan passed.

**Step 11 — Revise and verify** — Reran full Rust checks, SQLx preparation, applicable dependency/index/diff/ERD guards, audit tools and secret scan after implementation fixes. Missing pre-T-006 guards and runtime targets are explicitly marked not run.

**Step 12 — Report, commit** — Report completed; task status is set to `review` and the index regenerated. Commit and push are recorded in the run log.

## Questions

- **Self-resolved (implementer, run 5, needs architect ratification):** `ActorCtx::system(kind)` requests a kind but the task's required minimal struct and the actor schema define no kind field → accept the signature but do not store or serialize the kind; set role to `System` and keep the transaction actor id empty because the task requires the empty system actor id and spec 06/29 define no kind column. [TD-004]
- **Self-resolved (implementer, run 5, needs architect ratification):** T-003 requires Testcontainers integration tests but omits their path and manifest from `## Files`; the pinned workspace Testcontainers 0.23 graph fails enabled security audits → add only the relevant `Cargo.toml`, `crates/eq-core/Cargo.toml` test-only workspace dependency and the already-listed `crates/eq-core/tests/db.rs`, and use Testcontainers 0.28 because its compatible current Docker client graph avoids those advisories while keeping the task's stated test framework and audit gates intact. [TD-005]
- **Self-resolved (implementer, run 5, needs architect ratification):** the test note calls the RLS probe a temporary table but requires reading it through a second `eq_app` connection → use a regular probe table inside each disposable Testcontainers database, because PostgreSQL temporary tables are visible only to their creating session and cannot be read by the separate app pool. [TD-006]
