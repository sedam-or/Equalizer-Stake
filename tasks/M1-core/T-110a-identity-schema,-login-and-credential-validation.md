---
id: T-110a
title: Identity schema, login and credential validation
milestone: M1
status: review
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
| T-110a1 | Identity schema and RoleMap | T-102a, T-103, T-106, T-109d | mapped DDL and RLS; ERD migration check | review |
| T-110a1b | Validated identity policy model | T-110a1, T-106 | defaults/ranges/serde; `invalid_values_rejected_on_deserialize`, `out_of_range_timeout_rejected` | review |
| T-110a1c | Identity policy module schema and summaries | T-110a1b, T-106 | schema/summary; `defaults_schema_and_summary_match_contract` | review |
| T-110a2 | Password validation/hash and JWT primitive | T-110a1c, T-102a, T-103 | AC9; `common_rejected`, `max_len_128`, `argon_params`, `roundtrip`, `expired_rejected`, `wrong_tenant_rejected` | review |
| T-110a3 | Login/lockout domain service and persistence | T-110a2, T-106, T-109d | AC2 behavior; service and abuse tests | review |
| T-110a4 | HTTP login route and response | T-110a3 | AC1, AC2, AC6; `login_success`, `lockout_flow`, `login_constant_time` | review |
| T-110a5 | Actor extraction, permission derivation and role switching | T-110a4 | AC7, AC8; `switch_role`, `owner_gets_admin_perms_without_admin` | review |

## Files
| Path | Action | Purpose |
|---|---|---|
| `tasks/M1-core/T-110a-*.md` | modify | Track the implementation parts and final aggregate report. |
| `tasks/DECISIONS.md` | modify | Record TD-129 and TD-130 for the aggregate-only contract and current resource policy. |
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
- [x] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `scripts/check-literals.sh` passes. Not run: the full workspace runtime suite, because it would start Testcontainers resources prohibited by the disk-hygiene rule; focused unit and API suites passed below.
- [ ] Not run: `make api` is owned by T-114; its current stub writes `web/openapi.json`, outside this aggregate's Files.
- [x] Signed report commit `docs(identity): report login aggregate [T-110a]` pushed to `dev`.

## Report
**Plan** — (1) Read the direct dependency Reports, the seven split-part Reports and their shipped code → verify the original identity contract and inherited decisions. (2) Verify the mapped identity migration against a fresh PostgreSQL 16 task database and run `scripts/erd.py --migrations` → verify schema alignment. (3) Run the original RoleMap, policy, password/JWT, login/lockout, Actor/permission and role-switch tests → verify AC1/2/6/7/8/9. (4) Run formatting, workspace Clippy, dependency policy, task lint/index, guards, ERD and secret checks. (5) Record environment-limited checks, capture resources, drop the task database and commit this aggregate report.

**Assumptions** — The seven split parts own all implementation; this parent owns aggregate verification only. Refresh rotation and password-change invalidation remain with T-110b/T-111. The fixed Cargo target and PostgreSQL 16 container `ritma-db-1` may be reused; only `ritma_test_T-110a` was used for this task, and it was dropped after checks. No container, image, volume or target directory may be created. Business values, roles, permissions, error codes, events, audit actions and schema remain exactly those in the task/spec.

**Test plan** — AC1: Given valid admin credentials, when login succeeds, then the response carries the admin token/canonical refresh cookie and success audit → `login_success`. AC2: Given the policy lockout defaults, when five wrong passwords precede a correct one, then lockout returns 423, writes the event/audit, and wrong attempts remain generic 401 → `lockout_flow`. AC6: Given unknown-email and known-email/wrong-password inputs, when measured over 20 attempts each, then p50 differs by at most 50 ms → `login_constant_time`. AC7: Given active admin+guardian memberships, when switching to guardian, then the token and permission snapshot narrow → `switch_role`. AC8: Given an owner tenant without an active admin, then the specified Admin permission exists; after an admin is active it is absent → `owner_gets_admin_perms_without_admin`. AC9: Given `password123` at minimum length 10, then validation returns `password_weak` → `password::tests::common_rejected`. Edge cases include password length boundaries, invalid policy deserialization, malformed/expired/cross-tenant JWTs and inactive memberships. Error cases use only registered `invalid_credentials`, `locked`, `unauthenticated` and `forbidden`. Abuse cases cover lockout enumeration, cross-tenant/stale tokens, role escalation, missing CSRF and redacted secrets.

**Summary** — All seven implementation parts are `review`, and the aggregate contract checks passed across their shipped schema, policy, password/JWT, login and role-switch code. The fresh-database Identity API suite passed 16/16, and the focused core and identity unit suites passed 109/109 and 30/30. The full workspace runtime suite is Not run because it starts prohibited Testcontainers resources; the timing test remains locally ignored, and hosted CI was not run.

**Spec read** — Task-inline IDN/AUTH/STM/CAN contract; `spec/06-domain-model.md` §4.2; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md`; `spec/18-security-by-design.md` SBD-81; `spec/20-state-machines.md` STM-22/23; `spec/22-validation-and-sanitization.md`; `spec/23-contracts-and-canonical-forms.md` CAN-34; `spec/25-auth-and-access-control.md`; `spec/29-erd.md` §7. Also read the direct dependency and split-part Reports, including Deviations/Follow-ups, and reviewed their shipped code.

**Files** — `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md` (aggregate status/report); `tasks/DECISIONS.md` (TD-129/TD-130); `tasks/INDEX.md` (generated status index).

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1: successful admin login, cookie and audit | `login_success` | Passed in the Identity API suite. |
| AC2: lockout state and generic wrong-password response while locked | `lockout_flow` | Passed in the Identity API suite. |
| AC6: unknown-email timing matches known-email failure | `login_constant_time` | Not run: test is `#[ignore]` for local runs and the hosted CI runner is unavailable. |
| AC7: role switch narrows permissions and retains refresh family | `switch_role`; `switch_role_keeps_refresh_family` | Passed in the 16-test Identity API suite. |
| AC8: owner fallback permission depends on active-admin presence | `owner_gets_admin_perms_without_admin` | Passed in the 16-test Identity API suite. |
| AC9: common weak password is rejected | `password::tests::common_rejected` | Passed in the 30-test identity unit suite. |
| Schema, RoleMap, policy, password/JWT, bearer and abuse cases | `role_map_rejects_unknown_role_keys`; `invalid_values_rejected_on_deserialize`; `out_of_range_timeout_rejected`; `defaults_schema_and_summary_match_contract`; `max_len_128`; `argon_params`; `roundtrip`; `expired_rejected`; `wrong_tenant_rejected`; `bearer_actor_rejects_cross_tenant_token`; `bearer_actor_rejects_stale_password_version`; `switch_role_requires_csrf`; `switch_role_rejects_inactive_role` | Passed in the focused core/identity unit and API suites recorded below and in the split-part Reports. |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate `base64` lockfile warning remains.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-core --lib -- --test-threads=1 --quiet` — 109/109. Two `TestDb::new()` tests unexpectedly started `dazzling_darwin` because `RITMA_TEST_DATABASE_URL` was unset; its container and anonymous volume were removed after the suite completed and client connections reached zero.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --lib -- --test-threads=1` — 30/30; the immediately following API suite confirmed the sequential command completed successfully.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` — 16/16 against the fresh task database.
- ✅ `scripts/tasks-index.sh --check`; `python3 -I -B scripts/tasks-lint.py`; `python3 -I -B scripts/tests/test_tasks_lint.py` — 8/8.
- ✅ `scripts/check-deps.sh`; `scripts/check-literals.sh`; `scripts/check-web-boundaries.sh`; `scripts/check-layers.sh`.
- ✅ `python3 -I -B scripts/erd.py --migrations` — 25 migrated tables checked against spec; 40 not migrated yet. ✅ `python3 -I -B scripts/erd.py --check`.
- ✅ Per-file Gitleaks scans of the three changed task files — no leaks. An earlier multi-path invocation scanned the repository and reported the existing `Makefile:80` `curl-auth-user` pattern; no changed file was implicated.
- ✅ `git diff --check`.
- Not run: `cargo test --workspace`; workspace DB tests start Testcontainers containers, prohibited by the disk-hygiene rule. Focused unit/API suites and each split-part's named checks were run instead.
- Not run: `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check`; this aggregate changes no web source. The child browser run built the web app.
- Not run: `make api`; OpenAPI generation belongs to T-114 and its current stub writes outside this task's Files. No SQL query changed in this aggregate, so `cargo sqlx prepare --workspace` was not required.
- Not run: a separate `make dev` or `make e2e` invocation for this report-only aggregate; the child HTTP task exercised real localhost requests and ran E2E. Hosted GitHub Actions are unavailable locally.

**Manual checks** — Created `ritma_test_T-110a` from `template0` inside the existing `ritma-db-1` PostgreSQL 16.15 container, applied migrations with the owner URL, and ran `DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='https://example.test' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` using the `eq_app` app connection (`NOSUPERUSER`, `NOBYPASSRLS`). The suite passed 16/16 real HTTP tests covering login, lockout, bearer validation, owner permission fallback, role-switch CSRF, inactive membership, expiry/revocation, session family/digest preservation and audit. After confirming zero active connections, dropped only `ritma_test_T-110a` and verified it no longer exists. The focused `eq-core` test also created `eq_template` and two `ritma_test_<UUID>` databases in `dazzling_darwin`; exact UUIDs were not printed by the test helper. After confirming zero client backends, stopped/removed that test-only container and its anonymous volume. Child T-110a4a3 ran `E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4a3' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4a3' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' RITMA_LISTEN='127.0.0.1:8080' E2E_BASE_URL='http://127.0.0.1:8080' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true make e2e` on its fresh PostgreSQL 16.15 database `ritma_test_T-110a4a3`: 4/6 passed (Chromium 3/3; WebKit health 1/1); two WebKit page launches were Not run because `libWPEWebKit-2.0.so.1`, ICU 74 and Flite are absent.

**Debug log** — No implementation failure. The first `python3 -I -B -m unittest scripts/tests/test_tasks_lint.py` invocation could not import `scripts` under isolated mode; running the test file directly with `python3 -I -B scripts/tests/test_tasks_lint.py` passed 8/8. The initial database drop command omitted SQL identifier quotes around the hyphenated name and made no change; the quoted retry dropped it successfully. The focused `eq-core` test unexpectedly started Testcontainers because its `TestDb::new()` calls were not supplied `RITMA_TEST_DATABASE_URL`; I removed only the identified `dazzling_darwin` container and its anonymous volume after zero client backends were confirmed. Its internal `eq_template` and two UUID-named databases were removed with that volume; the UUIDs were not emitted by the test helper. The broad Gitleaks invocation also scanned the whole repository and reported an unchanged Makefile pattern; separate scans of every changed file were clean.

**Performance** — No aggregate runtime change. `login_constant_time` was not measured locally; its local-ignore/hosted-CI limitation is recorded under AC6.

**Security** — Triggers T1/T3/T4. SBD-81 aggregate review verified bearer signature, tenant, credential-version and membership checks; enabled-module permission derivation; conditional owner fallback; CSRF/Origin checks before tenant resolution; hashed/redacted refresh credentials; and transaction-bound role-switch audit. Abuse tests cover cross-tenant/stale/inactive tokens, lockout enumeration, invalid CSRF, expired/revoked sessions and permission fallback removal. No new permission, schema object, event, error code or secret behavior was introduced; no PII/secret was found in changed files.

**Deviations** — RATIFY inherited TD-063, TD-066–TD-068 and TD-096 from the original task split; TD-097–TD-116 from implementation parts; TD-117–TD-128 from T-110a5; and TD-129/TD-130 recorded here for aggregate commit type and fixed-resource policy. Rationale is in `tasks/DECISIONS.md` and owning task Questions. The focused `eq-core` suite unexpectedly created a Testcontainers container and volume because `RITMA_TEST_DATABASE_URL` was unset; the exact resources were removed after verification. Full workspace tests, local timing test, `make api`, and a second aggregate E2E run are Not run for the reasons above.

**Dependencies affected** — T-110b retains refresh rotation and credential invalidation; T-111 must invalidate the password-version cache after password change/reset. No shipped interface changed.

**Rollback plan** — Revert the signed aggregate report commit; no application or schema state requires rollback.

**Follow-ups** — Architect ratification of TD-063, TD-066–TD-068, TD-096–TD-130; T-110b/T-111 invalidation ownership; rerun the two WebKit page checks on a host with the required libraries.

**Resources** — Before: `du -sh "$HOME/.cache/ritma/target"` → 13G; `docker system df` → Images 20 total/7 active/15.5GB (2.073GB reclaimable), Containers 24/16/10.51MB, Local Volumes 450/21/75.29GB (68.81GB reclaimable), Build Cache 0. The focused `eq-core` test transiently added `dazzling_darwin` (`postgres:16-alpine`) and anonymous volume `676d2e0393964c920c5c58e0549a74bf51dfa6007375892ba5855fb68836a4dd`; both were removed after tests ended and client backends reached zero. After cleanup: target 13G; Docker → Images 20/7/15.5GB, Containers 24/16/10.51MB, Local Volumes 450/21/75.28GB (68.81GB reclaimable), Build Cache 0. Created/dropped `ritma_test_T-110a` inside existing `ritma-db-1`; the test helper also created `eq_template` and two UUID-named `ritma_test_<UUID>` databases in the removed test container. No image or target directory was created.

**Workflow steps 0–12** — 0: resumed on `dev`; confirmed dependencies/parts were `review`, promoted this aggregate under its Split condition and regenerated the index; preserved unrelated WIP. 1: read the task and its inline identity/auth/security contract and cited specs. 2: read dependency/split Reports (Deviations, Follow-ups, RATIFY) and the shipped code. 3: n/a, backend-only. 4: recorded Plan, Assumptions and AC-to-test plan above. 5: n/a, implementation belongs to the split parts; this parent only aggregates. 6: ran the real localhost API suite against the fresh PostgreSQL task database; child T-110a4a3 ran browser E2E. 7: no implementation failure; corrected the isolated unittest invocation and SQL identifier quoting, and documented the Testcontainers resource creation and cleanup plus the two unavailable WebKit checks. 8: reviewed authentication, tenant scope, role permissions, CSRF ordering and audit. 9: n/a, no runtime change or performance target. 10: completed the SBD-81 review above. 11: ran focused Rust suites, formatting, workspace Clippy, cargo-deny, task lint/index, guards, ERD, per-file secret scans and diff checks; recorded prohibited/hosted checks as Not run. 12: completed this Report, set `review`, regenerated the index and prepared the single signed report commit for `dev`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-068):** STM-22/D-37 describe `revoked_reason`, but spec 06 §4.2 and ERD §7 define the complete `refresh_sessions` schema without that column → add no column and detect reuse from `revoked_at` plus a newer row in the same `family_id`, because the mapped schema and IDN-04 behavior are the available source of truth.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

- **Self-resolved (implementer, run 7, needs architect ratification; TD-096):** this task combines five independently verifiable identity concerns and would exceed CQ-101 → split schema/policy, password/JWT, login service, login HTTP and actor/role-switch work into T-110a1–T-110a5, preserve all listed test names once, and run this parent only after all parts are review.
- **Self-resolved (implementer, run 13, needs architect ratification; TD-129):** the original DoD requires a `feat(identity)` commit although the split parent Scope is aggregate verification only → use `docs(identity): report login aggregate [T-110a]`, because this task changes only its report, decision register and generated index.
- **Self-resolved (implementer, run 13, needs architect ratification; TD-130):** the Report still says the disk rule prohibits creating a Cargo target cache, database or container → reuse the already existing fixed target and PostgreSQL container, and use/drop one `ritma_test_T-110a` database, because that is the current user-authorized resource policy.

## Review
n/a — pending review after implementation.
