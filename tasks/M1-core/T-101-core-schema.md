---
id: T-101
title: Add the core schema migrations (tenants, locations, resources, persons, memberships, links, modules, policies, closures, files, drafts, attention, consents, keys, idempotency)
milestone: M1
status: review
depends_on: [T-003, T-007]
spec_refs: [DOM-01, ARC-06, POL-03, SEC-02, API-08]
estimated_size: M
---

## Goal
The core tables exist exactly as specified, with RLS on every tenant table, and with Rust row structs and enums for them in `eq-core`.

## Context
- The normative DDL is in `spec/06-domain-model.md`:
  - §4.1: `tenants`, `locations`, `resources`, `persons`, `memberships`, `person_links`, `tenant_modules`, `closures`, `files`, `drafts`, `outbox`, `event_receipts`, `jobs`. **Leave `outbox`/`event_receipts`/`jobs` out of this task**; T-107/T-108 add them.
  - §2 of `spec/04-policy-engine.md`: `policies`.
  - §4.11: `attention_items`, `consents`, `tenant_keys`, `idempotency_keys`.
- **Copy the DDL verbatim.** Column names, types, checks and indexes are the contract for later tasks.
- **Addition to the spec DDL (approved):** `tenants` also gets `locale text NOT NULL DEFAULT 'en'`, `currency char(3) NOT NULL DEFAULT 'USD'` and `timezone text NOT NULL DEFAULT 'UTC'`. These are denormalised copies of `CorePolicy` for fast tenant resolution, kept in sync by T-113.
- RLS: call `SELECT eq_enable_rls('<table>');` (from T-003) for every table with `tenant_id`, **except** `tenants` and `tenant_keys`. Those get no RLS: `tenants` is read through `resolve_tenant`, and `tenant_keys` only through core crypto.
- `tenants` has no RLS. Add `CREATE FUNCTION resolve_tenant_by_slug(p_slug text) RETURNS TABLE(id uuid, slug text, status text) SECURITY DEFINER …` and `resolve_single_tenant()`, which returns the same columns for the only row or none (errors if more than one). Pin both functions' search path to `pg_catalog` and qualify the public table to prevent name shadowing (SBD-01).
- DOM-01: enums are `text` + CHECK. Rust maps them with `#[derive(sqlx::Type)] #[sqlx(type_name = "text", rename_all = "snake_case")]`.
- `policies` is append-only: `REVOKE UPDATE, DELETE ON policies FROM eq_app` (inside an exception block if the role is missing).

## Scope
### In
- `migrations/0002_core_tenancy.sql`: tenants, locations, resources, persons, memberships, person_links, tenant_modules, closures.
- `migrations/0003_core_policies_files.sql`: policies, files, drafts, attention_items, consents, tenant_keys, idempotency_keys.
- `eq_core::model::{Tenant, Location, Resource, Person (without decrypted fields), Membership, Role, PersonLink, Closure, FileRow, AttentionItem}` structs (`sqlx::FromRow`, serde) and enums `Role`, `TenantStatus`, `ResourceKind`, `MembershipStatus`, `FilePurpose`, `ClosureKind`, `Severity`.
- `eq_core::ids`: add `LocationId`, `ResourceId`, `FileId` newtypes.

### Out
- Repositories and services (later tasks create them where they're used).

## Files
| Path | Action |
|---|---|
| `scripts/erd.py` | update migration validation to enforce §7 table/column ownership |
| `scripts/tests/test_erd_map.py` | create migration-ownership regression tests |
| `migrations/0002_core_tenancy.sql`, `migrations/0003_core_policies_files.sql` | create |
| `crates/eq-core/src/model/*.rs`, `ids.rs` | create/edit |
| `crates/eq-core/tests/schema.rs` | create |

## Acceptance criteria
- AC1. Migrations apply on a fresh DB. → `eq-core/tests/schema.rs::migrations_apply`
- AC2. Every table with a `tenant_id` column except `tenant_keys` has `relrowsecurity` and `relforcerowsecurity` true (query `pg_class`). → `rls_enabled_on_tenant_tables`
- AC3. `person_links_one_primary`: inserting a second primary guardian for the same student fails. → `one_primary_guardian`
- AC4. `resolve_single_tenant()` errors when two tenants exist. → `single_tenant_guard`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/schema.rs` | `migrations_apply`, `rls_enabled_on_tenant_tables`, `one_primary_guardian`, `single_tenant_guard`, `policies_append_only_for_app_role` (connect as `eq_app` with a password set in the test, expect UPDATE to fail) |

## Definition of done
- [x] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass; DDL matches spec §4.1/§4.11 (reviewer diff)
- [x] Commit `feat(core): tenancy and policy schema [T-101]`, signed off

## Report

**Plan** — (1) Update `scripts/erd.py --migrations` to honor the exact table and column migration ownership in spec 29 §7, including a later `ALTER` column, with focused parser checks → verify the existing DDL is still checked and the future column is not demanded before `0012_ics.sql`. (2) Create `0002_core_tenancy.sql` by copying the eight tenancy tables from spec 06 §4.1, then add both tenant resolvers and forced RLS policies → verify against PostgreSQL 16 and the `eq_app` pool. (3) Create `0003_core_policies_files.sql` from spec 04 §2 and spec 06 §§4.1/4.11, including only the mapped tables, RLS and append-only privileges → verify migration drift and app-role denial. (4) Add the requested `eq_core::model` row structs/enums, three ID newtypes and five named database tests → verify migration behavior, constraints, tenant resolution and append-only access. (5) Run the ERD checks, Rust suite, guard scripts, local runtime and M1 Playwright E2E → record exact results before commit.

**Assumptions** — No business defaults or schema fields beyond the normative DDL are added. The explicit migration map in spec 29 §7 determines when a DDL column is required; the T-211-owned `persons.ics_secret_version` remains absent until `0012_ics.sql`. The two tenant resolver functions return `(id, slug, status)`; `resolve_single_tenant` uses PostgreSQL's standard cardinality violation when more than one tenant exists. The existing TestDb supplies an owner pool and a separately authenticated `eq_app` pool with fake credentials.

**Test plan** —
- Guard regression: Given `persons` is created by `0002_core_tenancy.sql` and `0012_ics.sql` is not yet present, when `scripts/erd.py --migrations` runs, then it validates the columns owned by `0002` without demanding the T-211 column; when `0012_ics.sql` is present, it requires that column in the mapped migration with the exact type, nullability and FK contract.
- AC1: Given a fresh TestDb template, when T-101 migrations are applied by the testkit, then all 15 mapped tables and migration versions 2 and 3 exist: `migrations_apply`.
- AC2: Given all T-101 tables with a `tenant_id`, when `pg_class` is queried, then each table except `tenant_keys` has RLS enabled and forced, and an app-role unscoped location read returns zero rows: `rls_enabled_on_tenant_tables`.
- AC3: Given an existing primary guardian link, when a second primary link is inserted for the same student, then PostgreSQL rejects it with the unique violation: `one_primary_guardian`.
- AC4: Given zero or one tenant, when the single-tenant resolver runs, then it returns no row or the tenant; given two tenants, it errors: `single_tenant_guard`.
- Additional DoD/security case: Given a policy row and an authenticated `eq_app` connection, when an update is attempted, then PostgreSQL denies it for lack of privilege: `policies_append_only_for_app_role`.
- Edges: zero/one/two tenants; paired and bounded geo coordinates; default versus explicit enum values; one versus duplicate primary guardians; all mapped tenant-table RLS flags; tenant key has no RLS.
- Errors: unique violation from duplicate primary guardian and PostgreSQL cardinality violation from multiple rows; permission denied for policy mutation. No new HTTP error code is introduced.
- Abuse: verify unscoped app-role access cannot read a seeded location through RLS; verify app-role update of append-only policy data is denied; verify tenant lookup works through the SECURITY DEFINER resolver as `eq_app`.

**Summary** — Added the core tenancy and policy schema in the migrations assigned by ERD-06, with forced RLS on tenant tables, append-only policy permissions, and security-definer tenant lookup functions. Added typed Rust row models and PostgreSQL tests for migration application, RLS, guardian uniqueness, tenant resolution, and policy immutability. Updated the ERD migration guard to enforce the §7 table/column map, which lets the T-211-owned `persons.ics_secret_version` remain in its later migration.

**Spec read** — `spec/04-policy-engine.md` §2; `spec/06-domain-model.md` §§1, 4.1, 4.11–4.12; `spec/10-security-privacy.md` §§1–5; `spec/16-code-quality.md` §§4, 9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§1, 4–5, 8; `spec/19-error-handling.md` §§1–5; `spec/29-erd.md` §§1, 3.1, 4–5, 7; `WORKFLOW.md` §§3–4.

**Files** — Created `migrations/0002_core_tenancy.sql`, `migrations/0003_core_policies_files.sql`, `crates/eq-core/src/model.rs`, `crates/eq-core/tests/schema.rs`, and `scripts/tests/test_erd_map.py`. Updated `crates/eq-core/src/{ids.rs,lib.rs}`, `scripts/erd.py`, this task report and `tasks/INDEX.md`.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| ERD-06 | `scripts.tests.test_erd_map` (2 cases) | Passed; all 65 DDL tables have a mapped migration and `persons.ics_secret_version` belongs to `0012_ics.sql`. |
| AC1 | `migrations_apply` | Passed against a fresh PostgreSQL 16 TestDb; all 15 mapped tables exist and the future ICS column is absent. |
| AC2 | `rls_enabled_on_tenant_tables` | Passed; all 13 tenant tables have RLS enabled and forced; the app role sees no unscoped location; `tenants` and `tenant_keys` have no RLS. |
| AC3 | `one_primary_guardian` | Passed; the second primary guardian insert returns PostgreSQL unique violation `23505`. |
| AC4 | `single_tenant_guard` | Passed; zero/one tenants resolve, multiple tenants return cardinality violation `21000`, and `eq_app` cannot read `tenants` directly. |
| DoD | `policies_append_only_for_app_role` | Passed; the authenticated app role's update is rejected with permission denied `42501`. |

**Checks run** —
- ✅ `python3 scripts/erd.py --migrations` — 15 migrated tables matched the DDL and ERD-06; 50 not-yet-migrated tables are reported as pending.
- ✅ `python3 scripts/erd.py --check`.
- ✅ `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts.tests.test_erd_map` — 2 passed.
- ✅ `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` — full workspace passed, including all DB-backed tests.
- ✅ `cargo test -p eq-core --test schema -- --nocapture` — all 5 schema tests passed against PostgreSQL 16.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `cargo deny check` — advisories, bans, licenses and sources passed; it printed non-failing duplicate-crate and unused-allowance warnings.
- ✅ `cargo audit` — no vulnerabilities reported.
- ✅ `docker run --rm postgres:16-alpine postgres --version` — PostgreSQL 16.15.
- ✅ `make e2e` — release server, built web app, Chromium and WebKit mobile; 6/6 browser cases passed.
- ✅ `scripts/tasks-index.sh --check` and `git diff --check` (rerun after setting `review`).
- ❌ `gitleaks detect --redact --no-banner` — one pre-existing false positive in `Makefile:80` from the Docker `--user` flag in the E2E browser container command; verified it contains no credential and did not suppress or alter the scanner.
- n/a `cargo sqlx prepare --workspace` — no `sqlx::query!` macro query or SQLx offline metadata changed.
- n/a `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check` — no web source changed; `make e2e` built the web app and exercised both browser projects.
- n/a `make dev` — no HTTP route or request behavior was added; the PostgreSQL integration tests exercised schema behavior and `make e2e` sent real readiness requests to the release server.
- Not run: hosted GitHub Actions; requires the hosted CI environment.

**Manual checks** — `TestDb` applied the migrations to fresh PostgreSQL 16 databases. The `eq_app` pool verified unscoped RLS denial, resolver-only tenant access, and append-only policy permissions. `make e2e` built the release server and web bundle and passed the live readiness, home page and SPA fallback cases in Chromium and WebKit mobile.

**Debug log** — Initial guard behavior demanded `persons.ics_secret_version` as part of T-101 even though ERD-06 assigns it to T-211. Rather than move or duplicate the column, updated the guard to honor the existing table/column map and added a regression test. The first Rust compile also showed `PgRange<NaiveDate>` lacks serde implementations; added a lossless serde adapter for the bound endpoints, then reran `cargo check`, focused schema tests and the full Rust suite successfully. PostgreSQL-backed suite startup took several minutes in this environment but completed green.

**Performance** — n/a; no implementation or query was retained.

**Security** — Triggers T2 (tenant resolution/RLS), T3 (S2 person data and S3 tenant keys) and T4 (database roles/privileges). RLS is enabled and forced on all 13 tenant tables; row-isolation checks run only on the `eq_app` pool (`NOSUPERUSER NOBYPASSRLS`). The two `SECURITY DEFINER` resolvers use `search_path = pg_catalog`, qualify `public.tenants`, and are executable only by `eq_app`; direct app-role table access is revoked. `tenant_keys` intentionally has no RLS per SEC-02. `policies` denies app-role UPDATE/DELETE. Tests cover unscoped reads, resolver access, guardian uniqueness and append-only denial. No decrypted person fields are present in `Person`. Gitleaks found only the pre-existing false positive listed above; no secret was added.

**Deviations** — RATIFY: the three mechanical task clarifications recorded in Questions remain for architect review: add the explicitly required schema-test path to Files; use the slug resolver's `(id, slug, status)` output for `resolve_single_tenant`; and pin SECURITY DEFINER resolver `search_path` while qualifying `public.tenants`. The operator-authorized guard correction follows the explicit ERD-06 mapping; it changes no schema ownership or DDL and is not a schema exception. Decision register references: TD-021, TD-022, TD-023 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).

**Dependencies affected** — T-103, T-104 and later tasks may now consume the core schema/model types. No schema field or migration number was added beyond ERD-06.

**Rollback plan** — Revert the signed T-101 commit. Migrations are forward-only; no compensating migration is needed before deployment because this task has not been applied to a persistent production database. Any deployed correction must use a forward migration.

**Follow-ups** — Review the three RATIFY items above. The full `gitleaks` scan has a known false positive at `Makefile:80` for the non-secret Docker `--user` argument; a maintainer should decide whether to change that command or record a narrowly scoped false-positive exception without weakening secret detection. Name validation maps the former product display name to **Ritma**; no active product-facing label uses the old name. The post-report audit found 18 occurrences across 11 files: six live GitHub slug URLs in `Cargo.toml`, `BRANCHING.md` and both README badges; six historical/provenance references in `brand/` and `spec/00-index.md`; and six task-report mentions, including this report. The current remote is `sedam-or/Equalizer-Stake`; `sedam-or/ritma` does not exist, and `brand/GITHUB.md` describes the planned transfer to `ritmahq/ritma`. Keep the current operational URLs valid until the repository is transferred/renamed, then update the metadata, branch helper and badges. Historical references stay as provenance. The repository description still uses the former display name; repository settings were not changed.

**Step 0 — Start** — Started from a clean `milestone/m1-core` branch at `f654774`; `scripts/tasks-index.sh --check` passed. T-003 and T-007 were both `review`, making T-101 eligible. The prior blocked commit was preserved; the operator authorized relevant blocker fixes.

**Step 1 — Task and spec** — Read the full T-101 contract and spec 04 §2, spec 06 §§1/4.1/4.11/4.12, spec 16 §12, spec 17 §§1/4, spec 18 §§4/5/8, spec 19 §10 and spec 29 §§1/3/4/5/6/7.

**Step 2 — Dependencies** — Read the full T-003 and T-007 Reports, including RATIFY items and Follow-ups, plus their shipped `eq-core` database/ID code, bootstrap migration and TestDb owner/app pool. The app role is authenticated with `NOSUPERUSER NOBYPASSRLS`; the bootstrap supplies default privileges and `eq_enable_rls`.

**Step 3 — Design reference** — n/a; this is a backend schema task.

**Step 4 — Plan and test plan** — Recorded above before implementation, including the guard regression, every AC, edge and abuse cases.

**Step 5 — Implement** — Added the migration-map-aware ERD guard, two migrations, core row models and ID wrappers, and all five required PostgreSQL tests.

**Step 6 — Run locally** — Applied migrations through the PostgreSQL 16 testkit; exercised RLS and privileges with real app-role queries. `make e2e` built the local release app and passed live HTTP/browser smoke tests.

**Step 7 — Debug loop** — Fixed the single serde compile error described above, then reran compile, focused PostgreSQL tests, full Rust checks and E2E successfully. No unresolved runtime failure remains.

**Step 8 — Self code review** — Reviewed the diff against CQ-16 §9, CA-01/30, ERR-10 and SBD-81. Verified the schema matches the DDL/map, no cross-module table was added, no `#[allow]` or decrypted person data was introduced, and app-role isolation tests use the non-superuser pool.

**Step 9 — Optimize** — n/a; no application list/lookup query was added or performance target introduced.

**Step 10 — Security audit** — Completed the applicable SBD-81 review and abuse tests summarized under Security. The secret scan's sole match is a verified pre-existing false positive.

**Step 11 — Revise and verify** — Reran the full Rust suite, all guard scripts, both ERD checks, the migration-owner regression, cargo audit/deny, task index, diff whitespace and E2E. Exact commands and the gitleaks exception are listed above.

**Step 12 — Report, commit** — Report completed, status set to `review`, and index regenerated. The signed-off task commit is pushed on `milestone/m1-core`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** The Tests section requires `crates/eq-core/tests/schema.rs`, but the Files table omitted it → add that exact test path because the named acceptance tests cannot be implemented without it. [TD-021]
- **Self-resolved (implementer, run 6, needs architect ratification):** `resolve_single_tenant()` had no declared return shape while the slug resolver returns `(id, slug, status)` → use the same row shape so the tenant resolver contract is consistent and can expose the status required by tenant resolution. [TD-022]
- **Self-resolved (implementer, run 6, needs architect ratification):** The SECURITY DEFINER resolver's search path was unspecified → pin it to `pg_catalog` and qualify `public.tenants`, because SBD-01 requires fail-closed security and prevents a shadow object from changing what the definer reads. [TD-023]
- **Resolved (operator-authorized implementation, 2026-10-25):** Q1 — Preserve the explicit T-211 mapping and update `scripts/erd.py --migrations` to honor the §7 table/column ownership map; the operator authorized relevant blocker fixes, and ERD-06 specifies the answer.
- **Blocker Q1 (history):** `scripts/erd.py --migrations` requires `persons.ics_secret_version` in a migration now, but spec 06 §4.12 and ERD-06 assign it to T-211's `0012_ics.sql`. Should the checker be updated to account for later mapped column migrations, or should the column be reassigned to T-101 and the T-211 contract changed? **Historical recommendation:** preserve the explicit spec/task ownership and align the guard with it.
