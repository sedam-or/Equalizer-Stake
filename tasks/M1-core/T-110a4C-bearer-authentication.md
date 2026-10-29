---
id: T-110a4C
title: Authenticate bearer requests before extracting Actor
milestone: M1
status: todo
depends_on: [T-110a4B]
spec_refs: [IDN-03, IDN-10, IDN-11, AUTH-01, AUTH-05, SBD-01, SBD-40, SBD-42, SBD-53, CAN-30]
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
| crates/eq-identity/tests/api_auth.rs | bearer_actor_rejects_cross_tenant_token, bearer_actor_rejects_inactive_role, bearer_actor_rejects_stale_password_version |

## Definition of done
- [ ] Named tests pass on a fresh ritma_test_T-110a4C database in the existing PostgreSQL 16 container; drop it afterward.
- [ ] SQLx metadata is regenerated/checked; cargo fmt, focused Clippy, cargo deny, task lint/index and guards pass.
- [ ] Real HTTP requests verify valid and invalid bearer behavior; no new schema or error code is added.
- [ ] Report includes Workflow steps 0–12 and resource snapshots.
- [ ] Signed commit feat(identity): authenticate verified bearer requests [T-110a4C] is pushed to dev.

## Report
**Plan** — (1) Validate token transport and JWT claims → verify parser and token tests. (2) Check active role and credential version through their owning repositories → verify database abuse tests. (3) Derive permission set and wire middleware after ModuleGate → verify actor insertion and response behavior. (4) Regenerate SQLx metadata and run security/guard checks.
**Assumptions** — Current API composition supplies the tenant-resolved extension and the already-compiled enabled-module set; no new role or permission is introduced.
**Test plan** — AC1: same-tenant current token is accepted; cross-tenant/stale versions are denied → bearer integration tests. AC2: inactive role or malformed header is 401 → bearer tests. AC3: enabled-module permissions and owner fallback remain exact → owner test. Edge: missing token, malformed scheme, missing credentials row, expired token and disabled module.
**Spec read** — spec/modules/identity.md, spec/25 §§3.1/4.6, CAN-30, SBD-81, spec 16 §9/12, spec 17 §7, spec 19 §10 and ERD core persons dictionary.
**Security** — SBD-81 review planned; JWT, tenant, membership, cache and secret handling are covered by the abuse tests.

## Questions
- **Self-resolved (implementer, run 9, needs architect ratification; TD-121):** parsed bearer text must not remain a plain String → wrap it in SecretString and expose only at the verifier call, per SBD-50.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** ordered child C must sort before lowercase aggregate T-110a4a → uppercase C does under the task index's fixed C byte order; no guard is changed.

## Review
n/a — awaiting maintainer review.
