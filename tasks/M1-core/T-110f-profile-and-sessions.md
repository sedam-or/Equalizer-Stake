---
id: T-110f
title: Implement self profile, password and device sessions
milestone: M1
status: todo
depends_on: [T-110e]
spec_refs: [IDN-08, IDN-13, API-07, AUTH-05, STM-22, ERD-02]
estimated_size: M
---

## Goal
Let an authenticated person view and update their own profile, change their password, list their active device sessions, and revoke one session family.

## Context
- T-111 defines GET /me → PersonMe with own profile, roles, active role, permissions and optional linked_students; email/phone are decrypted for self without a PII audit.
- PATCH /me accepts display_name?, phone?, locale?, normalizes phone to E.164 with T-103 and a new bidx, and uses If-Match on persons.version.
- POST /me/password {current_password,new_password} returns 204, revokes every other session and audits security.password.changed.
- GET /me/sessions lists only non-revoked, non-expired sessions; DELETE /me/sessions/{id} revokes that session family.

## Scope
### In
- Implement /me, profile mutation, self password change and owned session list/revocation.
- Use Core-owned person/link data only through the existing CoreIdentityDirectory port.
### Out
- Admin people/role management (T-112) and TOTP endpoints (T-111e/f).

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_me.rs | create | Self profile and session HTTP DTOs/handlers. |
| crates/eq-identity/src/service.rs | modify | Password and session services. |
| crates/eq-identity/src/repo.rs | modify | Refresh-session list/family revocation. |
| crates/eq-identity/src/module.rs | modify | Register routes and spec-defined self-read permissions. |
| crates/eq-core/src/auth/directory.rs | modify | Read/update Core-owned profile and linked-student facts through the port. |
| crates/eq-identity/tests/api_me.rs | create | Profile/password/session behavior tests. |
| tasks/M1-core/T-110f-profile-and-sessions.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
- GET /api/v1/me returns the PersonMe shape already used by the login response.
- PATCH /api/v1/me accepts display_name?, phone?, locale? and returns the updated PersonMe. Missing If-Match → 428; stale If-Match → 409 version_conflict.
- POST /api/v1/me/password accepts current_password/new_password → 204.
- GET /api/v1/me/sessions lists the caller's active sessions; DELETE /api/v1/me/sessions/{id} revokes only the caller's matching family.

## Business rules
Use the If-Match version of persons.version. Profile phone parsing and normalization use T-103. Password requirements and session expiry come from the existing IdentityPolicy and injected Clock.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S2 profile phone/user-agent/IP; S3 password hash; S1 IDs and locale.
- S/E: cross-tenant or other-person access → derive person and tenant only from Actor, check the exact spec-defined permission, and scope every query (AUTH-05, CA-30, SBD-40/41/42).
- I: PII and credentials → decrypt own PII only, secret-safe DTO Debug, no PII logs, verify current password, hash new password (SBD-10/21/50).
- Abuse cases: me_patch_requires_if_match verifies stale/missing version rejection; additional tests cover cross-person session revoke and version conflict.

## Implementation notes
Use the Core port for person and linked-student reads/writes; do not issue Identity SQL against Core-owned persons or person_links. Keep session ownership in Identity's own repository.

## Acceptance criteria
- AC1. GET /me returns only the authenticated person and authorized own fields → me_get_scoped_to_actor.
- AC2. PATCH without If-Match returns 428 and with a stale version returns 409 version_conflict → me_patch_requires_if_match.
- AC3. Password change revokes every other session, and session deletion revokes only the selected family → me_password_and_sessions.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_me.rs | me_get_scoped_to_actor, me_patch_requires_if_match, me_password_and_sessions |

## Definition of done
- [ ] Exact acceptance tests pass on the existing PostgreSQL 16 container using ritma_test_T-111.
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
