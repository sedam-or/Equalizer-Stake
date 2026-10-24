# Module: identity (always on)

Purpose: authentication, sessions, invites, password reset, RBAC and role switching.
Policy: `IdentityPolicy` (04 §6.2). Tables: `credentials`, `refresh_sessions`, `invites`, `password_resets`, plus `persons` and `memberships` from core.
Legacy reference: `v0-capstone:srs/01-authentication.md`, `backend/app/services/auth.py`.

## 1. Rules

- **IDN-01. Login identifier.** The login identifier is the email, normalised: trim, lowercase, and NFC Unicode normalisation. Lookup goes through `email_bidx` (SEC-04). In cloud mode, login happens on the tenant subdomain, so the same email may exist in several tenants.
- **IDN-02. Passwords.**
  - Hashing uses argon2id (m=19456 KiB, t=2, p=1; OWASP 2023). Minimum length comes from `password_min_length`; the maximum is 128.
  - Passwords are checked against a bundled top-10k common-password list.
  - There are no composition rules (NIST 800-63B).
- **IDN-03. Access token.**
  - Format: JWT, HS256, signed with a key derived as `HKDF(RITMA_SECRET_KEY, "jwt")`. TTL is **10 minutes** (technical constant).
  - Claims: `sub` (person_id), `tid` (tenant_id), `role` (active role), `sid` (refresh session id), `iat`, `exp`, `ver` (`credentials.password_changed_at` epoch, so a password change invalidates tokens).
  - The token travels in the response body only. The SPA keeps it **in memory** only, never in localStorage.
- **IDN-04. Refresh token.**
  - It is an opaque random 32-byte token in an `HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth` cookie named `eq_rt`. Only its SHA-256 is stored.
  - It **rotates on every refresh**, and the family is kept.
  - Reusing an already-rotated token revokes the whole family and audits `security.refresh_reuse`.
  - `idle_expires_at` is `now + idle_timeout_minutes[role]` and slides with each refresh. `absolute_expires_at` is `created + absolute_session_days`.
- **IDN-05. CSRF.** Refresh, logout and role switching are cookie-authenticated, so they require the header `X-EQ-CSRF: 1` and an `Origin` check. All other endpoints use the Bearer token only, so they aren't vulnerable to CSRF.
- **IDN-06. Lockout.** This follows `lockout`: `failed_count` within `window_minutes` reaching `max_failed` sets `locked_until`. Any response for a locked account is the generic 401 `invalid_credentials`, except the UI receives `locked_until` **only** after a correct password, so lock status isn't revealed to guessers.
  - Locking emits `identity.account_locked.v1`, which notify sends to admins when `notify_admins` is set.
  - Responses take constant time, using a dummy hash for unknown emails.
- **IDN-07. Invites replace temporary passwords (legacy rule 30).**
  - Admins invite a person with a role. The person gets a link `/{lang}/invite/{token}` by email or WhatsApp, or the admin copies it to share manually.
  - Accepting sets the password, marks the membership `active`, and logs the person in.
  - The token expires after `invite_expires_days`. Admins can resend, which revokes the previous invite.
- **IDN-08. Password reset.**
  - `POST /auth/forgot` always returns 202, whether or not the email exists. The token TTL is `reset_token_ttl_minutes`, limited to `reset_max_per_hour` per email.
  - Completing a reset revokes all refresh sessions.
- **IDN-09. Role switching.** A person with several active memberships picks a role at login. The default is the most privileged, ordered owner > admin > instructor > guardian > student. They can switch later with `POST /auth/switch-role`, which issues a new access token and keeps the same refresh family.
- **IDN-10. RBAC.** Permissions are strings `<module>.<entity>.<action>`. Each module declares its permissions with default role grants (`PermissionDef { key, roles: &[Role] }`). The effective set is the union over the **active role** only. **Scoping** (e.g. a guardian sees only their own children) is enforced in queries, not by the permission name. Every `list`/`get` service takes an `Actor` and applies a `Scope`:
  - `All` (owner, admin)
  - `Instructor(person_id)`: sessions they teach
  - `Guardian(person_id)`: themselves plus linked students
  - `Self_(person_id)`
- **IDN-11. Owner is read-mostly.** Owners have read on everything, plus `core.policy.write`, `core.modules.write`, admin management, overrides (unfreeze, cancel invoice, revert session) and audit read for all categories. Owners **do not** get day-to-day write permissions by default (v0 rule). The exception: in a tenant with no admin, the owner gets the admin permission set automatically. This is computed, never stored.
- **IDN-12. TOTP.**
  - Uses RFC 6238, 30 s steps, ±1 step. The secret is encrypted.
  - Setup shows a QR code and 10 recovery codes, stored as argon2 hashes.
  - It is required at login for owner/admin when the policy is `RequiredForOwnerAdmin`.
- **IDN-13. Deactivation.** A deactivated membership is rejected at the next refresh, and all sessions for that tenant are revoked immediately.

## 2. API

| Method | Path | Auth | Request → Response | Errors |
|---|---|---|---|---|
| POST | `/auth/login` | none | `{email, password, totp?}` → `{access_token, expires_in, person: PersonMe, roles: [Role], active_role, needs_totp?: bool}` + `eq_rt` cookie | 401 `invalid_credentials`, 401 `totp_required`, 423 `locked {until}` (only after correct pw) |
| POST | `/auth/refresh` | cookie + CSRF | — → same as login (new access, rotated cookie) | 401 `session_expired` |
| POST | `/auth/logout` | cookie + CSRF | — → 204 (revokes this refresh session) | |
| POST | `/auth/logout-all` | bearer | — → 204 | |
| POST | `/auth/switch-role` | cookie + CSRF | `{role}` → login response | 403 |
| POST | `/auth/forgot` | none | `{email}` → 202 | 429 |
| POST | `/auth/reset` | none | `{token, new_password}` → 204 | 400 `token_invalid`, 422 `password_weak` |
| GET | `/auth/invites/{token}` | none | → `{tenant_name, display_name, role, expires_at}` | 404, 410 `invite_expired` |
| POST | `/auth/invites/{token}/accept` | none | `{password}` → login response | 410 |
| GET | `/me` | bearer | → `PersonMe {id, display_name, email, phone, locale, avatar_url, roles, active_role, permissions: [string], linked_students?: [{id, display_name}]}` | |
| PATCH | `/me` | bearer | `{display_name?, phone?, locale?}` → PersonMe | |
| POST | `/me/password` | bearer | `{current_password, new_password}` → 204 (revokes other sessions) | 401, 422 |
| POST | `/me/totp/setup` → `/me/totp/confirm` → DELETE `/me/totp` | bearer | standard TOTP enrolment | |
| GET | `/me/sessions`, DELETE `/me/sessions/{id}` | bearer | list and revoke devices | |
| GET | `/people` | `identity.people.read` | `?role=&q=&status=&cursor=` → page of PersonSummary | |
| POST | `/people` | `identity.people.write` | `{display_name, legal_name?, email?, phone?, birth_date?, roles:[Role], invite: bool, guardian_of?: [student_id], guardians?: [{guardian_id, relationship, is_primary}]}` → Person | 409 `email_taken` |
| GET/PATCH | `/people/{id}` | read / write | Person detail incl. decrypted PII if permitted (`identity.people.read_pii`) | |
| POST | `/people/{id}/invite` | `identity.people.write` | `{role, channel: email\|whatsapp\|link}` → `{invite_url?}` | |
| POST | `/people/{id}/roles` / DELETE `/people/{id}/roles/{role}` | `identity.roles.write` (owner for admin role) | | 409 `last_owner` |
| POST | `/people/{id}/deactivate` / `/reactivate` | `identity.people.write` | | |
| POST/DELETE | `/people/{id}/links` | `identity.people.write` | guardian ↔ student link | 409 `primary_exists` |

## 3. Permissions (defaults)

| Permission | owner | admin | instructor | guardian | student |
|---|---|---|---|---|---|
| identity.people.read | ✅ | ✅ | own students | self + linked | self |
| identity.people.read_pii | ✅ | ✅ | — | self + linked | self |
| identity.people.write | — | ✅ | — | — | — |
| identity.roles.write | ✅ (incl. admin/owner) | ✅ (instructor/guardian/student only) | — | — | — |
| identity.admins.manage | ✅ | — | — | — | — |

## 4. Events and jobs

- **Emits:** `identity.user_invited.v1`, `identity.account_locked.v1`, `identity.password_reset_requested.v1`.
- **Jobs:** `identity.cleanup` runs daily at 03:30 tenant TZ. It deletes refresh sessions expired more than 30 days ago, and used or expired reset tokens and invites older than 30 days.

## 5. Acceptance criteria

1. **Login.** Given an active admin with valid credentials, When they POST /auth/login, Then the response has an access token with `role=admin`, a `eq_rt` cookie with HttpOnly/Secure/SameSite=Strict, and an audit `security.login.success`.
2. **Lockout.** Given lockout 5/15/15, When 5 wrong passwords are sent within 15 min, Then the 6th attempt with the *correct* password returns 423 with `until`, the event `identity.account_locked.v1` is emitted, and wrong passwords during the lock return the generic 401.
3. **Refresh reuse.** Given refresh token R1 was rotated to R2, When R1 is presented, Then 401 is returned, R2 is revoked as well, and an audit `security.refresh_reuse` is written.
4. **Password change.** Given a password change, When an old access token is used, Then 401 is returned, because `ver` no longer matches.
5. **Guardian scope.** Given a guardian, When GET /people?role=student, Then only their linked students are returned.
6. **Last owner.** Given the only owner, When removing their owner role, Then 409 `last_owner` is returned.
7. **Invite expiry.** Given an invite older than `invite_expires_days`, When it is accepted, Then 410 is returned.
8. **Account enumeration.** /auth/forgot returns 202 with identical latency (±50 ms, p50 over 20 tries) for known and unknown emails.
