---
id: T-110a3
title: Implement login service and lockout
milestone: M1
status: todo
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
**Plan** — Read the core/identity ownership map, add the typed core directory boundary, implement the lockout/login transaction, and verify the named state and rollback cases with available checks.
**Assumptions** — Core owns person and membership reads; identity owns credentials/session writes. The port exposes only data needed by these flows.
**Test plan** — Exercise unknown/known failures, lockout boundaries, successful reset, disabled membership and rollback when audit/event writes fail.

## Questions
None.

## Review
n/a — pending implementation.
