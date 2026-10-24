---
id: T-111
title: Add invites, password reset, /me endpoints, TOTP enrolment and device sessions
milestone: M1
status: todo
depends_on: [T-110]
spec_refs: [IDN-07, IDN-08, IDN-12, IDN-13, SEC-12]
estimated_size: M
---

## Goal
No password is ever sent. People join by invite link and recover by single-use reset link. Every person can manage their profile, password, TOTP and devices.

## Context
- IDN-07 invites:
  - `create_invite(tx, actor, person_id, role, channel)`: revokes earlier unaccepted invites for (person, role) by setting `expires_at = now`. It creates a 32-byte random token (sha256 stored) with `expires_at = now + invite_expires_days`, sets the membership `status='invited'` (insert if missing), emits `identity.user_invited.v1 {person_id, role, invite_id}`, and audits `identity.invite.created`.
  - The URL is `{EQ_PUBLIC_URL}/{lang}/invite/{token}`, where `lang` is the person's locale, then the tenant locale.
  - Channel `link` returns `invite_url` in the response (admin copies it). `email`/`whatsapp` don't return it; notify sends it (M5). Until notify exists, the event sits in the outbox.
  - `GET /auth/invites/{token}` → `{tenant_name, display_name, role, expires_at}`. Unknown → 404. Expired → 410 `invite_expired`. Accepted → 410.
  - `POST /auth/invites/{token}/accept {password}`: validates the password per policy, sets `password_hash` and `password_changed_at`, sets membership `active` and `accepted_at`, and returns the login response plus the cookie. Audit `identity.invite.accepted`.
- IDN-08 reset:
  - `POST /auth/forgot {email}` always returns 202, in constant time. If the person exists and has an active membership, and there were fewer than `reset_max_per_hour` resets in the last hour (count `password_resets` rows), create a token with TTL `reset_token_ttl_minutes` and emit `identity.password_reset_requested.v1 {person_id, token_id}`. **The raw token is put in the event payload** (field `token`) for notify to deliver. The outbox is a server-side table, never exposed. Document this in code comments.
  - `POST /auth/reset {token, new_password}` → 204. It sets the password, marks the token used, revokes **all** refresh sessions of the person, and audits `security.password.reset`. Invalid, expired or used token → 400 `token_invalid`. Weak password → 422 `password_weak`.
- `/me`:
  - `GET /me` → `PersonMe {id, display_name, email, phone, locale, avatar_url, roles, active_role, permissions, linked_students?}`. Email and phone are decrypted (own data, no PII audit). `linked_students` is present for guardians.
  - `PATCH /me {display_name?, phone?, locale?}`. Phone is normalised to E.164 (T-103), with a new bidx. Uses `If-Match` on `persons.version` (API-07).
  - `POST /me/password {current_password, new_password}` → 204. Revokes all **other** refresh sessions. Audit `security.password.changed`.
- IDN-12 TOTP:
  - `POST /me/totp/setup` → `{otpauth_url, qr_svg, secret_base32}`. Stores `totp_secret_enc` (pending; `totp_enabled` stays false).
  - `POST /me/totp/confirm {code}` → `{recovery_codes: [10 × 10 chars]}`. Sets `totp_enabled=true` and stores argon2 hashes of the recovery codes in a new table `totp_recovery_codes(person_id, tenant_id, code_hash, used_at)`, added in `migrations/0008_identity_totp_recovery.sql` with RLS.
  - `DELETE /me/totp {password}` disables it, unless the policy is `RequiredForOwnerAdmin` and the person holds owner/admin → 403 `policy_violation` rule `identity.totp`.
  - Login accepts a recovery code in place of `totp` (single use).
- Sessions: `GET /me/sessions` → `[{id, created_at, last_used_at, user_agent, ip, current: bool}]` (only non-revoked, non-expired). `DELETE /me/sessions/{id}` revokes it (its family).
- IDN-13: deactivating a membership revokes all refresh sessions for that person in the tenant (T-112 calls `AuthService::revoke_all(person)`, implemented here).

## Scope
### In
- The endpoints listed above, the services, the migration `0008`, and DTOs with `ToSchema`.
- i18n-free (server returns codes only).

### Out
- Sending email or WhatsApp (M5 notify). The people admin API (T-112).

## Acceptance criteria
- AC1. Invite accept within the TTL → login response, membership `active`. → `api_invites::accept_ok`
- AC2. Invite older than `invite_expires_days` → 410 `invite_expired`. → `accept_expired_410`
- AC3. Resending an invite makes the old token 410. → `resend_revokes_previous`
- AC4. `/auth/forgot` for unknown and known emails → both 202, and only the known one creates an outbox event. → `forgot_no_enumeration`
- AC5. The 4th reset request in an hour (policy 3) creates no token, still 202. → `forgot_rate_limited_silently`
- AC6. A reset revokes all refresh sessions. → `reset_revokes_sessions`
- AC7. TOTP setup → confirm with a correct code → login without TOTP → 401 `totp_required`. With the code → 200. With a recovery code → 200, and the same recovery code again → 401. → `totp_flow`
- AC8. `PATCH /me` without `If-Match` → 428. With a stale one → 409 `version_conflict`. → `me_patch_requires_if_match`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_invites.rs`, `api_reset.rs`, `api_me.rs` | AC names above |

## Definition of done
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(identity): invites, reset, profile and TOTP [T-111]`, signed off

## Report

## Questions
