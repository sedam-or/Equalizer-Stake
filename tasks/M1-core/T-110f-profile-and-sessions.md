---
id: T-110f
title: Implement self profile, password and device sessions
milestone: M1
status: review
depends_on: [T-110e]
spec_refs: [IDN-08, IDN-13, API-07, AUTH-05, STM-22, ERD-02]
estimated_size: M
---

## Goal
Let an authenticated person view and update their own profile, change their password, list their active device sessions, and revoke one session family.

## Context
- T-111 defines GET /me → PersonMe with own profile, roles, active role, permissions and optional linked_students; email/phone are decrypted for self without a PII audit.
- PATCH /me accepts display_name?, phone?, locale?, normalizes phone to E.164 with T-103 and a new bidx, and uses If-Match on persons.version.
- POST /me/password {current_password,new_password} returns 204, revokes every other session and audits security.password.changed.
- GET /me/sessions lists only non-revoked, non-expired sessions; DELETE /me/sessions/{id} revokes that session family.

## Scope
### In
- Implement /me, profile mutation, self password change and owned session list/revocation.
- Use Core-owned person/link data only through the existing CoreIdentityDirectory port.
### Out
- Admin people/role management (T-112) and TOTP endpoints (T-111e/f).

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_me.rs | create | Self profile and session HTTP DTOs/handlers. |
| crates/eq-identity/src/lib.rs | modify | Export the self-profile API module. |
| crates/eq-identity/src/service.rs | modify | Password and session services. |
| crates/eq-identity/src/repo.rs | modify | Refresh-session list/family revocation. |
| crates/eq-identity/src/module.rs | modify | Register routes and spec-defined self-read permissions. |
| crates/eq-core/src/crypto.rs | modify | Blind-index already-canonical E.164 phone values without a guessed country. |
| crates/eq-core/src/error.rs | modify | Register the spec-defined shared `version_conflict` code. |
| crates/eq-core/src/auth/directory.rs | modify | Read/update Core-owned profile and linked-student facts through the port. |
| crates/eq-identity/tests/api_me.rs | create | Profile/password/session behavior tests. |
| .sqlx/*.json | generate | Offline metadata for changed compile-time queries. |
| tasks/DECISIONS.md | modify | Record T-110f self-resolutions. |
| tasks/M1-core/T-110f-profile-and-sessions.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
- GET /api/v1/me returns the PersonMe shape already used by the login response.
- PATCH /api/v1/me accepts display_name?, phone?, locale? and returns the updated PersonMe. Missing If-Match → 428; stale If-Match → 409 version_conflict.
- POST /api/v1/me/password accepts current_password/new_password → 204.
- GET /api/v1/me/sessions lists the caller's active sessions; DELETE /api/v1/me/sessions/{id} revokes only the caller's matching family.

## Business rules
Use the If-Match version of persons.version. Profile phone parsing and normalization use T-103. Password requirements and session expiry come from the existing IdentityPolicy and injected Clock.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S2 profile phone/user-agent/IP; S3 password hash; S1 IDs and locale.
- S/E: cross-tenant or other-person access → derive person and tenant only from Actor, check the exact spec-defined permission, and scope every query (AUTH-05, CA-30, SBD-40/41/42).
- I: PII and credentials → decrypt own PII only, secret-safe DTO Debug, no PII logs, verify current password, hash new password (SBD-10/21/50).
- Abuse cases: me_patch_requires_if_match verifies stale/missing version rejection; additional tests cover cross-person session revoke and version conflict.

## Implementation notes
Use the Core port for person and linked-student reads/writes; do not issue Identity SQL against Core-owned persons or person_links. Keep session ownership in Identity's own repository.

## Acceptance criteria
- AC1. GET /me returns only the authenticated person and authorized own fields → me_get_scoped_to_actor.
- AC2. PATCH without If-Match returns 428 and with a stale version returns 409 version_conflict → me_patch_requires_if_match.
- AC3. Password change revokes every other session, and session deletion revokes only the selected family → me_password_and_sessions.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_me.rs | me_get_scoped_to_actor, me_patch_requires_if_match, me_password_and_sessions |

## Definition of done
- [x] Exact acceptance tests pass on the existing PostgreSQL 16 container using the task database recorded in TD-151.
- [x] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [x] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [x] OpenAPI generation is attempted; SQLx metadata is refreshed after query changes.
- [x] Signed-off commit and complete Step 12 Report.

## Report
**Plan** — 1. Extend the Core identity port with the profile version, active roles, guardian-linked student summaries, and optimistic profile update → verify every Core-owned query stays in Core and tenant scope. 2. Add a canonical E.164 blind-index helper and the shared conflict code required by the existing contracts → verify phone updates never infer a country and use the registered `version_conflict`. 3. Implement authenticated `/me` and owned-session routes plus transactional password/session mutations → verify If-Match, self-scope, audit, and session-family behavior. 4. Add the three exact acceptance tests with abuse and validation coverage → verify against one fresh database in the existing PostgreSQL container. 5. Run focused Rust, SQLx, security, guard, API, task and secret checks → record exact commands and clean up only the task database.

**Assumptions** — The `PersonMe` login wire shape is preserved; `/me` flattens it and adds the required `version` while returning the matching ETag. The task's explicit database constraint takes precedence over its stale T-111 database name: use a fresh `ritma_test_T-110d` only. Phone updates accept the canonical E.164 value because `TenantInfo` has no country field; the client uses the canonical phone form, and the server does not infer a national dialing region. Self-service routes are restricted to the authenticated Actor's own person/session and require the existing `identity.people.read` permission.

**Test plan** — AC1 Given authenticated actors for separate people and one guardian link, when each requests `/me`, then the response contains only their own profile, role/permission set, and guardian-linked student summaries where applicable → `me_get_scoped_to_actor`. AC2 Given a current profile version, when PATCH omits If-Match or supplies a stale version, then the API returns 428 `precondition_required` or 409 `version_conflict`; when the matching version is supplied, then validated fields update atomically and the new ETag/version is returned → `me_patch_requires_if_match`. AC3 Given valid and invalid current passwords and multiple refresh families, when the password/session routes run, then only other families are revoked on password change, deleting another person's session returns 404 without modifying it, and deleting an owned session revokes its family → `me_password_and_sessions`. Edge cases: malformed/weak ETags, null required display name, null clearable phone/locale, non-canonical phone, omitted/no-op patch, expired/revoked session rows, and concurrent stale writes. Error cases: 400 `bad_request`, 401 `invalid_credentials`, 403 `forbidden`, 404 `not_found`, 409 `version_conflict`, 422 `validation`/`password_weak`, and 428 `precondition_required`. Abuse cases: tenant and person identity are taken only from Actor; cross-person session revoke is not found; phone/user-agent/IP/password values never enter logs or Debug; sensitive session fields are returned only with `identity.people.read_pii`; PostgreSQL checks use the `eq_app` role.

**Summary** — Added authenticated `/me` profile read/update, password change and device-session listing/revocation. Core owns person/link reads and profile writes; Identity owns refresh-session mutations. Password changes keep the current session, revoke other sessions, update the password version and audit in one transaction. The login `PersonMe` response remains unchanged.

**Spec read** — `spec/07-api-conventions.md` API-07/API-15; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7; `spec/18-security-by-design.md` §§4/5/8; `spec/19-error-handling.md` §§1–5/10; `spec/20-state-machines.md` STM-22; `spec/22-validation-and-sanitization.md` VAL-20/21; `spec/23-contracts-and-canonical-forms.md` CAN-10/23; `spec/25-auth-and-access-control.md` permission matrix and session rules; `spec/29-erd.md` identity module / ERD-02.

**Files** — Created `crates/eq-identity/src/api_me.rs` for self-profile/session handlers and DTOs, and `crates/eq-identity/tests/api_me.rs` for live HTTP integration tests. Modified `crates/eq-core/src/auth/directory.rs` for Core profile/link ports and typed optimistic updates; `crates/eq-core/src/crypto.rs` for canonical E.164 blind-indexing; `crates/eq-core/src/error.rs` for `version_conflict`; `crates/eq-identity/src/lib.rs`, `module.rs`, `repo.rs` and `service.rs` for route registration and password/session operations. Regenerated `.sqlx/*.json` metadata (7 query entries added, 1 obsolete entry removed). Updated `tasks/DECISIONS.md`, this task Report/Questions and generated `tasks/INDEX.md`. No migration, schema or web source changed.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1: `/me` returns the caller's profile and only the caller's authorized linked students | `me_get_scoped_to_actor` | ✅ Passed |
| AC2: profile patch requires a current version and applies validated updates | `me_patch_requires_if_match` | ✅ Passed |
| AC3: password changes revoke other sessions and session deletion affects only the caller's selected family | `me_password_and_sessions` | ✅ Passed |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-identity --lib api_me::tests -- --nocapture` — 3/3 passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL=postgres://eq_app:eq_app@localhost:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://localhost:8080 SQLX_OFFLINE=true cargo test -p eq-identity --test api_me -- --nocapture` — 3/3 passed using `eq_app` against fresh database `ritma_test_T-110d` in existing `ritma-db-1`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --all-targets` — all workspace targets prepared against the approved database with owner access for test fixtures.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma_test_T-110d SQLX_OFFLINE=false cargo sqlx prepare --check --workspace -- --all-targets`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate-version warnings remain.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check` — the schema map matches; no migration changed.
- ✅ `git diff --check`.
- ✅ `EXPLAIN (ANALYZE, BUFFERS)` on the new profile, active-role, linked-student, active-session and session-family queries against the seeded task database; plans completed in 0.02–0.12 ms on this small fixture.
- ✅ `gitleaks protect --staged --redact` — staged task changes contain no detected secret. (Run before commit.)
- ✅ `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py`. (Run after setting `review`, before commit.)
- Not run: `cargo test --workspace`; its existing testcontainers would create additional containers, outside the operator's one-container limit. The three exact task integration tests and three focused unit tests passed.
- Not run: Web lint/typecheck/tests/i18n and browser E2E; no web files or UI flow changed, and `make e2e` would create an additional container.
- Not run: `make api`; it was attempted and exits 1 because the Makefile explicitly says OpenAPI generation is added by T-114. T-114 must generate these routes and the client.

**Manual checks** — The three acceptance tests issued real HTTP requests through a local TCP Axum listener. They covered own-profile reads and guardian links; missing, stale and matching ETags; validation errors; successful and rejected current passwords; session listing; cross-person revocation returning 404; and owned family revocation. Database assertions verified encrypted/blind-indexed phone storage, the `core.person.updated` audit with the caller as actor, and that the test pool uses `eq_app` with `NOSUPERUSER NOBYPASSRLS`. Query plans used seeded rows; the session list used the injected TST-20 clock instant. After the final database check, `ritma_test_T-110d` was dropped; `ritma-db-1` remains running and healthy. `make dev` was not run because it sources `.env` and launches the shared web dev server; the task's real HTTP fixture stayed on the approved database.

**Debug log** — The first all-target SQLx preparation with `DATABASE_URL` as `eq_app` failed because an existing `eq-core` test macro reads `public.tenants`; rerunning preparation with the owner URL passed, while the API tests still use `eq_app`. All-target compilation then exposed a stale unit-test reference, which was removed. Clippy found an oversized Core port, request parser and password service signature plus long integration helpers; the port now takes a private-field, canonical `LoginProfileUpdate`, and the remaining targeted complexity exceptions use `#[expect(..., reason = ...)]`. The final Clippy and tests passed. During query-plan measurement the first session-list plan used wall-clock `now()` and returned no active rows because the tests use the fixed TST-20 clock; rerunning with that injected instant exercised four active seeded rows.

**Performance** — No N+1 queries were introduced. `EXPLAIN (ANALYZE, BUFFERS)` returned plans within 0.02–0.12 ms on the seeded 72-session test database. PostgreSQL chose sequential scans for small tables in this fixture; no migration or index change is in this task's Files. Revisit indexing with representative production-scale data.

**Security** — Triggers: T1, T3 and T4. SBD-81: (1) each handler requires the existing permission before data access; (2) tenant/person/session scope comes from Actor and Core's tenant-checked port; (3) phone is encrypted and blind-indexed; email/phone and session UA/IP are returned only with `identity.people.read_pii`; display-name, phone, session and password Debug output is redacted and no PII is logged; (4) display name, E.164 phone, locale, password and ETag are validated at the boundary. **API-15 rate limiting is not implemented in the current shared API composition**; TD-152 records a required shared middleware follow-up and this control is not marked complete. (5) no HTML/export output is added; (6) password verification/hash use the existing secret/password pipeline and blocking worker; (7) no external calls; (8) profile updates use If-Match and optimistic version checks; (9) profile/password/session changes are audited in their transaction; the profile audit action follows STM-04 and Core ownership (TD-153); no outbox event is emitted because the event catalogue names none (TD-150); (10) the three HTTP tests cover actor scope, stale versions and cross-person session abuse. Gitleaks runs on staged task files before commit.

**Deviations** — RATIFY TD-147: keep login JSON stable and add the required version/ETag only to `/me`; TD-148: accept canonical E.164 because no tenant country is available; TD-149: use the existing self-read permission with strict Actor ownership because no self-write permission is specified; TD-150: audit state changes but do not invent an outbox event; TD-151: use only `ritma_test_T-110d` per the operator's database boundary; TD-153: name the profile audit action `core.person.updated` from ERD-01 ownership and STM-04's action format; TD-154: complete this already-assigned multi-endpoint task as one commit despite its 2,066-line implementation diff exceeding CQ-101's approximate 400-line guidance. TD-152 is a required maintainer decision: shared API-15 rate limiting is still absent and must be in place before deployment. No schema, migration, business value, permission key, role grant or event name was added.

**Dependencies affected** — T-111 should preserve the login response shape while consuming the `/me` profile contract; T-114 should include these handlers in OpenAPI/client generation. The T-110e follow-up about its IDs-only password-reset event remains unchanged.

**Rollback plan** — Revert the signed T-110f commit. No migration or persistent shared database change ships; the disposable `ritma_test_T-110d` database was dropped after verification and `ritma-db-1` remains running.

**Follow-ups** — Ratify TD-147–TD-151, TD-153 and TD-154 and review TD-152; implement shared API-15 per-user/per-IP limits and `RateLimit-*` headers before deployment; review the 2,066-line implementation size against CQ-101 and split future task contracts before implementation when needed; include the routes in T-114 OpenAPI generation; rerun the full workspace suite in an environment that permits its testcontainers.

**Workflow steps 0–12** — 0: confirmed `dev`, with `1e43e7e` in its ancestry; T-110d and T-110e are `review`, so T-110f is the lowest eligible task; unrelated WIP remains untouched. 1: read this task and its cited Identity, API, auth, state-machine, ERD, validation and security rules. 2: read T-110e's Reports, TD-143–146 and shipped reset/runtime code. 3: n/a (backend API). 4: recorded this plan, assumptions and AC/error/edge/abuse test plan before coding. 5: implemented the Core profile port, canonical phone blind index, self-profile/session handlers, transactional password change, repository operations and exact named tests. 6: exercised real HTTP requests against the API fixture using only `ritma_test_T-110d`; `make dev` was not run because it reads `.env` and starts the web dev server. 7: diagnosed and fixed the SQLx-role, unit-test, and Clippy issues described in the Debug log. 8: reviewed the diff against spec/16 §9, spec/17 §7, spec/19 and the SBD-81 checklist. 9: measured the new query plans against seeded rows; no index change fits the task's Files or available representative scale. 10: checked tenant scope, permissions, PII encryption/redaction, ETag integrity, audit atomicity and the explicit API-15 gap. 11: reran format, Clippy, focused unit/integration tests, SQLx metadata, cargo-deny, guards, ERD, Gitleaks, task index/lint and diff checks; workspace tests and OpenAPI generation are explicitly not run for the recorded reasons. 12: completed this Report, set `review`, regenerated the index, and prepared one signed-off commit for `dev`.

## Questions
- **Self-resolved (implementer, 2026-10-30; TD-147, needs architect ratification):** API-07/CAN-23 require a version and ETag for the mutable self profile, while the reused login `PersonMe` has neither and the shared `version_conflict` symbol is missing → preserve login JSON by flattening `PersonMe` into `/me`'s response with `version` and add the already-specified conflict code to Core's shared registry.
- **Self-resolved (implementer, 2026-10-30; TD-148, needs architect ratification):** phone normalization requires a tenant country, but the resolved `TenantInfo` has no country and POL-02 forbids deriving one from locale/currency/timezone → accept only the canonical validated E.164 form for this API and blind-index that value through a Core crypto helper.
- **Self-resolved (implementer, 2026-10-30; TD-149, needs architect ratification):** `/me` mutations are self-service but no self-write permission exists; the existing `identity.people.read` matrix defines the self scope → require that permission and bind every operation to Actor's own person/session so no cross-person mutation is reachable.
- **Self-resolved (implementer, 2026-10-30; TD-150, needs architect ratification):** the global state-change rule requests an outbox event but no event for profile/password/session changes is in the event catalog → record the task/spec-defined audit actions in the same transaction and do not invent an event name.
- **Self-resolved (implementer, 2026-10-30; TD-151, needs architect ratification):** this task's DoD says `ritma_test_T-111`, while the operator's explicit resource boundary permits creating only `ritma_test_T-110d` → create a fresh `ritma_test_T-110d` for this task's checks and drop it after the checks.

- **Decision (TD-152, needs maintainer ratification):** API-15's shared per-user/per-IP rate limiter is not wired into the current API composition, and a handler-local substitute would not cover the shared API contract → keep this task's scope unchanged, disclose the missing control, and require shared rate-limit middleware before deployment.
- **Self-resolved (implementer, 2026-10-30; TD-153, needs architect ratification):** the `/me` profile update mutates Core-owned `persons`, but T-110f names no audit action → use `core.person.updated`, because ERD-01 assigns `persons` to Core and STM-04 specifies `<module>.<entity>.<verb_past>`.
- **Decision (TD-154, needs architect ratification):** CQ-101's approximately 400-line guidance is exceeded by this task's 2,066-line implementation diff, but T-110f is already the assigned contract and this run requires one task commit → complete the contract and disclose the size for review; split equivalent future tasks before implementation.

The task database name above is superseded for this run by TD-151: use only `ritma_test_T-110d` in the existing PostgreSQL 16 container, then drop it.

## Review
n/a — pending review.
