---
id: T-110a4B
title: Register the spec-defined Identity write permission
milestone: M1
status: review
depends_on: [T-110a4A]
spec_refs: [IDN-10, IDN-11, SBD-40, SBD-42]
estimated_size: S
---

## Goal
Expose the existing identity.people.write permission with only its spec-defined Admin grant.

## Context
The permission key and Admin grant already appear in spec/modules/identity.md §3. The login permission snapshot and owner-without-admin fallback shipped in T-110a4 and are covered by TD-118.

## Scope
### In
- Register identity.people.write for ActorRole::Admin only.
- Verify the Identity module's registered permission list and the owner fallback when no active admin exists.
- Make the auth integration fixture accept the task-named ritma_test_<TASK_ID> database.
### Out
- New permission keys/grants, permission calculation changes, bearer middleware and role switching.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/module.rs | modify | Register the exact spec-defined permission and Admin grant. |
| crates/eq-identity/tests/api_auth.rs | modify | Owner/admin acceptance test and task-named DB guard/fixture. |
| tasks/M1-core/T-110a4B-identity-permissions.md, tasks/DECISIONS.md, tasks/INDEX.md | read/modify/generate | Reference TD-118 and update task state. |

## Interfaces
- Use Module::permissions and PermissionDef.
- Reuse the shipped LoginHttpState permission snapshot; owner fallback follows IDN-11.

## Business rules
identity.people.write is Admin-only in the Identity module. The owner receives that permission only under IDN-11's no-active-admin condition.

## Security
- Triggers: T1 and T8 (SBD-30).
- Register no additional grant; disabled modules continue to fail closed.
- Abuse test: owner permission is absent when an active tenant admin exists.

## Implementation notes
Preserve the existing permission calculation and test both sides of the owner fallback. Use only the task-specific disposable test database.

## Acceptance criteria
- AC1. IdentityModule exposes identity.people.write for Admin only → defaults_schema_and_summary_match_contract.
- AC2. Owner receives the Admin set only when no active Admin exists → owner_gets_admin_perms_without_admin.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/src/module.rs | defaults_schema_and_summary_match_contract |
| crates/eq-identity/tests/api_auth.rs | owner_gets_admin_perms_without_admin |

## Definition of done
- [x] Both named tests pass; `owner_gets_admin_perms_without_admin` ran on a fresh ritma_test_T-110a4B database in the existing PostgreSQL 16 container, then the database was dropped.
- [x] fmt, focused Clippy, task lint/index, cargo deny and applicable guards pass.
- [x] No other permission key, grant, role or schema object is introduced.
- [x] Report includes Workflow steps 0–12 and resource snapshots.
- [x] Signed commit feat(identity): register the spec-defined write permission [T-110a4B] is pushed to dev.

## Report
**Plan** — (1) Check the exact permission key and grant against the identity module spec → verify the manifest test. (2) Register only the Admin grant and update the stale route-surface assertion to match the already-shipped login route → verify the named unit test. (3) Make the fixture accept a task-named database and cover the owner fallback with/without an active admin → verify the named API integration test on fresh PostgreSQL 16. (4) Run Clippy, cargo deny, task checks, guards, secret scan and resource snapshots → verify the task Definition of done.
**Assumptions** — The owner fallback already shipped in T-110a4; this part registers only the permission and verifies the spec-defined fallback. The existing PostgreSQL container and `eq_template` contain the schema shipped through migration 0011.
**Test plan** — AC1: Given the Identity module registry, when its manifest is checked, then `identity.people.write` is present with only the Admin role and the shipped login route remains listed → `defaults_schema_and_summary_match_contract`. AC2: Given an owner tenant, when there is no active admin, then the login permission snapshot includes `identity.people.write`; when an active admin exists, it omits that permission → `owner_gets_admin_perms_without_admin`. Edges: separate fresh fixture tenants for both membership states. Errors: none introduced. Abuse: owner privilege is removed from the snapshot when an active admin exists.
**Summary** — Registered the existing `identity.people.write` permission for Admin only and verified IDN-11's conditional owner fallback through the login route. Updated the existing route-surface assertion to preserve the shipped `/api/v1/auth/login` path. Both named tests pass; the integration test used and then dropped `ritma_test_T-110a4B`.
**Spec read** — `spec/modules/identity.md` §§1–3; `spec/25-auth-and-access-control.md` §§1 and 3; `spec/18-security-by-design.md` §§5.4 and 8 (SBD-81); `spec/16-code-quality.md` §§9 and 12; `spec/17-clean-architecture.md` §§1 and 7; `spec/19-error-handling.md` §10. T-110a4A's Report and shipped Actor code were read before implementation.
**Files** — Modified `crates/eq-identity/src/module.rs` and `crates/eq-identity/tests/api_auth.rs`; updated this task, `tasks/DECISIONS.md` and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1: manifest exposes the spec-defined Admin-only write permission and shipped login route | `defaults_schema_and_summary_match_contract` | ✅ 1 passed |
| AC2: owner fallback is present without an active admin and absent when one exists | `owner_gets_admin_perms_without_admin` | ✅ 1 passed (two login requests on fresh database) |
**Checks run** — ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`; ✅ `DATABASE_URL='postgres://eq_app:test-secret-app@127.0.0.1:32768/ritma_test_T-110a4B' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@127.0.0.1:32768/ritma_test_T-110a4B' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked -p eq-identity defaults_schema_and_summary_match_contract -- --test-threads=1` (1 passed); ✅ `DATABASE_URL='postgres://eq_app:test-secret-app@127.0.0.1:32768/ritma_test_T-110a4B' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@127.0.0.1:32768/ritma_test_T-110a4B' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked -p eq-identity --test api_auth owner_gets_admin_perms_without_admin -- --exact --test-threads=1` (1 passed); ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked -p eq-identity --all-targets -- -D warnings`; ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` (advisories/bans/licenses/sources pass; existing duplicate-base64 warning); ✅ `scripts/tasks-index.sh --check`; ✅ `python3 -I -B scripts/tasks-lint.py`; ✅ `python3 -I -B scripts/tests/test_tasks_lint.py` (8 passed); ✅ `scripts/check-deps.sh`; ✅ `scripts/check-literals.sh`; ✅ `scripts/check-web-boundaries.sh`; ✅ `scripts/check-layers.sh`; ✅ `python3 -I -B scripts/erd.py --check`; ✅ `git diff --check`; ✅ Gitleaks scans of the modified module and integration test (no leaks). Not run: `scripts/erd.py --migrations` (no migration changed); hosted CI; a separate listening `make dev` process (the exact login route was exercised through the Axum router integration test using PostgreSQL); E2E (not a milestone integration task).
**Manual checks** — The named integration test sent `POST /api/v1/auth/login` through the Axum router for the owner-without-admin and owner-with-admin cases on PostgreSQL 16. No endpoint or browser UI was added.
**Resources** — Before: fixed target `9.2G`; `df -h .` showed `62G` available; 17 existing containers and 12 existing images. After: fixed target `9.2G`; `62G` available; the same 17 containers and 12 images. `timeout 30 docker system df` timed out (exit 124) before and after, so usage output is **Not run**. Reused container `ritma-t110a3-pg16-20261029` (not removed); created database `ritma_test_T-110a4B` from existing `eq_template`, then dropped it (verified absent). No new container, image, volume or Cargo target was created. Set the existing dev role passwords to obvious fake test values `test-secret-owner` and `test-secret-app` so host TCP connections authenticate; `eq_app` remains `NOSUPERUSER NOBYPASSRLS`.
**Debug log** — The first `defaults_schema_and_summary_match_contract` run failed because an old assertion expected no Identity routes. Replaced it with the spec-defined login path assertion under TD-123; retry passed 1/1. The first owner integration attempt failed to connect as the database owner because the role password was unknown for host TCP authentication. Set the two existing development roles to the documented fake test credentials, verified both TCP connections, and retried; the named test passed 1/1. No third attempt was needed.
**Performance** — n/a; no query or hot path was changed.
**Security** — Triggers T1/T8. SBD-81: only the existing identity write permission is registered for Admin; owner fallback is conditional on no active tenant admin and remains computed. No new key, role grant, schema object, error code or cross-module query was added. The owner-with-admin abuse case passes. The task database was accessed through `eq_app` with `NOSUPERUSER NOBYPASSRLS`; owner setup used the separate owner role. No audit event or state-changing production behavior was introduced.
**Deviations** — RATIFY TD-118 (register the already-specified permission required by the existing acceptance criterion), TD-122 (uppercase child ordering for the unchanged index guard), and TD-123 (replace the stale empty-route assertion with the shipped spec-defined login path).
**Dependencies affected** — T-110a4C uses the Identity permission manifest; T-110a4a and T-110a5 aggregate this permission behavior.
**Rollback plan** — Revert this task's signed commit; no persisted application schema changes exist. The fake development role passwords can be reset locally if needed.
**Follow-ups** — Architect ratification of TD-118, TD-122 and TD-123.
**Workflow steps 0–12** — 0: selected this lowest-ID eligible task after T-110a4A review, set in_progress and regenerated index; 1: read the full task and cited specs; 2: read T-110a4A Report and shipped code; 3: n/a (backend-only); 4: recorded plan, assumptions and AC/test mappings; 5: registered the exact Admin grant, corrected the stale route assertion, and expanded the fixture for active-admin state; 6: exercised login requests through the Axum router with a fresh database; separate listening `make dev` was not needed for a permission manifest change and is marked Not run above; 7: corrected the stale assertion and owner TCP credentials, with one retry per failure; 8: reviewed grant scope, owner fallback, RLS app role and schema scope; 9: n/a; 10: completed SBD-81 review above; 11: reran fmt, focused tests/Clippy, deny, task lint/index, four guards, ERD, diff and Gitleaks; 12: completed this Report, set status to review, regenerated index and prepared the signed task commit.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification; TD-118):** T-110a5 prohibits adding grants while AC2 requires identity.people.write → register only the already-specified Admin grant and use IDN-11's owner fallback; no permission is invented.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** this ordered child uses uppercase suffix B so it sorts before lowercase aggregate T-110a4a under LC_ALL=C without changing the DAG guard.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-123):** the existing `defaults_schema_and_summary_match_contract` helper expected Identity to expose no routes, but T-110a4 already shipped `POST /api/v1/auth/login` → assert that exact spec-defined login route instead, because the original empty-surface assertion predates the shipped endpoint and made the named test fail.

## Review
n/a — awaiting maintainer review.
