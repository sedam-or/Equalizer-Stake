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
- `tenant_name` comes from Core-owned `tenants.name` through the function `public.resolve_tenant_name(uuid)` added by migration `0012_core_tenant_name_resolver.sql` (TD-138). Identity reads it only through `CoreIdentityDirectory::tenant_name`.

## Scope
### In
- Implement the public invite preview and acceptance endpoints and the acceptance transaction.
- Add `tenant_name` to the invite preview via the Core port defined in TD-138.
- Keep invite lookup tenant-scoped and token comparison constant-work as required by STM-21/AUTH-06.
### Out
- Invite issuance/resend (T-110c, the shipped dependency), reset, profile or TOTP behavior.
- Any other migration, function, table or column than `0012_core_tenant_name_resolver.sql`.

## Files
| Path | Action | Purpose |
|---|---|---|
| migrations/0012_core_tenant_name_resolver.sql | create (shipped by architect, TD-138) | SECURITY DEFINER `resolve_tenant_name(uuid) → text`, grant to `eq_app`. |
| crates/eq-core/src/auth/directory.rs and its Postgres implementation (`grep -rn "impl CoreIdentityDirectory" crates/eq-core`) | modify | Add `tenant_name(tx, tenant) -> Result<Option<String>, AppError>`, implemented by `SELECT public.resolve_tenant_name($1)`. |
| crates/eq-identity/src/api_invites.rs | create | Preview and accept HTTP DTOs/handlers. |
| crates/eq-identity/src/invites.rs | modify | Accept and preview service behavior. |
| crates/eq-identity/src/repo.rs | modify | Invite lookup and single-use acceptance. |
| crates/eq-identity/src/module.rs | modify | Register routes. |
| crates/eq-identity/tests/api_invites.rs | modify | Exact acceptance tests. |
| tasks/M1-core/T-110d-invite-acceptance.md | modify | Plan and final Report. |

## Interfaces
- GET /api/v1/auth/invites/{token} returns {tenant_name, display_name, role, expires_at}.
- POST /api/v1/auth/invites/{token}/accept accepts {password} and returns the existing login response plus the refresh cookie.
- Errors are the spec-defined 404, 410 invite_expired and 422 password_weak; add no error code.
- SQL (Core only): `SELECT public.resolve_tenant_name($1)` inside `core::db::begin`, so the `app.tenant_id` setting restricts the result to the caller's tenant.

## Business rules
Use only IdentityPolicy.password_min_length, invite_expires_days and the injected Clock. Acceptance is single-use and activates the membership in the same transaction as credentials, refresh family and audit.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 invite token hash/password hash; S2 profile fields in the existing login response; S1 tenant name (non-personal).
- S/E: token guessing, replay or cross-tenant lookup → tenant-scoped hash lookup, single-use transaction and policy validation (SBD-40/41/42, STM-21). `resolve_tenant_name` returns a name only when the id equals the tenant set by `core::db::begin`, so it is not a cross-tenant lookup.
- I: secret leakage → hash-only token persistence, redacted Debug and no token/PII logging (SBD-10/21/50).
- Abuse cases: accept_ok and accept_expired_410 cover valid, expired and replayed invite paths.

## Implementation notes
Reuse the invite repository/service shipped by T-110c and the existing LoginService success response; do not query memberships or `tenants` in the Identity repository.

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
- [ ] Migration 0012 is applied in the task database, and a test confirms `eq_app` can call `resolve_tenant_name` for its own tenant and gets NULL for another tenant's id.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [ ] OpenAPI generation is attempted and T-114 placeholder recorded as Not run; SQLx metadata is updated if queries change.
- [ ] Signed-off commit and complete Step 12 Report.

## Report
(filled in by Codex. The earlier blocked Report is in git history at commit 80bf868.)

## Questions
- **Resolved by architect (TD-138):** `tenant_name` comes from the new Core function in migration 0012. Option (a) from the earlier Q1 was chosen, and the response contract is unchanged.
- **Resolved by architect (TD-139):** `blocked` is not a task status. Record a TD and continue.

## Review
(filled in by Codex)
