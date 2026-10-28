---
id: T-109b
title: Add audit tables, grants and controlled database functions
milestone: M1
status: review
depends_on: [T-109a]
spec_refs: [AUD-01, AUD-03, AUD-05, ERD-03, ERD-04, ERD-05, SEC-07]
estimated_size: S
---

## Goal
Create the mapped audit schema, immutable app-role entry table, checkpoints, row policies and controlled functions. The SQL must match spec 06 §4.9/§4.11 and the T-109 row in ERD §7.

## Context
Implement only `migrations/0006_audit.sql`. The parent T-109 Context contains the complete mapped DDL, grants, `seal`, `anonymize` and `purge_expired` contracts. `audit.entries` is tenant-scoped with RLS; checkpoints are system rows without RLS. SECURITY DEFINER functions use `search_path = pg_catalog, audit` and explicitly reject a tenant different from `app.tenant_id`.

## Scope
### In
- Exact audit tables/indexes, least-privilege grants, RLS policies, trigger-free controlled functions from parent T-109.
- Verify that `eq_app` cannot UPDATE/DELETE entries and cannot pass another tenant to the functions.
### Out
- Rust record/seal/policy/job behavior (T-109c–T-109h).

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.lock` | modify | Lock the integration-test dependency set. |
| `crates/eq-audit/Cargo.toml` | modify | Declare integration-test-only database dependencies. |
| `migrations/0006_audit.sql` | create | Exact mapped audit schema and functions. |
| `crates/eq-audit/tests/audit.rs` | modify | App-role privilege test. |
| `tasks/M1-core/T-109b-audit-schema.md` | modify | Report and status. |
| `tasks/DECISIONS.md` | modify | Record TD-077 for the missing test dependency declarations. |

## Interfaces
Use the table DDL and function signatures in parent T-109 Context. Do not add columns, tables, roles, policies or permission names.

## Business rules
None beyond the parent task's exact cutoff and anonymization semantics.

## Security
- Triggers: tenant isolation and immutable audit storage (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05; free-text `reason` follows ERD-05 anonymization requirements.
- Threat: cross-tenant mutation or procedure calls → RLS, revoked UPDATE/DELETE and explicit tenant-context checks.
- Abuse tests: `app_role_cannot_mutate`; cross-tenant procedure rejection is retained in T-109f.

## Implementation notes
Read the audit module section and T-109 migration mapping in `spec/29-erd.md`; run both ERD checks after the migration change.

## Acceptance criteria
- AC1. `eq_app` cannot UPDATE or DELETE audit entries → `app_role_cannot_mutate`.
- AC2. Migration matches the complete spec 06 DDL and ERD mapping → `scripts/erd.py --migrations`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `app_role_cannot_mutate` |

## Definition of done
- [x] `scripts/erd.py --migrations` and `scripts/erd.py --check` pass.
- [x] The app-role test uses `eq_app`, not a superuser connection.
- [x] Signed commit `feat(audit): add audit schema and functions [T-109b]`.

## Report
**Plan** — Verify the existing mapped migration against spec 06 and ERD §7, declare only the dev dependencies needed by the integration test (TD-077), and test UPDATE/DELETE rejection through `TestDb::app_pool`.
**Assumptions** — Parent T-109 is the source for exact signatures and values; the spec DDL is normative.
**Test plan** — AC1→`app_role_cannot_mutate`, using the non-superuser `eq_app` pool for both UPDATE and DELETE; AC2→`scripts/erd.py --migrations` plus `scripts/erd.py --check`.
**Summary** — Added the mapped audit migration with entries/checkpoint tables, indexes, entry RLS and least-privilege grants, plus tenant-context-checked seal/anonymize/purge functions. Added the app-role integration test and only test-scoped dependencies. ERD validation, the targeted PostgreSQL test and the final Rust workspace suite passed.
**Spec read** — `spec/06-domain-model.md` §§4.9/4.11; audit module and data-class sections in `spec/29-erd.md` §§3/4/5/7; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/8; `spec/19-error-handling.md` §10; parent T-109 contract and T-109a Report/code.
**Files** — Added `migrations/0006_audit.sql` and `crates/eq-audit/tests/audit.rs`; changed `crates/eq-audit/Cargo.toml`, `Cargo.lock`, `tasks/M1-core/T-109b-audit-schema.md`, `tasks/DECISIONS.md` and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `app_role_cannot_mutate` | Pass; `SELECT current_user` returned `eq_app`, and UPDATE and DELETE both returned SQLSTATE `42501`. |
| AC2 | `python3 -I scripts/erd.py --migrations`; `python3 -I scripts/erd.py --check` | Pass; 21 migrated tables checked, ERD matches. |
**Checks run** — `cargo fmt --all --check` ✅; `SQLX_OFFLINE=true cargo test -p eq-audit --test audit app_role_cannot_mutate -- --exact` ✅ (1/1; fresh Testcontainers PostgreSQL 16 `postgres:16-alpine`, generated per-test `ritma_test_<uuid>` database); `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` ✅; `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` ✅ on final run; `cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-CC0 warnings remain); `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh` ✅; `python3 -I scripts/erd.py --migrations` ✅ (21 migrated tables checked); `python3 -I scripts/erd.py --check` ✅; `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8); `scripts/tasks-index.sh --check` ✅; `git diff --check` ✅; `gitleaks dir --redact --no-banner` run separately on each of the seven changed paths ✅ (no leaks). The direct command `python3 -I -m unittest scripts/tests/test_tasks_lint.py` failed to import `scripts`; TD-078 records the equivalent isolated discovery invocation. Not run: `cargo sqlx prepare --workspace` (no compile-time SQL query changed); web checks (no web files changed); `make dev` and browser E2E (no server/UI behavior in this schema/privilege slice).
**Manual checks** — The database test ran on a new `eq_testkit::TestDb`, confirmed its least-privilege connection identity, and attempted both forbidden mutations.
**Debug log** — The first full workspace test attempt hit a Testcontainers startup timeout in the unrelated `eq-api::health::readyz_migrations_pending` test. That test passed on an isolated retry (1/1), and the full workspace test passed on rerun. The prescribed isolated unittest module path failed because `-I` removes the repository root from import lookup; discovery rooted at `scripts/tests` ran the same file and passed all 8 tests (TD-078).
**Performance** — n/a.
**Security** — Trigger: audit immutability and tenant isolation (SBD-30 T4). Entries are RLS-enabled and forced through the tenant policy; the app role receives SELECT/INSERT but no UPDATE/DELETE; SECURITY DEFINER functions compare the supplied tenant with `app.tenant_id`. `app_role_cannot_mutate` executes as `eq_app` and fails if either mutation is allowed. No new data class, handler, permission, external call or sensitive logging was added. Cross-tenant function argument abuse tests remain assigned to T-109f.
**Deviations** — RATIFY TD-077: add integration-test dependencies as dev dependencies because T-109b's exact `TestDb::app_pool` privilege test requires direct `eq-testkit`, `sqlx` and `tokio` declarations; production dependencies remain unchanged. RATIFY TD-078: isolated unittest module invocation could not import `scripts` from the filesystem path, so the equivalent unittest discovery command was used and passed all eight gate tests. The migration follows parent decisions TD-071 (`reason` anonymization key) and TD-072 (checkpoint `at DEFAULT now()`).
**Dependencies affected** — T-109c consumes the schema and functions; later audit parts use the same mapped migration.
**Rollback plan** — Revert this task commit; because the migration is forward-only and has not been applied to a shared database, no compensating migration is needed before merge.
**Follow-ups** — T-109f retains cross-tenant procedure abuse tests; T-109c implements the versioned retention policy.

**Workflow steps 0–12** — Step 0: continued the existing authorized T-109 work on `dev`; T-109a was `review`, T-109b was the only active task, and its task/index status was updated. Step 1: read the complete T-109b contract, parent T-109 DDL/function contract, and cited spec sections. Step 2: read T-109a's Report and shipped derive/trait code; confirmed the existing module stub. Step 3: n/a, database/backend task without a UI. Step 4: recorded this plan and AC-to-test map before implementation. Step 5: verified the mapped migration and added only direct integration-test dependencies and the named app-role test. Step 6: n/a for `make dev`; exercised database behavior through the fresh Testcontainers PostgreSQL test. Step 7: recorded and resolved the transient Testcontainers timeout with an isolated retry and a green full-suite rerun. Step 8: reviewed against CQ-101, CA-01/30, ERR-10, ERD and the SBD-81 checklist; no schema drift, cross-module SQL, new error code or privilege expansion was introduced. Step 9: n/a; no measured performance concern. Step 10: checked tenant policy, entry immutability, function tenant guards, data classes and abuse-test coverage. Step 11: reran Rust, ERD, dependency/guard, task hygiene and secret checks; final full workspace tests were green. Step 12: completed this Report, regenerated the task index, and prepared the signed commit and push.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-077):** T-109b requires an integration test using `TestDb::app_pool` and SQLx but omits the crate manifest from `## Files` → add `eq-testkit`, `sqlx` and `tokio` as dev dependencies and list the manifest here because Rust integration tests require direct crate dependencies and this leaves runtime dependencies unchanged.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-078):** `python3 -I -m unittest scripts/tests/test_tasks_lint.py` treats `scripts` as an importable package, which isolated mode cannot resolve → run `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'`, which exercises the same file and all eight tests successfully.

## Review
n/a until review.
