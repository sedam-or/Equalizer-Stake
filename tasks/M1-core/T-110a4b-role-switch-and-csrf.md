---
id: T-110a4b
title: Add cookie role switching and CSRF checks
milestone: M1
status: draft
depends_on: [T-110a4a3]
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
| `crates/eq-identity/src/module.rs` | modify | Register the role-switch route in the Identity module and retain its route-surface assertion. |
| `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/service.rs`, `crates/eq-identity/src/repo.rs`, `crates/eq-identity/src/jwt.rs` | modify | Cookie auth, role-switch transaction/repository, token issue and response. |
| `crates/eq-identity/tests/api_auth.rs` | modify | Role-switch, CSRF and inactive-role acceptance/abuse tests. |
| `.sqlx/` | modify | Generated metadata for changed SQL macros. |
| `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Aggregate report, split decision and generated status index. |

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
- [ ] Signed task-report commit `docs(identity): report role-switch aggregate [T-110a4b]` pushed to `dev` after all split parts are reviewed.

## Split
The implementation diff exceeded CQ-101's approximately 400-line review guide. It is split into three ordered parts so each commit has one small responsibility:

1. `T-110a4a1` owns the refresh-session repository and role-switch service transaction.
2. `T-110a4a2` owns cookie parsing, CSRF/Origin checks, the HTTP handler, and route registration.
3. `T-110a4a3` owns the four named localhost HTTP integration tests.

This aggregate remains `draft` until all three parts reach `review`; then it can be promoted and its report will summarize the combined acceptance results.

## Report
**Plan** — (1) Add the route and request type in `api_auth.rs`, validate `X-EQ-CSRF` and the configured Origin before cookie or database work → verify all CSRF failures return `forbidden`. (2) Parse the single canonical `eq_rt` token into a redacted `SecretString` and hash its decoded 32 bytes → verify malformed, missing and unknown cookies return `unauthenticated` without exposing the token. (3) Use a tenant-scoped read to resolve the session person, then begin the actor-scoped mutation transaction and lock/revalidate that same session → verify expiry, revocation and active membership fail closed. (4) Update only `active_role`, issue a role-bound access token, and record `security.role_switched` with `eq_audit::record` in the same transaction → verify the same row, family and token digest remain. (5) Build the existing login response shape and exercise success and rejection paths over localhost HTTP against a fresh named PostgreSQL database → verify the exact test cases. (6) Run focused Rust, SQLx, task, guard and secret checks; inspect diff and report resource snapshots.
**Assumptions** — The preliminary session lookup is read-only and only supplies the person id needed to begin the final transaction with the correct audit actor; the final transaction locks and revalidates the session before any write. The existing `unauthenticated` and `forbidden` codes cover the specified failures. The already-shipped `has_active_role` identity port will be used to enumerate the five spec-defined roles in IDN-09 order for the response, avoiding cross-module SQL or a new port.
**Test plan** — AC1: given active admin and guardian memberships, when the existing cookie switches to guardian, then response token/permissions and `active_role` are guardian-only while row id, family id and token hash remain constant → `switch_role`, `switch_role_keeps_refresh_family`. AC2: given a missing/wrong header or absent/mismatched Origin, when switching is requested, then it returns 403 `forbidden` before database mutation → `switch_role_requires_csrf`. AC3: given an inactive/nonmember target role, when switching is requested, then it returns 403 and leaves session state unchanged → `switch_role_rejects_inactive_role`. Edge/error cases: missing, duplicate, malformed or unknown cookie; expired/revoked session; malformed or unknown role; permission snapshot remains limited to the selected role. No schema, permission, event or error-code addition.

## Questions
- **Self-resolved (implementer, run 10, needs architect ratification; TD-125):** the task requires a new Identity HTTP route but omits `crates/eq-identity/src/module.rs` from `## Files` → add that route registry file because the handler would otherwise be unreachable; registering the requested route is required by the task's Scope and does not broaden it.
- **Self-resolved (implementer, run 10, needs architect ratification; TD-126):** SBD-40 generically requires `actor.require(perm)` for non-public handlers, but no self-role-switch permission is specified and this endpoint has cookie authentication before an Actor exists → authorize with a valid refresh session and the requested active membership, because IDN-05/IDN-09 and `spec/25` §4.4 define those controls and adding an unrelated permission would change the contract.
- **Self-resolved (implementer, run 11, needs architect ratification; TD-127):** the role-switch implementation exceeded CQ-101's approximately 400-line review guide → split it into ordered service/persistence, HTTP boundary, and HTTP-test tasks while preserving the original aggregate contract and all four named tests.

## Review
Pending the three split parts listed under `## Split`; this aggregate is intentionally draft until their Reports are complete.
