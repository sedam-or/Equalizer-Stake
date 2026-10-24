---
id: T-007
title: Create eq-testkit (TestDb with template database, ManualClock re-export, API test client scaffold)
milestone: M0
status: todo
depends_on: [T-003]
spec_refs: [TST-03, TST-20]
estimated_size: M
---

## Goal
Integration tests across crates share one fast way to get an isolated, migrated Postgres database and a controllable clock.

## Context
TST-20: `TestDb::new()` creates a fresh DB from a **template** that already has migrations applied, so each test gets isolation in about 50 ms. There is one Postgres container per test binary (lazy static). `TestClock` starts at `2026-01-05T00:00:00+07:00`, a Monday. Seeds are deterministic.

## Scope
### In
- `eq_testkit::TestDb`:
  - A global `tokio::sync::OnceCell` that starts `testcontainers_modules::postgres::Postgres` (image `postgres:16-alpine`) once per process, runs migrations into the DB `eq_template`, then marks it `IS_TEMPLATE true`.
  - `new()` creates a random DB name with `CREATE DATABASE … TEMPLATE eq_template` and returns a pool plus the URL.
  - `Drop` doesn't need to drop the DB (the container dies at process end).
- If the env var `EQ_TEST_DATABASE_URL` is set, use that server instead of a container (faster local loops).
- `eq_testkit::clock::test_clock() -> ManualClock` at the canonical start instant.
- `eq_testkit::rng::seeded(u64)` for deterministic `StdRng`.
- `eq_testkit::api::TestApp` scaffold: builds `eq_api::router` with a `TestDb` and a `ManualClock`, wrapped in `axum_test::TestServer`. Auth helpers come later (T-117).

### Out
- Seed builders for domain entities (added by later tasks next to their modules).

## Files
| Path | Action |
|---|---|
| `crates/eq-testkit/src/{lib.rs,db.rs,clock.rs,rng.rs,api.rs}` | create |
| `crates/eq-core/tests/db.rs` | edit: switch to TestDb |

## Interfaces
```rust
pub struct TestDb { pub pool: sqlx::PgPool, pub url: String }
impl TestDb { pub async fn new() -> TestDb; }
pub fn test_clock() -> eq_core::clock::test::ManualClock;
pub struct TestApp { pub server: axum_test::TestServer, pub db: TestDb, pub clock: ManualClock }
impl TestApp { pub async fn new() -> TestApp; }
```

## Business rules
None.

## Implementation notes
- Template creation must be guarded against parallel tests (OnceCell). Concurrent `CREATE DATABASE … TEMPLATE` needs no connections to the template, so close the template pool after migrating.

## Acceptance criteria
- AC1. Two tests using `TestDb::new()` concurrently see independent data. → `eq-testkit/tests/isolation.rs::independent_dbs`
- AC2. With `EQ_TEST_DATABASE_URL` set, no container is started (manual check, note it in the Report).

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-testkit/tests/isolation.rs` | `independent_dbs`, `template_has_migrations` |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `test(testkit): template databases and test clock [T-007]`, signed off

## Report

## Questions
