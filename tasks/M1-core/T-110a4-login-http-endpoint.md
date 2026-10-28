---
id: T-110a4
title: Add the identity login HTTP endpoint
milestone: M1
status: todo
depends_on: [T-110a3]
spec_refs: [IDN-01, IDN-03, IDN-04, IDN-06, IDN-12, CAN-34, ERR-61, SBD-31, SBD-81]
estimated_size: M
---

## Goal
Expose the task-owned `POST /auth/login` endpoint and response contract over the completed identity service. Keep credentials and access tokens out of logs and cookies other than the canonical refresh cookie.

## Context
- Request: `{email, password, totp?}`. Response: `{access_token, expires_in, person: PersonMe, roles, active_role, needs_totp?}` plus `eq_rt`.
- Cookie per CAN-34: `HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=<absolute_session_days·86400>`. `Secure` is omitted only for `http://localhost` development.
- Return only the errors named in spec/modules/identity.md and spec/19; malformed email maps to generic `invalid_credentials`.

## Scope
### In
- Add the OpenAPI-annotated login route, request/response DTOs and canonical cookie attributes.
- Write the exact login API tests owned by this part.
### Out
- Lockout service internals (T-110a3), refresh/logout endpoints (T-110b), and switch-role (T-110a5).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/module.rs`, `crates/eq-identity/src/lib.rs` | create/modify | Route definitions and Identity module wiring. |
| `crates/eq-identity/tests/api_auth.rs` | create | Login success, lockout and timing acceptance tests. |
| `crates/eq-api/src/modules.rs` | modify | Ensure the shipped API registry includes the implemented module routes. |
| `crates/eq-identity/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | modify | Only required runtime/test dependencies. |
| `tasks/M1-core/T-110a4-login-http-endpoint.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Task state and recorded corrections. |

## Interfaces
Register `POST /auth/login` through `utoipa-axum` 0.3/`utoipa` 6. Do not introduce a second login path or noncanonical cookie name.

## Business rules
All response values and auth behavior come from IDN-01–06, IDN-12, CAN-34 and the parent task. `expires_in` is the 600-second technical access-token TTL.

## Security
- Triggers: T1, T3, T4 (SBD-30).
- Data classes touched: S2/S3; do not log request bodies, tokens, email or TOTP.
- Authentication is tenant-resolved before lookup; audit is emitted by the service transaction (SBD-40/53).
- Abuse cases: invalid/malformed credentials are indistinguishable, and response/cookie does not expose the raw refresh token outside `Set-Cookie` (SBD-31/73/81).

## Implementation notes
1. Annotate the route with `#[utoipa::path]` and preserve problem+json behavior.
2. Apply exact CAN-34 cookie attributes, including the localhost-only Secure exception.
3. `make api` is currently an intentional stub until T-114; do not edit Makefile to force it. Record it as `Not run` and run the available OpenAPI equivalent when its generator exists.

## Acceptance criteria
- AC1. Valid credentials return the canonical login response, JWT role and cookie, with `security.login.success` recorded → `login_success`.
- AC2. Lockout responses and event/audit behavior match IDN-06 → `lockout_flow`.
- AC3. Unknown and known-wrong login p50 latency differs by at most 50 ms over 20 tries → `login_constant_time` (locally ignored only as the parent specifies).

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `login_success`, `lockout_flow`, `login_constant_time` |

## Definition of done
- [ ] Exact named endpoint tests pass, or are `Not run` with the fixed-cache/database reason.
- [ ] `scripts/check-literals.sh`, `scripts/check-layers.sh`, task lint/index and `cargo fmt --all --check` pass.
- [ ] `make api` is recorded `Not run: Makefile target is intentionally unavailable until T-114`; no generated contract is fabricated.
- [ ] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [ ] Signed commit `feat(identity): add login endpoint [T-110a4]` pushed to `dev`.

## Report
**Plan** — Implement the mapped login route and DTOs, test status/body/cookie/audit behavior and constant-time rejection, then run available web/API guards.
**Assumptions** — T-110a3 is the single service boundary; API routes do not read identity/core tables directly.
**Test plan** — Exercise successful login, each lockout response, malformed credentials, cookie flags and the ignored latency comparison.

## Questions
None.

## Review
n/a — pending implementation.
