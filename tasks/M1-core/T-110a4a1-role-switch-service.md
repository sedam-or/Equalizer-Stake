---
id: T-110a4a1
title: Add role-switch persistence and service
milestone: M1
status: review
depends_on: [T-110a4a]
spec_refs: [IDN-03, IDN-04, IDN-09, IDN-10, STM-22, SBD-40, SBD-50, SBD-53]
estimated_size: S
---

## Goal
Implement the internal refresh-session repository and service logic needed to switch the active role without changing the token or family.

## Context
The public contract is in T-110a4b. The schema is fixed by spec 06 §4.2 and ERD-04. The existing `CoreIdentityDirectory::has_active_role` port owns membership checks; the service uses the caller's transaction and the existing audit API.

## Scope
### In
- Add private refresh-session query result types and tenant-scoped reads, including a `FOR UPDATE` lookup.
- Add a role-switch service that validates the locked session and active membership, issues a token for the selected role, updates only `active_role`, and records the specified audit event in the same transaction.
- Add SQLx metadata for the new repository queries.
### Out
- HTTP routing, cookie parsing, CSRF, response construction, API integration tests, schema changes, new permission grants, and token rotation.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/repo.rs` | modify | Tenant-scoped refresh-session lookup, locking and active-role update. |
| `crates/eq-identity/src/service.rs` | modify | Validate session and membership, issue role-bound token, and audit transactionally. |
| `.sqlx/` | modify | Generated query metadata. |
| `tasks/M1-core/T-110a4a1-role-switch-service.md`, `T-110a4a2-role-switch-http.md`, `T-110a4a3-role-switch-http-tests.md`, `T-110a4b-role-switch-and-csrf.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Part report, ordered split plan and task index. |

## Interfaces
- Internal `RoleSwitchService::switch_role` accepts the tenant, person, refresh digest, target `Role`, and actor context through a private request value.
- Repository lookup is scoped by tenant and token digest; mutation changes only `refresh_sessions.active_role` on the locked row.

## Business rules
1. Missing, expired, revoked, or person-mismatched sessions return `unauthenticated`.
2. A target role without an active membership returns `forbidden`.
3. Preserve the session id, family id, token digest and all other columns; audit `security.role_switched` in the same transaction.

## Security
- Triggers: auth session, refresh token and role authorization (SBD-30).
- Treat the refresh token as S3: retain it only as `SecretString`, hash its canonical decoded bytes, and redact session/person identifiers in Debug output.
- Scope every query to the tenant and locked session; use the existing membership port. No client-supplied tenant or new permission is introduced.
- Record the security event through the audit API before transaction commit; failures roll back the role change.

## Implementation notes
1. Resolve the cookie digest to a person in a tenant-scoped read before beginning the final actor-scoped transaction.
2. Lock and revalidate the session in that final transaction, then use only the existing Core identity port for membership checks.
3. Generate the new access token before updating `active_role`; any later failure rolls the transaction back.

## Acceptance criteria
- AC1. A locked session is revalidated for tenant/person, revocation and expiry before service mutation → `refresh_session_for_update` and `switch_role`.
- AC2. Only an active target membership receives a role-bound access token and only `active_role` changes → `switch_role` and `switch_role_keeps_refresh_family`.
- AC3. The role change and `security.role_switched` audit row share one transaction → `switch_role_keeps_refresh_family`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/service.rs` | `role_switch_request_debug_redacts_sensitive_values` |
| `crates/eq-identity/tests/api_auth.rs` (dependent part) | `switch_role`, `switch_role_keeps_refresh_family`, `switch_role_rejects_expired_session`, `switch_role_rejects_revoked_session` |

## Definition of done
- [x] `role_switch_request_debug_redacts_sensitive_values` passes; SQLx metadata check and workspace Clippy pass with the fixed target and offline compile mode.
- [x] Service and repository add no migration, permission, event or error code; update only the selected session's role and audit it transactionally.
- [x] The named dependent integration tests pass in the 16-test auth API suite.
- [x] Report contains Workflow steps 0–12 and disk/resource snapshots.
- [x] Signed commit `feat(identity): add role-switch service [T-110a4a1]` pushed to `dev`.

## Report
**Plan** — (1) Add private refresh-session query values and tenant-scoped lookup/update queries → verify SQLx metadata and row-lock query. (2) Add the service transaction with expiry, revocation and membership checks → verify compilation and dependent HTTP tests. (3) Issue a role-bound access token, update only the active role, and audit before commit → verify session family/digest and audit count in dependent integration tests. (4) Run focused Rust, SQLx, task, guard and secret checks → verify no schema or permission drift.
**Assumptions** — The HTTP boundary resolves the session person before beginning the actor-scoped transaction, then the service locks and revalidates the same tenant/session before mutation. The shared membership port and existing audit action are authoritative.
**Test plan** — AC1: Given an expired or revoked session, when the service locks it, then it returns `unauthenticated` → `switch_role_rejects_expired_session`, `switch_role_rejects_revoked_session`. AC2: Given an active membership, when the service switches role, then only `active_role` changes and the access token carries that role → `switch_role`, `switch_role_keeps_refresh_family`. AC3: Given a successful role change, when audit recording runs, then the event commits with the mutation or both roll back → `switch_role_keeps_refresh_family`. Edge/error/abuse cases include missing rows, expiry, revocation, inactive role and tenant scoping.
**Summary** — Added tenant-scoped refresh-session lookups, a row-locking mutation path, and a service that validates expiry/revocation/membership, issues the selected role's token, changes only `active_role`, and records the audit event in the same transaction. All 16 Identity auth API tests passed over localhost HTTP against the existing PostgreSQL 16.15 task database, including expired/revoked-session rejection and refresh-family preservation.
**Workflow steps 0–12** — 0: Confirmed `dev`, fetched origin, preserved the preexisting role-switch WIP and unrelated audit/Python WIP; split the parent implementation under CQ-101 and regenerated the index. 1: Read the full service part and its identity/auth/security/schema references. 2: Read T-110a4a's Report and shipped Actor/auth code; confirmed `CoreIdentityDirectory::has_active_role`, audit `record`, and refresh-session DDL. 3: n/a, backend-only. 4: Recorded the parent plan before implementation; narrowed it to this service/persistence part and mapped ACs to exact tests. 5: Added private repository values/queries, service validation/audit, SQLx metadata, and a Debug-redaction unit test. 6: Ran the production router through a real localhost TCP listener in the auth API integration suite. 7: Fixed the missing test `RITMA_SECRET_KEY` environment and replaced test UUID numeric fixtures flagged by `check-literals.sh`; reran affected checks green. 8: Reviewed tenant scoping, row locking, parameterized SQL, audit transaction and error mapping against specs 16/17/19. 9: n/a; no measured bottleneck or list query, and no optimization was made. 10: Completed the SBD-81 audit below. 11: Reran formatting, Clippy, tests, task guards, SQLx metadata, dependency policy and secret scans; workspace runtime tests were not run because they would create prohibited Testcontainers resources. 12: Completed this Report, set `review`, regenerated the index, and made the signed task commit pushed to `dev`.
**Spec read** — `spec/modules/identity.md` §§1–3; `spec/06-domain-model.md` §4.2; `spec/29-erd.md` ERD-04; `spec/20-state-machines.md` STM-22; `spec/23-contracts-and-canonical-forms.md` CAN-30/CAN-34; `spec/25-auth-and-access-control.md` §§3.3/4.4; `spec/18-security-by-design.md` §§5/8; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7; `spec/19-error-handling.md` §10.
**Files** — Modified `crates/eq-identity/src/repo.rs`, `crates/eq-identity/src/service.rs`, and three `.sqlx/query-*.json` metadata files. Added the three ordered child task files, updated the T-110a4b aggregate to draft, recorded TD-127 in `tasks/DECISIONS.md`, and regenerated `tasks/INDEX.md`.
**Resources** — Before: `du -sh "$HOME/.cache/ritma/target"` → 11G; `docker system df` → Images 20/15.5GB (7 active), Containers 17/10.49MB (9 active), Volumes 443/74.64GB (14 active), Build Cache 0; `df -h /` → 57G available. After tests/commit/push: target 11G; Docker Images 20/15.5GB, Containers 17/10.49MB, Volumes 443/74.56GB, Build Cache 0; filesystem 57G available. Reused the existing `ritma-db-1` PostgreSQL 16.15 container and existing `ritma_test_T-110a4b` database; created/removed no container, image, volume or database, and created no Cargo target directory.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `switch_role_rejects_expired_session`, `switch_role_rejects_revoked_session` | Passed; both return 401 `unauthenticated`. |
| AC2 | `switch_role`, `switch_role_keeps_refresh_family` | Passed; role-bound guardian token and permission snapshot returned while session id/family/digest stayed unchanged. |
| AC3 | `switch_role_keeps_refresh_family` | Passed; exactly one `security.role_switched` audit entry committed with the change. |
| SBD-50 | `role_switch_request_debug_redacts_sensitive_values` | Passed; tenant/person/digest are redacted. |
**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --lib service::tests::role_switch_request_debug_redacts_sensitive_values -- --exact --test-threads=1` (1 passed).
- ✅ `DATABASE_URL=postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4b RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4b RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=https://example.test CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` (16 passed, 0 failed; real localhost HTTP; PostgreSQL 16.15 in existing `ritma-db-1`).
- ✅ `DATABASE_URL=postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4b CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=false cargo sqlx prepare --workspace --check -- --all-targets --all-features`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` (advisories, bans, licenses and sources passed; existing duplicate-base64 warning only).
- ✅ `scripts/tasks-index.sh --check`; ✅ `python3 -I scripts/tasks-lint.py`; ✅ `python3 -I -B scripts/tests/test_tasks_lint.py` (8 passed).
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && python3 -I -B scripts/erd.py --check`.
- ✅ `git diff --check`; ✅ `gitleaks dir --redact --no-banner` on `crates/eq-identity`, this task's three split files, the parent task, and `tasks/DECISIONS.md` (no leaks).
- Not run: `cargo test --workspace`; workspace tests start per-suite Testcontainers databases/containers, prohibited by the disk rule. The focused 16-test suite used the existing fixed PostgreSQL container.
- Not run: `scripts/erd.py --migrations`; no migration changed. Web checks and Playwright are n/a (backend only). Hosted GitHub Actions is not available locally.
**Manual checks** — The integration tests bound a local TCP listener and used an HTTP client against the production Axum router. The successful path returned a guardian role/token without `Set-Cookie`; CSRF and inactive, expired, and revoked session paths returned the specified problem codes; session snapshots and audit counts were checked. Database: existing disposable `ritma_test_T-110a4b`, PostgreSQL 16.15, in reused `ritma-db-1`; no database/container was created or removed in this part.
**Debug log** — First integration invocation lacked `RITMA_SECRET_KEY` and failed during fixture configuration; reran with an obvious fake key and URL, then all 16 passed. `check-literals.sh` first rejected large numeric UUID fixtures; replaced them with `Uuid::nil()` and reran the full guard set successfully. No product-code failure remained.
**Performance** — No hot list query or measured bottleneck; membership enumeration is bounded to the five spec roles. No optimization was made.
**Security** — Triggers: T1 auth, T3 token/session, T4 role authorization, T8 audit. SBD-81: refresh credentials are `SecretString` and hashed before lookup; session/person/digest Debug output is redacted; repository SQL is parameterized and tenant-scoped; the locked session is revalidated for person, expiry and revocation; active membership comes from the existing Core port; only `active_role` changes; audit is recorded in the same transaction. The parent task's TD-126 records why no unspecified self-switch permission is added. Abuse tests cover CSRF, inactive membership, expiry and revocation with no mutation/audit on rejection; success checks role-only permissions, unchanged family/digest, and one audit entry. No secret/PII leaks found.
**Deviations** — RATIFY TD-127: the original role-switch diff exceeded CQ-101 and was split into ordered service, HTTP, and test tasks. No schema, permission, event, error-code, or business-rule deviation.
**Dependencies affected** — T-110a4a2 and T-110a4a3 consume the private service/repository interface; T-110a4b is the aggregate; T-110a5 remains dependent on that aggregate.
**Rollback plan** — Revert this task's signed commit; no migration or external resource needs reversal.
**Follow-ups** — T-110b owns refresh-token invalidation; T-111 owns password-version cache invalidation as noted by T-110a4a. The existing duplicate-base64 `cargo deny` warning is outside this task.

## Questions
- **Self-resolved (implementer, run 11, needs architect ratification; TD-127):** the original role-switch implementation exceeded CQ-101's approximately 400-line review guide → split it into ordered service/persistence, HTTP boundary, and HTTP-test tasks while preserving the original aggregate contract and named tests.

## Review
Ready for maintainer review.
