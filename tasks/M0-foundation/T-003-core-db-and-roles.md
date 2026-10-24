---
id: T-003
title: Add the DB pool, tenant-scoped transaction helper and bootstrap migration (extensions, roles, RLS helper)
milestone: M0
status: todo
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
  - `Tx` newtype over `sqlx::Transaction<'static, Postgres>` with `Deref`/`DerefMut`, and a `commit()` method.
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
- In dev and docker the app and owner may be the same DB user. RLS is still enforced because it's forced. Document this in the Report.
- `after_release`: `Box::pin(async move { sqlx::query("RESET ALL").execute(&mut *conn).await.map(|_| true).or(Ok(false)) })`. Returning `false` discards the connection.
- Integration tests need Postgres. Use `testcontainers` directly here (eq-testkit arrives in T-007).

## Acceptance criteria
- AC1. Migrations apply twice without error (idempotent bootstrap). → `db::tests::bootstrap_idempotent`
- AC2. Given a temp table with RLS enabled through `eq_enable_rls` and rows for tenants A and B, When `begin(A)` selects, Then only A's rows are visible. When it selects without `begin` (plain pool connection), Then 0 rows. → `db::tests::rls_scopes_rows`
- AC3. After a transaction is committed and the connection reused from the pool, `current_setting('app.tenant_id', true)` is empty. → `db::tests::settings_do_not_leak`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/db.rs` | `bootstrap_idempotent`, `rls_scopes_rows`, `settings_do_not_leak` |

## Definition of done
- [ ] Rust checks pass (tests need Docker running)
- [ ] `cargo sqlx prepare --workspace` run (if any `query!` used)
- [ ] Commit `feat(core): db pool, tenant tx and bootstrap migration [T-003]`, signed off

## Report

## Questions
