---
id: T-111
title: Add invites, password reset, /me endpoints, TOTP enrolment and device sessions
milestone: M1
status: todo
depends_on: [T-103, T-110b]
spec_refs: [IDN-07, IDN-08, IDN-12, IDN-13, SEC-12, STM-21, STM-24, STM-25, AUTH-06]
estimated_size: M
---

## Goal
No password is ever sent. People join by invite link and recover by single-use reset link. Every person can manage their profile, password, TOTP and devices.

## Context
- IDN-07 invites:
  - `create_invite(tx, actor, person_id, role, channel)`: revokes earlier unaccepted invites for (person, role) by setting `expires_at = now`. It creates a 32-byte random token (sha256 stored) with `expires_at = now + invite_expires_days`, sets the membership `status='invited'` (insert if missing), emits `identity.user_invited.v1 {person_id, role, invite_id}`, and audits `identity.invite.created`.
  - The URL is `{RITMA_PUBLIC_URL}/{lang}/invite/{token}`, where `lang` is the person's locale, then the tenant locale.
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
  - `POST /me/totp/confirm {code}` → `{recovery_codes: [10 × 10 chars]}`. Sets `totp_enabled=true` and stores argon2 hashes of the recovery codes in a new table `totp_recovery_codes(person_id, tenant_id, code_hash, used_at)`, added in `migrations/0008_identity_totp_recovery.sql` with RLS (DDL verbatim from `spec/06-domain-model.md` §4.12; PK `(person_id, code_hash)`; `code_hash` is S3).
  - `DELETE /me/totp {password}` disables it, unless the policy is `RequiredForOwnerAdmin` and the person holds owner/admin → 422 `policy_violation` rule `identity.totp` (ERR-10; STM-25 t4).
  - Login accepts a recovery code in place of `totp` (single use).
- Sessions: `GET /me/sessions` → `[{id, created_at, last_used_at, user_agent, ip, current: bool}]` (only non-revoked, non-expired). `DELETE /me/sessions/{id}` revokes it (its family).
- IDN-13: deactivating a membership revokes all refresh sessions for that person in the tenant (T-112 calls `AuthService::revoke_all(person)`, implemented here).
- Invite, reset and TOTP flows reveal nothing about whether an account exists (AUTH-06); the full flow table is spec 25 §4.

## Scope
### In
- The endpoints listed above, the services, the migration `0008`, and DTOs with `ToSchema`.
- i18n-free (server returns codes only).

### Out
- Sending email or WhatsApp (M5 notify). The people admin API (T-112).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/tests/api_invites.rs`, `api_reset.rs`, `api_me.rs` | test | Task-listed verification file. |
| `migrations/0008_identity_totp_recovery.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3, T4 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `forgot_rate_limited_silently` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(identity): invites, reset, profile and TOTP [T-111]`, signed off

## Report

**Deviations** — RATIFY: TD-063, dependency references now target the final split part that provides the referenced contract.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** this task depended on the original parent contract after its CQ-101 split → depend on the final implementation part that provides the contract, because the parent now owns only the split map.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
