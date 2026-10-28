---
id: T-110a
title: Identity schema, login and credential validation
milestone: M1
status: todo
depends_on: [T-102a, T-103, T-106, T-109]
spec_refs: [IDN-01, IDN-02, IDN-03, IDN-04, IDN-05, IDN-06, IDN-09, SEC-10, SEC-12, STM-22, STM-23, CAN-34, D-37, AUTH-02, AUTH-03, AUTH-05, AUTH-06]
estimated_size: M
---

## Goal
People log in with email and password, get a 10-minute in-memory access token plus a rotating HttpOnly refresh cookie, and every authenticated request yields an `Actor` with the active role and permission set.

## Context
- DDL from spec 06 §4.2, verbatim, into `migrations/0007_identity.sql`: `credentials`, `refresh_sessions`, `invites`, `password_resets` (see the SQL below). RLS via `eq_enable_rls` on all four.
```sql
CREATE TABLE credentials (person_id uuid PRIMARY KEY REFERENCES persons(id), tenant_id uuid NOT NULL REFERENCES tenants(id),
  password_hash text, password_changed_at timestamptz, totp_secret_enc bytea, totp_enabled boolean NOT NULL DEFAULT false,
  failed_count int NOT NULL DEFAULT 0, failed_window_start timestamptz, locked_until timestamptz);
CREATE TABLE refresh_sessions (id uuid PRIMARY KEY, tenant_id uuid NOT NULL REFERENCES tenants(id), person_id uuid NOT NULL REFERENCES persons(id),
  family_id uuid NOT NULL, token_hash bytea NOT NULL UNIQUE, active_role text NOT NULL, created_at timestamptz NOT NULL DEFAULT now(),
  last_used_at timestamptz NOT NULL DEFAULT now(), idle_expires_at timestamptz NOT NULL, absolute_expires_at timestamptz NOT NULL,
  revoked_at timestamptz, user_agent text, ip inet);
CREATE TABLE invites (id uuid PRIMARY KEY, tenant_id uuid NOT NULL REFERENCES tenants(id), person_id uuid NOT NULL REFERENCES persons(id),
  role text NOT NULL, token_hash bytea NOT NULL UNIQUE, expires_at timestamptz NOT NULL, accepted_at timestamptz,
  created_by uuid REFERENCES persons(id), created_at timestamptz NOT NULL DEFAULT now());
CREATE TABLE password_resets (id uuid PRIMARY KEY, tenant_id uuid NOT NULL REFERENCES tenants(id), person_id uuid NOT NULL REFERENCES persons(id),
  token_hash bytea NOT NULL UNIQUE, expires_at timestamptz NOT NULL, used_at timestamptz, created_at timestamptz NOT NULL DEFAULT now());
```
  **Login lookup happens before a tenant user is known, but the tenant is known from the host.** It runs inside `begin(tenant, ActorCtx::anonymous(ip, request_id))`, so RLS still applies.
- `IdentityPolicy` (04 §6.2), defined here in `eq_identity::policy` implementing `PolicyDoc` (MODULE Identity, SCHEMA_VERSION 1). All fields are x-ui `advanced`.
```rust
pub struct IdentityPolicy {
  pub idle_timeout_minutes: RoleMap<u16>,  // 5..=1440; default owner 30, admin 30, instructor 60, guardian 120, student 120
  pub absolute_session_days: u16,          // 1..=90, default 30
  pub lockout: Lockout,
  pub password_min_length: u8,             // 8..=64, default 10
  pub invite_expires_days: u16,            // 1..=30, default 7
  pub reset_token_ttl_minutes: u16,        // 10..=1440, default 60
  pub reset_max_per_hour: u8,              // 1..=10, default 3
  pub totp: TotpPolicy,                    // Off | OptionalForStaff | RequiredForOwnerAdmin; default OptionalForStaff
}
pub struct Lockout { pub max_failed: u8 /*3..=20, 5*/, pub window_minutes: u16 /*1..=120, 15*/, pub lock_minutes: u16 /*1..=1440, 15*/, pub notify_admins: bool /*true*/ }
pub struct RoleMap<T> { pub owner: T, pub admin: T, pub instructor: T, pub guardian: T, pub student: T } // put RoleMap in eq_core::policy::types
```
  Summaries: `policy.identity.summary.sessions {idle_admin, idle_guardian, absolute_days}` and `policy.identity.summary.lockout {max_failed, window, lock}`.
- IDN-01: email normalisation and lookup via `bidx_email` (T-103). The login identifier is the email only.
- IDN-02: argon2id m=19456 KiB, t=2, p=1 (technical consts). The max password length is 128. The common-password list is bundled at `crates/eq-identity/data/common-passwords.txt` (top 10k, from SecLists `10k-most-common.txt`, MIT), loaded into a `HashSet` once. No composition rules.
- IDN-03:
  - JWT HS256 with the key `Keys.jwt` (T-103), TTL 600 s (technical const).
  - Claims `sub, tid, role, sid, iat, exp, ver` (`ver` = `password_changed_at` unix seconds, 0 if null).
  - Verify: signature, `exp` with 30 s leeway, `tid` == resolved tenant, `ver` == current `credentials.password_changed_at`. `ver` is checked by a cached lookup (moka, 60 s TTL, invalidated on password change in-process).
- IDN-04:
  - The refresh token is 32 random bytes, base64url, in cookie `eq_rt` (`HttpOnly; SameSite=Strict; Path=/api/v1/auth; Secure` when `RITMA_PUBLIC_URL` is https). Only `sha256` is stored.
  - **Rotate on every refresh:** insert a new row in the same `family_id` and revoke the old one.
  - **Reuse:** presenting a revoked token whose family has a newer row → revoke the whole family, audit `security.refresh_reuse` (category security), and return 401 `session_expired`.
  - Idle expiry slides: `now + idle_timeout_minutes[role]`. Absolute = `created + absolute_session_days` of the family's first row.
- IDN-05: `/auth/refresh`, `/auth/logout` and `/auth/switch-role` require the header `X-EQ-CSRF: 1`, and `Origin` (if present) must equal `RITMA_PUBLIC_URL`'s origin. Otherwise 403 `forbidden`.
- IDN-06 lockout:
  - Within `window_minutes` from `failed_window_start`, `failed_count` is incremented. At `max_failed`, set `locked_until = now + lock_minutes`, emit `identity.account_locked.v1 {person_id, until}`, and audit `security.account.locked`.
  - **While locked:** a wrong password → generic 401 `invalid_credentials`. A correct password → 423 `locked {until}`.
  - A successful login resets the counters.
  - Unknown email → verify against a precomputed dummy argon2 hash (constant time), then 401.
- IDN-09: with several active memberships, `active_role` defaults to the most privileged (owner > admin > instructor > guardian > student). `POST /auth/switch-role {role}` issues a new access token, updates `refresh_sessions.active_role`, and keeps the family.
- TOTP check at login: if `credentials.totp_enabled`, or the policy is `RequiredForOwnerAdmin` and the active role is owner/admin, a missing `totp` → 401 `totp_required` with `needs_totp: true`. **TOTP enrolment endpoints are out of scope (T-111).** Here, implement only the verification function `totp_verify(secret, code, now)` (RFC 6238, 30 s, ±1 step) using `totp-rs`. If the policy requires TOTP but the user hasn't enrolled, login succeeds with `needs_totp_setup: true` in the response.
- Audits (category security): `security.login.success`, `security.login.failure` (entity person if known, else entity_type `login` without id), `security.refresh_reuse`, `security.logout`, `security.role_switched`.
- `Actor` extractor (in `eq-core::auth`, so every crate can use it):
```rust
pub struct Actor { pub person_id: PersonId, pub tenant: Arc<TenantInfo>, pub role: Role, pub session_id: Uuid, pub perms: Arc<PermSet>, pub ctx: ActorCtx }
impl Actor { pub fn require(&self, perm: &'static str) -> Result<(), AppError>; pub fn has(&self, perm: &str) -> bool; }
pub struct MaybeActor(pub Option<Actor>);
```
  `PermSet` is computed from the registry: the union of `PermissionDef.roles` containing the active role, across **enabled** modules. Owner without any active admin in the tenant → add the admin set (IDN-11, computed, cached 30 s). Deactivated membership → 401.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Do not add `revoked_reason`: spec 06 §4.2 and ERD §7 define `refresh_sessions` without it. Detect reuse from a revoked token whose `family_id` has a newer `created_at` row, then revoke the family, audit `security.refresh_reuse`, and return 401 `session_expired`; this preserves the existing IDN-04 behavior without an unmapped column.
  - Lock counters follow STM-23 (`FOR UPDATE` on the credentials row). Login parses `email` with the T-102a `Email` kind but maps **every** failure, including a malformed email, to 401 `invalid_credentials` (VAL-25 §5.1).
  - Cookie attributes exactly as CAN-34.
- Consolidated view of this mechanism (keys, token anatomy, timings, invalidation): spec 25 §2–§4 and ADR-0016. The values in this task are the same; if they ever differ, block and ask.

## Scope
### In
- Implement only AC 1, 2, 6, 7, 8, 9 and the tests assigned to T-110a in this split.
- Use the task's inline context and canonical interface definitions for those criteria.
### Out
- AC and behavior owned by the sibling split part(s) listed in T-110.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/jwt.rs` | create or modify | Path named in this task's contract. |
| `crates/eq-identity/src/password.rs` | create or modify | Path named in this task's contract. |
| `crates/eq-identity/tests/api_auth.rs` | test | Task-listed verification file. |
| `migrations/0007_identity.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3, T4 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `expired_rejected` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Valid admin login → 200, a JWT with `role=admin`, an `eq_rt` cookie with HttpOnly + SameSite=Strict (+ Secure under an https public URL), and audit `security.login.success`. → `api_auth::login_success`
- AC2. Lockout 5/15/15: 5 wrong passwords, then the correct one → 423 with `until`, an outbox `identity.account_locked.v1`, and a wrong password during the lock → 401 `invalid_credentials`. → `lockout_flow`
- AC6. Unknown email vs known email + wrong password: p50 latency differs ≤ 50 ms over 20 tries each. → `login_constant_time` (`#[ignore]` locally; runs in CI)
- AC7. A person with admin + guardian logs in as admin, then switches to guardian → the new token has `role=guardian` and the permissions shrink. → `switch_role`
- AC8. An owner in a tenant with no admin has `identity.people.write`. After an admin exists, they don't. → `owner_gets_admin_perms_without_admin`
- AC9. Common password `password123` with min length 10 → 422 `password_weak` (verified via the `validate_new_password` unit). → `password::tests::common_rejected`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `login_success`, `lockout_flow`, `login_constant_time`, `switch_role`, `owner_gets_admin_perms_without_admin` |
| `crates/eq-identity/src/password.rs` | `common_rejected`, `max_len_128`, `argon_params` |
| `crates/eq-identity/src/jwt.rs` | `roundtrip`, `expired_rejected`, `wrong_tenant_rejected` |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `scripts/check-literals.sh` passes
- [ ] `make api` regenerated
- [ ] Commit `feat(identity): login, tokens and refresh rotation [T-110a]`, signed off

## Report
**Deviations** — RATIFY: TD-063

**Deviations** — RATIFY: TD-068, use the canonical refresh-session DDL and detect reuse from the existing revoked timestamp and family successor row., CQ-101 split of the original task contract; the assigned AC and test names are preserved in this part.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-068):** STM-22/D-37 describe `revoked_reason`, but spec 06 §4.2 and ERD §7 define the complete `refresh_sessions` schema without that column → add no column and detect reuse from `revoked_at` plus a newer row in the same `family_id`, because the mapped schema and IDN-04 behavior are the available source of truth.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
