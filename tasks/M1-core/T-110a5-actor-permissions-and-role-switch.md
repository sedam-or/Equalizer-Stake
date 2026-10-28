---
id: T-110a5
title: Add Actor permissions and role switching
milestone: M1
status: todo
depends_on: [T-110a4, T-110a2]
spec_refs: [IDN-05, IDN-09, IDN-10, IDN-11, AUTH-02, AUTH-05, STM-22, CAN-34, SBD-40, SBD-42, SBD-53]
estimated_size: M
---

## Goal
Provide the shared `eq_core::auth::Actor` extractor and identity role-switch behavior. Derive effective permission strings from active role and enabled module permissions, including the computed owner-without-admin rule.

## Context
The parent T-110a contract defines `Actor { person_id, tenant, role, session_id, perms, ctx }` and `MaybeActor`. `Actor::require` returns `AppError::Forbidden`; `has` checks the permission set. Permissions are the union of `PermissionDef.roles` containing the active role in enabled modules. If the tenant has no active admin, owner receives the admin permission set; this is computed, never stored. Deactivated membership is rejected. `POST /auth/switch-role {role}` issues a new token, updates `refresh_sessions.active_role`, and retains the same refresh family.

## Scope
### In
- Add Actor/MaybeActor extraction, permission computation and tests.
- Add role-switch endpoint and CSRF checks required by IDN-05/CAN-34.
### Out
- Refresh rotation, logout and password-change invalidation (T-110b); people APIs (T-112).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/auth.rs`, `crates/eq-core/src/lib.rs`, `crates/eq-core/src/app_state.rs` | create/modify | Shared Actor extraction and request auth context. |
| `crates/eq-api/src/lib.rs`, `crates/eq-api/src/modules.rs` | modify | Inject registry/enabled-module context for permission calculation. |
| `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/service.rs`, `crates/eq-identity/src/jwt.rs` | modify | Role switch, token claims and current session checks. |
| `crates/eq-identity/tests/api_auth.rs` | modify | Role-switch and permission acceptance tests. |
| `crates/eq-identity/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | modify | Only dependencies required by this contract. |
| `tasks/M1-core/T-110a5-actor-permissions-and-role-switch.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Task state and decisions. |

## Interfaces
Use `ActorRole`/`Role`, `ModuleRegistry`, `EnabledModuleSet`, `PermissionDef`, the existing tenant resolver and the parent `POST /auth/switch-role` wire shape. Do not add permission keys or role grants.

## Business rules
The role privilege ordering and owner fallback are exactly IDN-09/11. Role switch accepts only an active membership role; no new role is granted.

## Security
- Triggers: T1, T3, T4, T8 (SBD-30).
- Data classes touched: S1/S2/S3; bearer/refresh tokens remain secret and PII remains tenant-scoped.
- Role change requires cookie CSRF header and matching Origin where supplied; permission checks use the active tenant and enabled module set (IDN-05/SBD-40/42).
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
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `switch_role`, `owner_gets_admin_perms_without_admin`, `switch_role_requires_csrf` |
| `crates/eq-core/src/auth.rs` | `actor_requires_registered_permission`, `maybe_actor_is_optional` |

## Definition of done
- [ ] Actor, permission and role-switch tests pass, or database/runtime checks are `Not run` under the resource rule.
- [ ] `scripts/check-layers.sh`, `scripts/check-web-boundaries.sh`, task lint/index and `cargo fmt --all --check` pass.
- [ ] No new permission key, role grant, schema, error code or cross-module SQL is introduced.
- [ ] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [ ] Signed commit `feat(identity): add Actor permissions and role switching [T-110a5]` pushed to `dev`.

## Report
**Plan** — Add the shared verified Actor extractor, permission derivation and role-switch endpoint; test permissions, role membership and CSRF behavior.
**Assumptions** — The API composition layer supplies the compiled module registry and enabled-module service; the owner grant is computed from current tenant membership.
**Test plan** — Exercise active and inactive roles, permission shrink, owner fallback before/after admin membership, and CSRF/Origin rejection.

## Questions
None.

## Review
n/a — pending implementation.
