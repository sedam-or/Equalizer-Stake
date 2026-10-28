---
id: T-110a
title: Identity schema, login and credential validation
milestone: M1
status: draft
depends_on: [T-102a, T-103, T-106, T-109d]
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
- Final aggregate verification that the five implementation parts below satisfy this contract and its exact named tests.
### Out
- Reimplementing behavior owned by T-110a1 through T-110a5; fix only a defect required for the aggregate contract.

## Split
The combined schema, policy model/module, password/JWT, login persistence, HTTP login and actor/role-switch implementation exceeds CQ-101's approximately 400 changed-line review guide. Preserve all original ACs and test names, each assigned once. This parent remains `draft` until every part is `review`, then returns to `todo` for aggregate verification.

| Part | Owns | Depends on | Original ACs/tests | Status |
|---|---|---|---|---|
| T-110a1 | Identity schema and RoleMap | T-102a, T-103, T-106, T-109d | mapped DDL and RLS; ERD migration check | in_progress |
| T-110a1b | Validated identity policy model | T-110a1, T-106 | defaults/ranges/serde; `invalid_values_rejected_on_deserialize`, `out_of_range_timeout_rejected` | todo |
| T-110a1c | Identity policy module schema and summaries | T-110a1b, T-106 | schema/summary; `defaults_schema_and_summary_match_contract` | todo |
| T-110a2 | Password validation/hash and JWT primitive | T-110a1c, T-102a, T-103 | AC9; `common_rejected`, `max_len_128`, `argon_params`, `roundtrip`, `expired_rejected`, `wrong_tenant_rejected` | todo |
| T-110a3 | Login/lockout domain service and persistence | T-110a2, T-106, T-109d | AC2 behavior; service and abuse tests | todo |
| T-110a4 | HTTP login route and response | T-110a3 | AC1, AC2, AC6; `login_success`, `lockout_flow`, `login_constant_time` | todo |
| T-110a5 | Actor extraction, permission derivation and role switching | T-110a4 | AC7, AC8; `switch_role`, `owner_gets_admin_perms_without_admin` | todo |

## Files
| Path | Action | Purpose |
|---|---|---|
| `tasks/M1-core/T-110a-*.md` | modify | Track the implementation parts and final aggregate report. |
| `tasks/INDEX.md` | generate | Reflect part status through `scripts/tasks-index.sh`. |

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
**Plan** — (1) Verify the four identity tables against `spec/06` §4.2 and T-110's exact `spec/29` §7 row, then add `0007_identity.sql` → verify `scripts/erd.py --migrations`. (2) Implement password validation and Argon2id hashing with the task's exact parameters and common-password source → verify `common_rejected`, `max_len_128`, and `argon_params`. (3) Implement the JWT claims and checks using the shipped `Keys.jwt` → verify `roundtrip`, `expired_rejected`, and `wrong_tenant_rejected`. (4) Implement the identity policy, role grants, login/lockout/role-switch application flow, transaction-bound audit/outbox effects, and the `Actor` extractor → verify the exact AC1/2/6/7/8 API tests and the policy/security boundaries. (5) Wire the module routes into the shipped registry and generate OpenAPI → verify `make api` and the generated contract. (6) Run the available static checks, ERD and secret scans; record build, database and runtime checks as `Not run` wherever the active disk/resource restrictions prohibit their prerequisites.

**Assumptions** — The parent split is authoritative: this part owns initial login, lockout, password/JWT behavior, role switch, owner permission fallback and the assigned tests; refresh rotation, CSRF enforcement and password-change invalidation remain with T-110b. All policy values, roles, problem codes, events, audit actions and table columns come from the task/spec; no new business value or schema object will be introduced. The current run's recorded disk restriction continues to prohibit creating a Cargo target cache, database, or container.

**Test plan** — AC1: Given a tenant with an active admin and valid credentials, when `POST /auth/login` runs, then it returns 200 with an admin JWT and canonical `eq_rt` cookie and writes `security.login.success` → `api_auth::login_success`. AC2: Given the default lockout policy, when five wrong passwords are followed by a correct password, then the result is 423 with `until`, the account-locked event/audit are written, and wrong credentials while locked remain generic 401 → `lockout_flow`. AC6: Given unknown-email and known-email/wrong-password attempts, when each is measured 20 times, then p50 latency differs by at most 50 ms → `login_constant_time` (ignored locally as specified). AC7: Given active admin and guardian memberships, when the person logs in then switches to guardian, then the token role changes and permissions shrink → `switch_role`. AC8: Given an owner with no active tenant admin, when permissions are computed, then `identity.people.write` is present; after an admin exists it is absent → `owner_gets_admin_perms_without_admin`. AC9: Given `password123` and policy minimum 10, when a new password is validated, then the result is `password_weak` → `password::tests::common_rejected`. Unit edges cover empty, 1, 128, and 129 Unicode-scalar password lengths; Argon2 parameter stability; malformed/expired/wrong-tenant JWTs; email normalization and unknown-email dummy verification. Abuse cases cover unknown-user timing, lockout enumeration, cross-tenant token use, role escalation, redacted secret formatting, and no PII/secret logging. Database-backed cases use a disposable PostgreSQL 16 fixture only if the active resource restrictions permit creating it; otherwise report the exact tests as `Not run`.

**Deviations** — RATIFY: TD-063, TD-066–TD-068 inherited from T-110; TD-096 splits this implementation into five reviewable parts while preserving every original AC and named test exactly once. Additional implementation decisions are recorded in the owning part and `tasks/DECISIONS.md`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-068):** STM-22/D-37 describe `revoked_reason`, but spec 06 §4.2 and ERD §7 define the complete `refresh_sessions` schema without that column → add no column and detect reuse from `revoked_at` plus a newer row in the same `family_id`, because the mapped schema and IDN-04 behavior are the available source of truth.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

- **Self-resolved (implementer, run 7, needs architect ratification; TD-096):** this task combines five independently verifiable identity concerns and would exceed CQ-101 → split schema/policy, password/JWT, login service, login HTTP and actor/role-switch work into T-110a1–T-110a5, preserve all listed test names once, and run this parent only after all parts are review.

## Review
n/a — pending review after implementation.
