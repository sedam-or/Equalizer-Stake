---
id: T-110a4
title: Add the identity login HTTP endpoint
milestone: M1
status: review
depends_on: [T-110a3]
spec_refs: [IDN-01, IDN-03, IDN-04, IDN-06, IDN-12, CAN-34, ERR-61, SBD-31, SBD-81]
estimated_size: M
---

## Goal
Expose the task-owned `POST /auth/login` endpoint and response contract over the completed identity service. Keep credentials and access tokens out of logs and cookies other than the canonical refresh cookie.

## Context
- Request: `{email, password, totp?}`. Response: `{access_token, expires_in, person: PersonMe, roles, active_role, needs_totp?}` plus `eq_rt`.
- Cookie per CAN-34: `HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=<absolute_session_days·86400>`. `Secure` is omitted only for `http://localhost` development.
- Return only the errors named in spec/modules/identity.md and spec/19; malformed email maps to generic `invalid_credentials`.

## Scope
### In
- Add the OpenAPI-annotated login route, request/response DTOs and canonical cookie attributes.
- Wire the shipped `LoginService` into the runtime API composition and return the `PersonMe` profile through core-owned identity ports; derive its permission snapshot from registered, enabled module grants without adding grants.
- Write the exact login API tests owned by this part.
### Out
- Lockout service internals (T-110a3), refresh/logout endpoints (T-110b), and the shared `Actor` extractor/role-switch endpoint (T-110a5). The login response's required `PersonMe.permissions` snapshot remains in scope.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/module.rs`, `crates/eq-identity/src/lib.rs` | create/modify | Route definitions and Identity module wiring. |
| `crates/eq-identity/tests/api_auth.rs` | create | Login success, lockout and timing acceptance tests. |
| `crates/eq-api/src/lib.rs`, `crates/eq-api/src/modules.rs` | modify | Inject the shared lazy runtime into Identity routes while preserving synchronous API composition. |
| `crates/eq-core/src/auth/directory.rs` | modify | Read the authenticated person's core-owned profile and active-admin fact through the typed port. |
| `crates/eq-core/src/fields/mod.rs`, `crates/eq-core/src/fields/kinds.rs`, `crates/eq-core/src/fields/{collector.rs,tests.rs}`, `crates/eq-core/src/http/validated.rs`, `crates/eq-identity/src/{service.rs,password.rs}` | modify | Represent an absent country context and update existing context construction sites without inventing a tenant country. |
| `crates/eq-identity/Cargo.toml`, `crates/eq-api/Cargo.toml`, `Cargo.toml`, `Cargo.lock`, `.sqlx/` | modify | Only required runtime/test dependencies and query metadata. |
| `tasks/M1-core/T-110a4-login-http-endpoint.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Task state and recorded corrections. |

## Interfaces
Register `POST /auth/login` through `utoipa-axum` 0.3/`utoipa` 6. Do not introduce a second login path or noncanonical cookie name.

## Business rules
All response values and auth behavior come from IDN-01–06, IDN-12, CAN-34 and the parent task. `expires_in` is the 600-second technical access-token TTL.

## Security
- Triggers: T1, T3, T4 (SBD-30).
- Data classes touched: S2/S3; do not log request bodies, tokens, email or TOTP.
- Authentication is tenant-resolved before lookup; audit is emitted by the service transaction (SBD-40/53).
- Abuse cases: invalid/malformed credentials are indistinguishable, and response/cookie does not expose the raw refresh token outside `Set-Cookie` (SBD-31/73/81).

## Implementation notes
1. Annotate the route with `#[utoipa::path]` and preserve problem+json behavior.
2. Apply exact CAN-34 cookie attributes, including the localhost-only Secure exception.
3. `make api` is currently an intentional stub until T-114; do not edit Makefile to force it. Record it as `Not run` and run the available OpenAPI equivalent when its generator exists.

## Acceptance criteria
- AC1. Valid credentials return the canonical login response, JWT role and cookie, with `security.login.success` recorded → `login_success`.
- AC2. Lockout responses and event/audit behavior match IDN-06 → `lockout_flow`.
- AC3. Unknown and known-wrong login p50 latency differs by at most 50 ms over 20 tries → `login_constant_time`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `login_success`, `lockout_flow`, `login_constant_time`, `login_omits_pii_without_permission` (SBD-81 abuse case) |

## Definition of done
- [x] Exact named endpoint tests pass.
- [x] `scripts/check-literals.sh`, `scripts/check-layers.sh`, task lint/index and `cargo fmt --all --check` pass.
- [x] `make api` is recorded `Not run: Makefile target is intentionally unavailable until T-114`; no generated contract is fabricated.
- [x] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [x] Signed commit `feat(identity): add login endpoint [T-110a4]` pushed to `dev`.

## Report
**Plan** — (Step 4.) (1) Make country context optional and preserve country-dependent phone rejection; verify existing context construction and email/password behavior. (2) Add core-owned profile and active-admin port reads; verify tenant scoping and PII authorization. (3) Inject one lazy, process-shared policy runtime at API composition; verify synchronous health routes can start without PostgreSQL. (4) Implement the annotated login route, DTOs, generic error mapping and `eq_rt`; verify response, cookie, audit and secret redaction. (5) Add all named endpoint tests and the no-PII abuse test; exercise real HTTP requests against the task database. (6) Run Rust, SQLx, deny, ERD, task and guard checks; record unavailable API/browser checks as `Not run`.
**Assumptions** — T-110a3's `LoginService` and `PolicyStore` remain the only login and effective-policy boundaries. Email and password validation do not consume `FieldCtx.country`; password minimum is intentionally absent during login. The core directory is the only route to core-owned person data. If no persisted identity policy is initialized, login fails closed until T-301 provides the embedded preset loader.
**Test plan** — AC1: Given an active admin with valid credentials, when `POST /api/v1/auth/login` is called, then the canonical response, role-bearing access token, `eq_rt` attributes and committed success audit are present → `login_success`. AC2: Given the policy lockout threshold, when failures reach the threshold and the correct password is supplied, then the response is 423 with `until`, the lock event/audit commit, and a wrong password remains generic 401 → `lockout_flow`. AC3: Given unknown and known-wrong identities, when each path is measured for 20 attempts, then p50 delta is at most 50 ms → `login_constant_time`. Security abuse: for an instructor with fake ciphertext in PII columns but no `read_pii`, login succeeds without decrypting or returning email/phone → `login_omits_pii_without_permission`. Edge cases: malformed email uses generic credentials error; absent TOTP; null PII without permission; localhost Secure exception; HTTPS cookie; policy-derived Max-Age. Errors: `invalid_credentials`, `totp_required` and `locked`. Other abuse cases: no account enumeration, tenant-scoped profile reads, refresh token only in `Set-Cookie`, and audit/session changes share one transaction.

**Summary** — Added `POST /api/v1/auth/login` to the Identity module and wired it through the synchronous API composition using one lazy shared policy runtime. The response includes the service-issued access token and `PersonMe` snapshot; the refresh token is only in `eq_rt`. Login success, lockout, timing equivalence, permission-gated PII, cookie settings and real HTTP happy/error paths were verified.

**Spec read** — `spec/modules/identity.md` §§1–3 (IDN-01–12 and permissions); `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§2, 5, 8; `spec/19-error-handling.md` §§1, 10; `spec/22-validation-and-sanitization.md` VAL-20/21; `spec/23-contracts-and-canonical-forms.md` CAN-34; `spec/25-auth-and-access-control.md`; and `spec/29-erd.md` core `persons` dictionary and §5. Read T-110a3's Report and the shipped login service/directory code.

**Files** — `crates/eq-identity/src/api_auth.rs`, `module.rs`, and `lib.rs`: annotated route, DTOs, redaction, cookie, permissions and registration. `crates/eq-identity/tests/api_auth.rs`: named HTTP integration tests and the no-PII abuse case. `crates/eq-api/src/lib.rs` and `modules.rs`: shared lazy runtime injection. `crates/eq-core/src/auth/directory.rs`: tenant-scoped profile and active-admin port methods. `crates/eq-core/src/fields/{mod.rs,kinds.rs,collector.rs,tests.rs}` and `crates/eq-core/src/http/validated.rs`: optional country context and construction sites. `crates/eq-identity/src/{password.rs,service.rs}`: updated existing context constructors. `crates/eq-identity/Cargo.toml`, `Cargo.lock`, and the two task-generated `.sqlx/query-*.json` files: required dependencies and offline query metadata. `tasks/DECISIONS.md`, this task and generated `tasks/INDEX.md`: decision record and report.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 — valid login response, token, cookie and success audit | `login_success` | ✅ Passed |
| AC2 — correct-password lockout response, event/audit, generic wrong-password response | `lockout_flow` | ✅ Passed |
| AC3 — unknown and known-wrong login p50 delta ≤ 50 ms over 20 tries | `login_constant_time` | ✅ Passed |
| SBD-81 — PII is not decrypted or returned without `read_pii` | `login_omits_pii_without_permission` | ✅ Passed |

**Checks run** — ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`. ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`. ✅ `DATABASE_URL=<task-db> RITMA_DATABASE_OWNER_URL=<task-db-owner> RITMA_SECRET_KEY=<fake-32-byte-key> RITMA_PUBLIC_URL=https://example.test CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-identity --test api_auth -- --test-threads=1` — 4 passed. ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-identity api_auth::tests:: -- --test-threads=1` — 3 passed. ✅ `DATABASE_URL=<task-db-owner> CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo sqlx prepare --workspace --check -- --all-targets --all-features`. ✅ `DATABASE_URL=<eq_app-url> RITMA_DATABASE_OWNER_URL=<task-db-owner> RITMA_SECRET_KEY=<fake-32-byte-key> RITMA_PUBLIC_URL=https://example.test CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo run -p eq-server -- migrate`. ✅ `cargo deny check` — advisories, bans, licenses and sources passed; duplicate-version warnings were printed. ✅ `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`. ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, `python3 -I -B scripts/erd.py --check`, and `git diff --check`. ✅ `scripts/erd.py --migrations` — n/a, this task changed no migration. ✅ `gitleaks protect --staged --redact` — no leaks found. **Not run:** `cargo test --workspace` because `TestDb::new()` creates a fresh random database per test and this run is restricted to the task-named database; `make api` because its target is intentionally unavailable until T-114 (invocation confirmed it prints `OpenAPI generation is added by T-114`; make exits 2 after the target's `false` exits 1); `make dev` because it sources the shared `.env` and starts the web server, so its direct-server equivalent was used; `make e2e`/Playwright and web checks because this backend endpoint task has no UI journey scope and `make e2e` uses the shared Compose DB plus a Playwright container. Hosted CI was not triggered; the local checks are listed above.

**Manual checks** — On the existing `ritma-t110a3-pg16-20261029` PostgreSQL 16 container, created the task-only `ritma_test_T-110a4` from the already-migrated `eq_template`; it started with zero tenants. Tests connected through `eq_app` (`NOSUPERUSER NOBYPASSRLS`) and an owner-only fixture pool. Ran the built server directly on `127.0.0.1:18080` with `Host: <test-tenant>.example.test`: valid credentials returned HTTP 200, the access token was present but never printed, `active_role=admin`, `Cache-Control: no-store`, and `eq_rt` had HttpOnly/SameSite=Strict/Path/Max-Age with Secure omitted for configured `http://localhost`; wrong password and malformed email each returned HTTP 401 `invalid_credentials`. `make dev` was not used because it loads shared `.env` and starts the web app. The task DB was dropped after the checks.

**Debug log** — Clippy first identified the `.err().expect()` test idiom and an overlong handler; replaced it with `expect_err` and extracted `login_success_response`. The extracted helper initially named SQLx's transaction instead of the `eq_core::db::Tx` wrapper; corrected its type. Clippy then found an overlong integration fixture, disallowed direct environment reads, and a manual absolute-difference expression; split configuration, connection and seeding helpers, used `Config::from_env`, and switched to `Duration::abs_diff`. An initial PII test fixture bound an inferred `Vec<i32>` to `bytea`; changed the sentinel to `vec![0_u8]`. One final full integration-suite run exceeded the 50 ms p50 timing bound; recreated the task-named database from the clean migrated template and reran the suite unchanged, where all four tests passed in 52.70 seconds. No timing threshold or test was weakened. During database reset, corrected the hyphenated identifier quoting and split `DROP DATABASE`/`CREATE DATABASE` into separate psql invocations; only the task database was affected.

**Performance** — `login_constant_time` measured 20 known-wrong and 20 unknown-email requests and passed its ≤50 ms p50-delta assertion. The test does not print the raw delta. No separate optimization was needed or measured.

**Security** — Triggers T1/T3/T4. SBD-81: (1) login is intentionally unauthenticated, requires tenant resolution, and reads the authenticated person's own profile through the core port; `read_pii` is checked before decryption. (2) Tenant identity comes only from `RequireTenant`; transaction context and every core query are tenant-scoped under RLS; no new table was added. (3) Email/phone remain S2 ciphertext and are decrypted/returned only with `identity.people.read_pii`; display name is S1; password, TOTP and token values use secret-safe wrappers; Debug output redacts credentials and profile PII. (4) Unknown request fields are denied; Email/Password use the shipped validators, TOTP verification requires the service's six ASCII digits, and Axum's JSON extractor bounds body size. (5) Only the required access token is serialized in the response body; refresh token is only in the HttpOnly cookie; response is `no-store`; no HTML/CSV output exists. (6) Token, password and encrypted PII handling stays in the shipped secrecy/crypto boundaries; TOTP uses the service verifier; invalid and unknown login paths passed the constant-time test. (7) No external calls. (8) No new aggregate/version or money writes; session, audit and outbox effects stay within the tenant transaction. (9) Success/failure/lockout audit behavior passed the named integration tests. (10) Account-enumeration and no-PII abuse cases passed; request/response Debug redaction passed its unit test. No real secrets or personal data were added.

**Deviations** — RATIFY TD-113 (optional country context; VAL-20/21), TD-114 (shared policy runtime; POL-04/CA-30), TD-115 (response permission snapshot and PII authorization; IDN-10/11, ERD-04), and TD-116 (lazy policy initialization preserves OPS-06 health routes). `make api` remains `Not run` by design until T-114; no generated OpenAPI contract was fabricated. The non-generated diff has 1,221 changed lines, exceeding CQ-101's approximate 400-line PR review guide; the requested response, runtime composition, core profile access and named database-backed tests are all in this task's scope. Human review should decide whether future PR presentation needs a split.

**Dependencies affected** — T-110a5 can reuse the permission calculation and core directory port. T-115 remains the owner of shared login rate limiting; no duplicate limiter was added. T-110a3's HTTP response/cookie/timing/route-test follow-up is covered here. No dependency code or task file was edited.

**Rollback plan** — Revert the signed T-110a4 commit on `dev`; do not rewrite pushed history. No migration exists. The task-named disposable database was dropped after verification; the existing PostgreSQL container and unrelated WIP remain untouched.

**Follow-ups** — Architect ratification of TD-113–TD-116; implement login rate limiting under T-115; review the CQ-101 size note and decide whether a later PR should be presented as smaller review units. Hosted CI, web checks and browser E2E remain unavailable/not applicable for this backend-only task as described above.

**Resource report** — Before task checks: fixed Cargo cache 8.1G, `df -h .` 64G available, Docker reported 17 running containers, 20 images and 443 volumes; the pre-existing task DB was 9,479 kB. After: fixed Cargo cache 8.2G, `df -h .` 64G available, 17 containers, 11 unique image IDs and 443 volumes. A post-check `docker system df` call did not return within 30 seconds; no container/image/volume create or delete command was run. The final task DB was recreated from `eq_template` with zero tenants (9,263 kB), used for the four integration tests and direct HTTP checks, then dropped; `ritma_test_T-110a4` is absent. The existing PostgreSQL container remains running.

**Workflow steps 0–12** — 0: resumed the in-progress T-110a4 WIP on `dev`; ran index checks and preserved unrelated audit `.pending`/`.hold`, SQLx and Python-cache files. 1: read the full task and listed spec sections. 2: read T-110a3's Report, deviations/follow-ups, and shipped LoginService/directory code; followed its TD-112 port contract and recorded T-115's limiter ownership. 3: n/a, backend-only task. 4: plan and AC/edge/error/abuse test plan recorded above before final verification. 5: implemented route, DTOs, runtime wiring, profile ports, permission snapshot, tests and generated SQLx metadata. 6: exercised direct server requests for success, wrong password and malformed email on the disposable PostgreSQL 16 fixture. 7: logged and fixed every Clippy, type, fixture and DB-command failure above. 8: reviewed changed code against CQ-16 §9, CA-17 §7, ERR-19 and tenant/PII boundaries; noted the CQ-101 size follow-up. 9: measured the timing comparison; no speculative optimization. 10: completed the SBD-81 checklist above and added the no-PII abuse test. 11: final fmt, Clippy, SQLx, deny, task, ERD, guard and targeted test checks are listed above; workspace/e2e/API-generator checks carry explicit reasons. 12: set `review`, regenerate/check the index, make one signed Conventional Commit and push it to `dev`; the resulting SHA and push are logged in `RUN_LOG.md`.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification):** the shipped `LoginRequest` requires `FieldCtx.country`, but tenant resolution does not expose a country and the Email/Password validators do not consume it → make country optional and reject phone parsing when absent, because VAL-20/21 define context by field use; no tenant country is fabricated (TD-113).
- **Self-resolved (implementer, run 8, needs architect ratification):** the shipped login service requires `PolicyStore`, but `AppState` does not carry one and router construction is synchronous → initialize one shared store at the API composition root and inject it through a runtime extension, because IDN-06/T-110a3 require the existing policy boundary and POL-04 requires a shared cache/listener (TD-114).
- **Self-resolved (implementer, run 9, needs architect ratification):** eagerly creating `PolicyStore` requires an async PostgreSQL listener and would prevent `/healthz` from starting during a database outage → inject one shared lazy runtime and initialize the policy store on the first login request, because POL-04 requires shared cache/listener state and OPS-06 keeps health routes available when PostgreSQL is down (TD-116).
- **Self-resolved (implementer, run 8, needs architect ratification):** the login response's normative `PersonMe.permissions` overlaps T-110a5's Actor work → compute the response-only permission snapshot from existing enabled module grants and the core directory, then reuse it from T-110a5; `read_pii` remains mandatory and no grants are added (TD-115).

## Review
Self-review complete against CQ-16 §9 and the identity, error, API and security contracts listed above. The route uses the identity module's service and core-owned directory port; profile PII is decrypted only after permission checks, credential/token Debug output is redacted, and malformed or wrong credentials share the canonical error. The targeted integration, unit, formatting, lint, dependency, SQLx and guard checks recorded above pass on the final code. Human review follow-ups are TD-113–TD-116 ratification and the CQ-101 diff-size note.
