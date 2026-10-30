---
id: T-110d
title: Preview and accept identity invites
milestone: M1
status: todo
depends_on: [T-110c]
spec_refs: [IDN-02, IDN-07, STM-20, STM-21, AUTH-06]
estimated_size: M
---

## Goal
Allow an invite holder to preview a pending invitation and accept it once by setting a policy-valid password and starting an authenticated session.

## Context
- T-111 defines GET /auth/invites/{token} → tenant_name, display_name, role, expires_at. Unknown tokens return 404; expired or accepted tokens return 410 invite_expired.
- POST /auth/invites/{token}/accept {password} validates Password against the current IdentityPolicy, marks the invite accepted, sets the credential password and timestamp, activates the invited membership through Core, creates the login response and refresh cookie, and audits identity.invite.accepted.
- Invite tokens are 32 random bytes; only their SHA-256 hash is stored. The raw token is never logged.

## Scope
### In
- Implement the public invite preview and acceptance endpoints and the acceptance transaction.
- Keep invite lookup tenant-scoped and token comparison constant-work as required by STM-21/AUTH-06.
### Out
- Invite issuance/resend (T-111a), reset, profile or TOTP behavior.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_invites.rs | create | Preview and accept HTTP DTOs/handlers. |
| crates/eq-identity/src/invites.rs | modify | Accept and preview service behavior. |
| crates/eq-identity/src/repo.rs | modify | Invite lookup and single-use acceptance. |
| crates/eq-identity/src/module.rs | modify | Register routes. |
| crates/eq-identity/tests/api_invites.rs | modify | Exact acceptance tests. |
| tasks/M1-core/T-110d-invite-acceptance.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
- GET /api/v1/auth/invites/{token} returns {tenant_name, display_name, role, expires_at}.
- POST /api/v1/auth/invites/{token}/accept accepts {password} and returns the existing login response plus the refresh cookie.
- Errors are the spec-defined 404, 410 invite_expired and 422 password_weak; add no error code.

## Business rules
Use only IdentityPolicy.password_min_length, invite_expires_days and the injected Clock. Acceptance is single-use and activates the membership in the same transaction as credentials, refresh family and audit.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 invite token hash/password hash; S2 profile fields in the existing login response.
- S/E: token guessing, replay or cross-tenant lookup → tenant-scoped hash lookup, single-use transaction and policy validation (SBD-40/41/42, STM-21).
- I: secret leakage → hash-only token persistence, redacted Debug and no token/PII logging (SBD-10/21/50).
- Abuse cases: accept_ok and accept_expired_410 cover valid, expired and replayed invite paths.

## Implementation notes
Reuse T-111a's invite repository/service and the existing LoginService success response; do not query memberships in the Identity repository.

## Acceptance criteria
- AC1. A valid invite accepted inside its TTL returns the login response and activates the membership → api_invites::accept_ok.
- AC2. Expired or accepted invite tokens return 410 invite_expired → accept_expired_410.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_invites.rs | accept_ok, accept_expired_410 |

## Definition of done
- [ ] Exact acceptance tests pass against the task database in the existing PostgreSQL 16 container.
- [ ] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [ ] OpenAPI generation is attempted and T-114 placeholder recorded as Not run; SQLx metadata is updated if queries change.
- [ ] Signed-off commit and complete Step 12 Report.

## Report
_Filled when this task starts, using the complete Step 12 structure from WORKFLOW.md._

## Questions
None known. Preserve the decisions recorded in T-111 and TD-134.

## Review
n/a — pending review.
