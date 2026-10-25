---
id: T-101
title: Add the core schema migrations (tenants, locations, resources, persons, memberships, links, modules, policies, closures, files, drafts, attention, consents, keys, idempotency)
milestone: M1
status: blocked
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
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; DDL matches spec §4.1/§4.11 (reviewer diff)
- [ ] Commit `feat(core): tenancy and policy schema [T-101]`, signed off

## Report

**Plan** — (1) Create `0002_core_tenancy.sql` by copying the eight tenancy tables from spec 06 §4.1, then add the two resolver functions and forced RLS policies exactly where required → verify the migration contents and resolver behavior against PostgreSQL 16. (2) Create `0003_core_policies_files.sql` from spec 04 §2 and spec 06 §§4.1/4.11, including only the listed tables, RLS and append-only policy privileges → verify schema drift and the app-role mutation denial. (3) Add the requested `eq_core::model` row structs/enums and the three ID newtypes → verify formatting, Clippy and workspace compilation. (4) Add the five named database tests using the existing isolated TestDb and its `eq_app` pool → verify migrations, RLS, the primary guardian constraint, single-tenant resolution and append-only access. (5) Run the migration guard, full Rust suite, guard scripts, dev runtime and M0 Playwright E2E against the migrated server → record exact results before commit.

**Assumptions** — No business defaults or schema fields beyond the normative DDL are added. The two tenant resolver functions return `(id, slug, status)`; `resolve_single_tenant` uses PostgreSQL's standard cardinality violation when more than one tenant exists. The existing TestDb supplies an owner pool and a separately authenticated `eq_app` pool with fake credentials.

**Test plan** —
- AC1: Given a fresh TestDb template, when T-101 migrations are applied by the testkit, then all 15 mapped tables and migration versions 2 and 3 exist: `migrations_apply`.
- AC2: Given all T-101 tables with a `tenant_id`, when `pg_class` is queried, then each table except `tenant_keys` has RLS enabled and forced, and an app-role unscoped location read returns zero rows: `rls_enabled_on_tenant_tables`.
- AC3: Given an existing primary guardian link, when a second primary link is inserted for the same student, then PostgreSQL rejects it with the unique violation: `one_primary_guardian`.
- AC4: Given zero or one tenant, when the single-tenant resolver runs, then it returns no row or the tenant; given two tenants, it errors: `single_tenant_guard`.
- Additional DoD/security case: Given a policy row and an authenticated `eq_app` connection, when an update is attempted, then PostgreSQL denies it for lack of privilege: `policies_append_only_for_app_role`.
- Edges: zero/one/two tenants; paired and bounded geo coordinates; default versus explicit enum values; one versus duplicate primary guardians; all mapped tenant-table RLS flags; tenant key has no RLS.
- Errors: unique violation from duplicate primary guardian and PostgreSQL cardinality violation from multiple rows; permission denied for policy mutation. No new HTTP error code is introduced.
- Abuse: verify unscoped app-role access cannot read a seeded location through RLS; verify app-role update of append-only policy data is denied; verify tenant lookup works through the SECURITY DEFINER resolver as `eq_app`.

**Summary** — Blocked before implementation. `scripts/erd.py --migrations` rejects the currently correct T-101 table set because it expects `persons.ics_secret_version` before the migration assigned to T-211 exists. The schema remains unchanged; the ERD map and migration ownership need an architect-approved resolution before this task can pass its required check.

**Spec read** — `spec/04-policy-engine.md` §2; `spec/06-domain-model.md` §§1, 4.1, 4.11–4.12; `spec/10-security-privacy.md` §§1–5; `spec/16-code-quality.md` §§4, 9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§1, 4–5, 8; `spec/19-error-handling.md` §§1–5; `spec/29-erd.md` §§1, 3.1, 4–5, 7; `WORKFLOW.md` §§3–4.

**Files** — Updated this task's Files, Plan, Questions and Report; regenerated `tasks/INDEX.md`. The trial migration/model/test changes were removed after the required migration guard exposed the ownership contradiction. No schema or source implementation remains in this task commit.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `migrations_apply` | Not run: implementation is blocked; migrations cannot pass the required ERD guard under the current migration map. |
| AC2 | `rls_enabled_on_tenant_tables` | Not run: implementation is blocked. |
| AC3 | `one_primary_guardian` | Not run: implementation is blocked. |
| AC4 | `single_tenant_guard` | Not run: implementation is blocked. |
| DoD | `policies_append_only_for_app_role` | Not run: implementation is blocked. |

**Checks run** —
- ❌ `python3 scripts/erd.py --migrations` — reports `persons.ics_secret_version: missing in migrations`; `spec/29-erd.md` §7 assigns this column to T-211's `0012_ics.sql`, while the checker compares every currently migrated table against all columns in the final DDL.
- ✅ `scripts/tasks-index.sh --check` — index is current after setting this task to `blocked`.
- Not run: `cargo fmt --all --check`; the combined command stopped at the failing ERD guard before invoking rustfmt.
- Not run: Clippy, workspace tests, the five schema tests, `make dev`, `make e2e`, `cargo sqlx prepare --workspace`, and the other guard scripts; no implementation remains to verify.
- Not run: hosted GitHub Actions; local-only task run.

**Manual checks** — n/a; no migration was applied to the dev or test database. The guard's message was compared with spec 06 §4.12, spec 29 §7, and T-211's own migration contract.

**Debug log** — The initial ERD check failed on the future `persons.ics_secret_version` column. Adding it to T-101 would violate the explicit `0012_ics.sql`/T-211 ownership in spec 06 §4.12 and ERD-06. A trial schema implementation was removed rather than shipping a migration that conflicts with the map. No repeated run could change this deterministic contradiction.

**Performance** — n/a; no implementation or query was retained.

**Security** — Triggers would include T2 (tenant resolution/RLS), T3 (S2/S3 person fields and S3 tenant key), and T4 (RLS/role privileges). No security controls or tests were implemented because this task is blocked. The intended controls remain those in the task: forced RLS on tenant tables except tenants and tenant_keys, SECURITY DEFINER tenant resolution, and no UPDATE/DELETE for `eq_app` on policies. SBD-81 checks were not run.

**Deviations** — RATIFY: the three mechanical task clarifications are recorded in Questions: add the explicitly required schema-test path to Files; use the slug resolver's `(id, slug, status)` output for `resolve_single_tenant`; and pin SECURITY DEFINER resolver `search_path` while qualifying `public.tenants`. Blocking question: the ERD checker requires a column assigned to a later migration. No schema or guard was changed.

**Dependencies affected** — T-103 and T-104 cannot start until T-101 is resolved; tasks depending on them remain ineligible. T-102a remains independently eligible.

**Rollback plan** — No database or source changes remain. To resume safely, resolve the checker/migration ownership question first, then return T-101 to `todo` and rerun its migration check.

**Follow-ups** — Add an architect-scoped task to align `scripts/erd.py --migrations` with ERD-06's later column migrations, or explicitly reassign the column migration in the specs/tasks. This run cannot edit the guard, spec, or T-211 task.

**Step 0 — Start** — Clean `milestone/m1-core` tree; `scripts/tasks-index.sh --check` passed; T-101 was the lowest-ID eligible task because T-003 and T-007 were `review`. The required milestone merge was pushed as `ce30c82` and the combined M0/T-102 Rust checks passed.

**Step 1 — Task and spec** — Read the entire T-101 task and the normative DDL, table data dictionary, migration map, security, SQL and code-quality rules listed above.

**Step 2 — Dependencies** — Read T-003 and T-007 Reports, including Deviations and Follow-ups, plus their shipped `eq-core` database/ID code, bootstrap migration and TestDb owner/app pool implementation. Their `eq_app` pool is authenticated with `NOSUPERUSER NOBYPASSRLS`; the bootstrap uses default privileges and `eq_enable_rls`.

**Step 3 — Design reference** — n/a; this is a backend schema task.

**Step 4 — Plan and test plan** — Recorded above before implementation. The ERD guard then exposed the blocking ownership conflict.

**Step 5 — Implement** — n/a; trial code was removed because satisfying the guard by moving the column into T-101 would contradict ERD-06 and spec 06 §4.12.

**Step 6 — Run locally** — n/a; no migration or endpoint was retained, and the task is blocked before database behavior can be exercised.

**Step 7 — Debug loop** — The deterministic guard failure was cross-checked against the explicit future migration assignment. No alternative within T-101's scope preserves both the migration map and a green required check; further attempts would require changing a guard or another task.

**Step 8 — Self code review** — n/a; there is no implementation diff to review.

**Step 9 — Optimize** — n/a; no query or runtime code was retained.

**Step 10 — Security audit** — n/a; no implementation was retained. The applicable triggers and intended controls are listed under Security.

**Step 11 — Revise and verify** — n/a; no implementation is available for full verification. The failed migration guard and unrun checks are recorded above.

**Step 12 — Report, commit** — Report completed, status set to `blocked`, and index regenerated. A signed-off blocking commit and push follow.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** The Tests section requires `crates/eq-core/tests/schema.rs`, but the Files table omitted it → add that exact test path because the named acceptance tests cannot be implemented without it.
- **Self-resolved (implementer, run 6, needs architect ratification):** `resolve_single_tenant()` had no declared return shape while the slug resolver returns `(id, slug, status)` → use the same row shape so the tenant resolver contract is consistent and can expose the status required by tenant resolution.
- **Self-resolved (implementer, run 6, needs architect ratification):** The SECURITY DEFINER resolver's search path was unspecified → pin it to `pg_catalog` and qualify `public.tenants`, because SBD-01 requires fail-closed security and prevents a shadow object from changing what the definer reads.
- **Blocker Q1:** `scripts/erd.py --migrations` requires `persons.ics_secret_version` in a migration now, but spec 06 §4.12 and ERD-06 assign it to T-211's `0012_ics.sql`. Should the checker be updated to account for later mapped column migrations, or should the column be reassigned to T-101 and the T-211 contract changed? **Recommendation:** preserve the explicit spec/task ownership and create a separate authorized guard-fix task, then resume T-101.
