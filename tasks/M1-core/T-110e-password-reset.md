---
id: T-110e
title: Implement password reset and recovery
milestone: M1
status: todo
depends_on: [T-110d]
spec_refs: [IDN-02, IDN-08, SEC-12, STM-24, AUTH-06]
estimated_size: M
---

## Goal
Provide non-enumerating password recovery with policy-bound, single-use reset tokens, rate limits, and session revocation.

## Context
- T-111 defines POST /auth/forgot {email} as 202 for known/unknown addresses with constant work. For an active person below IdentityPolicy.reset_max_per_hour, create a reset token with TTL IdentityPolicy.reset_token_ttl_minutes and emit identity.password_reset_requested.v1 {person_id, token_id, token}. The raw token is retained only in the server-side outbox payload for Notify; it is never returned to the caller.
- POST /auth/reset {token, new_password} returns 204, updates password/version, marks the token used, revokes every refresh session for that person and audits security.password.reset. Invalid/expired/used token → 400 token_invalid; weak password → 422 password_weak.

## Scope
### In
- Implement forgot/reset routes, service, persistence, event and audit behavior and the three exact acceptance tests.
### Out
- Invite, profile, sessions and TOTP behavior.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_reset.rs | create | Forgot/reset HTTP DTOs and handlers. |
| crates/eq-identity/src/service.rs | modify | Non-enumerating reset service and session/cache invalidation. |
| crates/eq-identity/src/repo.rs | modify | Tenant-scoped reset token operations. |
| crates/eq-identity/src/module.rs | modify | Register routes. |
| crates/eq-identity/src/errors.rs | modify | Reuse existing spec-defined errors only. |
| crates/eq-identity/tests/api_reset.rs | create | Exact acceptance tests. |
| tasks/M1-core/T-110e-password-reset.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
- POST /api/v1/auth/forgot {email} → 202 with an identical response body for known and unknown addresses.
- POST /api/v1/auth/reset {token,new_password} → 204; use the existing errors token_invalid and password_weak.
- Outbox payload includes the raw token only as required for Notify delivery; the outbox remains server-side and no endpoint exposes it.

## Business rules
Use only IdentityPolicy.reset_max_per_hour and reset_token_ttl_minutes. Count password_resets rows in the last policy window defined by the task; do not create a new policy field or default.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 reset token hash, password hash and outbox token; S1 person id.
- S/E: account enumeration, replay and tenant crossover → constant-work 202 response, policy cap, tenant-scoped hash lookup and single-use transaction (AUTH-06, SBD-43, STM-24).
- I: token/password leak → hash persisted in table, outbox-only raw token, secret-safe Debug and no logs (SBD-10/21/23/50).
- Abuse-case acceptance criterion: forgot_rate_limited_silently verifies the policy cap is enforced without revealing account existence.

## Implementation notes
Use the existing Core identity directory for canonical email resolution and existing AuthState cache invalidation. Do not add an event name, table, column, permission or error code.

## Acceptance criteria
- AC1. Known and unknown addresses receive the same 202 response; only a known active account under the cap creates an event → forgot_no_enumeration.
- AC2. The request beyond reset_max_per_hour creates no token and still returns 202 → forgot_rate_limited_silently.
- AC3. Reset revokes all refresh sessions and consumes the token atomically → reset_revokes_sessions.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_reset.rs | forgot_no_enumeration, forgot_rate_limited_silently, reset_revokes_sessions |

## Definition of done
- [ ] Exact acceptance tests pass on ritma_test_T-111 in the existing PostgreSQL 16 container.
- [ ] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [ ] OpenAPI generation is attempted; SQLx metadata is refreshed after query changes.
- [ ] Signed-off commit and complete Step 12 Report.

## Report
_Filled when this task starts, using the complete Step 12 structure from WORKFLOW.md._

## Questions
None known. Preserve the decisions recorded in T-111 and TD-134.

## Review
n/a — pending review.
