---
id: T-110a4a
title: Verify Actor authentication and permission composition
milestone: M1
status: draft
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
- [ ] T-110a4A/B/C are all review; each named check is green or explicitly Not run under the disk rules.
- [ ] Combined auth tests pass on a fresh `ritma_test_T-110a4a` database in the existing PostgreSQL 16 container, and that database is dropped afterward.
- [ ] Manual HTTP checks cover a valid bearer, invalid/cross-tenant bearer, and the owner permission snapshot.
- [ ] No schema change, unregistered permission, new error code, or cross-module SQL was introduced.
- [ ] Report includes Workflow steps 0–12, exact checks, and resource before/after data.
- [ ] Signed aggregate commit `test(identity): verify Actor authentication and permissions [T-110a4a]` pushed to `dev`.

## Report
**Plan** — (1) Read all child Reports and shipped code → verify the split preserves the original contract. (2) Run the combined identity API tests on a new task-named database inside the existing PostgreSQL 16 container → verify AC1–AC3. (3) Send real requests for valid, malformed and cross-tenant bearer credentials → verify fail-closed transport behavior. (4) Re-run Rust, task, SQLx, security and guard checks → record each result and the resource snapshot.
**Assumptions** — The child tasks preserve the original ACs and named tests. Cache intervals and permission grants are fixed by the cited specs.
**Test plan** — AC1: missing Actor extension is rejected while MaybeActor is optional → the A tests. AC2: Admin gets the defined write permission and owner fallback is conditional → the B tests. AC3: invalid or stale bearer credentials never insert an Actor → the C tests. Edges: empty/malformed header, absent credential row, expired token, disabled module and active/inactive membership. Errors: existing unauthenticated/forbidden codes only.

**Summary** — Pending aggregate verification after T-110a4A/B/C.
**Spec read** — spec/modules/identity.md §3, spec/25-auth-and-access-control.md §§3.1/4.6, and the referenced IDN/AUTH/SBD/CAN sections in the child tasks.
**Files** — Task reports and code are owned by the three split parts; this task verifies their combined behavior and updates its report/index.
**Acceptance criteria → tests** — Pending aggregate run; see the ownership table above.
**Checks run** — Pending aggregate run; each unavailable check will be marked Not run with its reason.
**Manual checks** — Pending valid/invalid bearer and permission HTTP checks.
**Debug log** — None.
**Performance** — n/a; no query or hot path is added by the aggregate.
**Security** — Auth and permission controls are reviewed in each split task; aggregate SBD-81 check remains pending.
**Deviations** — RATIFY TD-118, TD-121 and TD-122.
**Dependencies affected** — T-110a4b and T-110a5 consume this aggregate Actor/permission contract.
**Rollback plan** — Revert the signed aggregate commit; no database schema changes exist.
**Follow-ups** — Human ratification of TD-118, TD-121 and TD-122; review exact cache invalidation ownership with T-110b.
**Workflow steps 0–12** — Pending aggregate completion.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification; TD-118):** the parent task says not to add permission keys/grants, while its AC requires `identity.people.write` and the IdentityModule currently declares none → register only the already-specified key with its exact admin grant and rely on the mapped IDN-11 owner exception, because `spec/modules/identity.md` §3 and `spec/25` §6 define both; no permission is invented.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-121):** bearer tokens enter through an HTTP header as plain text, while SBD-50 requires secret-safe handling → wrap the parsed token in `SecretString` before verification and test that Debug formatting omits a fake token.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** the original lowercase aggregate ID must sort after new subtasks under the task-index DAG guard → preserve `T-110a4a` and use uppercase-suffix `T-110a4A/B/C`, which sort before lowercase `a` with the script's fixed C byte order; this avoids renumbering the existing role-switch dependency and leaves the guard unchanged.

## Review
n/a — pending implementation.
