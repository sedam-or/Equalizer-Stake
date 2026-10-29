---
id: T-110a4a2
title: Expose role-switch HTTP route with CSRF checks
milestone: M1
status: todo
depends_on: [T-110a4a1]
spec_refs: [IDN-05, IDN-09, AUTH-03, CAN-30, CAN-34, SBD-40, SBD-50, SBD-53]
estimated_size: S
---

## Goal
Expose the role-switch service through `POST /api/v1/auth/switch-role`, authenticating with the existing refresh cookie and enforcing the exact CSRF and Origin contract.

## Context
T-110a4a1 supplies the service. IDN-05 and `spec/25` §3.3 require both `X-EQ-CSRF: 1` and an `Origin` equal to the configured public URL origin. The route returns the existing login-response shape and must preserve the current cookie.

## Scope
### In
- Parse one canonical `eq_rt` cookie into secret-safe storage and compute its canonical digest.
- Require exactly one CSRF header with value `1` and exactly one matching Origin before database access.
- Register the handler and return a no-store login-shaped response without setting a replacement cookie.
- Add unit and route-surface tests.
### Out
- Session persistence/service logic, new schema, role grants, token rotation, and database-backed HTTP acceptance tests (T-110a4a3).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_auth.rs` | modify | Request boundary, cookie/CSRF validation and response construction. |
| `crates/eq-identity/src/module.rs` | modify | Register the endpoint in the Identity module. |
| `tasks/M1-core/T-110a4a2-role-switch-http.md`, `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Part report, aggregate split record and task index. |

## Interfaces
- `POST /api/v1/auth/switch-role` accepts exactly `{ "role": <canonical Role> }`.
- Headers: `Cookie: eq_rt=…`, `X-EQ-CSRF: 1`, and `Origin` equal to `RITMA_PUBLIC_URL`'s origin.
- Errors reuse `unauthenticated` for missing/malformed credentials and `forbidden` for CSRF or inactive target membership.

## Business rules
1. The target role and response use canonical `Role` values.
2. A successful response keeps the browser's refresh cookie and existing session family unchanged.
3. The endpoint adds no permission key or role grant.

## Security
- Triggers: cookie authentication, CSRF, refresh/access tokens (SBD-30).
- Require exact CSRF/Origin values and reject duplicate headers; perform these checks before any database query.
- Keep the cookie credential in `SecretString` with redacted Debug output; accept only canonical base64url encoding of the expected digest input.
- The control is a valid refresh session plus active target membership defined in IDN-05/09; do not invent an `actor.require` permission.

## Implementation notes
1. Reuse the shared login state, permission snapshot and response types where their contracts match.
2. Validate exact CSRF/Origin headers before reading the cookie or opening a database transaction.
3. Preserve the existing refresh cookie by omitting `Set-Cookie` on success.

## Acceptance criteria
- AC1. The module exposes the documented POST path and the handler returns the existing login response shape → `defaults_schema_and_summary_match_contract`.
- AC2. Missing, duplicate, malformed or mismatched CSRF/Origin values fail closed before database mutation → `role_switch_csrf_requires_exact_header_and_origin`.
- AC3. Missing or malformed refresh cookie never exposes its value and canonical digest validation rejects invalid encodings → `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/api_auth.rs` | `role_switch_csrf_requires_exact_header_and_origin`, `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required` |
| `crates/eq-identity/src/module.rs` | `defaults_schema_and_summary_match_contract` |

## Definition of done
- [ ] The named unit and route-surface tests pass.
- [ ] Focused Clippy, fmt, task lint/index and required guard scripts pass.
- [ ] No schema, permission, event, cookie rotation or new error code is introduced.
- [ ] Report contains Workflow steps 0–12 and disk/resource snapshots.
- [ ] Signed commit `feat(identity): expose role-switch endpoint [T-110a4a2]` pushed to `dev`.

## Report
**Plan** — (1) Add the strict cookie/CSRF boundary to the existing auth router → verify exact header and Origin rejection. (2) Connect the handler to the service and existing login response shape → verify route surface and serialization. (3) Register the route in Identity → verify its manifest test. (4) Run focused unit, Clippy, task, guard and secret checks → verify the endpoint contract without a migration.
**Assumptions** — API composition supplies the configured public URL, tenant context and request actor; the service's transaction handles final session validation and audit.
**Test plan** — AC1: Given the Identity module is built, when routes are inspected, then the POST path exists → `defaults_schema_and_summary_match_contract`. AC2: Given a missing, duplicate or wrong CSRF/Origin value, when the handler validates headers, then it returns `forbidden` before database work → `role_switch_csrf_requires_exact_header_and_origin`. AC3: Given a missing, duplicate, malformed or noncanonical cookie, when parsing/digest validation runs, then the credential is not exposed and invalid input is rejected → `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required`. Database-backed success and active-membership rejection are covered by T-110a4a3.

## Questions
- **Self-resolved (implementer, run 10, needs architect ratification; TD-125):** the parent task omitted the route registry from its Files table → register the handler in `crates/eq-identity/src/module.rs`, the existing composition point required by Scope.
- **Self-resolved (implementer, run 10, needs architect ratification; TD-126):** generic SBD-40 says `actor.require(perm)`, but this cookie-auth endpoint has no Actor before refresh authentication and no self-switch permission is specified → enforce valid refresh session, exact CSRF/Origin and active membership per IDN-05/09 and `spec/25` §4.4; add no permission.

## Review
Pending implementation checks.
