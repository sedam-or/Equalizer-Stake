---
id: T-110d
title: Preview and accept identity invites
milestone: M1
status: review
depends_on: [T-110c]
spec_refs: [IDN-02, IDN-07, STM-20, STM-21, AUTH-06]
estimated_size: M
---

## Goal
Allow an invite holder to preview a pending invitation and accept it once by setting a policy-valid password and starting an authenticated session.

## Context
- T-111 defines GET /auth/invites/{token} → tenant_name, display_name, role, expires_at. Unknown tokens return 404; expired or accepted tokens return 410 invite_expired.
- POST /auth/invites/{token}/accept {password} validates Password against the current IdentityPolicy, marks the invite accepted, sets the credential password and timestamp, activates the invited membership through Core, creates the login response and refresh cookie, and audits identity.invite.accepted.
- Invite tokens are 32 random bytes; only their SHA-256 hash is stored. The raw token is never logged.
- `tenant_name` comes from Core-owned `tenants.name` through the function `public.resolve_tenant_name(uuid)` added by migration `0012_core_tenant_name_resolver.sql` (TD-138). Identity reads it only through `CoreIdentityDirectory::tenant_name`.

## Scope
### In
- Implement the public invite preview and acceptance endpoints and the acceptance transaction.
- Add `tenant_name` to the invite preview via the Core port defined in TD-138.
- Keep invite lookup tenant-scoped and token comparison constant-work as required by STM-21/AUTH-06.
### Out
- Invite issuance/resend (T-110c, the shipped dependency), reset, profile or TOTP behavior.
- Any other migration, function, table or column than `0012_core_tenant_name_resolver.sql`.

## Files
| Path | Action | Purpose |
|---|---|---|
| migrations/0012_core_tenant_name_resolver.sql | create (shipped by architect, TD-138) | SECURITY DEFINER `resolve_tenant_name(uuid) → text`, grant to `eq_app`. |
| crates/eq-core/src/auth/directory.rs and its Postgres implementation (`grep -rn "impl CoreIdentityDirectory" crates/eq-core`) | modify | Add `tenant_name(tx, tenant) -> Result<Option<String>, AppError>`, implemented by `SELECT public.resolve_tenant_name($1)`. |
| crates/eq-identity/src/api_invites.rs | create | Preview and accept HTTP DTOs/handlers. |
| crates/eq-identity/src/invites.rs | modify | Accept and preview service behavior. |
| crates/eq-identity/src/repo.rs | modify | Invite lookup and single-use acceptance. |
| crates/eq-identity/src/errors.rs | modify | Register the existing spec-defined `invite_expired` code. |
| crates/eq-identity/Cargo.toml | modify | Declare the existing workspace `subtle` dependency for constant-time digest comparison. |
| Cargo.lock | modify | Record the existing workspace `subtle` dependency in the locked identity graph. |
| `.sqlx/*.json` | generate | Offline query metadata for all workspace targets after adding queries. |
| crates/eq-identity/src/service.rs | modify | Reuse login session creation for an invite-accepted identity. |
| crates/eq-identity/src/api_auth.rs | modify | Reuse the existing login response and refresh-cookie builder. |
| crates/eq-identity/src/module.rs | modify | Register routes. |
| crates/eq-identity/tests/api_invites.rs | modify | Exact acceptance tests. |
| tasks/M1-core/T-110d-invite-acceptance.md | modify | Plan and final Report. |
| tasks/DECISIONS.md | modify | Record TD-140 for the Core-owned acceptance transition. |

## Interfaces
- GET /api/v1/auth/invites/{token} returns {tenant_name, display_name, role, expires_at}.
- POST /api/v1/auth/invites/{token}/accept accepts {password} and returns the existing login response plus the refresh cookie.
- Errors are the spec-defined 404, 410 invite_expired and 422 password_weak; add no error code.
- SQL (Core only): `SELECT public.resolve_tenant_name($1)` inside `core::db::begin`, so the `app.tenant_id` setting restricts the result to the caller's tenant.

## Business rules
Use only IdentityPolicy.password_min_length, invite_expires_days and the injected Clock. Acceptance is single-use and activates the membership in the same transaction as credentials, refresh family and audit.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 invite token hash/password hash; S2 profile fields in the existing login response; S1 tenant name (non-personal).
- S/E: token guessing, replay or cross-tenant lookup → tenant-scoped hash lookup, single-use transaction and policy validation (SBD-40/41/42, STM-21). `resolve_tenant_name` returns a name only when the id equals the tenant set by `core::db::begin`, so it is not a cross-tenant lookup.
- I: secret leakage → hash-only token persistence, redacted Debug and no token/PII logging (SBD-10/21/50).
- Abuse cases: accept_ok and accept_expired_410 cover valid, expired and replayed invite paths.

## Implementation notes
Reuse the invite repository/service shipped by T-110c and the existing LoginService success response; do not query memberships or `tenants` in the Identity repository.

## Acceptance criteria
- AC1. A valid invite accepted inside its TTL returns the login response and activates the membership → api_invites::accept_ok.
- AC2. Expired or accepted invite tokens return 410 invite_expired → accept_expired_410.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_invites.rs | accept_ok, accept_expired_410 |

## Definition of done
- [x] Exact acceptance tests pass against the task database in the existing PostgreSQL 16 container.
- [x] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [x] Migration 0012 is applied in the task database, and a test confirms `eq_app` can call `resolve_tenant_name` for its own tenant and gets NULL for another tenant's id.
- [x] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [x] OpenAPI generation is attempted and T-114 placeholder recorded as Not run; SQLx metadata is updated if queries change.
- [x] Signed-off commit and complete Step 12 Report.

## Report
**Plan**
1. Add the Core directory port for the `invited → active` transition and tenant-name lookup → verify Core has the only membership/tenant SQL.
2. Add tenant-scoped invite preview and single-use acceptance repository operations → verify pending, expired, accepted, and unknown token results.
3. Add acceptance service behavior for password policy, membership activation, audit, and login-session creation → verify all effects share one transaction.
4. Add public preview/accept HTTP handlers and register them → verify request/response shapes, problem codes, refresh-cookie behavior, and invitee audit attribution within the actor-scoped transaction.
5. Extend the existing invite integration fixture and add acceptance/resolver tests → verify only `ritma_test_T-110d` is used.
6. Run focused Rust, database, security, OpenAPI, schema, and repository checks → update this Report and the generated task index.
**Assumptions** — Membership activation follows STM-20 t3 and is performed only through the Core directory port. The existing login success DTO, session policy, permissions, and refresh-cookie behavior are reused.
**Test plan**
- AC1: Given a pending invite inside its TTL and a policy-valid password, when the holder accepts it, then the endpoint returns the existing login response and refresh cookie, marks the invite accepted, sets the credential, activates the invited membership, creates a refresh family, and writes the acceptance audit attributed to the invitee → `accept_ok`.
- AC2: Given an expired or already accepted invite, when the holder accepts it, then the endpoint returns 410 `invite_expired` and creates no second session → `accept_expired_410`.
- Preview: Given a pending invite, when its token is previewed, then tenant name, display name, role, and expiry are returned without token or PII logging → `preview_ok`.
- Edge cases: unknown/malformed token; expiry boundary; replay; token presented in another tenant; wrong membership state; policy minimum password boundary.
- Error cases: 404 unknown token, 410 expired/accepted token, 422 weak password.
- Abuse cases: guessing, replay, and cross-tenant lookup; ensure raw tokens and passwords stay redacted and absent from audit/event/log output; verify acceptance audit actor matches the invitee and `eq_app` resolver returns its own tenant name and NULL for another tenant.

**Summary** — Added invite preview and one-use acceptance over the existing invite store. Acceptance validates the current password policy, activates the invited membership through Core, writes credentials/session/audit in one transaction, and returns the existing login response. The preview exposes only the specified fields and resolves the tenant name through Core's tenant-scoped function.

**Spec read** — `spec/06-domain-model.md` identity DDL; `spec/07-api-conventions.md` API-15; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 7; `spec/18-security-by-design.md` §§4, 5, 8; `spec/19-error-handling.md` §§1–5, 10; `spec/20-state-machines.md` STM-20/21; `spec/21-data-flows.md` FLW-12; `spec/25-auth-and-access-control.md` IDN-07/AUTH-06; and the ERD reference required by TD-138.

**Files** — Modified `crates/eq-core/src/auth/directory.rs` — Core activation and tenant-name ports.
- Created `crates/eq-identity/src/api_invites.rs` — invite preview and acceptance handlers.
- Modified `crates/eq-identity/src/api_auth.rs` — reusable login response and cookie helpers.
- Modified `crates/eq-identity/src/errors.rs` — existing `invite_expired` error code.
- Modified `crates/eq-identity/src/invites.rs` — invite preview, validation and acceptance services.
- Modified `crates/eq-identity/src/lib.rs` — module registration.
- Modified `crates/eq-identity/src/module.rs` — route registration.
- Modified `crates/eq-identity/src/repo.rs` — invite lookup and single-use acceptance persistence.
- Modified `crates/eq-identity/src/service.rs` — password, TOTP and login-session handling.
- Modified `crates/eq-identity/tests/api_invites.rs` — acceptance and abuse-case integration tests.
- Modified `crates/eq-identity/Cargo.toml` — existing workspace `subtle` dependency.
- Modified `Cargo.lock` — locked identity dependency graph.
- Created 17 `.sqlx/*.json` files — offline metadata for new/changed queries.
- Modified `tasks/DECISIONS.md` — TD-140 through TD-142.
- Modified `tasks/M1-core/T-110d-invite-acceptance.md` — plan and final Report.
- Modified `tasks/INDEX.md` through `scripts/tasks-index.sh` — generated task status.
- No migration was changed; architect-provided migration 0012 was applied to the disposable task database.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1 — valid invite returns login response and activates membership | `api_invites::accept_ok` | ✅ pass; response cookie, session, credentials, active membership and invitee audit verified |
| AC2 — expired or accepted invite returns 410 and replay creates no second session | `api_invites::accept_expired_410` | ✅ pass |
| Preview, resolver isolation, unknown and cross-tenant tokens | `api_invites::preview_ok` | ✅ pass as `eq_app`; own tenant name returned, other tenant returned NULL |
| Weak password is rejected without activation | `api_invites::accept_password_weak` | ✅ pass |
| Existing TOTP factor is preserved | `api_invites::accept_requires_totp_for_enrolled_credential` | ✅ pass; acceptance rolled back with `totp_required` |
| Secret values are redacted from Debug | invite module unit tests | ✅ 2/2 pass |

**Checks run**
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-identity --lib invites::tests:: -- --nocapture` — 2 passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true DATABASE_URL=postgres://eq_app:eq_app@127.0.0.1:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@127.0.0.1:5433/ritma_test_T-110d RITMA_SECRET_KEY=test-secret-T-110d-fake-key-00000000000000000000000000000000 RITMA_PUBLIC_URL=http://localhost:8080 cargo test -p eq-identity --test api_invites -- --nocapture` — 7 passed against the disposable task database in existing `ritma-db-1`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo deny check` — advisories, bans, licences and sources passed; existing duplicate-version warnings remain.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check && python3 -I scripts/erd.py --migrations` — spec map matches; 25 migrated tables checked. No migration was changed by this task.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL=postgres://eq:eq@127.0.0.1:5433/ritma_test_T-110d SQLX_OFFLINE=false cargo sqlx prepare --workspace --check -- --all-targets --all-features`.
- ❌ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 DATABASE_URL=postgres://eq_app:eq_app@127.0.0.1:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@127.0.0.1:5433/ritma_test_T-110d RITMA_SECRET_KEY=test-secret-T-110d-fake-key-00000000000000000000000000000000 RITMA_PUBLIC_URL=http://localhost:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off E2E_BASE_URL=http://127.0.0.1:8080 make e2e` — against fresh `ritma_test_T-110d` in `ritma-db-1`, 4/6 passed (3 Chromium tests and WebKit health check); 2 WebKit page tests could not launch because this host lacks `libicu74` and `libflite1`. The first run used an unprovisioned DB and root returned the expected `setup_required` 503; after seeding one test-only tenant from the existing `crates/eq-api/tests/spa.rs` fixture, the second run produced 4/6. No extra container was started.
- **Not run:** OpenAPI generation. `make api` was attempted and returned its explicit T-114 placeholder.
- ✅ `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py` passed after setting T-110d to `review` and again after the final Report edits, before commit.
- ✅ `/home/kepolu/.local/bin/gitleaks protect --staged --redact` passed on only the files staged for this task; no leaks found.
- ✅ `git diff --check` and `git diff --cached --check` passed after the final Report edits.
- **Not run:** `cargo test --workspace` because workspace `TestDb` tests start extra testcontainers and databases beyond the allowed single-container/task-DB boundary. `make dev` was not run because it sources shared `.env`; `make e2e` started the real server and exercised browser requests against the disposable task DB instead. Web lint/typecheck/unit tests are not applicable because no web source changed; the E2E target built the production web bundle. Full WebKit browser rendering is not run on this host due the missing system dependencies; no privileged package install or extra Playwright container was used.

**Manual checks** — `api_invites.rs` issued real Axum router requests for preview, valid acceptance, weak-password rejection, TOTP rejection, expired invite and replay using separate owner and `eq_app` pools. `preview_ok` verifies migration 0012 is applied and that `eq_app` resolves its own tenant name but receives NULL for another tenant. The E2E run built the SPA and release server, then used the single task DB; its synthetic tenant values match the existing SPA integration fixture. After all tests, confirmed zero connections to `ritma_test_T-110d` and dropped only that database from `ritma-db-1` using `DROP DATABASE "ritma_test_T-110d"`. The temporary `target` symlink to `$HOME/.cache/ritma/target` was removed; the target cache and existing container/volume were retained.

**Debug log** — `preview_ok` first used a slug longer than the existing 40-character constraint; shortened the test-only slug and reran the suite. Valid acceptance first returned 500 due audit actor/transaction mismatch; TD-142 resolved it with a read-only invitee lookup followed by actor-bound revalidation, after which the API suite passed 7/7. Workspace Clippy found a stale `#[expect(clippy::too_many_arguments)]`, which was removed. The first migration command lacked the required fake `RITMA_SECRET_KEY`; reran with the documented test-only value and migration succeeded. Initial `make e2e` hit the unprovisioned-tenant setup gate; seeded a single tenant with the already-used test fixture, then reran. The final E2E run passed 4/6; 2 WebKit page tests could not launch because host browser libraries are missing. The database name contains hyphens, so the first drop SQL needed quoting; confirmed zero sessions and the quoted drop then succeeded.

**Performance** — No latency benchmark was warranted for this write/preview endpoint task. `EXPLAIN` on the tenant-scoped invite hash lookup used a sequential scan on the tiny test fixture; the existing unique constraint on `invites.token_hash` supplies the lookup index.

**Security** — Triggers T1, T3 and T4. SBD-81: (1) both public operations explicitly declare no HTTP auth scheme because the opaque invite token is the bearer capability; tenant context is required before data access and unknown/cross-tenant invites return 404; shared API-15 rate limiting remains a T-115 follow-up. (2) No tenant id comes from the client; all queries use resolved tenant context/RLS. (3) Password hashes are persisted through the existing argon2 path, invite hashes are held in `SecretBox<Zeroizing<_>>`, and preview returns only specified name/role/expiry fields. (4) Password input is parsed against current policy; token parser requires the encoded form of 32 bytes; roles use the closed enum. (5) No HTML/CSV output or unsafe URL rendering was added. (6) Tokens use existing `OsRng` issuance, SHA-256 storage and `subtle::ConstantTimeEq`; token/password Debug is hand-redacted and no raw secret is logged. (7) No external calls. (8) Invite conditional updates, credential/session writes and audits share one transaction; TOTP-enabled identities roll back. (9) `identity.invite.accepted` and login success are audited as the invitee, matching transaction actor context. (10) Abuse tests cover guessing/unknown tokens, cross-tenant lookup, replay, TOTP preservation and secret redaction. `invite_expired` is the existing spec-defined code; its English/Indonesian catalogue entry awaits the T-306 error catalogue.

**Deviations** — RATIFY TD-140: add Core-owned invited-membership activation port per STM-20/21. RATIFY TD-141: retain the enrolled TOTP requirement and roll back because the accept DTO has no TOTP field and TOTP behavior is out of scope. RATIFY TD-142: resolve the bearer invite in a read-only transaction, then bind and revalidate the atomic transaction as the invitee to satisfy STM-20 and audit actor-context enforcement. The existing task-sized diff is 1,153 additions/24 deletions in implementation and tests, above CQ-101's approximate 400-line review guide; the named files/tests form the required acceptance flow.

**Dependencies affected** — T-110e is now eligible because it depends on T-110d; aggregate T-111 still waits for T-110e through T-110h.

**Rollback plan** — Revert the single T-110d task commit. No schema change was created, so no compensating migration is needed; migration 0012 predates this task.

**Follow-ups** — T-115 should apply the shared API-15 rate limiter and `RateLimit-*` response headers to both invite endpoints. T-306 should add `errors.invite_expired` in English and Indonesian when its catalogue is introduced. Human review should assess the CQ-101 diff-size note, the three RATIFY decisions, and rerun the two WebKit page checks on a host with Playwright WebKit dependencies.

## Questions
- **Resolved by architect (TD-138):** `tenant_name` comes from the new Core function in migration 0012. Option (a) from the earlier Q1 was chosen, and the response contract is unchanged.
- **Resolved by architect (TD-139):** `blocked` is not a task status. Record a TD and continue.
- **Self-resolved (implementer, 2026-10-30; TD-140, needs architect ratification):** the acceptance contract requires Core to activate the invited membership and supply the resulting active roles for the login session, but the port only exposes `ensure_invited_membership` → add a Core-owned `activate_invited_membership` port method that transitions only `invited` to `active` and returns the active `LoginIdentity`, because STM-20 t3 and STM-21 t2 require this exact transition in the acceptance transaction.
- **Self-resolved (implementer, 2026-10-30; TD-141, needs architect ratification):** the accept request has no TOTP field while an existing credential may already have TOTP enabled → preserve the existing `totp_required` control and roll back acceptance for that credential, because T-110d puts TOTP behavior out of scope and SBD-20 forbids bypassing the enrolled factor.
- **Self-resolved (implementer, 2026-10-30; TD-142, needs architect ratification):** invite acceptance must be audited as the invitee under STM-20, but the anonymous transaction actor cannot be changed for audit → resolve the tenant-scoped invite token in a read-only transaction, then start the atomic acceptance transaction as that invitee and revalidate the invitation before mutations, because the audit adapter requires actor/context equality and STM-20 names the invitee as the transition actor.

## Review
Ready for human review. T-110d is `review`; generated-index and staged-check results are recorded in this Report and the task run log.
