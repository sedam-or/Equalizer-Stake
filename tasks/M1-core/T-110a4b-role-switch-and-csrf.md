---
id: T-110a4b
title: Add cookie role switching and CSRF checks
milestone: M1
status: todo
depends_on: [T-110a4a]
spec_refs: [IDN-05, IDN-09, IDN-10, AUTH-03, CAN-30, CAN-34, STM-22, SBD-40, SBD-53]
estimated_size: M
---

## Goal
Implement `POST /api/v1/auth/switch-role` using the existing refresh cookie, issue an access token for the selected active role, and retain the existing refresh-session family. Require the exact CSRF header and matching configured Origin.

## Context
IDN-09 allows switching only to an active membership role and requires the new token to carry that role while retaining the same refresh family. IDN-05 and `spec/25` §3.3 require both `X-EQ-CSRF: 1` and an `Origin` equal to the configured public URL origin. `spec/modules/identity.md` defines the request `{role}` and login-response body; `security.role_switched` is the mapped audit action. The refresh session's columns are fixed by spec 06 §4.2 and ERD-04; no schema change is allowed.

## Scope
### In
- Add the role-switch HTTP route with request validation, refresh-cookie authentication and exact CSRF/Origin validation.
- Verify requested role is an active membership; update only the existing refresh session's `active_role`, keep its `family_id` and refresh token unchanged, and issue a new role-bound access token.
- Record `security.role_switched` through the shipped audit API in the same transaction.
- Add the named role-switch API tests and abuse tests.
### Out
- Refresh rotation, logout, password-change invalidation (T-110b), people APIs (T-112), new schema, new event names or new permission grants.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/service.rs`, `crates/eq-identity/src/repo.rs`, `crates/eq-identity/src/jwt.rs` | modify | Cookie auth, role-switch transaction/repository, token issue and response. |
| `crates/eq-identity/tests/api_auth.rs` | modify | Role-switch, CSRF and inactive-role acceptance/abuse tests. |
| `.sqlx/` | modify | Generated metadata for changed SQL macros. |
| `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/M1-core/T-110a5-actor-permissions-and-role-switch.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Task report, aggregate status and recorded decisions. |

## Interfaces
- `POST /api/v1/auth/switch-role` accepts a JSON object containing exactly the canonical `Role` and returns the existing login-response body shape.
- Cookie: `eq_rt`; required headers: `X-EQ-CSRF: 1`, `Origin: <RITMA_PUBLIC_URL origin>`.
- Invalid/missing session → existing `unauthenticated`; invalid CSRF or inactive target role → existing `forbidden`.

## Business rules
1. Role must be an active membership of the cookie session's person and tenant (IDN-09).
2. The selected role is the only permission-grant source (IDN-10); no role grant is added by switching.
3. The same `refresh_sessions` row and family remain active; only `active_role` changes.

## Security
- Triggers: T1, T3, T4, T8 (SBD-30).
- Data classes touched: S1 role, S2 person/tenant ids, S3 refresh token and access token.
- Spoofing → hash and compare the opaque cookie credential; require the role to be an active membership (IDN-04/09).
- CSRF → require both exact header and matching Origin; missing or malformed values fail closed (IDN-05/AUTH-05).
- Tampering/replay → update session and audit in one tenant transaction; do not rotate or change its family (STM-22/SBD-53).
- Abuse acceptance: `switch_role_requires_csrf`, `switch_role_rejects_inactive_role`, and `switch_role_keeps_refresh_family`.

## Implementation notes
1. Parse the `eq_rt` cookie as the existing canonical base64url refresh token and hash the decoded 32 bytes as T-110 IDN-04 does.
2. Lock the matching active session row; compare expiry and role membership before changing its role.
3. Keep the audit write in the transaction and use the exact `security.role_switched` action from the parent contract.
4. Build the response with the already-shipped login-response fields and the new access token; preserve the existing cookie/family.

## Acceptance criteria
- AC1. Given active admin and guardian memberships, a valid session can switch to guardian; response token role and permissions are guardian-only, `active_role` is updated, and the same refresh family remains → `switch_role`.
- AC2. Missing/wrong CSRF header or absent/mismatched Origin is rejected before DB mutation → `switch_role_requires_csrf`.
- AC3. A requested inactive/nonmember role is forbidden and leaves session state unchanged → `switch_role_rejects_inactive_role`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `switch_role`, `switch_role_requires_csrf`, `switch_role_rejects_inactive_role`, `switch_role_keeps_refresh_family` |

## Definition of done
- [ ] Named role-switch and abuse tests pass on a disposable PostgreSQL 16 task database; drop the task database after checks.
- [ ] Real HTTP request verifies success plus CSRF and inactive-role error paths.
- [ ] `scripts/erd.py --migrations` is not applicable because no migration changes; SQLx metadata is regenerated for query changes.
- [ ] `cargo fmt --all --check`, focused Clippy, task lint/index and required guard scripts pass.
- [ ] No new schema, permission grant, event or error code; audit is transaction-bound.
- [ ] Report includes Workflow steps 0–12, resource snapshots, and exact database/test pass count.
- [ ] Signed commit `feat(identity): add role switching endpoint [T-110a4b]` pushed to `dev`.

## Report
**Plan** — (1) Validate CSRF header and Origin before reading cookie/session data → verify missing, wrong and cross-origin requests fail. (2) Add cookie digest/session lock and active-role validation in the Identity service/repository → verify invalid role does not mutate state. (3) Update the existing session role, audit in the same transaction and issue new token → verify refresh family is unchanged. (4) Return login response shape and exercise the endpoint over real HTTP on a fresh task database → verify `switch_role` and abuse cases. (5) Run focused Rust, SQLx, guard and secret checks.
**Assumptions** — The caller's refresh cookie remains the same credential after role change; the endpoint returns the same response body contract as login. Existing `unauthenticated` and `forbidden` codes cover the required errors.
**Test plan** — AC1: admin+guardian session switches to guardian; access token and permission list shrink and family id/token hash remain constant → `switch_role`, `switch_role_keeps_refresh_family`. AC2: CSRF missing/wrong and Origin missing/mismatched → `switch_role_requires_csrf`. AC3: inactive role attempt returns forbidden without state change → `switch_role_rejects_inactive_role`. Edge cases: malformed cookie, expired/revoked session, malformed/unknown role. No schema or event is added.

## Questions
None.

## Review
n/a — pending implementation.
