---
id: T-110a5
title: Add Actor permissions and role switching
milestone: M1
status: review
depends_on: [T-110a4a, T-110a4b]
spec_refs: [IDN-05, IDN-09, IDN-10, IDN-11, AUTH-02, AUTH-05, STM-22, CAN-34, SBD-40, SBD-42, SBD-53]
estimated_size: S
---

## Goal
Verify the shared `eq_core::auth::Actor` extractor, permission derivation and identity role-switch behavior shipped by the two implementation parts below. Preserve this aggregate task so the original identity-auth contract can be checked end to end after both parts are reviewed.

## Context
The parent T-110a contract defines `Actor { person_id, tenant, role, session_id, perms, ctx }` and `MaybeActor`. `Actor::require` returns `AppError::Forbidden`; `has` checks the permission set. Permissions are the union of `PermissionDef.roles` containing the active role in enabled modules. If the tenant has no active admin, owner receives the admin permission set; this is computed, never stored. Deactivated membership is rejected. `POST /auth/switch-role {role}` issues a new token, updates `refresh_sessions.active_role`, and retains the same refresh family.

## Split
The actor authentication/permission path and cookie role-switch endpoint are separate review concerns and together exceed CQ-101's approximately 400-line guide. The original acceptance criteria and named tests remain assigned exactly once to the parts. This aggregate stays `draft` until both parts are `review`, then returns to `todo` for final contract verification.

| Part | Responsibility | Depends on | Owned acceptance/tests |
|---|---|---|---|
| T-110a4a (parts T-110a4A/B/C) | Verified Actor extraction, permission grants and bearer authentication | T-110a4, T-110a2 | AC2; Actor extractor, permission and all bearer abuse/redaction tests listed below |
| T-110a4b | Cookie role switch and CSRF | T-110a4a | AC1, AC3; `switch_role`, `switch_role_requires_csrf` |

## Scope
### In
- Verify the original AC1–AC3 against the completed T-110a4a and T-110a4b reports and shipped code.
- Run the combined identity auth tests on a disposable PostgreSQL 16 task database.
### Out
- Reimplementing behavior owned by T-110a4a or T-110a4b; refresh rotation, logout and password-change invalidation (T-110b); people APIs (T-112).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/tests/api_auth.rs` | read/run | Combined identity API acceptance and abuse tests from both parts. |
| `tasks/M1-core/T-110a5-actor-permissions-and-role-switch.md`, `tasks/M1-core/T-110a4a-actor-auth-and-permissions.md`, `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/create/read/generate | Split implementation contract, aggregate report and generated task state. |

## Interfaces
Use `ActorRole`/`Role`, `ModuleRegistry`, `EnabledModuleSet`, `PermissionDef`, the existing tenant resolver and the parent `POST /auth/switch-role` wire shape. Do not add permission keys or role grants beyond the module definitions in the cited specs.

## Business rules
The role privilege ordering and owner fallback are exactly IDN-09/11. Role switch accepts only an active membership role; no new role is granted.

## Security
- Triggers: T1, T3, T4, T8 (SBD-30).
- Data classes touched: S1/S2/S3; bearer/refresh tokens remain secret and PII remains tenant-scoped.
- Role change requires both `X-EQ-CSRF: 1` and an `Origin` equal to `RITMA_PUBLIC_URL`'s origin; permission checks use the active tenant and enabled module set (IDN-05/SBD-40/42).
- Abuse tests reject a role outside active memberships, cross-tenant JWTs, missing CSRF, and owner permission retention after an admin becomes active (SBD-31/73).

## Implementation notes
1. Actor extraction trusts only a token verified by the identity authentication boundary; never build an Actor directly from caller-supplied role/tenant fields.
2. Cache invalidation/TTL follows the parent contract; do not persist the computed owner grant.
3. Add CSRF for switch-role here because this child owns that cookie-authenticated endpoint; T-110b owns refresh/logout checks.

## Acceptance criteria
- AC1. Admin+guardian role switching yields a guardian token and narrower permissions → `switch_role`.
- AC2. Owner receives `identity.people.write` only while no active tenant admin exists → `owner_gets_admin_perms_without_admin`.
- AC3. Missing/invalid CSRF and inactive roles are rejected with the specified error.

## Tests to write
The original test names are implemented by their named split part and rerun in the aggregate verification:

| Part | File | Test names |
|---|---|---|
| T-110a4a / T-110a4A | `crates/eq-core/src/auth.rs` | `actor_requires_authenticated_extension`, `actor_requires_registered_permission`, `maybe_actor_is_optional` |
| T-110a4a / T-110a4B | `crates/eq-identity/src/module.rs`, `crates/eq-identity/tests/api_auth.rs` | `defaults_schema_and_summary_match_contract`, `owner_gets_admin_perms_without_admin` |
| T-110a4a / T-110a4C | `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/tests/api_auth.rs` | `bearer_actor_rejects_cross_tenant_token`, `bearer_actor_rejects_inactive_role`, `bearer_actor_rejects_stale_password_version`, `bearer_token_debug_redacts_credentials`, `malformed_bearer_header_rejected` |
| T-110a4b | `crates/eq-identity/tests/api_auth.rs` | `switch_role`, `switch_role_requires_csrf`, `switch_role_rejects_inactive_role`, `switch_role_keeps_refresh_family` |

## Definition of done
- [x] All split-part tests pass on a disposable PostgreSQL 16 task database; the database is dropped after use.
- [x] `scripts/check-layers.sh`, `scripts/check-web-boundaries.sh`, task lint/index and `cargo fmt --all --check` pass.
- [x] No permission key or role grant beyond those explicitly defined in the cited specs, schema, error code or cross-module SQL is introduced.
- [x] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [x] Signed aggregate commit `test(identity): verify Actor permissions and role switching [T-110a5]` pushed to `dev`.

## Report
**Plan** — (1) Keep this task as the final aggregate after splitting authentication/permission derivation and cookie role switching into T-110a4a and T-110a4b → verify their Reports and dependency edges. (2) Run both parts' named tests plus the identity integration suite on a fresh task database → verify original AC1–AC3. (3) Review the joined implementation against IDN-05/09/10/11, AUTH-05, CAN-34, SBD-40/42/53 and the parent T-110 contract → verify no permission or tenant-boundary drift. (4) Regenerate/check the task index and produce the original signed aggregate commit on `dev`.
**Assumptions** — The split preserves each original AC and named test once; both parts use the already-compiled module registry and enabled-module service. Permission derivation and the owner fallback remain governed by the specs.
**Test plan** — AC1: active admin+guardian membership can switch to guardian and receives narrower permissions → `switch_role`. AC2: owner gets the spec-defined admin permission set only while the tenant lacks an active admin, under the specified 30-second permission cache → `owner_gets_admin_perms_without_admin`. AC3: missing/mismatched CSRF and inactive target role are rejected → `switch_role_requires_csrf` and role-switch abuse test. Edge cases: malformed/expired/cross-tenant tokens, missing tenant, disabled module, no active role. Error cases: registered `unauthenticated` and `forbidden` codes only. Security abuse: cross-tenant JWT, inactive membership, missing CSRF, owner permission removal after cache expiry.

**Summary** — The Actor/permission and cookie role-switch split parts satisfy the parent contract together: verified Actors receive only the enabled-module permissions for their active role, owner fallback is conditional, and role switching enforces CSRF, active membership, refresh-family preservation and transactional audit. All listed unit and HTTP tests passed, including the combined Identity API suite at 16/16 on PostgreSQL 16.15. The sibling role-switch test task also ran the browser E2E suite at 4/6, with two WebKit page launches recorded as Not run because required host libraries are absent.
**Spec read** — `spec/modules/identity.md` §§1–3; `spec/25-auth-and-access-control.md` §§1–4.6/6; `spec/23-contracts-and-canonical-forms.md` CAN-30/CAN-34; `spec/26-containers-and-caching.md` CACHE-14; `spec/18-security-by-design.md` §§3–5/8 (SBD-81); `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7; `spec/19-error-handling.md` §§2/10; `spec/20-state-machines.md` STM-22. Read T-110a4a and T-110a4b Reports, their child Reports and the shipped Actor, bearer middleware, role-switch service and route code.
**Files** — Updated this aggregate's status and Report; regenerated `tasks/INDEX.md`. No application source, migration, SQLx metadata or web source changed. `RUN_LOG.md` remains append-only and untracked.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 — role switch returns the selected role and narrower permissions while retaining the refresh family | `switch_role`; `switch_role_keeps_refresh_family` | Passed in the 16-test API suite. |
| AC2 — owner fallback grants Admin permissions only without an active admin | `owner_gets_admin_perms_without_admin` | Passed in the 16-test API suite. |
| AC3 — CSRF/Origin and inactive membership are rejected without mutation | `switch_role_requires_csrf`; `switch_role_rejects_inactive_role`; `role_switch_csrf_guard_runs_before_tenant_resolution` | Passed. |
| Verified Actor extraction and permission checks | `actor_requires_authenticated_extension`; `actor_requires_registered_permission`; `maybe_actor_is_optional` | Passed, 3/3. |
| Bearer/role abuse paths | `bearer_actor_rejects_cross_tenant_token`; `bearer_actor_rejects_inactive_role`; `bearer_actor_rejects_stale_password_version`; malformed transport and redaction tests | Passed. |
| Session abuse paths | `switch_role_rejects_expired_session`; `switch_role_rejects_revoked_session` | Passed. |
**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-core auth::tests -- --test-threads=1` — 3 passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity defaults_schema_and_summary_match_contract -- --test-threads=1` — 1 passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity api_auth::tests:: -- --test-threads=1` — 8 passed.
- ✅ `DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a5' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a5' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='https://example.test' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` — 16 passed, 0 failed on a fresh PostgreSQL 16.15 task database. `cargo run --locked --offline -p eq-server -- migrate` applied migrations first; the database was dropped after confirming zero active connections.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline -p eq-core -p eq-identity --all-targets -- -D warnings`.
- ✅ `scripts/tasks-index.sh --check`; `python3 -I scripts/tasks-lint.py`; `python3 -I -B scripts/tests/test_tasks_lint.py` (8/8).
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && python3 -I -B scripts/erd.py --check && git diff --check`.
- ✅ Child task Reports record their staged Gitleaks checks; ✅ `gitleaks dir --redact --no-banner tasks/M1-core/T-110a5-actor-permissions-and-role-switch.md` — no leaks. The final staged scan is run before this task's commit.
- ✅ Child T-110a4a3 ran `E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4a3' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4a3' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' RITMA_LISTEN='127.0.0.1:8080' E2E_BASE_URL='http://127.0.0.1:8080' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true make e2e` — 4/6 on the child task's fresh PostgreSQL 16.15 database. Chromium 3/3 and WebKit health 1/1 passed; WebKit `loads home` and `spa fallback` are Not run because `libWPEWebKit-2.0.so.1`, ICU 74 and Flite are absent on this host.
- Not run: `cargo test --workspace`; workspace DB tests start Testcontainers containers, prohibited by the disk-hygiene rule. All split-part named unit and API tests were run using the fixed PostgreSQL container.
- Not run: `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check`; this aggregate changes no web source. The child E2E ran `pnpm -C web build` successfully.
- Not run: a separate `make dev` server process or a second `make e2e` invocation for this report-only aggregate; the child API task already exercised the production route over TCP and ran the browser E2E command above.
- n/a: `scripts/erd.py --migrations` (no migration changed); SQLx prepare and `make api` (no query/API contract changed); hosted GitHub Actions (not available locally).
**Manual checks** — Created `ritma_test_T-110a5` from `template0` inside the existing `ritma-db-1` container, applied the current migrations using the owner URL, and ran the 16-test Identity API suite against PostgreSQL 16.15 with the app URL using `eq_app` (`NOSUPERUSER`, `NOBYPASSRLS`). The suite issued real localhost HTTP requests and verified current-token Actor insertion, cross-tenant/stale/inactive rejection, owner permission fallback, role-switch CSRF, inactive membership, expiry/revocation, session family/digest preservation and audit. Dropped the task database after confirming no active connections. The child A3 Report records its separate `make e2e` command and 4/6 result.
**Debug log** — None; task-scoped migrations, all named tests, Clippy and guards passed without a failed attempt. The inherited child A3 Report records the two unavailable WebKit page launches and their missing host-library reason.
**Performance** — n/a; this aggregate adds no runtime code or query.
**Security** — Triggers T1/T3/T4/T8. SBD-81 aggregate review: verified Actor construction remains behind bearer signature, tenant, credential-version and active-membership checks; permissions come from enabled modules and the active role; owner fallback is conditional on no active admin; cookie role switch requires exact CSRF/Origin and active membership; refresh credentials remain hashed/redacted; only `active_role` changes and audit shares its transaction. Abuse tests cover cross-tenant/stale/inactive tokens, missing auth, invalid CSRF, expired/revoked sessions, role isolation and owner fallback removal. No new grant, schema object, event, error code or secret behavior was introduced, and no PII/secret was found in the scanned task report.
**Deviations** — RATIFY inherited TD-117, TD-118, TD-119, TD-120, TD-121, TD-122, TD-123, TD-124, TD-125, TD-126, TD-127 and TD-128. They record the approved split, spec-defined Admin grant, secret redaction, child ordering, shipped login route, CACHE-14 keys, route registration, cookie-session authorization, split size and pre-tenant CSRF guard. Full rationale is in `tasks/DECISIONS.md` and the owning task Questions. This aggregate adds no new decision. The two WebKit page checks remain Not run for missing host libraries.
**Dependencies affected** — T-110a aggregates all identity implementation parts; T-110b remains responsible for refresh rotation and token invalidation; T-111 must invalidate the password-version cache after password change/reset. No shipped interface changed.
**Rollback plan** — Revert the signed aggregate report commit; no application or schema state needs rollback.
**Follow-ups** — Architect ratification of TD-117–TD-128; T-110b/T-111 follow-ups noted above; rerun the two WebKit page checks on a host with Playwright's required libraries.
**Resources** — Before: `du -sh "$HOME/.cache/ritma/target"` → 13G; `docker system df` → Images 20/7 active/15.5GB (2.073GB reclaimable), Containers 22/14/10.51MB, Local Volumes 448/19/75.09GB (68.81GB reclaimable), Build Cache 0; filesystem had 54G available. After: target 13G; Docker → Images 20/7/15.5GB, Containers 22/14/10.51MB, Local Volumes 448/19/75.09GB, Build Cache 0; filesystem had 54G available. Created and dropped only `ritma_test_T-110a5` inside existing `ritma-db-1`; no container, image, volume or target directory was created or removed. All Docker container/image inventories were unchanged.
**Workflow steps 0–12** — 0: confirmed the current repo and `dev`, fetched origin, verified both dependencies were `review`, promoted this aggregate under its Split condition and regenerated the index; unrelated `.pending`, `.hold` and Python cache WIP stayed untouched. 1: read this task and its cited identity/auth/canonical/security/quality specs. 2: read T-110a4a and T-110a4b Reports, all relevant child Reports, inherited deviations/follow-ups and shipped Actor, permission, bearer and role-switch code. 3: n/a, backend-only. 4: recorded the aggregate Plan, Assumptions and AC-to-test plan above. 5: n/a, implementation was completed by the two split parts; this aggregate performed verification only. 6: ran the production HTTP routes through the localhost TCP integration suite on a fresh PostgreSQL database; the child A3 part ran the browser E2E. 7: no failure required retries; host-limited WebKit checks are recorded as Not run. 8: reviewed token validation, permission derivation, tenant scope, CSRF ordering, session mutation and audit against the cited standards. 9: n/a, no runtime change or performance target. 10: completed the SBD-81 review above. 11: reran actor/permission/API tests, fmt, focused Clippy, task lint/index and tests, all four guards, ERD check and diff; recorded workspace, web, WebKit and hosted-CI checks as Not run with reasons. 12: completed this Report, set `review`, regenerated the index, and prepared the single signed task commit for `dev`.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification; TD-117):** Actor verification/permission caching and cookie role switching exceed CQ-101's approximate 400-line guide when reviewed together → split into T-110a4a and T-110a4b, preserve original AC/test ownership, and leave this parent as a final aggregate because CQ-101 requires large concerns to be split before implementation.
- **Self-resolved (implementer, run 8, needs architect ratification; TD-119):** the task said Origin matching was required only "where supplied", while IDN-05 and AUTH-05 require the cookie endpoint's Origin to match the configured public origin → reject a missing or mismatched Origin together with a missing/invalid CSRF header, because `spec/25` §3.3 and `spec/23` CAN-34 define the stricter rule.
- **Self-resolved (implementer, run 8, needs architect ratification; TD-120):** task-index validation rejects a task depending on a higher-sorting ID → place the two split parts at T-110a4a/T-110a4b, after their T-110a4 dependency and before aggregate T-110a5, because `scripts/tasks-index.sh` enforces this acyclic ordering.

## Review
Ready for maintainer review. All tests listed across both split parts passed in the focused suites. The sibling E2E report is 4/6 with two WebKit page tests Not run for missing host libraries. Ratify TD-117–TD-128.
