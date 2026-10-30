---
id: T-110h
title: Accept one-time TOTP recovery codes at login
milestone: M1
status: review
depends_on: [T-110g]
spec_refs: [IDN-12, STM-25, AUTH-06, ERD-03]
estimated_size: S
---

## Goal
Allow an enrolled person to use one unused recovery code instead of a TOTP code exactly once.

## Context
- T-111 AC7 requires TOTP login without a code → 401 totp_required; valid TOTP → 200; a recovery code → 200; reuse of that code → 401.
- T-111e stores each recovery code as an Argon2 hash in the ERD-mapped identity.totp_recovery_codes table with used_at.
- Reuse the existing LoginService, Clock, tenant transaction and session creation path.

## Scope
### In
- Verify a submitted recovery code against unused hashes, mark exactly one matching row used in the login transaction, and continue through normal login success/audit/session behavior.
- Implement the original named totp_flow acceptance test.
### Out
- Generating or displaying recovery codes (T-111e).

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/service.rs | modify | Consume one recovery code during TOTP verification. |
| crates/eq-identity/src/repo.rs | modify | Atomically match and mark one unused hash. |
| crates/eq-identity/src/password.rs | modify | Reuse the configured Argon2id verifier for recovery-code secrets. |
| crates/eq-identity/tests/api_auth.rs | modify | End-to-end TOTP/recovery-code login flow using the existing auth fixture. |
| .sqlx/ | generate | Refresh offline metadata for recovery-hash queries. |
| tasks/M1-core/T-110h-recovery-code-login.md | modify | Plan and final Report. |
| tasks/DECISIONS.md | modify | Record TD-161–TD-162. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
The existing login request accepts the TOTP input field; when TOTP is enabled, a valid unused recovery code may be supplied in its place. Reuse returns the existing 401 totp_required/invalid-credentials behavior defined by Identity.

## Business rules
One successful recovery-code login consumes one code in the same transaction. No new policy field or retry count is introduced.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 recovery-code hashes and submitted secret.
- S/E: replay or cross-tenant code use → tenant/person-scoped atomic consume within the authentication transaction (SBD-40/41/42, STM-25).
- I: code disclosure → Argon2 verification, no raw code persistence or logging, redacted request Debug (SBD-10/21/50).
- Abuse-case acceptance criterion: totp_flow proves first use succeeds and replay fails.

## Implementation notes
Do not change the event or error-code catalog. A submitted code is accepted only when it verifies against an unused hash belonging to the authenticated tenant/person.
Read unused hashes through the tenant-scoped login transaction, verify them with the existing Argon2id parameters off the async runtime, and mark the matching row used with the injected login time before continuing to session creation.

## Acceptance criteria
- AC1. TOTP remains required without a valid TOTP or recovery code; a valid recovery code succeeds once and replay fails → totp_flow.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_auth.rs | totp_flow |

## Definition of done
- [x] totp_flow passes on the task DB in the existing PostgreSQL 16 container.
- [x] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [x] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [x] SQLx metadata is refreshed/checked after query changes; OpenAPI generation is attempted.
- [x] Signed-off commit and complete Step 12 Report.

## Report
**Plan** — 1. Update this task's file list to match the shipped auth boundary and record TD-161–TD-162 → verify the index and task linter. 2. Add a recovery-code verifier beside the configured Argon2id password verifier and repository reads/conditional consume for unused hashes → verify hashes remain secret-wrapped, tenant/person-scoped and transaction-bound. 3. Extend login TOTP verification to accept a valid unused recovery code, then continue through the existing session and audit path → verify missing, valid TOTP, recovery, replay and foreign-tenant-code behavior. 4. Add the exact `totp_flow` API integration test using the existing auth fixture → run it over the task-scoped database in the existing PostgreSQL container. 5. Refresh SQLx metadata and run the browser E2E suite against the same disposable task database → record browser coverage and environment-limited WebKit tests. 6. Run task/full applicable checks → verify no new container/image/volume and record before/after disk evidence.

**Assumptions** — The existing `totp` request field carries either a six-digit TOTP code or the ten-character recovery code specified by VAL-25; the handler already retains it in a redacting `SecretString`. T-110g stores Argon2id PHC hashes using the existing identity password parameters. The only new test database is `ritma_test_T-110h` inside the existing `ritma-db-1`; no schema change is needed. API-15 rate limiting remains the shared follow-up recorded by T-110f TD-152.

**Test plan** — AC1: Given an enrolled account with TOTP enabled, when the user logs in with a correct password but no code, a valid TOTP code, a valid unused recovery code, and then the same recovery code again, then the responses are respectively 401 `totp_required`, 200, 200, and 401 `invalid_credentials` → `totp_flow`. Edge cases: six-digit TOTP continues through the existing RFC verifier; only ten-character inputs are considered for recovery; a used code is excluded; one match marks exactly one row. Error cases: absent code retains `totp_required`; wrong TOTP/recovery and malformed values return generic 401 `invalid_credentials`; wrong password remains unchanged. Abuse cases: a matching code hash belonging to another tenant/person cannot authenticate or be consumed; raw codes never enter storage, audit or Debug output; the recovery login uses the normal session and success-audit path.

**Summary** — Login now accepts one unused TOTP recovery code after the normal password check, verifies the Argon2id hashes on a blocking worker, and consumes the matching tenant/person row in the same login transaction. Successful recovery continues through the existing session and audit flow; missing codes still return `totp_required`, and used or out-of-scope codes fail with generic `invalid_credentials`.

**Spec read** — `spec/modules/identity.md` IDN-12; `spec/20-state-machines.md` STM-23/25; `spec/22-validation-and-sanitization.md` VAL-25; `spec/25-auth-and-access-control.md` AUTH-06 §§4.1–4.2; `spec/29-erd.md` identity dictionary for `totp_recovery_codes` and §7; `spec/18-security-by-design.md` SBD-10/20/30/40/43/44/50/53/54/56/81; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7.

**Files** — Modified `crates/eq-identity/src/password.rs` (shared Argon2 verifier and focused test), `crates/eq-identity/src/repo.rs` (scoped unused-hash read and conditional consume), `crates/eq-identity/src/service.rs` (recovery fallback before normal session creation), and `crates/eq-identity/tests/api_auth.rs` (live HTTP `totp_flow`). Added `.sqlx/query-3b8803cdb3663385e57d310802c0dd140a2f67abf44ae6657d0c85f80d82902c.json` and `.sqlx/query-6cb675651829d4cfdbfb10f664673b7ceeb5cdacb7b0f3bee56094813d2500dd.json`; updated this task, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md`. No migration or schema object changed.

**Acceptance criteria → tests** —

| AC | Test | Result |
|---|---|---|
| AC1: TOTP remains required; TOTP and recovery login succeed, recovery replay fails | `totp_flow` | Pass: 1/1 on fresh `ritma_test_T-110h` in `ritma-db-1` (PostgreSQL 16); `api_auth` suite 20/20 |

**Checks run** — ✅ `cargo fmt --all --check`; ✅ `cargo clippy --workspace --all-targets -- -D warnings`; ✅ `cargo test -p eq-identity --lib password::tests::recovery_code_hash_verifies_only_the_matching_secret -- --exact` (1/1); ✅ `cargo test -p eq-identity --test api_auth totp_flow -- --exact` (1/1, final run); ✅ `cargo test -p eq-identity --test api_auth` (20/20); ✅ `cargo deny check` (advisories, bans, licenses and sources pass; pre-existing duplicate-version warnings remain); ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`; ✅ `python3 -I scripts/erd.py --check`; ✅ `cargo sqlx prepare --workspace -- --all-targets` and `cargo sqlx prepare --check --workspace -- --all-targets`; ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, and `git diff --check`; ✅ staged Gitleaks scan (before commit). ✅ `cargo run -p eq-server -- migrate` ran against `ritma_test_T-110h` after preparing query metadata. `E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 CARGO_TARGET_DIR="$HOME/.cache/ritma/target" make e2e` ran Playwright against freshly created `ritma_test_T-110h` in the existing PostgreSQL 16 container `ritma-db-1`: 4/6 passed (all 3 WebKit-independent Chromium cases and WebKit health check); 2 WebKit page tests could not launch because host libraries `libicu74` and `libflite1` are unavailable. The Chromium local equivalent `pnpm -C web exec playwright test --config=e2e/playwright.config.ts --project=chromium` passed 3/3 against the same disposable database. The task database was dropped after the tests. ✅ OpenAPI generation attempted with `SQLX_OFFLINE=true CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo run -p eq-server -- openapi >/dev/null`; the CLI returned exit 2 because `openapi` is still a placeholder. `scripts/erd.py --migrations` is n/a because no migration changed. Not run: `cargo test --workspace` because workspace tests can create additional testcontainers, outside the one-container resource limit; `make dev` because the test fixture exercised live local HTTP on the disposable database without reading shared `.env`; web lint/typecheck/unit checks because no web source changed (the E2E run built and exercised the SPA); `pnpm -C web gen:api` because OpenAPI is a placeholder and generated web outputs are outside this task's Files; the 2 WebKit page launches in the full E2E run because the host lacks `libicu74` and `libflite1`.

**Manual checks** — `totp_flow` sent real HTTP requests through a local TCP listener to the auth router, connected as `eq_app` to `ritma_test_T-110h` in the existing `ritma-db-1` container. It exercised initial login and TOTP setup/confirmation, missing code → 401 `totp_required`, valid RFC TOTP → 200, a foreign-tenant-only recovery code → 401 `invalid_credentials` without consuming the foreign row, recovery code → 200 with a normal access token, replay → 401 `invalid_credentials`, exactly one local code consumed, and three normal login-success audit entries. The browser E2E run used the same database after recreating it from empty and seeding the single fake tenant required by the SPA fixture; Chromium passed 3/3, while `make e2e` passed 4/6 because two WebKit page tests could not launch without the host libraries noted above.

**Debug log** — First `cargo run -p eq-server -- migrate` attempt stopped at compile because the new query macros had no SQLx offline metadata. Applied migrations with `sqlx migrate run` to the task DB, then `cargo sqlx prepare --workspace` succeeded. The first focused library test compile then showed existing unit-test query metadata was missing because preparation had not included test targets; reran `cargo sqlx prepare --workspace -- --all-targets`, which generated the two missing entries, and the focused test and final SQLx check passed. The first E2E invocation stopped before Playwright because Corepack tried to use an unavailable pnpm 12.16.1 launcher; activated the repository-pinned pnpm 9.15.9. The next invocation exposed the Makefile's local `target/release/ritma` path while the permitted Cargo target is external; created a temporary `target` symlink to the fixed cache and removed it after E2E. The first browser run against the empty task DB returned 503 for the tenant-guarded SPA routes; recreated the task database and inserted only the fake tenant fixture required by the checked-in SPA tests. The subsequent `make e2e` run reached 4/6; two WebKit page tests could not start because `libicu74` and `libflite1` are not installed on the host. A Chromium-only local run then passed 3/3. Temporary E2E data and server-log paths were removed; no WebKit dependency was installed.

**Performance** — No list endpoint or index was added. Recovery verification reads at most the already-issued unused hashes for one person and performs Argon2id off the async runtime. The final `totp_flow` completed in 19.12s and the full auth integration suite in 31.69s locally; these test timings include fixture hashing and are not a production latency benchmark.

**Security** — Triggers T1/T3/T4. Recovery hashes remain S3 and incoming codes stay in the existing redacting `SecretString` request wrapper. The repository reads and conditionally consumes only the current tenant/person's rows on the `eq_app` RLS transaction; the credential row and matching code row are locked, and the consume occurs before session creation in that transaction. The Argon2 verifier reuses configured parameters and never logs or persists the submitted value. `totp_flow` covers replay, foreign-tenant isolation and the normal success-audit/session path; the password unit test also checks secret Debug redaction.

**Deviations** — RATIFY TD-161: place the login acceptance test in `api_auth.rs`, where login behavior and its email-backed auth fixture live, rather than `api_me.rs`, whose fixture has no login identity. RATIFY TD-162: add the narrow recovery-code verification helper to `password.rs`, the owner of configured Argon2id parameters, instead of duplicating cryptographic configuration in `service.rs`.

**Dependencies affected** — T-111 aggregate's AC7 becomes implementable; T-112 depends on the completed aggregate. T-110g supplies the Argon2id PHC format and the existing API-15 follow-up.

**Rollback plan** — Revert the signed T-110h commit. No migration or schema change is made; an unused recovery hash remains valid if the application rollback occurs before a successful recovery login.

**Follow-ups** — Keep the shared API-15 per-user/per-IP rate limiter from T-110f TD-152 visible for deployment review. No new event, error code, permission, policy field or schema object is added.

**Workflow steps 0–12** — 0: resumed `dev` at `90f730a`; verified the index/linter and preserved unrelated WIP; 1: read the full task and cited specs; 2: read T-110g Report, Deviations/Follow-ups, TD-155–TD-160, shipped login/TOTP/password/repository code and T-111 AC7; 3: n/a (backend API); 4: recorded plan, assumptions and AC/error/abuse tests above before implementation; 5: added the recovery hash repository operations, secret verifier, login fallback and exact acceptance test; 6: exercised local TCP HTTP and browser E2E paths against disposable PostgreSQL database `ritma_test_T-110h`; 7: resolved SQLx metadata and E2E setup failures described in Debug log, then recorded the WebKit host-library limitation; 8: reviewed the diff against spec/16 and spec/17, preserving module ownership, transaction boundaries, redaction and the RLS-safe app connection; 9: measured only local test runtime, no production optimization; 10: audited the S3 secret flow and abuse cases; 11: reran format, workspace Clippy, focused/unit/auth tests, dependency/guard/schema/task and SQLx metadata checks; recorded `make e2e` at 4/6 and Chromium-only at 3/3, with the two WebKit launches not run due to missing host libraries; 12: completed this report, set status to review, regenerated and checked the index, and prepared one signed task commit.

**Disk and resource accounting** — Before T-110h at 2026-10-30 16:51 +07, `du -sh "$HOME/.cache/ritma/target"` reported `8.1G`; `docker system df` reported Images 7/10.96GB/0B reclaimable, Containers 9/10.49MB/0B, Local Volumes 6/5.065GB/0B, and Build Cache 0. `ritma-db-1` was healthy; `ritma_test_T-110h` was absent, and only the pre-existing `ritma_test_T-110a4b` task database was present. After the required Rust and E2E builds, the fixed target measured 8.5G; after dropping the task database Docker reported Images 7/10.96GB/0B reclaimable, Containers 9/10.49MB/0B, Local Volumes 6/5.032GB/0B, and Build Cache 0. Created only database `ritma_test_T-110h` inside existing container `ritma-db-1`, ran migrations, API tests, SQLx checks and E2E there, then dropped only that database; the pre-existing `ritma_test_T-110a4b` remains. The temporary `target` symlink was removed. No image, container, volume or cache was removed or created. Target growth came from required compilation; no `cargo clean` was run.

## Questions
- **Self-resolved (implementer, 2026-10-30; TD-161, needs architect ratification):** `totp_flow` is listed in `api_me.rs`, but the implemented login route and email-backed login fixture live in `api_auth.rs` → keep the exact acceptance test name and place it with the auth adapter fixture.
- **Self-resolved (implementer, 2026-10-30; TD-162, needs architect ratification):** recovery codes require Argon2id verification, while the existing shared verifier accepts only the domain `Password` type → add a `SecretString` recovery-code verifier in `password.rs` and reuse its existing Argon2id parameters rather than duplicating them in the service.
- Preserve the TOTP policy and schema decisions shipped by T-110g; no new migration or policy value is required.

## Review
n/a — awaiting human review.
