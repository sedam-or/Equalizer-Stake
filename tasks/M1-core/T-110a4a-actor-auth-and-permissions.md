---
id: T-110a4a
title: Verify Actor authentication and permission composition
milestone: M1
status: review
depends_on: [T-110a4C]
spec_refs: [IDN-03, IDN-10, IDN-11, AUTH-01, AUTH-05, SBD-01, SBD-40, SBD-42, SBD-53, CAN-30]
estimated_size: S
---

## Goal
Verify the shared Actor request extractors, permission grants, and bearer authentication implemented by the three parts below. Keep this aggregate as the original contract and run the complete auth acceptance set after every part is reviewed.

## Context
The T-110 parent defines `Actor { person_id, tenant, role, session_id, perms, ctx }`, `Actor::require`, `Actor::has`, and `MaybeActor(pub Option<Actor>)`. IDN-03 requires signature, expiry, tenant and password-version validation. IDN-10 grants only the active role's permissions from enabled modules. IDN-11 adds the admin set to an owner only when there is no active tenant admin; the result is computed and cached for 30 seconds. Spec 25 §3.1 requires active membership verification and a 60-second password-version cache.

## Split
This task exceeded CQ-101's approximately 400-line review guide. Keep the original lowercase ID as the aggregate; complete its ordered implementation parts first and preserve each original acceptance criterion and named test.

| Part | Responsibility | Depends on | Owned tests |
|---|---|---|---|
| T-110a4A | Core Actor types and request extractors | T-110a4, T-110a2 | `actor_requires_authenticated_extension`, `actor_requires_registered_permission`, `maybe_actor_is_optional` |
| T-110a4B | Spec-defined Identity write permission and owner permission test | T-110a4A | `defaults_schema_and_summary_match_contract`, `owner_gets_admin_perms_without_admin` |
| T-110a4C | Verified bearer authentication, caches and middleware wiring | T-110a4B | bearer abuse and redaction tests |

## Scope
### In
- Re-run the original auth acceptance tests after T-110a4A, T-110a4B and T-110a4C are all reviewed.
- Review the composed implementation against the parent T-110 contract and the cited authentication/permission specs.
### Out
- Reimplementing behavior owned by the split parts; cookie role switching and CSRF (T-110a4b); refresh rotation/logout/password-change behavior (T-110b); and people APIs (T-112).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/auth.rs`, `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/tests/api_auth.rs` | read/run | Verify all three parts together against the original contract. |
| `tasks/M1-core/T-110a4a-actor-auth-and-permissions.md`, `tasks/M1-core/T-110a4A-actor-extractors.md`, `tasks/M1-core/T-110a4B-identity-permissions.md`, `tasks/M1-core/T-110a4C-bearer-authentication.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | read/generate | Split map, decisions and aggregate verification. |

## Interfaces
- `Actor` exposes typed getters for `person_id`, tenant, role, session id, permission set and `ActorCtx`; its authenticated construction path is used only after the identity boundary verifies the token and membership.
- `Actor::has(&str) -> bool` checks the computed permission set; `Actor::require(&'static str) -> Result<(), AppError>` returns the registered `forbidden` error when absent.
- `MaybeActor(pub Option<Actor>)` returns `None` only when no bearer credential was supplied; a malformed or invalid supplied credential is rejected.
- Identity authentication is composed through the shipped API module registry, tenant resolver and enabled-module service. Core continues to own its membership query through `CoreIdentityDirectory`.

## Business rules
1. Effective permissions are only grants for the active role in enabled modules (IDN-10).
2. Scopes remain enforced by later services.
3. Owner fallback uses exactly IDN-11 and the 30-second cache specified in `spec/25` §3.1/§4.6; never persist the fallback grant.
4. Credential `ver` follows the parent T-110 contract: `password_changed_at` Unix seconds, or zero when absent; its cache is 60 seconds as specified by IDN-03.
5. The aggregate adds no permission key, grant, role, schema object or error code.

## Security
- Triggers: T1, T3, T8 (SBD-30).
- Data classes touched: S1 actor context, S2 tenant/person identifiers, S3 bearer token and derived credential material.
- Spoofing → verify HS256 signature and canonical Bearer transport before constructing Actor (IDN-03/CAN-30); extracted bearer material uses `SecretString` and redacts Debug output (SBD-50).
- Elevation → compare signed tenant with resolved tenant, password version with current credential version, and role with an active membership (AUTH-05/SBD-01).
- Information disclosure → never include token material in Actor or middleware Debug/logging (SBD-21/50).
- Abuse acceptance: `bearer_actor_rejects_cross_tenant_token`, `bearer_actor_rejects_inactive_role`, `bearer_actor_rejects_stale_password_version`, and `owner_gets_admin_perms_without_admin`.

## Implementation notes
1. Treat the child Reports as the implementation record and this task as the end-to-end acceptance check.
2. Preserve module-gate ordering, tenant context and the Core-owned membership boundary.

## Acceptance criteria
- AC1. The required extractor rejects a missing verified Actor, the optional extractor returns None, and permission checks deny absent grants → T-110a4A tests.
- AC2. Identity exposes only the spec-defined Admin grant, while the owner fallback applies only when no active admin exists → T-110a4B tests.
- AC3. Cross-tenant, stale-version, inactive-role and malformed bearer credentials fail closed; a valid token produces the verified Actor extension → T-110a4C tests and manual checks.

## Tests to write
| File | Test names |
|---|---|
| T-110a4A | `crates/eq-core/src/auth.rs` | `actor_requires_authenticated_extension`, `actor_requires_registered_permission`, `maybe_actor_is_optional` |
| T-110a4B | `crates/eq-identity/src/module.rs` and `crates/eq-identity/tests/api_auth.rs` | `defaults_schema_and_summary_match_contract`, `owner_gets_admin_perms_without_admin` |
| T-110a4C | `crates/eq-identity/src/api_auth.rs` and `crates/eq-identity/tests/api_auth.rs` | `bearer_actor_rejects_cross_tenant_token`, `bearer_actor_rejects_inactive_role`, `bearer_actor_rejects_stale_password_version`, `bearer_token_debug_redacts_credentials`, `malformed_bearer_header_rejected` |

## Definition of done
- [x] T-110a4A/B/C are all review; each named check is green or explicitly Not run under the disk rules.
- [x] Combined auth tests pass on a fresh `ritma_test_T-110a4a` database in the existing PostgreSQL 16 container, and that database is dropped afterward.
- [x] Manual HTTP checks cover a valid bearer, invalid/cross-tenant bearer, and the owner permission snapshot.
- [x] No schema change, unregistered permission, new error code, or cross-module SQL was introduced.
- [x] Report includes Workflow steps 0–12, exact checks, and resource before/after data.
- [x] Signed aggregate commit `test(identity): verify Actor authentication and permissions [T-110a4a]` pushed to `dev`.

## Report
**Plan** —
1. Read the A/B/C Reports, decisions and shipped code → verify the split covers the original Actor/auth contract.
2. Run the named extractor, permission and bearer tests together → verify AC1–AC3.
3. Run the auth API suite against a fresh task-named database in the existing PostgreSQL 16 container → verify the HTTP acceptance paths and owner permission snapshot.
4. Run formatting, workspace Clippy, task checks, guards, ERD and a secret scan → verify the aggregate against the quality and security gates.
**Assumptions** — The split parts preserve the original acceptance criteria. Cache intervals and permission grants are fixed by the cited specs; this aggregate introduces no new behavior.
**Test plan** —
- AC1, Given no verified Actor extension or a missing grant, When the required extractor or permission check runs, Then it returns the registered error; the optional extractor returns `None` without an extension → `actor_requires_authenticated_extension`, `actor_requires_registered_permission`, `maybe_actor_is_optional`.
- AC2, Given the Identity module registry and owner/admin membership state, When permission behavior is checked, Then the exact Admin grant is registered and the owner fallback is conditional → `defaults_schema_and_summary_match_contract`, `owner_gets_admin_perms_without_admin`.
- AC3, Given valid, malformed, cross-tenant, stale-version or inactive-role bearer credentials, When the protected route is requested, Then only the valid token inserts Actor and the invalid cases return 401 → C bearer tests.
- Edges: missing Authorization passes public login; absent credential row, expiry, invalid signature, malformed transport and inactive role fail closed; owner fallback is absent when an active admin exists.
- Error cases reuse `unauthenticated` and `forbidden`; no new code is introduced. Abuse cases are the A/B/C negative and redaction tests.
**Summary** — The combined Actor, permission and bearer contract passes its named unit and API tests. A current same-tenant token produces a verified Actor, while malformed, cross-tenant, stale and inactive credentials fail closed; the existing owner fallback remains conditional.
**Workflow steps 0–12** — 0: confirmed `dev`, fetched origin/index, and preserved the existing unrelated audit/Python WIP; promoted this aggregate after its dependency T-110a4C was `review`, then set it `in_progress`. 1: read the full task and cited contracts. 2: read all three child Reports, shipped code and deviations/follow-ups. 3: n/a, backend-only. 4: recorded the Plan and Given/When/Then test plan above before verification. 5: n/a, no source changes; all implementation remains in the child tasks. 6: sent real localhost HTTP requests through the production bearer middleware in the task test; the owner snapshot test exercised the login route. 7: all task-specific tests passed on the first run; no debug retries were needed. 8: reviewed the combined implementation against the original parent contract and cited architecture/security rules. 9: n/a; this verification adds no query or hot path. 10: completed the aggregate SBD-81 check below. 11: ran the aggregate Rust, task, guard and resource checks below. 12: completed this Report, set status to review, regenerated the index; one signed commit and push follow.
**Spec read** — `spec/modules/identity.md` §§1–3; `spec/25-auth-and-access-control.md` §§1–4.6 and §6; `spec/23-contracts-and-canonical-forms.md` CAN-30; `spec/26-containers-and-caching.md` CACHE-14; `spec/18-security-by-design.md` §§5 and 8 (SBD-81); `spec/16-code-quality.md` §§9 and 12; `spec/17-clean-architecture.md` §§1, 4 and 7; `spec/19-error-handling.md` §§2 and 10. The child Reports also record their spec excerpts.
**Files** — Modified `tasks/M1-core/T-110a4a-actor-auth-and-permissions.md` and generated `tasks/INDEX.md`; no application source, migration or API contract changed.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `actor_requires_authenticated_extension`, `actor_requires_registered_permission`, `maybe_actor_is_optional` | Passed, 3/3. |
| AC2 | `defaults_schema_and_summary_match_contract`, `owner_gets_admin_perms_without_admin` | Passed, 1 manifest test plus owner fallback in the 10-test API suite. |
| AC3 | `bearer_actor_accepts_current_token`, `bearer_actor_rejects_cross_tenant_token`, `bearer_actor_rejects_inactive_role`, `bearer_actor_rejects_stale_password_version`, `bearer_actor_rejects_malformed_transport`, parser/redaction tests | Passed; valid token reached the Actor handler, invalid credentials returned 401 `unauthenticated`. |
**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-core auth::tests -- --test-threads=1` (3 passed).
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity defaults_schema_and_summary_match_contract -- --test-threads=1` (1 passed).
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity api_auth::tests:: -- --test-threads=1` (5 passed).
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` (10 passed, 0 failed) on fresh `ritma_test_T-110a4a` in reused PostgreSQL 16.15 container `ritma-db-1`; migrations were applied and the database was dropped.
- ✅ `cargo deny check` (advisories, bans, licenses and sources passed; existing duplicate-base64 warning only).
- ✅ `scripts/tasks-index.sh --check`; ✅ `python3 -I scripts/tasks-lint.py`; ✅ `python3 -I -B scripts/tests/test_tasks_lint.py` (8 passed).
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && python3 -I -B scripts/erd.py --check && python3 -I -B scripts/erd.py --migrations`.
- ✅ `git diff --check`.
- Not run: `cargo test --workspace`; the suite uses `TestDb::new()`/testcontainers and would create per-invocation PostgreSQL containers, prohibited by the disk rule. The aggregate's named A/B/C tests were run against the existing fixed container instead.
- Not run: web checks; `web/` was not touched. Not run: `make api`/SQLx regeneration; this aggregate changes neither API code nor queries. Not run: `make dev`/`make e2e`; the named reqwest test sends actual local HTTP requests and this is not the milestone integration task. Not run: hosted CI; requires GitHub Actions.
- ✅ `gitleaks protect --staged --redact` (no leaks found).
**Manual checks** — The task-scoped reqwest suite sent HTTP requests over localhost TCP through the production bearer middleware. A valid current token reached the protected test handler (200 with the expected permission); malformed, cross-tenant, inactive-role and stale-version tokens returned 401 `unauthenticated`. The owner-without-admin API test verified the login permission snapshot against the fresh task database; the owner-with-admin variant omitted the Admin grant.
**Debug log** — None; all named tests and checks passed on the first run. Full workspace runtime tests were not run for the explicit testcontainer restriction above.
**Performance** — n/a; this aggregate adds no runtime code or query.
**Security** — Triggers T1/T3/T8. SBD-81 aggregate review: the ModuleGate precedes the bearer middleware; tenant comes from resolved context and must match JWT `tid`; Actor is inserted only after signature, expiry, credential-version and active-membership checks; the Identity permission is the spec-defined Admin grant with the conditional IDN-11 owner fallback; secrets use redacted storage; invalid auth uses existing 401/403 codes; SQL remains bound and module-owned. Abuse tests cover absent identity/grant, owner-with-admin, malformed transport, cross-tenant, stale version, inactive role and token redaction. No new schema, grant, error code, secret or SQL behavior was added.
**Deviations** — RATIFY inherited from the child parts: TD-118 (register the already-specified Identity write permission), TD-121 (use `SecretString` and redacted Debug), TD-122 (uppercase child suffix ordering), TD-123 (assert the shipped login route rather than the stale empty-route assertion), and TD-124 (use CACHE-14 auth-cache keys). The owning child tasks and `tasks/DECISIONS.md` contain the full decisions.
**Dependencies affected** — T-110a4b and T-110a5 consume the verified Actor/permission contract.
**Rollback plan** — Revert the signed aggregate commit; it changes no database schema or application code.
**Follow-ups** — Ratify TD-118/121/122/123/124. T-111 must invalidate the password-version cache after password changes/resets; T-110b must preserve its own IDN-03 token-invalidation requirements. No follow-up changes were made here.
**Resource and disk hygiene** — Before: fixed Cargo target `11G`; `df -h .` showed 58G available; Docker usage: Images 20 / 15.5GB (2.073GB reclaimable), Containers 17 / 10.48MB, Volumes 443 / 74.55GB (68.81GB reclaimable), Build Cache 0. After: target `11G`, 58G available, and the same Docker counts/sizes; container/image inventories were unchanged. Reused `ritma-db-1` and its existing volume. Created `ritma_test_T-110a4a` once from `template0`, applied migrations, ran the combined tests and dropped the DB; query confirmed it is absent. No image, container, volume or Cargo target directory was created or removed.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification; TD-118):** the parent task says not to add permission keys/grants, while its AC requires `identity.people.write` and the IdentityModule currently declares none → register only the already-specified key with its exact admin grant and rely on the mapped IDN-11 owner exception, because `spec/modules/identity.md` §3 and `spec/25` §6 define both; no permission is invented.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-121):** bearer tokens enter through an HTTP header as plain text, while SBD-50 requires secret-safe handling → wrap the parsed token in `SecretString` before verification and test that Debug formatting omits a fake token.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** the original lowercase aggregate ID must sort after new subtasks under the task-index DAG guard → preserve `T-110a4a` and use uppercase-suffix `T-110a4A/B/C`, which sort before lowercase `a` with the script's fixed C byte order; this avoids renumbering the existing role-switch dependency and leaves the guard unchanged.

## Review
n/a — awaiting maintainer review.
