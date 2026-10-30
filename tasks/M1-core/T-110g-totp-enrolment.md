---
id: T-110g
title: Implement TOTP enrollment and recovery codes
milestone: M1
status: review
depends_on: [T-110f]
spec_refs: [IDN-12, STM-25, SEC-12, ERD-03]
estimated_size: M
---

## Goal
Allow a person to set up and confirm TOTP, show ten one-time recovery codes, and disable TOTP only under the state-machine guards.

## Context
- T-111 defines TOTP setup, confirmation and disable. Setup stores totp_secret_enc encrypted with totp_enabled false; confirm verifies a 6-digit code and enables TOTP, then shows ten 10-character recovery codes whose argon2 hashes are stored in totp_recovery_codes.
- Migration 0008 must exactly match spec/06 §4.12 and the T-111 row in spec/29 §7, with RLS.
- STM-25 t4 requires disable to check both current password and TOTP code, clear secret/recovery codes, audit security.totp.disabled and revoke other sessions. It refuses disable under RequiredForOwnerAdmin for owner/admin.
- totp-rs 6 is the mapped dependency; its qr feature provides QR PNG output.

## Scope
### In
- Implement setup/confirm/disable, recovery-code generation/storage, and migration 0008 exactly as mapped.
- Return the declared otpauth_url, qr_svg and secret_base32 fields.
### Out
- Accepting recovery codes during login (T-111f).

## Files
| Path | Action | Purpose |
|---|---|---|
| migrations/0008_identity_totp_recovery.sql; .sqlx/ | create/modify | Exact ERD-mapped recovery-code table/RLS and offline query metadata. |
| crates/eq-identity/src/api_me.rs | modify | TOTP request/response DTOs and handlers. |
| crates/eq-identity/src/service.rs | modify | TOTP enrollment and disable service behavior. |
| crates/eq-identity/src/password.rs | modify | Reuse the existing Argon2id implementation for one-time recovery-code hashes. |
| crates/eq-core/src/error.rs; crates/eq-core/src/error/tests.rs | modify | Register and test the spec-defined shared `invalid_transition` conflict used for invalid enrollment states. |
| crates/eq-identity/src/repo.rs | modify | Recovery-code persistence. |
| crates/eq-identity/src/module.rs | modify | Register TOTP routes. |
| Cargo.toml and Cargo.lock | modify | Enable the mapped totp-rs qr feature. |
| crates/eq-identity/tests/api_me.rs | modify | TOTP enrollment/disable tests. |
| tasks/M1-core/T-110g-totp-enrolment.md | modify | Plan and final Report. |
| tasks/DECISIONS.md | modify | Record TOTP-specific decisions TD-155–TD-160. |
| tasks/INDEX.md | generated | Reflect task status after script regeneration. |

## Interfaces
- POST /api/v1/me/totp/setup → {otpauth_url, qr_svg, secret_base32}.
- POST /api/v1/me/totp/confirm {code} → {recovery_codes}.
- DELETE /api/v1/me/totp accepts current password and the current 6-digit code per STM-25; required-policy denial uses policy_violation rule identity.totp.

## Business rules
TOTP policy is IdentityPolicy.totp. The code time window and duration follow totp-rs/spec/25. Recovery-code count and character length are exactly the task's ten codes of ten characters.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 encrypted TOTP secret and Argon2 recovery-code hashes.
- S/E: unauthorized setup/disable or cross-person access → Actor ownership and policy/state-machine checks (SBD-40/41/42, STM-25).
- I: seed/recovery-code leakage → encrypt seed with Keyring, use secret-safe Debug, show recovery codes once, persist only Argon2 hashes and never log secrets (SBD-10/21/23/50).
- Abuse cases: reject incorrect code, replay a used recovery code in T-111f, and reject disable when policy requires TOTP.

## Implementation notes
Use exact schema from spec/06 §4.12; run scripts/erd.py --migrations. Enable only the totp-rs qr feature named in spec/24; no new dependency or migration object.

## Acceptance criteria
- AC1. Setup stores a pending encrypted seed and returns an otpauth URL, QR SVG and base32 seed → totp_setup_confirm.
- AC2. Correct confirmation enables TOTP and returns ten 10-character codes once; disable requires the current password and TOTP code and obeys STM-25 → totp_setup_confirm and totp_disable_requires_code.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_me.rs | totp_setup_confirm, totp_disable_requires_code |

## Definition of done
- [x] scripts/erd.py --migrations and --check pass.
- [x] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [x] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [x] SQLx offline metadata is refreshed and checked; OpenAPI generation is attempted and T-114 placeholder recorded as Not run.
- [x] Signed-off commit and complete Step 12 Report.

## Report

**Plan** — 1. Add the exact totp_recovery_codes DDL and tenant RLS policy in migration 0008 → verify against spec/06 §4.12, ERD §4/§7 and both ERD scripts. 2. Add repository operations for pending encrypted TOTP seeds and Argon2 recovery hashes → verify SQL is parameterised, tenant-scoped and uses the caller transaction. 3. Add setup, confirmation and disable services plus authenticated HTTP handlers → verify STM-25 guards, audits, policy checks and secret redaction. 4. Register routes and add the exact totp_setup_confirm and totp_disable_requires_code HTTP tests → verify with real HTTP requests against the operator-approved database through eq_app. 5. Refresh SQLx metadata, review the security-sensitive diff and run all task checks → verify no unrelated Docker resources or files changed.

**Assumptions** — T-110d is already review at c158b56 and T-110f is review at 66c944e; this resumed task is the lowest eligible todo. The operator's resource boundary overrides stale task fixture naming: use only fresh ritma_test_T-110d inside the existing ritma-db-1, then drop only that database. Follow the task's qr_svg response shape and STM-25 password-plus-code guard (TD-155). The authenticated setup response is the single enrollment disclosure path under FLW-60 (TD-156). Existing shared API-15 rate limiting is still absent per dependency TD-152; this task records it as a follow-up and does not claim it is implemented. No UI files are in scope.

**Test plan** — AC1: Given an authenticated person and a non-Off TOTP policy, when setup returns a URI/QR/seed and a valid current code confirms it, then TOTP is enabled and exactly ten ten-character recovery codes are returned once → totp_setup_confirm. Cover pending-seed replacement, malformed/wrong code, no-store, encrypted pending seed, one-time confirmation, Argon2-only stored hashes, secret-safe audit data and app-role RLS isolation. AC2: Given an enabled TOTP secret, when disable is requested without a valid current password or code, it fails without changing state; when valid credentials are supplied, it applies RequiredForOwnerAdmin, clears the seed/recovery rows, audits and revokes other sessions → totp_disable_requires_code. Error cases: malformed/wrong confirm code returns 422 validation at /code; wrong disable password/code returns invalid_credentials; required-policy denial returns policy_violation rule identity.totp; invalid state returns the spec-defined 409 invalid_transition. Abuse cases: Actor-derived tenant/person scope, a cross-tenant RLS probe via eq_app, no secret in Debug/audit/logs and no disable for an owner/admin when policy requires TOTP.

**Summary** — Added the identity TOTP setup, confirmation and disable endpoints, encrypted seed persistence, one-time Argon2 recovery-code storage, audits and route registration. The HTTP tests verify setup replacement, validation, enrollment, role-policy denial, credential rejection, successful disable, RLS, session revocation and secret redaction. Migration 0008 matches the mapped DDL and all task changes are limited to the listed files.

**Spec read** — spec/06-domain-model.md §4.12; spec/18-security-by-design.md §§2, 4, 5.3 and 8; spec/19-error-handling.md ERR-10/21/30; spec/20-state-machines.md STM-25; spec/21-data-flows.md FLW-60; spec/22-validation-and-sanitization.md VAL-20/25; spec/24-open-source-stack.md OSS-02; spec/25-auth-and-access-control.md AUTH-01/02 and §§2/4; spec/29-erd.md identity diagram/dictionary and §§4–7; spec/modules/identity.md IDN-12; spec/16-code-quality.md §§9/12; spec/17-clean-architecture.md §§1/4/7.

**Files** — Added migrations/0008_identity_totp_recovery.sql and five SQLx query metadata files. Modified Cargo.toml/Cargo.lock to enable the mapped totp-rs qr feature; crates/eq-identity/src/api_me.rs, module.rs, password.rs, repo.rs and service.rs for DTOs, handlers, route registration, Argon2 reuse, tenant-scoped persistence and the transitions; crates/eq-identity/tests/api_me.rs for the two acceptance tests; crates/eq-core/src/error.rs and error/tests.rs for the existing spec-defined invalid_transition symbol; tasks/DECISIONS.md for TD-155–TD-160; this task Report/Questions and tasks/INDEX.md via its generator.

**AC → tests** —

| AC | Test | Result |
|---|---|---|
| AC1: TOTP setup and confirmation return one set of ten recovery codes | totp_setup_confirm | Passed 1/1 on ritma_test_T-110d |
| AC2: Disable requires password/code and obeys STM-25 | totp_disable_requires_code | Passed 1/1 on ritma_test_T-110d |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-identity --lib api_me::tests` (4/4); `cargo test -p eq-identity --lib service::tests::totp_verify_uses_injected_time_and_rfc_window -- --exact` (1/1); `cargo test -p eq-core --lib error::tests::invalid_transition_is_a_registered_conflict -- --exact` (1/1).
- ✅ With `DATABASE_URL=postgres://eq_app:eq_app@localhost:5433/ritma_test_T-110d`, `RITMA_DATABASE_URL` set to the same app URL, and `RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d`: `cargo test -p eq-identity --test api_me totp_setup_confirm -- --exact` (1/1) and `cargo test -p eq-identity --test api_me totp_disable_requires_code -- --exact` (1/1). Both used PostgreSQL 16 in the existing `ritma-db-1` and the fresh `ritma_test_T-110d` database.
- ✅ `cargo deny check` (advisories, bans, licenses and sources); existing duplicate-version warnings remain, with no exception added.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --migrations` and `python3 -I scripts/erd.py --check` (26 migrated tables checked; ERD matches DDL).
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=false DATABASE_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d cargo sqlx prepare --check --workspace -- --all-targets`; refreshed metadata was also generated with `cargo sqlx prepare --workspace -- --all-targets`.
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, `git diff --check`, and staged `gitleaks protect --staged --redact` (final results recorded after status update and staging).
- Passed: sqlx migrate run --no-dotenv --source migrations --database-url postgres://eq:eq@localhost:5433/ritma_test_T-110d applied migrations 0001–0009 and 0011–0012, including 0008. Then CARGO_TARGET_DIR=$HOME/.cache/ritma/target SQLX_OFFLINE=false DATABASE_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_DATABASE_URL=postgres://eq_app:eq_app@localhost:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_SECRET_KEY=<fake> RITMA_PUBLIC_URL=http://localhost:8080 cargo run -p eq-server -- migrate completed successfully.
- Passed: CARGO_TARGET_DIR=$HOME/.cache/ritma/target SQLX_OFFLINE=false DATABASE_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d cargo sqlx prepare --workspace -- --all-targets; query metadata was written to .sqlx.
- Passed: CARGO_TARGET_DIR=$HOME/.cache/ritma/target SQLX_OFFLINE=true cargo test -p eq-identity --lib api_me::tests (4/4), cargo test -p eq-identity --lib service::tests::totp_verify_uses_injected_time_and_rfc_window -- --exact (1/1), and cargo test -p eq-core --lib error::tests::invalid_transition_is_a_registered_conflict -- --exact (1/1).
- The exact HTTP integration commands are listed above; they used DATABASE_URL and RITMA_DATABASE_URL as eq_app and RITMA_DATABASE_OWNER_URL as eq against ritma_test_T-110d.
- OpenAPI generation attempt: CARGO_TARGET_DIR=$HOME/.cache/ritma/target SQLX_OFFLINE=true cargo run -p eq-server -- openapi > /dev/null exited 2 because the CLI command is the T-114 placeholder. Not run: make api, because its Makefile target also stops at that placeholder and writes web/openapi.json outside this task's Files.
- Not run: make dev; it sources the shared .env and starts services against its configured database. The two integration tests instead issued real HTTP requests through a local Axum listener against the isolated task database.
- Not run: cargo test --workspace and make e2e; their Testcontainers/Playwright container paths would create additional containers beyond the operator's one-container limit. The task-specific HTTP integration tests passed against PostgreSQL 16.15. Not run: web lint/typecheck/tests/i18n and browser journeys because no web/UI file or flow changed.

**Manual checks** — The integration tests sent real HTTP requests to the local Axum router with authenticated actors and PostgreSQL 16.15 in the existing ritma-db-1 container. They verified two setup calls replace the pending seed, setup/confirmation responses are no-store, malformed/wrong codes fail, the pending seed is encrypted, confirmation enables TOTP and returns ten recovery codes once, stored values are Argon2id hashes, and audit rows do not contain seed/URI values. The eq_app pool was verified NOSUPERUSER/NOBYPASSRLS; it saw its tenant's ten recovery rows and could not see a second tenant's row. Disable tests proved current password/code rejection, owner denial under RequiredForOwnerAdmin, Guardian disable success, seed/recovery deletion, current-session retention, other-session-family revocation and the security.totp.disabled audit. The fresh database ritma_test_T-110d was dropped after checks; ritma-db-1 and the pre-existing ritma_test_T-110a4b remain untouched.

**Debug log** — Initial cargo run -- migrate compilation used inherited SQLX_OFFLINE=true before new query metadata existed; the first compile also exposed a missing helper import, Zeroizing slice/format coercions and a utoipa request-schema bound. After applying the mapped migrations, the input DTO kept raw String at the HTTP boundary and passed it through Collector/FieldKind, the library byte/slice conversions were corrected, and SQLx metadata was generated; compilation passed. The first server migration run then lacked RITMA_SECRET_KEY; rerunning with the obviously fake test key and public URL passed. The first HTTP test run intentionally caught a fixture configuration error: DATABASE_URL pointed at owner role eq, and verify_app_pool rejected it; rerunning with DATABASE_URL=eq_app and RITMA_DATABASE_OWNER_URL=eq passed both acceptance tests. After the tests, the first cleanup command omitted SQL identifier quoting around the hyphenated database name and PostgreSQL rejected it without changing state; the quoted DROP DATABASE command succeeded. No secrets were logged.

**Performance** — No new list/search query was added. Credential changes use the existing person primary key; recovery inserts are ten fixed writes in the same transaction and deletion is scoped by tenant/person with the person_id primary-key prefix. No extra index or speculative cache was added; a representative production-scale benchmark is not available in this fixture.

**Security** — Triggers T1/T3/T4. Data classes: S3 for encrypted TOTP seed and Argon2 recovery hashes; S1 for enable state/identifiers. SBD-81 review: (1) each endpoint checks the existing identity.people.read permission before data access; (2) tenant/person scope comes from the authenticated Actor and every repo statement binds tenant/person; (3) Keyring encrypts the seed, only Argon2id hashes are stored, secret-bearing DTOs use SecretString and hand-written redacting Debug, secret responses are no-store, and setup audit payloads contain booleans only; (4) six ASCII digits and current password are validated at the boundary; (5) SQL is parameterised and the HTTP RLS probe uses eq_app, not a superuser; (6) shared API-15 per-user/per-IP rate limiting is still absent and remains a deployment follow-up per TD-152, not reported as implemented; (7) no third-party call or export is added; (8) enable/disable and setup audits run in the same transaction as state changes, and no outbox event is invented (T-110f TD-150); (9) tests cover invalid code/password, required owner/admin denial, cross-tenant RLS and secret-safe Debug/audit. No real secrets or personal data were added.

**Deviations** — RATIFY TD-155 preserves the task-declared qr_svg wire type and STM-25 current-password-plus-code disable contract while using the mapped library's PNG output inside a minimal SVG. RATIFY TD-156 allows only FLW-60's authenticated one-time setup disclosure for S3 seed material, with no-store and redaction. RATIFY TD-158 adds the shared symbol for the exact 409 invalid_transition code/status already defined in spec/19. RATIFY TD-159 names the pending setup audit action and limits its payload to state booleans; no outbox event is emitted because the catalog defines none (existing TD-150). TD-157 records the operator's single-database boundary. TD-160 records the approximately 1.3k non-generated changed-line deviation from CQ-101's ~400-line review guidance for architect review. No new table/column, business value, permission key, role grant, error code value or outbox event is introduced.

**Dependencies affected** — T-110h can verify recovery codes using the same existing Argon2id PHC format; T-111 aggregates these routes; T-114 still needs to include the registered paths in generated OpenAPI/client output. T-110f TD-150 supplies the no-invented-outbox-event boundary and TD-152 records the shared rate-limit follow-up.

**Rollback plan** — Revert the signed T-110g commit. Migration 0008 is forward-only and adds only the ERD-mapped table; if ever applied to shared data, use a separately reviewed compensating migration rather than deleting production state. The only database used here was the disposable ritma_test_T-110d, which was dropped after checks. The existing ritma-db-1 container and all images/volumes remain untouched.

**Follow-ups** — Architect/maintainer ratification of TD-155, TD-156, TD-158–TD-160 and database approval TD-157; implement the shared API-15 per-user/per-IP rate limiter before deployment (T-110f TD-152); T-110h must verify and consume recovery hashes; T-114 must generate and check the OpenAPI/client artifacts. No follow-up changes are made in this task.

**Workflow steps 0–12** — 0: resumed T-110g on dev at 66c944e, verified it equals origin/dev and that 1e43e7e is an ancestor; preserved unrelated WIP, ran index/lint and schema preflight, verified ritma-db-1/eq_app and that ritma_test_T-110d was absent; 1: read the task and cited specifications including the cross-cutting standards; 2: read T-110f's Report/deviations/follow-ups and shipped code plus T-111 aggregate behavior; 3: n/a (backend API); 4: plan, assumptions and AC/edge/error/abuse cases recorded above; 5: implemented only the listed files; 6: exercised happy/error paths through real HTTP requests against the fresh database; 7: corrected the compile/config issues in the Debug log and reran green checks; 8: reviewed scope, transaction boundaries, registered errors, SQL ownership, secrets and diff size; 9: confirmed no list query or measured optimization opportunity; 10: completed SBD-81 review and documented API-15 follow-up; 11: reran formatting, Clippy, focused tests, denial/guards, ERD, SQLx, task lint/index, diff and staged secret scan; 12: set status review, regenerated the index, committed one signed task commit and pushed dev.

## Questions
- **Self-resolved (implementer, 2026-10-30; TD-155, needs architect ratification):** the task context says the mapped TOTP QR feature returns PNG and the parent T-111 shorthand says disable takes only a password, while the task interface/acceptance and STM-25 require `qr_svg` plus current password and code → retain the declared API/state-machine contracts; embed the generated PNG data URI in the SVG and require both credentials.
- **Self-resolved (implementer, 2026-10-30; TD-156, needs architect ratification):** SBD-10 says S3 is never returned, while IDN-12 and FLW-60 require a QR/secret disclosure once during enrollment → limit disclosure to the authenticated setup response, mark it no-store, redact Debug and never log it; do not add a seed-read endpoint.
- **Decision (TD-157, needs maintainer ratification):** task acceptance tests use only the operator-authorized fresh database `ritma_test_T-110d` in the existing `ritma-db-1`; never touch the pre-existing `ritma_test_T-110a4b`.

- **Self-resolved (implementer, 2026-10-30; TD-158, needs architect ratification):** the task uses the spec-defined 409 invalid_transition code but the shared Core registry lacks its symbol → register the exact code and status from spec/19 and add a mapping test.

- **Decision (TD-159, needs architect ratification):** STM-25 names enable/disable audit actions but no pending setup action → record `security.totp.setup_started` with only the pending/enabled booleans; keep the seed, URI, QR and codes out of audit data. Do not emit an outbox event because no event is in the catalogue (T-110f TD-150).
- **Decision (TD-160, needs maintainer ratification):** this assigned multi-endpoint task exceeds CQ-101 approximate 400-line guidance → complete the given contract in one commit and disclose the size for architect review, following the recorded T-110f TD-154 precedent.

## Review
n/a — awaiting human review.
