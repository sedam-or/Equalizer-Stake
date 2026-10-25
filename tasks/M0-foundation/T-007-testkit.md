---
id: T-007
title: Create eq-testkit (TestDb with template database, ManualClock re-export, API test client scaffold)
milestone: M0
status: review
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
  - A global `tokio::sync::OnceCell` that starts `testcontainers::GenericImage` for `postgres:16-alpine` once per process, runs migrations into the DB `eq_template`, then marks it `IS_TEMPLATE true`.
  - After migrating the template, run `ALTER ROLE eq_app LOGIN PASSWORD 'test-secret-app'` (fake value; roles are cluster-wide, so once is enough).
  - `new()` creates a random DB name with `CREATE DATABASE … TEMPLATE eq_template` and returns **two pools**: `pool` (the container superuser, for seeding and DDL) and `app_pool` (logged in as `eq_app`, `NOSUPERUSER NOBYPASSRLS`). **Code under test, the router and every RLS or privilege assertion must use `app_pool`**; a test that checks isolation on the superuser pool is invalid because superusers ignore RLS.
  - `Drop` doesn't need to drop the DB (the container dies at process end).
- If the env var `RITMA_TEST_DATABASE_URL` is set (a superuser URL), use that server instead of a container (faster local loops); the `eq_app` login is derived from its host and port.
- `eq_testkit::clock::test_clock() -> ManualClock` at the canonical start instant.
- `eq_testkit::rng::seeded(u64)` for deterministic `StdRng`.
- `eq_testkit::api::TestApp` scaffold: builds `eq_api::router` with a `TestDb` (using its `app_pool`) and a `ManualClock`, wrapped in `axum_test::TestServer`. Auth helpers come later (T-117).

### Out
- Seed builders for domain entities (added by later tasks next to their modules).

## Files
| Path | Action |
|---|---|
| `crates/eq-testkit/src/{lib.rs,db.rs,clock.rs,rng.rs,api.rs}` | create |
| `crates/eq-testkit/Cargo.toml`, `crates/eq-testkit/tests/isolation.rs` | modify/create |
| `Cargo.toml`, `crates/eq-core/Cargo.toml` | modify |
| `crates/eq-core/tests/db.rs` | edit: switch to TestDb |

## Interfaces
```rust
pub struct TestDb { pub pool: sqlx::PgPool /* superuser: seed + DDL only */, pub url: secrecy::SecretString, pub app_pool: sqlx::PgPool /* eq_app: use for everything under test */, pub app_url: secrecy::SecretString }
impl TestDb { pub async fn new() -> TestDb; }
pub fn test_clock() -> eq_core::clock::test::ManualClock;
pub struct TestApp { pub server: axum_test::TestServer, pub db: TestDb, pub clock: ManualClock } // the router is built on db.app_pool; hand-written redacted Debug
impl TestApp { pub async fn new() -> TestApp; }
```

`TestDb` also has a hand-written redacting `Debug`; it does not derive `Clone`, `Serialize` or `Debug`. `TestApp` must not derive `Clone` or `Serialize`; its hand-written `Debug` must not expose its server or database URL.

## Business rules
None.

## Implementation notes
- Template creation must be guarded against parallel tests (OnceCell). Concurrent `CREATE DATABASE … TEMPLATE` needs no connections to the template, so close the template pool after migrating.

## Acceptance criteria
- AC1. Two tests using `TestDb::new()` concurrently see independent data. → `eq-testkit/tests/isolation.rs::independent_dbs`
- AC2b. `app_pool` runs as a role with `rolsuper = false` and `rolbypassrls = false`, and with `app.tenant_id` unset a tenant table (created by the test as owner, RLS via `eq_enable_rls`) returns 0 rows on `app_pool` but all rows on `pool`. → `eq-testkit/tests/isolation.rs::app_pool_enforces_rls`
- AC2. With `RITMA_TEST_DATABASE_URL` set, no container is started (manual check, note it in the Report).

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-testkit/tests/isolation.rs` | `independent_dbs`, `template_has_migrations`, `app_pool_enforces_rls`, `debug_redacts_database_urls`, `test_app_uses_app_pool` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass
- [x] Commit `test(testkit): template databases and test clock [T-007]`, signed off

## Report

**Plan** — (1) Add only the required workspace/testkit dependencies and listed test paths, preserving the Testcontainers 0.28 workspace pin and using its `GenericImage` API for `postgres:16-alpine` → verify dependency resolution and audit. (2) Implement one async `OnceCell` template initializer that selects either the container or explicit `RITMA_TEST_DATABASE_URL`, applies migrations, sets the fake app-role password, and disconnects from the template before marking it as a template database → verify migration/template tests. (3) Create per-test database clones with separate superuser and `eq_app` pools, keeping credential-bearing URLs secret and adding redacting `Debug` implementations → verify independent database, RLS, and redaction tests. (4) Add the manual clock, seeded RNG and TestApp router scaffold; compile the same TestDb source directly into eq-core's integration target to preserve the no-`eq-*` dependency guard → verify all named tests and `scripts/check-deps.sh`. (5) Run the listed tests, including a no-container external-DB run, then full Rust, web, guard, audit, ERD/index and secret checks.

**Assumptions** — Test role and database credentials are obvious fake values and never logged; database URLs are credentials and follow SBD-50. The fixed template name and canonical Monday start instant come from the task. `GenericImage::new("postgres", "16-alpine")` preserves the required image and uses the workspace's audit-clean Testcontainers 0.28 API.

**Test plan** —
- AC1: Given two concurrent `TestDb::new()` calls, when one database gets a probe table and row, then the other database has no such table → `independent_dbs`.
- AC2b: Given an owner-created RLS table and an unset tenant context, when `app_pool` and the superuser pool query it, then `eq_app` sees zero rows, the superuser sees both rows, and `eq_app` is not superuser or RLS-bypass → `app_pool_enforces_rls`.
- AC2: Given `RITMA_TEST_DATABASE_URL` points to an external PostgreSQL server, when the integration tests run, then they pass without adding a Docker container → manual external-database run with container-ID comparison.
- Template/secret/API cases: the migrated clone contains SQLx migration records → `template_has_migrations`; formatting TestDb and TestApp does not expose fake credentials or URLs → `debug_redacts_database_urls` and `test_app_uses_app_pool`; a readiness request through TestApp succeeds → `test_app_uses_app_pool`.
- Existing core cases remain exact: rerunning migrations is safe → `bootstrap_idempotent`; scoped/unscoped tenant reads follow RLS → `rls_scopes_rows`; transaction settings do not leak to a reused pool connection → `settings_do_not_leak`.
- Edges/errors: concurrent database clones; template initialization failure is reported without database credentials; the external URL must be valid and privileged enough to create the template and role.
- Abuse cases: a superuser-only RLS test is invalid, so all isolation assertions run on `app_pool`; fake credentials must not appear in Debug output.

**Step 0 — Start** — On `milestone/m0-foundation` after T-006 was pushed; the tree was clean, `scripts/tasks-index.sh --check` passed, and T-007 is the lowest eligible task. T-003 is `review`, which counts as complete under the unattended run ordering. Claimed T-007 as `in_progress` and regenerated `tasks/INDEX.md`.

**Step 1 — Task/spec** — Read all of T-007 and its inlined TST-03/TST-20 requirements. Consulted `spec/12-testing.md` §§1–3, `spec/16-code-quality.md` §§6 and 12, `spec/18-security-by-design.md` §§5.3/SBD-50–51 and §8/SBD-81, plus `AGENTS.md` and the Rust dependency map in `spec/24-open-source-stack.md`.

**Step 2 — Dependencies** — Read T-003's complete Report, especially its three RATIFY decisions and the follow-up to centralize the app-role test pool. Inspected its shipped `eq_core::db::{run_migrations,connect,begin}`, bootstrap migration's `eq_enable_rls` helper and `eq-core/tests/db.rs`; TestDb uses the existing migration API, resets app-pool session state on release, keeps RLS assertions on `app_pool`, and preserves the existing test names. The dependency guard forbids all `eq-core -> eq-*` package edges, so the core integration target compiles the shared `eq-testkit/src/db.rs` source directly rather than adding a forbidden crate edge. T-005/T-006 were checked for the current API router/runtime and check commands as recorded in T-003's transitive history.

**Step 3 — Design reference** — n/a; this task adds Rust test infrastructure and no user interface.

**Step 4 — Plan and test plan** — Recorded above before implementation. Three mechanical conflicts are self-resolved: secret URLs per SBD-50, required paths omitted from Files, and the task's module-crate API conflicting with the dependency pin established by T-003. Each is listed under Questions and appears as a RATIFY item in Deviations.

**Summary** — Added a shared testkit with a migrated PostgreSQL template, isolated owner/app pools, deterministic clock/RNG helpers, and an API TestApp backed by the non-superuser pool. The new RLS, template, independence, redaction, and readiness tests pass, and the existing core database tests now use the same TestDb implementation.

**Spec read** — `spec/12-testing.md` (TST-03/20); `spec/16-code-quality.md` (CQ-80/81, §12); `spec/18-security-by-design.md` (SBD-50/51, §8/SBD-81); `spec/24-open-source-stack.md` (workspace dependency map). T-003 shipped code and report as detailed in Step 2.

**Files** —
- Modified `Cargo.toml` and `crates/eq-core/Cargo.toml` for the `eq-testkit` workspace package and the Testcontainers test dependency.
- Created `crates/eq-testkit/Cargo.toml`, `src/{lib.rs,db.rs,clock.rs,rng.rs,api.rs}` and `tests/isolation.rs`.
- Updated `crates/eq-core/tests/db.rs` to use the shared TestDb source while preserving all three test names; `Cargo.lock` now includes the resolved TestApp dependencies.
- Updated this task's scope/report and regenerated `tasks/INDEX.md` through `scripts/tasks-index.sh`.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1 | `eq-testkit/tests/isolation.rs::independent_dbs` | Passed; two concurrent clones were isolated. |
| AC2b | `eq-testkit/tests/isolation.rs::app_pool_enforces_rls` | Passed; `eq_app` was LOGIN/NOSUPERUSER/NOBYPASSRLS, saw zero unscoped rows, and the owner saw both. |
| AC2 | Manual external `RITMA_TEST_DATABASE_URL` run with Docker-container before/after comparison | Passed; all five integration tests passed and container IDs did not change. |

**Checks run** —
- ✅ `cargo fmt --all --check`.
- ✅ `cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `cargo test --workspace` — all workspace tests passed, including 5 testkit integration tests and 3 core DB integration tests.
- ✅ `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh`.
- ✅ `scripts/tasks-index.sh --check`, `python3 scripts/erd.py --check`, and `git diff --check`.
- ✅ `cargo audit`, `cargo deny check advisories`, `cargo machete`, and `gitleaks detect --redact --no-banner`.
- ✅ `cargo test -p eq-testkit --test isolation` with Testcontainers and with `RITMA_TEST_DATABASE_URL` pointing at an external PostgreSQL 16 server; external mode kept Docker container IDs unchanged.
- n/a: `python3 scripts/erd.py --migrations`; this task adds no migration.
- n/a: `cargo sqlx prepare --workspace`; no `sqlx::query!` query or migration changed.
- n/a: web lint/typecheck/test/i18n checks; this task changes no web files.
- Not run: hosted GitHub Actions; it cannot be launched from this local task run.

**Manual checks** — `test_app_uses_app_pool` sent a real `/readyz` request through `axum_test::TestServer` backed by `TestDb::app_pool`. Repeated all five isolation tests against a disposable external PostgreSQL 16 server set through `RITMA_TEST_DATABASE_URL`; the container IDs before and after matched. Removed the disposable containers created by the local Testcontainers runs after verification.

**Debug log** —
- The initial `testcontainers-modules` 0.11 resolution conflicted with T-003's Testcontainers 0.28 pin; the newest module line also selected an exact `bollard-stubs` version incompatible with 0.28. Switched the task's minimum interface to Testcontainers 0.28 `GenericImage` while keeping the exact `postgres:16-alpine` image.
- SQLx 0.9 rejected generated DDL strings without an explicit safety assertion. The `ALTER ROLE` statement now uses only fixed role/password constants, and the generated database identifier uses a fixed prefix plus UUID hex; both are wrapped in `AssertSqlSafe` with comments explaining the constrained input.
- Initial compile/lint findings (missing `ImageExt` import, host type conversion, `manual_let_else`, and UUID v4 unavailable under workspace features) were fixed by importing the trait, stringifying the validated host, using let-else, and using the workspace-supported UUID v7 generator.
- `scripts/check-deps.sh` first rejected a direct eq-core dev dependency on eq-testkit. Removed that edge and compiled the shared database helper source into the eq-core integration target, so the shipped guard remains unchanged and passes.
- The first short external-Postgres readiness window expired during initdb; the retry used a longer allowance and then passed all tests.

**Performance** — No application query path was added or optimized. The test suite confirms template clones work; no standalone clone-latency benchmark was taken.

**Security** — Triggers T2 and T4 (database credentials and tenant isolation). Credential URLs use `SecretString`; TestDb and TestApp have hand-written redacting Debug implementations and do not derive Clone/Serialize. The owner/superuser pool is only for setup and seeding; the router and RLS tests use `app_pool`; the tests verify the app role is NOSUPERUSER/NOBYPASSRLS. Bound SQL values are parameterized; the only dynamic SQL is fixed-role/fake-password DDL and a UUID-derived identifier, each explicitly audited with `AssertSqlSafe`. Abuse tests verify unscoped RLS denial and secret-free Debug output. The final gitleaks scan passed.

**Deviations** — RATIFY items are recorded in Questions: redact credential-bearing URL fields with `SecretString`/manual Debug per SBD-50; add directly required manifests and the test file omitted from Files but referenced by Tests/Interfaces; use Testcontainers 0.28 `GenericImage` for the specified PostgreSQL 16 image because the task's `testcontainers-modules` 0.11 pin conflicts with T-003's audit-driven 0.28 pin, and the newest module release also requires an incompatible Bollard stub version.

**Dependencies affected** — T-101 and later tests can use TestDb's app-role pool; T-117 can use the TestApp router scaffold. No domain behavior is changed.

**Rollback plan** — Revert the task commit; no production schema or database is modified. Test-created database clones and containers are disposable; the local containers created during verification were removed manually.

**Follow-ups** —
- Testcontainers is stored in a process-global OnceCell, so the async container handle is not dropped on ordinary Rust process exit; the local verification containers remained running until removed manually. Confirm whether CI runner teardown is the intended cleanup boundary or add a supported process-shutdown cleanup path in a separate task.
- Ritma naming validation: canonical product display name is **Ritma**, mapped from the former project/capstone name `Equalizer-Stake`. A case-insensitive repository scan found 14 remaining occurrences across 9 files: six active GitHub repository-slug references in Cargo metadata, BRANCHING.md and the English/Indonesian README badges; six capstone/provenance references in brand/spec records; one earlier T-006 mapping and this report's mapping. `git remote -v` confirms the active slug is still `sedam-or/Equalizer-Stake`, so changing those URLs now would break links; the provenance references explicitly describe the former name. No active product label uses the old display name. Re-scan with `rg -n -i 'equalizer[- _]stake'` after the eventual repository transfer and update only the live slug references then. T-006 already records the GitHub description follow-up.

**Step 5 — Implement** — Added `eq-testkit` with one `OnceCell` template initializer, per-test UUID-named cloned databases, redacted URL storage, the owner/app pool split, deterministic clock/RNG helpers, and the TestApp router wrapper. Updated core integration tests while keeping their exact names and enforcing RLS through the app pool.

**Step 6 — Run locally** — The TestApp test issued a real readiness request through `TestServer`; both the normal Testcontainers path and the external-database override passed all listed tests. `make dev` is not needed for this test-only infrastructure task and no web files changed.

**Step 7 — Debug loop** — Resolved the dependency conflict and compile/lint findings listed in Debug log. The external Postgres readiness retry used a longer startup allowance after the first bounded attempt ended during initdb. Final targeted and workspace runs are green.

**Step 8 — Self code review** — Reviewed the task diff against `spec/16-code-quality.md` §9, `spec/17-clean-architecture.md` anti-patterns, `spec/19-error-handling.md`, and SBD-81. Confirmed no `eq-core -> eq-*` dependency edge, no `#[allow]`, no production schema changes, app-pool-only RLS checks, redacted secret-bearing structs, and fixed/UUID-derived dynamic DDL inputs.

**Step 9 — Optimize** — n/a; no application query workload or performance issue was introduced or measured.

**Step 10 — Security audit** — Completed the applicable SBD-81 review and abuse cases described above; the final gitleaks scan passed.

**Step 11 — Revise and verify** — Reran full formatting, Clippy, workspace tests, all four guard scripts, task index/ERD checks, Cargo dependency audits, and secret scanning after the final code changes. No migration, SQLx macro, or web check applies.

**Step 12 — Report, commit** — Report completed; status set to `review` and `tasks/INDEX.md` regenerated. A signed-off task commit and branch push follow this report.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** The interface sketch stores credential-bearing PostgreSQL URLs as public `String` fields and offers no redaction contract → use `secrecy::SecretString`, hand-written redacting `Debug`, and no `Clone`/`Serialize`/derived `Debug` for secret-bearing structs; test that formatting leaks neither fake credential, because SBD-50 governs secret storage.
- **Self-resolved (implementer, run 6, needs architect ratification):** The task's Tests section requires `crates/eq-testkit/tests/isolation.rs`, the implementation depends on Cargo manifests, but Files omits those paths → add only the directly required workspace/testkit manifests and test file to Files; this follows the test/interface contract and keeps the manifest edits to dependencies needed for the listed code.
- **Self-resolved (implementer, run 6, needs architect ratification):** The task's container interface specifies `testcontainers-modules` 0.11, which requires the incompatible Testcontainers 0.23 API, while T-003 pins Testcontainers 0.28 to keep enabled dependency audits clean; the newest module release also selects an incompatible exact `bollard-stubs` version → use Testcontainers 0.28 `GenericImage` with the exact `postgres:16-alpine` image because TST-03 requires PostgreSQL 16 through Testcontainers and the workspace pin is the established dependency contract.
