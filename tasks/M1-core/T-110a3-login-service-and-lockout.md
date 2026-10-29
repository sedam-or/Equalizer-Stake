---
id: T-110a3
title: Implement login service and lockout
milestone: M1
status: blocked
depends_on: [T-110a2, T-106, T-109d, T-102a, T-103]
spec_refs: [IDN-01, IDN-03, IDN-06, IDN-12, STM-23, SEC-04, SEC-05, SBD-40, SBD-53]
estimated_size: M
---

## Goal
Implement the tenant-scoped login state transition and persistence beneath the HTTP route. Failed attempts, successful session creation, audit entries and lockout events must commit atomically in the caller's tenant transaction.

## Context
- Resolve tenant from the request host and begin with `ActorCtx::anonymous(ip, request_id)` before any person is known.
- Normalize/parse email with T-102a's `Email`; lookup uses T-103 `bidx_email`. Any malformed or missing identity maps to generic 401 `invalid_credentials` after dummy Argon2 verification.
- Lock counters are updated with `FOR UPDATE` per STM-23. Policy defaults are max failures 5, window 15 minutes, lock 15 minutes. While locked, a wrong password remains generic 401; a correct password receives 423 with `until`.
- At threshold, emit exactly `identity.account_locked.v1` and audit `security.account.locked`; success resets counters and creates a refresh family/session as stated in T-110a.
- Identity owns `credentials` and `refresh_sessions`; person/membership data is core-owned. Cross-module lookups use a typed core-owned port, never identity SQL joining core tables (CA-30/ERD-02).

## Scope
### In
- Implement login service, credential/session repository adapters, TOTP verification, lockout transitions and transaction-bound audit/outbox effects.
- Add the minimal core-owned identity-directory port/adapter needed to read core-owned person and membership data without cross-module SQL.
### Out
- HTTP response/cookies and browser route tests (T-110a4); Actor permission derivation and switch-role (T-110a5); refresh rotation/CSRF for refresh and logout (T-110b).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/service.rs`, `crates/eq-identity/src/repo.rs`, `crates/eq-identity/src/lib.rs` | create/modify | Identity-owned login service and queries. |
| `crates/eq-core/src/auth.rs`, `crates/eq-core/src/auth/directory.rs`, `crates/eq-core/src/app_state.rs`, `crates/eq-core/src/lib.rs` | create/modify | Core-owned directory port/adapter and state injection. |
| `crates/eq-identity/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | modify | Mapped SQLx/TOTP and service dependencies only. |
| `.sqlx/` | modify | Exact SQLx metadata generated from the mapped DDL; never hand-fabricate entries. |
| `crates/eq-identity/src/errors.rs` | create | Use only codes registered by spec 19/20. |
| `tasks/M1-core/T-110a3-login-service-and-lockout.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Task tracking and decisions. |

## Interfaces
- Implement against `PolicyStore::current::<IdentityPolicy>`, `eq_core::db::begin`, `eq_audit::record`, and `eq_core::outbox::emit`.
- Core directory lookup returns only the tenant-scoped person fields, active roles and membership state needed by identity; its SQL stays in core's adapter.
- Login failure action follows STM-23 `security.login.failed`; account lock and success actions remain `security.account.locked` and `security.login.success`.

## Business rules
Use only the IdentityPolicy values, IDN-06 transition, and RFC 6238 TOTP step/tolerance specified in the parent. No rate-limit threshold is added; T-115 owns rate limiting.

## Security
- Triggers: T1, T3, T4, T8 (SBD-30).
- Data classes touched: S1/S2/S3 per ERD; never log email, password, token, phone or TOTP material.
- Tenant is taken only from `TenantInfo`; every table access is tenant-scoped and runs under RLS (SEC-05/SBD-40/42).
- Abuse tests cover unknown-user timing, lockout enumeration, tenant mismatch and transaction rollback (SBD-31/53/73).

## Implementation notes
1. Use the same Argon2id parameters for known, unknown and password-null accounts.
2. Audit failure from an unauthenticated request without falsely attributing the attempt to the supplied identity; retain the known person only as the audit entity when appropriate.
3. Use only compile-time SQLx query macros and refresh metadata from a database the disk rule permits; if unavailable, do not invent metadata.

## Acceptance criteria
- AC1. Five wrong passwords transition to locked at the policy threshold; a correct password during lock returns 423, a wrong one returns generic 401, and the lock event/audit share the transaction → service tests.
- AC2. Unknown email and known wrong password both execute Argon2 verification and return the same authentication error → timing/API test in T-110a4.
- AC3. Successful verification clears counters and produces a session in the same tenant transaction → service tests.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/service.rs` | `lockout_threshold_emits_and_audits`, `locked_wrong_password_is_generic`, `successful_login_resets_failures`, `unknown_email_runs_dummy_hash` |
| `crates/eq-core/src/auth/directory.rs` | `login_directory_is_tenant_scoped`, `deactivated_membership_is_not_active` |

## Definition of done
- [ ] Named service/port tests pass on fake or allowed PostgreSQL fixtures.
- [ ] Query changes have generated `.sqlx` entries and `scripts/erd.py --migrations` passes.
- [ ] `cargo fmt --all --check`, dependency/guard/task checks pass; database/build checks prohibited by resource policy are `Not run`.
- [ ] Audit/outbox effects remain in the same transaction; no cross-module SQL or new schema item.
- [ ] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [ ] Signed commit `feat(identity): implement login service and lockout [T-110a3]` pushed to `dev`.

## Report
**Plan** — (Step 4; implementation is blocked before code changes.) Read the identity ownership and login-flow specs → verify existing table ownership and login transitions. Read all dependency Reports and shipped interfaces → confirm policy, email, crypto, password/JWT, transaction, audit and outbox APIs. Implement the core-owned identity-directory port and identity login service/repository → verify tenant-scoped reads, STM-23 transitions, audit/outbox atomicity, dummy-hash behavior and session creation. Generate exact SQLx metadata from the permitted database → verify offline query metadata. Run the six named tests and permitted Rust checks → verify the acceptance criteria. The final two implementation checks cannot be performed under the current resource rule.

**Assumptions** — Core owns person/membership reads; identity owns credential and refresh-session reads/writes. Tenant identity is derived only from the resolved request host and `TenantInfo`. No business-rule assumptions were added.

**Test plan** — AC1: Given credentials reach the configured threshold, When a wrong password is checked, Then lockout, `identity.account_locked.v1` and `security.account.locked` are in one transaction (`lockout_threshold_emits_and_audits`); while locked, a wrong password remains generic (`locked_wrong_password_is_generic`). AC2: Given an unknown email, When login verifies the supplied password, Then the dummy Argon2 hash is checked (`unknown_email_runs_dummy_hash`); HTTP timing and response equivalence remain T-110a4's test. AC3: Given valid credentials and any required TOTP, When login succeeds, Then counters reset and a refresh session is created in the same tenant transaction (`successful_login_resets_failures`). Directory edge cases: tenant scoping and inactive membership (`login_directory_is_tenant_scoped`, `deactivated_membership_is_not_active`). Abuse cases include tenant mismatch, lockout enumeration and rollback when audit/outbox effects fail.

**Summary** — Blocked before implementation. The task requires new compile-time SQLx queries with metadata generated from an allowed database and passing service/port tests; current resource rules prohibit creating the fixed Cargo target and disposable database/container, and no safe disposable database is available. No source, schema, dependency or `.sqlx` files were changed, and no metadata was fabricated.

**Spec read** — `spec/modules/identity.md` §§1–2 (IDN-01/02/03/06/12 and login API); `spec/21-data-flows.md` §2 (FLW-10); `spec/20-state-machines.md` §STM-23; `spec/18-security-by-design.md` §§5.1–5.4 and §8 (SBD-40/42/43/48–50/53/81); `spec/10-security-privacy.md` SEC-04/05; `spec/06-domain-model.md` §4.2; `spec/29-erd.md` §3.2 identity dictionary and §7; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` CA-30/31; and `spec/19-error-handling.md` ERR-72.

**Files** — Modified only `tasks/M1-core/T-110a3-login-service-and-lockout.md` and regenerated `tasks/INDEX.md`. No implementation or generated SQLx files changed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `lockout_threshold_emits_and_audits`; `locked_wrong_password_is_generic` | Not run: blocked before implementation; no permitted build target or disposable PostgreSQL fixture. |
| AC2 | `unknown_email_runs_dummy_hash` | Not run: blocked before implementation; timing/API equivalence is assigned to T-110a4. |
| AC3 | `successful_login_resets_failures` | Not run: blocked before implementation; no permitted build target or disposable PostgreSQL fixture. |
| Directory | `login_directory_is_tenant_scoped`; `deactivated_membership_is_not_active` | Not run: blocked before implementation; no permitted build target or disposable PostgreSQL fixture. |

**Checks run** — Before setting `status: blocked`, `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py` passed with the task in `in_progress`. After the block edit: `scripts/tasks-index.sh` regenerated the 156-task index; `python3 -I -B -m unittest discover -s scripts/tests` passed 10/10; `python3 -I -B scripts/erd.py --check`, `cargo fmt --all --check`, `cargo deny check`, all four guards (`scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`), task-file Gitleaks, and `git diff --check` passed. `scripts/tasks-index.sh --check` and `python3 -I -B scripts/tasks-lint.py` fail with `status blocked is not allowed`; the generated index itself lists T-110a3 as blocked. Not run: `cargo sqlx prepare --workspace`, compilation, Clippy, Rust/service/port tests, database tests, `make dev`, and `make e2e`; the fixed target is absent and the resource rule prohibits creating it, and the visible `eqs-postgres` container is pre-existing rather than a fresh disposable fixture. No database or container was accessed or modified.

**Manual checks** — None. No application server or database was started; this task is blocked before implementation.

**Debug log** — No test attempt or failure loop was started. The task was blocked at pre-implementation resource verification, so no implementation error was observed.

**Performance** — n/a; no implementation or query was added.

**Security** — Triggers T1, T3, T4 and T8. No implementation exists to audit. The implementation must preserve SBD-43's dummy-hash and non-enumerating behavior, derive tenant only from context (SBD-42/SEC-05), keep identity SQL on identity-owned tables and core directory SQL in core (CA-30), keep secrets redacted (SBD-50), and write security audit/outbox effects in the caller transaction (SBD-53). The named abuse tests are listed above but were not implemented or run.

**Deviations** — No implementation deviation or self-resolution. The task hygiene guard rejects `status: blocked`, although `WORKFLOW.md` §2 explicitly defines `blocked` as a task status; the status and index were kept correct and the guard was not weakened.

**Dependencies affected** — T-110a4 and T-110a5 remain ineligible while T-110a3 is blocked. Other task interfaces are unchanged.

**Rollback plan** — No source or schema change exists to revert; the block can be resolved by answering Q1 and returning this task to `todo`.

**Follow-ups** — Resume only when the fixed Cargo target and a fresh disposable PostgreSQL 16 database are explicitly permitted/available. Then implement without fabricating `.sqlx` metadata, run the named tests and workspace checks, and record the database and resource snapshots. A separate authorized tooling task must make `scripts/tasks-lint.py` accept the workflow-defined `blocked` status so `scripts/tasks-index.sh --check` can pass while a task is blocked; this task's scope does not include the guard. Read and retain the dependency follow-ups recorded in T-102a, T-103, T-106, T-109d and T-110a2; no decision from those tasks is changed here.

**Resource report** — Before: `df -h .` reported 80G available on the repository filesystem; the fixed `$HOME/.cache/ritma/target` and repository `target/` were absent. Read-only `docker ps` showed an existing `eqs-postgres` (`postgres:16-alpine`) alongside unrelated containers; it was not treated as fresh/disposable, contacted, migrated or modified. After the task checks, no target, database, container, image or volume was created or removed; the repository filesystem still reported 80G available on the final `df -h .` check. The prior read-only `docker system df` query timed out (exit 124); that inventory was not retried.

**Workflow steps 0–12** — 0: confirmed `dev`, retained unrelated untracked WIP, verified no task was already `in_progress`, ran index/task hygiene checks, and claimed T-110a3 before evaluating the block. 1: read the full task and the listed identity, login-flow, lockout, security, tenancy, ownership, ERD, error and code-quality sections recorded under Spec read. 2: read the Reports, Deviations, Follow-ups and shipped code for T-110a2, T-106, T-109d, T-102a and T-103; confirmed the shipped `Email`, `bidx_email`, `Password`/dummy-hash, `IdentityPolicy`, `Keys`, `Tx`, `PolicyStore`, audit and outbox APIs. Preserved dependency RATIFY records TD-026–034, TD-041–053, TD-081–083 and TD-106–109; no mismatch required a self-resolution. 3: n/a; backend task with no UI. 4: recorded the implementation/test plan above. 5: n/a; blocked before implementation. 6: n/a; no endpoint/server was added. 7: n/a; no test failure occurred. 8: n/a; no implementation diff exists. 9: n/a; no query or runtime code was added. 10: reviewed the task's required security controls; implementation audit is pending because no code was written. 11: `cargo fmt --all --check`, `cargo deny check`, the four guards, ERD check, task-lint unit fixtures, targeted Gitleaks and diff check ran as listed above; `scripts/tasks-index.sh --check`/task lint report the blocked-status allow-list mismatch. Rust build/tests, SQLx metadata and database checks are Not run because the resource rule prohibits their prerequisites. 12: set `blocked`, regenerated the task index, and included only this task record and generated index in the signed commit pushed to `dev`.

## Questions
- **Q1 (blocking):** May T-110a3 proceed only after an approved fixed Cargo target and a fresh disposable PostgreSQL 16 database are available for query-macro metadata generation and required tests?
  - Context: T-110a3 requires compile-time SQLx macros, exact generated `.sqlx` metadata, and named service/port tests on fake or allowed PostgreSQL fixtures; its implementation note explicitly forbids inventing metadata. The fixed Cargo target and repository target are absent, and current resource rules prohibit creating targets, databases or containers. The only visible PostgreSQL 16 container is the pre-existing `eqs-postgres`; it is not a disposable fixture and was not accessed or migrated. Existing dependency Reports likewise leave database/build tests not run under this restriction.
  - Options: (a) explicitly permit the fixed target and a fresh disposable PostgreSQL 16 fixture so the task can be implemented and verified; (b) keep the current resource rule and leave the task blocked until those resources are permitted/available.
  - Recommendation: (b) for this run; resume with (a) only after the resource restriction changes. Never hand-create `.sqlx` entries or use the existing shared container as a test database.

## Review
n/a — pending implementation.
