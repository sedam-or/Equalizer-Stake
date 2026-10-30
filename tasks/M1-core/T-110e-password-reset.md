---
id: T-110e
title: Implement password reset and recovery
milestone: M1
status: review
depends_on: [T-110d]
spec_refs: [IDN-02, IDN-08, SEC-12, STM-24, AUTH-06]
estimated_size: M
---

## Goal
Provide non-enumerating password recovery with policy-bound, single-use reset tokens, rate limits, and session revocation.

## Context
- T-111 defines POST /auth/forgot {email} as 202 for known/unknown addresses with constant work. For an active person below IdentityPolicy.reset_max_per_hour, create a reset token with TTL IdentityPolicy.reset_token_ttl_minutes and emit identity.password_reset_requested.v1 {person_id, token_id}. CAN-60, FLW-60 and spec/03 prohibit the raw token in the event payload; see TD-143. The token is never returned to the caller.
- POST /auth/reset {token, new_password} returns 204, updates password/version, marks the token used, revokes every refresh session for that person and audits security.password.reset. Invalid/expired/used token → 400 token_invalid; weak password → 422 password_weak.

## Scope
### In
- Implement forgot/reset routes, service, persistence, event and audit behavior and the three exact acceptance tests.
### Out
- Invite, profile, sessions and TOTP behavior.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_reset.rs | create | Forgot/reset HTTP DTOs and handlers. |
| crates/eq-identity/src/service.rs | modify | Non-enumerating reset service and session/cache invalidation. |
| crates/eq-identity/src/repo.rs | modify | Tenant-scoped reset token operations. |
| crates/eq-identity/src/module.rs | modify | Register routes. |
| crates/eq-identity/src/errors.rs | modify | Reuse existing spec-defined errors only. |
| crates/eq-identity/tests/api_reset.rs | create | Exact acceptance tests. |
| .sqlx/*.json | modify | Offline SQL metadata for query! statements changed by this task. |
| tasks/DECISIONS.md | modify | Record TD-143 through TD-146. |
| tasks/M1-core/T-110e-password-reset.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
- POST /api/v1/auth/forgot {email} → 202 with an identical response body for known and unknown addresses.
- POST /api/v1/auth/reset {token,new_password} → 204; use the existing errors token_invalid and password_weak.
- Outbox payload includes only the spec-defined person_id and token_id; the raw token is never persisted, logged, returned, or emitted (TD-143).

## Business rules
Use only IdentityPolicy.reset_max_per_hour and reset_token_ttl_minutes. Count password_resets rows in the last policy window defined by the task; do not create a new policy field or default.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 reset token hash, password hash and outbox token; S1 person id.
- S/E: account enumeration, replay and tenant crossover → constant-work 202 response, policy cap, tenant-scoped hash lookup and single-use transaction (AUTH-06, SBD-43, STM-24).
- I: token/password leak → hash persisted in table, the raw token stays transient, the event contains only IDs, and Debug is redacted with no logs (SBD-10/21/23/50).
- Abuse-case acceptance criterion: forgot_rate_limited_silently verifies the policy cap is enforced without revealing account existence.

## Implementation notes
Use the existing Core identity directory for canonical email resolution and existing AuthState cache invalidation. Do not add an event name, table, column, permission or error code.

## Acceptance criteria
- AC1. Known and unknown addresses receive the same 202 response; only a known active account under the cap creates an event → forgot_no_enumeration.
- AC2. The request beyond reset_max_per_hour creates no token and still returns 202 → forgot_rate_limited_silently.
- AC3. Reset revokes all refresh sessions and consumes the token atomically → reset_revokes_sessions.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_reset.rs | forgot_no_enumeration, forgot_rate_limited_silently, reset_revokes_sessions |

## Definition of done
- [x] Exact acceptance tests pass on the disposable ritma_test_T-110e database in the existing PostgreSQL 16 container.
- [x] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [x] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [x] OpenAPI generation is attempted; SQLx metadata is refreshed after query changes.
- [x] Signed-off commit and complete Step 12 Report.

## Report
**Plan** — 1. Reuse T-110d's API/runtime and task-database setup → verify only Identity owns reset SQL and the existing `ritma-db-1` is reused. 2. Add tenant-scoped reset token issue/count/consume/revoke repository operations and service orchestration → verify token hashing, rate cap, single-use transition, password version update, audit and IDs-only event. 3. Add forgot/reset DTOs and routes → verify identical 202 responses, 204 success, and registered token/password errors. 4. Add the named PostgreSQL-backed API tests plus edge and secret-redaction cases → verify against fresh `ritma_test_T-110e`. 5. Run focused Rust, SQLx, security, guard, E2E and task checks → record the exact commands, database, browser results and resource cleanup.

**Assumptions** — The dependency's shipped `LoginHttpState` and Core `by_email` port are the runtime contracts. Event payloads follow the exact spec-defined `{person_id, token_id}` shape; the raw-token delivery handoff remains a follow-up because no secure handoff is specified (TD-143). A password-reset bearer request uses the existing `System` actor role; the audit identifies the affected person as the entity and does not guess an authenticated role (TD-145). The task database is named for this task and `.sqlx` metadata is included in Files (TD-144).

**Test plan** — AC1 Given known and unknown email addresses, when `POST /forgot` is called, then both return the same 202 body and only the known active account under the cap inserts a reset row and IDs-only event → `forgot_no_enumeration`. AC2 Given the account has reached `reset_max_per_hour`, when one more request arrives, then it still returns the same 202 and creates no additional row or event → `forgot_rate_limited_silently`. AC3 Given a valid unused reset token and active refresh sessions, when reset succeeds, then the new password is stored, the token is consumed, all refresh sessions are revoked, the whole-second password version advances, the password version cache is invalidated, and the security audit is committed atomically → `reset_revokes_sessions`. Edge/error cases: malformed, unknown, expired, used, and cross-tenant tokens return 400 `token_invalid`; weak passwords return 422 `password_weak`; concurrent consume attempts allow at most one success. Abuse cases: known/unknown and rate-limited requests remain indistinguishable; S3 reset tokens are absent from event payloads, responses, logs and Debug; all SQL remains tenant-scoped under `eq_app` RLS.

**Spec read** — `spec/03-module-system.md` §6; `spec/06-domain-model.md` §4.2; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4; `spec/18-security-by-design.md` §§4, 5, 8; `spec/19-error-handling.md` §§1–5, 10; `spec/20-state-machines.md` STM-24; `spec/21-data-flows.md` FLW-12 and FLW-60; `spec/22-validation-and-sanitization.md` VAL-21/22; `spec/23-contracts-and-canonical-forms.md` CAN-60/61/63; `spec/25-auth-and-access-control.md` AUTH-06/IDN-08; and `spec/29-erd.md` identity module / ERD-06 row for `0007_identity.sql`.

**Summary** — Added non-enumerating forgot/reset routes backed by hash-only, tenant-scoped reset tokens, the existing Identity policy, and single-use transactional consumption. Successful reset updates the password, advances its whole-second access-token version, revokes all refresh sessions, records a security audit, and invalidates the in-process password-version cache. The request event contains only the spec-defined person and token ids; no raw token handoff was added.

**Files** — Created `crates/eq-identity/src/api_reset.rs` for HTTP DTOs and handlers; created `crates/eq-identity/tests/api_reset.rs` for HTTP acceptance, abuse and rollback coverage. Modified `crates/eq-identity/src/service.rs` and `repo.rs` for reset flow and persistence; `module.rs`, `lib.rs` and `errors.rs` for route and error registration; added 17 task query metadata files under `.sqlx/`; updated `tasks/DECISIONS.md`, this task report and generated `tasks/INDEX.md`. No migration or schema object changed.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1: known and unknown email return identical 202 responses; only an eligible account emits a reset request | `forgot_no_enumeration` | ✅ Passed; empty 202 bodies match, one hash-only reset row and IDs-only event are created for the known account |
| AC2: rate-capped request remains non-enumerating and creates no token | `forgot_rate_limited_silently` | ✅ Passed; response remains 202 and the cap prevents another reset row |
| AC3: valid token is consumed atomically, password version advances, sessions are revoked and reset is audited | `reset_revokes_sessions` | ✅ Passed; replay is invalid, refresh sessions are revoked and the audit is recorded |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true DATABASE_URL='postgres://eq_app:eq_app@127.0.0.1:5433/ritma_test_T-110e' RITMA_DATABASE_OWNER_URL='postgres://eq:eq@127.0.0.1:5433/ritma_test_T-110e' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' RITMA_BACKUP_SCHEDULE=off cargo test -p eq-identity --test api_reset -- --nocapture` — 3/3 passed on PostgreSQL 16 in `ritma_test_T-110e` inside the existing `ritma-db-1`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-identity --lib redacts -- --nocapture` — 8/8 secret-debug tests passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate-version warnings remain.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check` and `python3 -I scripts/erd.py --migrations` — 25 mapped tables checked; no migration was changed by this task.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL='postgres://eq:eq@127.0.0.1:5433/ritma_test_T-110d' SQLX_OFFLINE=false cargo sqlx prepare --workspace --check -- --all-targets`; the fresh task database was migrated, then dropped. SQLx metadata generation `cargo sqlx prepare --workspace -- --all-targets` had completed after the final query change.
- ✅ `pnpm -C web build`; no web source was changed. ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo build --release -p eq-server`.
- ⚠️ Local browser E2E command in Manual checks exited nonzero after 4/6 passed. The two WebKit page tests are **Not run** because Playwright cannot start WebKit without host `libicu74` and `libflite1`; Chromium passed 3/3 and WebKit health passed 1/1. `make e2e` itself was not used because it hard-codes `target/release/ritma` outside the fixed Cargo cache and writes Playwright artifacts into existing workspace paths; the local equivalent was run with its output under the cache.
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, `python3 -I -B scripts/tests/test_tasks_lint.py` (8/8), `git diff --check`, `git diff --cached --check`, and `gitleaks protect --staged --redact` (no leaks).
- `make api` was attempted and exits 2 with `OpenAPI generation is added by T-114`; OpenAPI artifacts are **Not run** because the repository target is still the T-114 placeholder. `cargo test --workspace` was not run because it launches additional Testcontainers PostgreSQL containers, outside the one-container limit. Web lint/typecheck/unit/i18n checks were not run because no web source changed.

**Manual checks** — The exact browser command was `E2E_BASE_URL='http://127.0.0.1:18080' pnpm -C web exec playwright test --config=e2e/playwright.config.ts --reporter=line --output="$HOME/.cache/ritma/target/t110e-e2e-output"`. It exercised the release server over HTTP against a fresh PostgreSQL 16 database `ritma_test_T-110d` in the existing container `ritma-db-1`; the database was migrated and seeded with one obviously synthetic active tenant so the SPA gate could serve pages. Result: 4/6 passed (Chromium 3/3 and WebKit health 1/1); WebKit `loads home` and `spa fallback` could not start because the host lacks `libicu74` and `libflite1`. `/readyz` and `/` returned 200 before Playwright. Both task databases, `ritma_test_T-110e` and the E2E-only `ritma_test_T-110d`, were dropped after checks; `ritma-db-1` remained running. The API tests used a real local TCP Axum server and verified `eq_app` is non-superuser/non-bypass-RLS.

**Disk hygiene** — Before T-110e: `du -sh "$HOME/.cache/ritma/target"` → `6.4G`; `docker system df` → Images 7 total / 10.96GB / 0B reclaimable, Containers 9 / 10.49MB / 0B reclaimable, Local Volumes 6 / 5.032GB / 0B reclaimable, Build Cache 0. After T-110e: `du -sh "$HOME/.cache/ritma/target"` → `6.5G`; `docker system df` remained Images 7 / 10.96GB / 0B reclaimable, Containers 9 / 10.49MB / 0B reclaimable, Local Volumes 6 / 5.032GB / 0B reclaimable, Build Cache 0. Reused existing container `ritma-db-1`; created no Docker container, image or volume. Created databases `ritma_test_T-110e` (API tests) and `ritma_test_T-110d` (fresh E2E/SQLx verification); both were dropped. The shared Cargo target grew by 0.1G while compiling the required test/release artifacts; `cargo clean` is forbidden, so the no-growth target-size rule could not be met. The E2E output and data folders remain under the fixed target cache.

**Debug log** — The first migration attempt used an owner password that did not match the existing container; rerunning with its configured `eq:eq` owner credentials succeeded. The first browser attempt used an unprovisioned database and correctly returned `setup_required` for SPA requests; a fresh T-110d E2E database with one synthetic tenant made the app routes available. The final browser run passed the Chromium flows; WebKit could not launch due missing host libraries. An earlier Clippy iteration identified oversized reset helpers/tests; the logic was split into small helpers and the final workspace Clippy check passed. No failure remained in the named API tests.

**Performance** — n/a; this task defines no performance target and no benchmark was requested.

**Security** — Triggers: T1, T3, T4. SBD-81 review: (1) reset endpoints are public bearer/email operations; tenant scope comes from `RequireTenant` and the resolved context, while the reset token is the only authority for password mutation; (2) all repository queries filter by tenant, execute through the tenant transaction and remain subject to the mapped RLS policy; (3) reset tokens are 32 random bytes, canonically encoded, hashed before persistence and held in `SecretString`/`SecretBox` with handwritten redacting Debug; passwords are validated against current policy and hashed off-thread; (4) forgot always returns the same empty 202 for known, unknown, malformed and rate-limited addresses, and `forgot_rate_limited_silently` covers the abuse path; (5) no user-controlled URL, HTML or export path was added; (6) token consumption is locked and revalidated inside the mutation transaction; password update, token use, session revocation and audit commit atomically, and the reset event carries only IDs; (7) no third-party call is introduced; (8) password-version cache invalidation runs only after commit; (9) failed audit rolls back the reset transaction; (10) acceptance and secret Debug tests passed. Gitleaks found no secrets in staged task changes.

**Deviations** — RATIFY TD-143: emit the normative `{person_id, token_id}` event and defer the raw-token handoff because CAN-60/FLW-60 prohibit S3 event payloads and no secure handoff is specified. RATIFY TD-144: retain the task-scoped T-110e test database and include the required SQLx metadata. RATIFY TD-145: attribute the bearer reset audit to the existing `System` actor and affected person entity because token possession does not establish a membership role. RATIFY TD-146: advance `password_changed_at` by one technical second when needed so the shipped whole-second JWT version changes without a new column. No business value, permission, role grant, error code, event name or schema object was added.

**Dependencies affected** — T-111 should reconcile its raw-token event wording with TD-143 and arrange a spec-approved Notify token handoff. T-114 should include the two reset routes in OpenAPI and generated client output.

**Rollback plan** — Revert the signed T-110e commit. No migration or persistent database change ships; task databases were dropped after verification.

**Follow-ups** — Architect ratification of TD-143–TD-146; design a secure reset-token handoff for Notify without placing S3 values in events; reconcile T-111's copied raw-token wording; regenerate OpenAPI under T-114; run WebKit page checks on a host with the required libraries; run container-backed workspace tests in an environment that allows their additional containers.

**Workflow steps 0–12** — 0: resumed `dev` at T-110d review commit `c158b565`, preserved unrelated WIP, and continued the already-claimed lowest eligible task T-110e. 1: reread this task and its cited identity, security, validation, canonical-form, error, state-machine and quality rules. 2: read T-110d's Report, TD-140–TD-142 deviations/follow-ups and the shipped Core directory, login runtime, invite transaction and integration patterns. 3: n/a, backend task. 4: recorded this plan, assumptions, AC mapping, edges and abuse cases before implementation. 5: implemented only the reset handlers, service, repository operations, tests, metadata, decisions and task report listed in Files. 6: exercised real HTTP forgot/reset flows and the SPA/browser routes locally. 7: recorded the incorrect DB credential and initial unprovisioned E2E setup; fixed both, then documented the host-limited WebKit launch. 8: reviewed tenant scoping, transaction/audit boundaries, password-version invalidation, secret redaction, registered errors and exact task scope against spec/16, spec/17 and spec/19. 9: n/a; no performance target or benchmark applies. 10: completed the SBD-81 checks above and scanned staged changes for secrets. 11: reran focused API/unit tests, format, Clippy, cargo-deny, SQLx metadata, guards, ERD, E2E, task index/lint, diff and Gitleaks; unavailable checks are explicitly recorded. 12: completed the Report, set `review`, regenerated the index, and prepared the single signed-off task commit for `dev`.

## Questions
- **Self-resolved (implementer, 2026-10-30; TD-143, needs architect ratification):** T-110e and T-111 request a raw S3 reset token in `identity.password_reset_requested.v1`, but CAN-60 and FLW-60 forbid S2/S3 event values and spec/03 defines only `{person_id, token_id}` → emit the exact spec-defined IDs-only payload; defer a secure Notify token handoff because neither a port nor protected persistence exists in this task's scope/schema.
- **Self-resolved (implementer, 2026-10-30; TD-144, needs architect ratification):** the DoD names another task's `ritma_test_T-111` database and Files omits the required offline metadata → use fresh `ritma_test_T-110e` and list `.sqlx/*.json`, because task-scoped database isolation and SQLx offline builds require task-owned names and metadata.
- **Self-resolved (implementer, 2026-10-30; TD-145, needs architect ratification):** the bearer reset route has no authenticated role, while the shipped audit adapter rejects an `Anonymous` actor and the Core port exposes no role lookup by token owner → use the existing `System` actor and record the target person as the audit entity, because claiming a user role would invent identity facts and the audit adapter already supports system actions.
- **Self-resolved (implementer, 2026-10-30; TD-146, needs architect ratification):** `password_changed_at` is converted to a whole-second JWT version, so a reset in the same second as the prior password change could leave access tokens valid → set the timestamp to the later of the injected current time and the prior whole-second timestamp plus one technical second, because the shipped IDN-03 version query floors epoch seconds and requires an observable password-version change.

## Review
Ready for maintainer review. The three named API acceptance tests pass; the local browser run is 4/6 with two WebKit page checks unavailable because required host libraries are absent. Please ratify TD-143–TD-146 and review the reset-token handoff follow-up.
