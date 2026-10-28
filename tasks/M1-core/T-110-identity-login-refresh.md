---
id: T-110
title: Add identity schema, IdentityPolicy, password hashing, login, JWT access tokens, rotating refresh sessions, logout and the Actor extractor
milestone: M1
status: review
depends_on: [T-102a, T-103, T-106, T-109d]
spec_refs: [IDN-01, IDN-02, IDN-03, IDN-04, IDN-05, IDN-06, IDN-09, SEC-10, SEC-12, STM-22, STM-23, CAN-34, D-37, AUTH-02, AUTH-03, AUTH-05, AUTH-06]
estimated_size: M
---

## Goal
People log in with email and password, get a 10-minute in-memory access token plus a rotating HttpOnly refresh cookie, and every authenticated request yields an `Actor` with the active role and permission set.

## Context
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
- Track the split task contract and dependency order only.
### Out
- All implementation and tests, which are assigned to the child parts below.

## Split
| Part | Owns | Depends on | Status |
|---|---|---|---|
| T-110a | Identity schema, login and credential validation; AC 1, 2, 6, 7, 8, 9 | T-102a, T-103, T-106, T-109d | todo |
| T-110b | Refresh rotation, CSRF and token invalidation; AC 3, 4, 5 | T-110a | todo |

## Files
| Path | Action | Purpose |
|---|---|---|
| `tasks/M1-core/T-110-identity-login-refresh.md` | modify | Track the split contract, report and status. |
| `tasks/INDEX.md` | regenerate | Reflect this task and the child dependency chain. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No runtime security control is implemented by this tracker-only parent. T-110a and T-110b carry the SBD-30 triggers, protections and abuse tests for their implementation.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Every original acceptance criterion and test is assigned exactly once to a child part in the Split table.

## Tests to write
| File | Test names |
|---|---|
| n/a | Child parts own all implementation tests. |

## Definition of done
- [x] Every original criterion and test name is assigned once to a dependent child.
- [x] The task index reflects the child dependency chain.

## Report
**Plan** — (1) Compare this parent's tracker rows and Files table with its tracker-only Scope and the T-110a/T-110b frontmatter → verify child statuses, dependencies and implementation ownership. (2) Map every original acceptance criterion and named test to exactly one child → verify the complete AC/test inventory. (3) Read dependency Reports and shipped interfaces, then reconcile the parent tracker without changing implementation behavior → verify T-102a Email, T-103 crypto helpers/Keys, T-106 PolicyStore, and T-109d record API are available. (4) Regenerate the index and run task, ERD-document, diff and secret checks → verify parent and child statuses match the task index.

**Assumptions** — T-110's Scope is authoritative: this parent tracks the split contract and dependency order only. Runtime authentication, database changes, endpoint behavior and security controls belong to T-110a and T-110b. Their existing todo frontmatter is the current state; the parent tracker is updated to match it.

**Test plan** — AC1: Given the parent contract and its two child tasks, when original criteria and test names are inventoried, then AC1, AC2, AC6–AC9 belong to T-110a and AC3–AC5 belong to T-110b exactly once. Verify T-110a tests login_success, lockout_flow, login_constant_time, switch_role, owner_gets_admin_perms_without_admin, common_rejected, max_len_128, argon_params, roundtrip, expired_rejected and wrong_tenant_rejected; verify T-110b tests refresh_reuse_revokes_family, refresh_requires_csrf and password_change_invalidates_access. Edge cases: absent or repeated AC/test ownership, stale child status, or a dependency row that differs from child frontmatter. Error/abuse tests execute in the implementation children; no runtime test belongs to this tracker parent.

**Summary** — Aligned the T-110 split table with the two child tasks: both are todo, and T-110a depends on T-109d as its frontmatter specifies. Replaced the misplaced migration path with the tracker task and generated index, and clarified that runtime security controls are owned by the children. The nine original behavior criteria and all fourteen named implementation tests map to one child each; no identity code or schema changed.

**Spec read** — spec/06-domain-model.md §4.2; spec/16-code-quality.md §§9/12; spec/17-clean-architecture.md §§1/4; spec/18-security-by-design.md §§5.1/5.4/8; spec/19-error-handling.md §10; spec/20-state-machines.md STM-22/23 and D-37; spec/23-contracts-and-canonical-forms.md CAN-34; spec/25-auth-and-access-control.md §§1–3; spec/29-erd.md §§3.2/5/7; spec/adr/0016-in-memory-jwt-with-rotating-refresh-cookie.md.

**Files** —
- tasks/M1-core/T-110-identity-login-refresh.md — corrected tracker files, split statuses/dependency, security scope, DoD and Report.
- tasks/DECISIONS.md — record the tracker reconciliation as TD-095.
- tasks/INDEX.md — regenerate status from task frontmatter.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1 | Split ownership inventory across T-110a and T-110b | Pass: AC1, AC2, AC6–AC9 are assigned to T-110a; AC3–AC5 are assigned to T-110b, each once. |
| AC1 | Named test ownership inventory | Pass: the eleven T-110a names and three T-110b names listed in the Test plan are each present once in their child task contracts. |

**Checks run** — Passed: `scripts/tasks-index.sh --check`; `python3 -I scripts/tasks-lint.py`; `PYTHONDONTWRITEBYTECODE=1 python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` (8/8); `python3 -I scripts/erd.py --check`; `git diff --check`; targeted Gitleaks scans of the parent task, decisions register and generated index (0 findings after replacing a generic detector false positive). Rust, database, server and browser checks are n/a for this tracker-only Scope.

**Manual checks** — Read dependency Reports and follow-ups for T-102a, T-103, T-106 and T-109d, and inspected their shipped Email field kind, Keys/bidx_email/normalize_email APIs, PolicyStore and transaction-bound audit record API. T-103's phone parser is the resolved rlibphonenumber 2 dependency; no spec dependency override is needed. T-109d's historical follow-up about record/sealer locking was superseded by parent T-109 TD-093 and was preserved as history. Compared T-110a/b frontmatter, split Scope, ACs, Files, Security sections and test lists. No database, server or browser behavior is in this parent Scope.

**Debug log** — No check failed. The initial tracker had stale child statuses/dependency and an implementation migration in its Files table; TD-095 corrects those mechanical inconsistencies.

**Performance** — n/a; documentation-only tracker task.

**Security** — No runtime security trigger is implemented by this parent. T-110a and T-110b have explicit T1/T3/T4 Security sections, S2/S3 classifications, tenant-bound controls, token handling and abuse tests; this task only reconciles their ownership map. No secret, PII fixture, role, permission or security control was changed.

**Deviations** — RATIFY TD-063: split the oversized identity contract across dependent children while assigning each behavior and test once. RATIFY TD-068: preserve the mapped refresh-session schema without revoked_reason and use revoked_at plus a family successor to detect reuse. RATIFY TD-066/TD-067: classify the original behavior's security triggers and complete the required task headings from the existing contract. RATIFY TD-095: reconcile the tracker Files, child statuses and T-110a dependency with its Scope and shipped child frontmatter; no business, schema or authorization rule changed.

**Dependencies affected** — T-110a is now accurately recorded as depending on T-102a, T-103, T-106 and T-109d; T-110b depends on T-110a. T-111 and later identity consumers continue to depend on the child implementation chain, not this tracker parent.

**Rollback plan** — Revert this documentation-only commit to restore the previous parent tracker. No code, database schema, persistent data or deployed behavior changes.

**Follow-ups** — T-110a owns login, identity schema, credentials, password validation, JWTs and the Actor/permissions behavior assigned in its Scope. T-110b owns refresh rotation, CSRF and token invalidation. Execute their database, Rust, OpenAPI and security tests in those tasks; the parent adds no runtime checks.

**Workflow steps 0–12** — Step 0: selected the lowest eligible task T-110 on dev; dependencies were review, the task index was current, and the pre-existing SQLx/.pending/.hold/cache WIP remained untouched. Set this task in_progress and regenerated the index. Step 1: read the whole parent and both child contracts plus their cited identity/schema/security rules. Step 2: read T-102a, T-103, T-106 and T-109d Reports, Deviations, Follow-ups, RATIFY items and shipped interfaces. Step 3: n/a, tracker-only task without UI work. Step 4: recorded this plan and the AC/test ownership audit before editing task content. Step 5: changed only the parent tracker, decision register and generated index; no implementation or test behavior changed. Step 6: n/a, no runtime behavior in this Scope. Step 7: no failure occurred. Step 8: reviewed the documentation diff for exact child frontmatter alignment, unique AC/test ownership, and no schema/security invention. Step 9: n/a. Step 10: confirmed the parent has no runtime security control; both implementation children carry SBD-30 sections and abuse tests. Step 11: ran the available task, ERD-document, diff and secret checks; Rust, SQL and browser checks are n/a for this documentation-only parent. Step 12: completed this Report, set review, regenerated the index, and prepared one signed commit and push to dev.
## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-068):** STM-22/D-37 describe `revoked_reason`, but spec 06 §4.2 and ERD §7 define the complete `refresh_sessions` schema without that column → add no column and detect reuse from `revoked_at` plus a newer row in the same `family_id`, because the mapped schema and IDN-04 behavior are the available source of truth.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending human review.
