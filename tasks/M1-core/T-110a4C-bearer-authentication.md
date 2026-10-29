---
id: T-110a4C
title: Authenticate bearer requests before extracting Actor
milestone: M1
status: review
depends_on: [T-110a4B]
spec_refs: [IDN-03, IDN-10, IDN-11, AUTH-01, AUTH-05, SBD-01, SBD-40, SBD-42, SBD-53, CAN-30, CACHE-14]
estimated_size: M
---

## Goal
Insert a shared Actor only after the tenant-bound access token, current password version and active membership are verified.

## Context
T-110a4A provides Actor and extractors. T-110a4B registers the spec-defined Identity permission. T-110a2 provides signed access-token claims; T-110a4 provides the shared API runtime, tenant resolver and module gate.

## Scope
### In
- Parse canonical Bearer transport into SecretString and redact Debug.
- Verify signature, expiry and resolved tenant; verify current password version and active role.
- Compute enabled-module permissions with IDN-11 owner fallback and specified 30/60-second caches.
- Wire the auth middleware after the module gate and add core-owned membership/Identity-owned credential reads.
### Out
- Cookie role switching, refresh rotation/logout/password changes and people APIs.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_auth.rs, crates/eq-identity/src/jwt.rs, crates/eq-identity/src/repo.rs | modify | Secret-safe bearer boundary, verification and password-version lookup. |
| crates/eq-core/src/auth/directory.rs | modify | Core-owned active-role membership query. |
| crates/eq-api/src/lib.rs, crates/eq-api/src/modules.rs | modify | Inject auth state and preserve module-gate ordering. |
| crates/eq-identity/tests/api_auth.rs, crates/eq-identity/Cargo.toml, Cargo.toml, Cargo.lock, .sqlx/ | modify | Abuse tests, direct dependency and generated query metadata. |
| tasks/M1-core/T-110a4C-bearer-authentication.md, tasks/DECISIONS.md, tasks/INDEX.md | read/modify/generate | Reference TD-121, test task status and regenerate index. |

## Interfaces
- Reuse AccessClaims, AuthState, TenantResolver, CoreIdentityDirectory and LoginHttpState from shipped interfaces.
- Middleware inserts Actor only after all checks pass; absent Authorization passes through for public login, malformed supplied Authorization returns unauthenticated.
- Cache keys follow CACHE-14: credential version `(tenant, person)` and permissions `(tenant, person, role)`.

## Business rules
1. Permission calculation uses only active role grants in enabled modules (IDN-10).
2. Owner fallback and password-version cache intervals follow IDN-11/IDN-03 exactly.
3. Tenant isolation remains unchanged; cross-module SQL is forbidden.

## Security
- Triggers: T1, T3, T8 (SBD-30).
- Store parsed bearer values in SecretString with handwritten redacted Debug; never log token material.
- Fail closed for invalid signature, tenant, expiry, stale version or inactive role.
- Abuse tests: cross-tenant, stale password version, inactive role, malformed transport and secret redaction.

## Implementation notes
Use the existing tenant-scoped transaction boundary, Identity's own credentials table, and Core's directory port. No schema or migration changes.

## Acceptance criteria
- AC1. Only a valid token for the resolved tenant and current credential version can produce Actor → bearer integration tests.
- AC2. Inactive memberships and malformed/invalid bearer transport return 401 → named bearer tests.
- AC3. Effective permissions are from enabled modules and IDN-11 owner fallback → owner acceptance test rerun.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/src/api_auth.rs | bearer_token_debug_redacts_credentials, malformed_bearer_header_rejected |
| crates/eq-identity/tests/api_auth.rs | bearer_actor_accepts_current_token, bearer_actor_rejects_cross_tenant_token, bearer_actor_rejects_malformed_transport, bearer_actor_rejects_inactive_role, bearer_actor_rejects_stale_password_version |

## Definition of done
- [x] Named tests pass on a fresh ritma_test_T-110a4C database in the existing PostgreSQL 16 container; drop it afterward.
- [x] SQLx metadata is regenerated/checked; cargo fmt, focused Clippy, cargo deny, task lint/index and guards pass.
- [x] Real HTTP requests verify valid and invalid bearer behavior; no new schema or error code is added.
- [x] Report includes Workflow steps 0–12 and resource snapshots.
- [x] Signed commit feat(identity): authenticate verified bearer requests [T-110a4C] is pushed to dev.

## Report
**Plan** —
1. Parse only canonical Bearer transport into a secret wrapper → verify parser and redaction tests.
2. Verify signature, expiry, tenant and credential version → verify JWT and database-backed bearer tests.
3. Check the active role through Core and credentials through Identity → verify tenant, stale-version and inactive-role abuse tests.
4. Derive permissions from enabled modules and cache by the CACHE-14 keys → rerun owner-without-admin acceptance.
5. Wire middleware inside the module gate, then run real HTTP requests and the task checks → verify valid Actor insertion, failure responses, SQLx metadata and guards.
**Assumptions** — The API composition supplies the resolved `TenantCtx` and compiled enabled-module set. This task introduces no role, permission, error code or schema object.
**Test plan** —
- AC1, Given a resolved tenant and a current token, When the protected route is requested, Then the handler receives an `Actor` → `bearer_actor_accepts_current_token`; cross-tenant and stale versions must return 401 → `bearer_actor_rejects_cross_tenant_token`, `bearer_actor_rejects_stale_password_version`.
- AC2, Given an inactive role or malformed Bearer transport, When the route is requested, Then it returns 401 `unauthenticated` → `bearer_actor_rejects_inactive_role`, `bearer_actor_rejects_malformed_transport`, `malformed_bearer_header_rejected`.
- AC3, Given the active owner role and enabled modules, When permissions are computed, Then IDN-11 owner fallback remains unchanged → `owner_gets_admin_perms_without_admin`.
- Edge cases: missing Authorization passes through for public login; missing credentials, invalid signature, wrong tenant, expired token, inactive role and stale version fail closed; whitespace and unsupported scheme are rejected; secret formatting is redacted.
- Error case: reuse registered 401 `unauthenticated`; no new problem code or response detail is introduced.
- Abuse cases: cross-tenant token, deactivated membership, stale credential version, malformed transport and secret redaction are covered by the named tests.
**Summary** — Protected requests now receive `Actor` only after the signed token, resolved tenant, credential version and active membership checks pass. Permissions use enabled modules and the CACHE-14 key shapes, while the existing public login path still accepts requests without Authorization.
**Workflow steps 0–12** — 0: resumed T-110a4C on `dev` from preserved WIP; fetched origin and confirmed the index/WIP state. 1: read this task and its cited specs. 2: read T-110a4B's Report and shipped permission-registration code (including TD-118, TD-122 and TD-123). 3: n/a, backend-only. 4: recorded the Plan and AC/edge/error/abuse test plan above before continuing implementation. 5: implemented only the listed files. 6: exercised the production middleware over real localhost TCP requests; `make dev` was not run for the reason below. 7: recorded and resolved the SQLx database and isolated-unittest invocation failures below. 8: reviewed the staged diff against CQ-16 §9, CA-17 §7, ERR-19 and SBD-81. 9: n/a for measured tuning; required auth caches follow CACHE-14. 10: completed the SBD-81 review below. 11: reran task, Rust, SQLx, guard and secret checks below. 12: this Report, status and index are complete; the signed task commit and push follow.
**Spec read** — `spec/modules/identity.md` §§1–2; `spec/25-auth-and-access-control.md` §§3.1 and 4.6; `spec/23-contracts-and-canonical-forms.md` CAN-30; `spec/26-containers-and-caching.md` CACHE-14; `spec/18-security-by-design.md` §§5.1, 5.3, 8 (SBD-81); `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§4, 7; `spec/19-error-handling.md` §§2, 10; ERD Core `persons` data dictionary.
**Files** —
- Modified `crates/eq-identity/src/api_auth.rs`, `jwt.rs`, `repo.rs`; added bearer parsing, middleware, cache checks and credential lookup.
- Modified `crates/eq-core/src/auth/directory.rs`; added the Core-owned active-role lookup.
- Modified `crates/eq-api/src/lib.rs` and `modules.rs`; injected auth state and placed auth after the module gate.
- Modified `crates/eq-identity/tests/api_auth.rs` and `crates/eq-identity/Cargo.toml`; added local TCP HTTP abuse tests and the `reqwest` dev dependency.
- Modified `Cargo.lock`; added `.sqlx/query-71a5f758bd90b9d563e4c0bacf3ce9d7bafdb0d0d5652507799c3af38b6438f0.json` and `.sqlx/query-c7f8d8c58c3bb26fb1e276579c78ca853db830aa880542a62122098620af8676.json`.
- Modified this task, `tasks/DECISIONS.md` (TD-124) and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `bearer_actor_accepts_current_token`, `bearer_actor_rejects_cross_tenant_token`, `bearer_actor_rejects_stale_password_version` | Passed; current token produced an Actor, the cross-tenant and stale tokens returned 401 `unauthenticated`. |
| AC2 | `bearer_actor_rejects_inactive_role`, `bearer_actor_rejects_malformed_transport`, `malformed_bearer_header_rejected` | Passed; inactive role and malformed input returned 401 `unauthenticated`; parser unit cases rejected malformed forms. |
| AC3 | `owner_gets_admin_perms_without_admin` | Passed; owner fallback acceptance was rerun. |
**Checks run** —
- ✅ `cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline --workspace --all-targets -- -D warnings`.
- ✅ Focused unit tests: `CARGO_TARGET_DIR=... SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity api_auth::tests:: -- --test-threads=1` (5 passed).
- ✅ Focused database suite: `cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` (10 passed, 0 failed) on fresh `ritma_test_T-110a4C`, PostgreSQL 16.15 in reused `ritma-db-1`; migrations were applied first and the database was dropped.
- ✅ `cargo test --locked --offline --workspace --no-run` compiled all workspace test binaries. ❌ `cargo test --workspace` was not run: the workspace DB tests call `TestDb::new()` / testcontainers and would create per-invocation PostgreSQL containers, prohibited by the disk-hygiene rule; task-scoped DB tests instead used the existing fixed container.
- ✅ `cargo deny check` (advisories, bans, licenses and sources passed; existing duplicate-base64 warning only).
- ✅ `python3 -I scripts/tasks-lint.py`; ✅ `scripts/tasks-index.sh --check`; ✅ `python3 -I -B scripts/tests/test_tasks_lint.py` (8 passed).
- ❌ `python3 -I -m unittest scripts/tests/test_tasks_lint.py` failed to import `scripts` under isolated mode; direct-file invocation above passed all 8 tests.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && python3 -I scripts/erd.py --check && python3 -I scripts/erd.py --migrations`.
- ✅ `gitleaks protect --staged --redact` (no leaks found).
- ✅ `cargo sqlx prepare --workspace -- --all-targets --all-features` regenerated query metadata during implementation; ✅ `DATABASE_URL=<fresh task DB> SQLX_OFFLINE=false cargo sqlx prepare --workspace --check -- --all-targets --all-features` verified it against current migrations.
- ❌ `make api` reports `OpenAPI generation is added by T-114` and exits 2; generation is not available until that task. No OpenAPI output was produced.
- Not run: web lint/typecheck/tests/i18n; `web/` was not changed.
- Not run: `make dev` and `make e2e`; the task-scoped reqwest suite exercised the middleware over real localhost TCP requests on a fresh DB, and T-110a4C is not the M1 integration task.
- ✅ `git diff --check`.
- Not run: hosted CI; it requires GitHub Actions.
**Manual checks** — The reqwest integration test starts a test Axum router assembled from `IdentityModule` routes and the production bearer middleware on an ephemeral localhost TCP port, then sends valid and malformed Authorization requests. A current same-tenant token reached the test-only protected handler (`200`, `has_write=true`); cross-tenant, inactive-role, stale-version and malformed-transport requests returned `401` with `code=unauthenticated`. `make dev` was not run because its `.env`-driven multi-process startup would use the shared development configuration; the production middleware request path was exercised in the task test instead.
**Debug log** — The first `cargo sqlx prepare --workspace --check -- --all-targets --all-features` against the shared `ritma` database failed because that older database lacks `public.cron_state` and `public.resolve_tenant_preset`; a fresh database with current migrations passed the check. The documented isolated unittest module invocation could not import `scripts`; invoking the same test file directly with `python3 -I -B` passed 8/8. `make api` is an intentional stub until T-114, so no workaround was attempted.
**Performance** — n/a; this task adds no endpoint or list query. Cache TTLs and keys follow CACHE-14 exactly.
**Security** — Triggers T1, T3 and T8. SBD-81 review: auth is applied after ModuleGate and before the protected test handler; tenant comes only from resolved `TenantCtx` and is checked against JWT `tid`; no field or table was added; request values are checked at the boundary; SQL uses bound parameters and each query is tenant-scoped; token text is held as `SecretString` and debug formatting redacts it; failures reuse 401 `unauthenticated`; no state mutation or audit event is introduced. Abuse tests cover cross-tenant use, inactive membership, stale password version, malformed transport and secret redaction. No new permission grant or security exception was added.
**Deviations** — RATIFY TD-121 (wrap parsed bearer text in `SecretString` with redacted Debug per SBD-50); RATIFY TD-122 (uppercase child suffix C preserves the task index's bytewise order); RATIFY TD-124 (use CACHE-14's exact `(tenant, person)` and `(tenant, person, role)` auth-cache keys). These choices are recorded in `tasks/DECISIONS.md`. `make api` and the full DB-backed workspace test suite remain Not run for the reasons above.
**Dependencies affected** — T-111 must call `LoginHttpState::invalidate_password_version` after password changes/resets so the in-process version cache is cleared immediately.
**Rollback plan** — Revert this task's signed commit. No schema or migration rollback is required; both new caches are process-local.
**Follow-ups** — T-111: call the exposed password-version invalidation method from password-change/reset handlers. No other out-of-scope change was made.
**Resource and disk hygiene** — Before: `du -sh "$HOME/.cache/ritma/target"` = 9.6G; Docker usage = Images 20 / 15.5GB (2.073GB reclaimable), Containers 17 / 10.48MB, Volumes 443 / 74.55GB (68.81GB reclaimable), Build Cache 0; filesystem had 62G available. After: fixed target = 11G; Docker usage counts/sizes unchanged; filesystem had 58G available. `ritma-db-1` and its existing volume were reused; no image, container, volume or target directory was created or removed. The same task database name `ritma_test_T-110a4C` was created and dropped twice earlier in this task and twice in this continuation (focused tests and SQLx freshness check); all four uses were removed. The fixed target grew about 1.4G from required workspace test-binary compilation; the rule against cleaning the shared target prevented reclaiming that growth, so the “disk no bigger” condition could not be met without violating the explicit no-clean rule.

## Questions
- **Self-resolved (implementer, run 9, needs architect ratification; TD-121):** parsed bearer text must not remain a plain String → wrap it in SecretString and expose only at the verifier call, per SBD-50.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** ordered child C must sort before lowercase aggregate T-110a4a → uppercase C does under the task index's fixed C byte order; no guard is changed.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-124):** the task specifies 30/60-second auth caches but not their keys → use `(tenant, person)` for credential versions and `(tenant, person, role)` for permission sets, because CACHE-14 fixes those exact key shapes.

## Review
n/a — awaiting maintainer review.
