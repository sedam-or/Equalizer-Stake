---
id: T-110a5
title: Add Actor permissions and role switching
milestone: M1
status: draft
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
- [ ] All split-part tests pass on a disposable PostgreSQL 16 task database; the database is dropped after use.
- [ ] `scripts/check-layers.sh`, `scripts/check-web-boundaries.sh`, task lint/index and `cargo fmt --all --check` pass.
- [ ] No permission key or role grant beyond those explicitly defined in the cited specs, schema, error code or cross-module SQL is introduced.
- [ ] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [ ] Signed aggregate commit `test(identity): verify Actor permissions and role switching [T-110a5]` pushed to `dev`.

## Report
**Plan** — (1) Keep this task as the final aggregate after splitting authentication/permission derivation and cookie role switching into T-110a4a and T-110a4b → verify their Reports and dependency edges. (2) Run both parts' named tests plus the identity integration suite on a fresh task database → verify original AC1–AC3. (3) Review the joined implementation against IDN-05/09/10/11, AUTH-05, CAN-34, SBD-40/42/53 and the parent T-110 contract → verify no permission or tenant-boundary drift. (4) Regenerate/check the task index and produce the original signed aggregate commit on `dev`.
**Assumptions** — The split preserves each original AC and named test once; both parts use the already-compiled module registry and enabled-module service. Permission derivation and the owner fallback remain governed by the specs.
**Test plan** — AC1: active admin+guardian membership can switch to guardian and receives narrower permissions → `switch_role`. AC2: owner gets the spec-defined admin permission set only while the tenant lacks an active admin, under the specified 30-second permission cache → `owner_gets_admin_perms_without_admin`. AC3: missing/mismatched CSRF and inactive target role are rejected → `switch_role_requires_csrf` and role-switch abuse test. Edge cases: malformed/expired/cross-tenant tokens, missing tenant, disabled module, no active role. Error cases: registered `unauthenticated` and `forbidden` codes only. Security abuse: cross-tenant JWT, inactive membership, missing CSRF, owner permission removal after cache expiry.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification; TD-117):** Actor verification/permission caching and cookie role switching exceed CQ-101's approximate 400-line guide when reviewed together → split into T-110a4a and T-110a4b, preserve original AC/test ownership, and leave this parent as a final aggregate because CQ-101 requires large concerns to be split before implementation.
- **Self-resolved (implementer, run 8, needs architect ratification; TD-119):** the task said Origin matching was required only "where supplied", while IDN-05 and AUTH-05 require the cookie endpoint's Origin to match the configured public origin → reject a missing or mismatched Origin together with a missing/invalid CSRF header, because `spec/25` §3.3 and `spec/23` CAN-34 define the stricter rule.
- **Self-resolved (implementer, run 8, needs architect ratification; TD-120):** task-index validation rejects a task depending on a higher-sorting ID → place the two split parts at T-110a4a/T-110a4b, after their T-110a4 dependency and before aggregate T-110a5, because `scripts/tasks-index.sh` enforces this acyclic ordering.

## Review
n/a — pending implementation.
