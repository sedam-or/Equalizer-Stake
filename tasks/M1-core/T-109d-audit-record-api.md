---
id: T-109d
title: Add transaction-bound audit recording
milestone: M1
status: review
depends_on: [T-109c]
spec_refs: [AUD-02, ARC-05, FLW-01, SEC-07]
estimated_size: S
---

## Goal
Add the typed audit event and `record` operation so an audit insert shares the caller's transaction and failure rolls the business write back.

## Context
Use the exact `AuditEvent` and `record(tx: &mut Tx, event) -> Result<(), AppError>` signature from parent T-109. `AuditSink` wiring belongs to T-109h.

## Scope
### In
- Transaction-bound insert into `audit.entries`, actor/request metadata from the supplied actor context, and the rollback acceptance test.
### Out
- Derive macro (T-109a), schema (T-109b), sealer and jobs (T-109e–T-109h).

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.lock`, `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/record.rs`, `crates/eq-audit/src/repo.rs` | modify/create | Record API, SQL adapter, direct dependencies and exports. |
| `crates/eq-audit/tests/audit.rs` | modify | Transaction rollback case. |
| `.sqlx/` | create | Offline metadata for the new SQLx query macros. |
| `tasks/M1-core/T-109d-audit-record-api.md` | modify | Report and status. |

## Interfaces
Preserve `AuditEvent<'a>` and `record(&mut Tx, AuditEvent<'_>)` exactly as stated in the parent contract. No endpoint or public error code is added.

## Business rules
Use only `Category`, actor data and event fields defined by parent T-109.

## Security
- Triggers: audit disclosure, tampering and tenant context (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: cross-tenant or unpaired audit writes → use caller `Tx`, verify tenant context, and rely on transaction rollback.
- Abuse case: forced audit insert failure leaves no business row → `audit_failure_rolls_back`.

## Implementation notes
SQL stays in the audit module's own adapter. Do not open a second transaction for the audit insert.

## Acceptance criteria
- AC1. A forced audit insert failure rolls back the business write → `audit_failure_rolls_back`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `audit_failure_rolls_back` |
| `crates/eq-audit/src/record.rs` | `debug_redacts_audit_payload_and_actor` |
| `crates/eq-audit/src/repo.rs` | `anonymous_actor_role_is_rejected` |

## Definition of done
- [ ] The database integration test passes on a fresh PostgreSQL 16 testcontainer.
- [ ] Rust architecture/security checks pass with no new `#[allow]`.
- [x] Signed commit `feat(audit): add transaction-bound record API [T-109d]`.

## Report
**Plan** — (1) Confirm audit DDL, grants, RLS and parent event fields → verify against spec 06 §4.9/§4.11, ERD-04 and ERD §7. (2) Add the typed event and redacted `Debug` in `record.rs`, with the public operation delegating to `repo.rs` → verify the exact signature and no SQL outside the adapter. (3) Insert through the supplied transaction after validating its tenant/actor context → verify parameterized query types and fail-closed role mapping. (4) Add the forced-trigger rollback integration test and Debug/role unit tests → verify the named rollback behavior and output/role boundaries. (5) Add only the offline metadata required by the two query macros, then run available formatting, policy, guard, task, diff and secret checks; record build/database checks as Not run if the fixed cache/resource rule prevents them.
**Assumptions** — The supplied `ActorCtx` is the actor for this transaction; its person ID must match the transaction-local actor setting. The audit tenant is read from the transaction-local setting, and RLS remains the database enforcement boundary. An anonymous actor cannot create an audit entry because the mapped DDL lists only owner, admin, instructor, guardian, student, system and platform roles.
**Test plan** — AC1: Given a business row written inside an `eq_app` transaction, when the audit insert trigger rejects the marker action, then rollback removes the business row → `audit_failure_rolls_back` on a disposable PostgreSQL 16 database. Error case: a forced insert error is returned and does not commit the caller transaction. Abuse case: `audit_failure_rolls_back` proves the business row disappears after a forced audit error; the actor-ID comparison is checked in the repository adapter. Edge case: system actors use the documented NULL actor ID; anonymous actors are rejected rather than persisted as an unmapped role. Debug-output case: `debug_redacts_audit_payload_and_actor` proves fake PII/IP do not appear in formatted event output.
**Summary** — Added the public typed `AuditEvent`/`record` contract and a private repository adapter that reads tenant/actor context and inserts through the caller's transaction. The event's payload and actor are redacted in `Debug`; mismatched actor context and the unmapped anonymous role fail closed. The required rollback test and additional actor/debug tests are written, but database/build checks were not executed under the current disk and Docker constraints.
**Spec read** — `spec/02-architecture.md` ARC-05; `spec/06-domain-model.md` §§4.9/4.11; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/5/8; `spec/21-data-flows.md` FLW-01; `spec/29-erd.md` §§3.9/4/5/7; parent T-109 record contract; dependency T-109c Report and shipped `Category`/`AuditPolicy` code.
**Files** — Created `crates/eq-audit/src/record.rs` and `crates/eq-audit/src/repo.rs`; modified `crates/eq-audit/src/lib.rs`, `crates/eq-audit/Cargo.toml`, `crates/eq-audit/tests/audit.rs`, `Cargo.lock`, the two query metadata entries for this task, `tasks/DECISIONS.md`, this task Report/status, and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `audit_failure_rolls_back` | Not run: requires a fresh PostgreSQL 16 TestDb and a Cargo build; the fixed target cache is absent and the disk rule prohibits creating either test resources or a replacement target. |

Additional tests written: `debug_redacts_audit_payload_and_actor` and `anonymous_actor_role_is_rejected`; both are Not run for the same build restriction. The DB integration command planned was `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test -p eq-audit --test audit audit_failure_rolls_back -- --exact --test-threads=1`; database used: none; task integration pass count: 0/1 executed (not a test failure).
**Checks run** — `cargo fmt --all --check` ✅; `cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-CC0 warnings remain); `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` ✅; `python3 -I scripts/erd.py --check` ✅; `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8); `scripts/tasks-index.sh --check` ✅; `SQLX_OFFLINE=true cargo metadata --locked --format-version 1 --no-deps` ✅; manual SHA-256 comparison of both SQL strings to their `.sqlx` metadata ✅; `git diff --check` ✅; targeted `gitleaks dir --redact --no-banner` on each of the eleven changed paths ✅ (no leaks). Not run: `cargo clippy --workspace --all-targets -- -D warnings`, `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1`, and the task's PostgreSQL integration tests, because the fixed Cargo target cache does not exist and disk rules prohibit creating it or a disposable DB/container. `cargo sqlx prepare --workspace` Not run for the same missing build cache and unavailable database; the two exact existing metadata records were retained and their query hashes verified, but they were not regenerated here. `make dev` Not run because it would require the unavailable DB; no endpoint is in scope. `make e2e` n/a because this task adds no API/UI journey. `scripts/erd.py --migrations` n/a because no migration changed.
**Manual checks** — Compared every inserted column with the mapped `audit.entries` dictionary and T-109 DDL; confirmed `tenant_id` comes only from the transaction-local setting, `eq_app` RLS checks that tenant, actor ID is compared with the transaction setting, SQL is confined to `repo.rs`, the insert uses the supplied `Tx`, and the two query macros match their offline metadata. No live database was available for runtime inspection.
**Debug log** — No test failure loop was entered because build/database tests were prohibited before execution. One formatting check initially showed module-item ordering in `lib.rs`; rustfmt corrected it, and the final `cargo fmt --all --check` passed.
**Performance** — n/a; one insert and one context query, no list query or performance target.
**Security** — SBD-30 T4 trigger. SBD-81: (1) auth/route/permission chain n/a because there is no endpoint; (2) tenant comes from the transaction, no client tenant is accepted, and the mapped RLS insert policy remains active; (3) no schema field was added, `before`/`after` are S1 and IP is S2 per ERD, event Debug redacts snapshots and actor, and no PII is logged or returned; (4) there is no HTTP DTO, role mapping is closed and rejects the unmapped anonymous role, and SQL parameters are bound. `audit_failure_rolls_back` exercises the forced-failure abuse case; the Debug and anonymous-role unit tests were written but not executed due the disk restriction. Targeted Gitleaks scans were clean; no permission, role grant, external call or security control changed.
**Deviations** — RATIFY TD-081: reject `ActorRole::Anonymous` because the mapped audit role set excludes it. RATIFY TD-082: include only the required `.sqlx` metadata and `Cargo.lock` dependency edge for the offline query macros omitted from the task Files table. RATIFY TD-083: keep SQL in private `repo.rs` per CA-11/CQ-24 while preserving the public record interface. T-109c TD-079/080 were read and do not affect this task; parent T-109 decisions TD-071–075 and TD-066/067 remain recorded in their owning Reports. Build and database checks are Not run under the disk rule; this task created no build cache, database, container, image or volume.
**Dependencies affected** — T-109e consumes the public record API; T-109h's `AuditSink` will use it. No dependency interface is changed.
**Rollback plan** — Revert this part commit; the schema remains from T-109b.
**Follow-ups** — T-109e owns the `audit.seal` transaction advisory lock and sealing behavior in the parent T-109 contract; this record API intentionally does not add that lock.

**Disk hygiene** — Before and after checks, repository `target/` and `$HOME/.cache/ritma/target` were absent. `timeout 5s docker system df` returned exit 124 with no output, so Docker usage totals were unavailable. No target directory, database, container, image or volume was created or removed for T-109d. The existing `.pending`, `.hold`, jobs WIP, unrelated SQLx metadata and Python caches were preserved and not staged.

**Workflow steps 0–12** — Step 0: selected the lowest eligible task T-109d because T-109c is `review`; confirmed branch `dev`, current index and one active task, preserving pre-existing T-109g/OpenCode WIP. Step 1: read the full task, parent record contract, mapped audit DDL, cited specs and CQ/CA/ERR rules. Step 2: read T-109c's Report, deviations/follow-ups and shipped policy/category code; read T-109b's Report, grants and migration; noted T-109c TD-079/080 and T-109b TD-077/078. Step 3: n/a, backend-only task. Step 4: wrote the plan, assumptions, AC mapping and error/abuse/edge cases before implementation. Step 5: added the exact public event/record contract, kept SQL in the private adapter, added the named rollback test and Debug/role unit tests, and included only the required SQLx metadata. Step 6: the database test was Not run because the fixed cache is absent and creating a disposable DB/container is prohibited; no server endpoint or UI journey is in scope. Step 7: no test failure loop was entered; static formatting was corrected and the final formatter passed. Step 8: reviewed the scoped diff against CQ-22/23/24/27, CA-11, ERR-10/05, ERD-04 and the audit migration; no `#[allow]`, cross-module SQL, unbound SQL or new error code. Step 9: n/a; no measurable performance question and no DB execution was allowed. Step 10: completed the SBD-81 checklist as recorded above and scanned all changed paths. Step 11: all available static, dependency, guard, task, metadata and secret checks are green; compilation, Clippy, database tests and SQLx prepare are explicitly Not run. Step 12: completed this Report, set `review`, regenerated the index, and prepared one signed commit and push.

## Questions
- **Self-resolved (implementer, run 6; TD-081, needs architect ratification):** `ActorRole` includes `Anonymous`, but the mapped `audit.entries.actor_role` values omit it → reject anonymous audit writes using the existing internal error rather than persist an unmapped role, because spec 06 §4.9 and ERD-04 list the supported audit roles and fail-closed behavior preserves the audit boundary.
- **Self-resolved (implementer, run 6; TD-082, needs architect ratification):** the task Files table omits `.sqlx` and `Cargo.lock` although the implementation needs SQLx macros in an offline workspace → include only the exact query metadata and direct dependency edge required by this task, because `.cargo/config.toml` sets `SQLX_OFFLINE=true` and AGENTS.md requires refreshed metadata for changed queries.
- **Self-resolved (implementer, run 6; TD-083, needs architect ratification):** the task's Files table omits the repository adapter although CA-11/CQ-24 require SQL in `repo.rs` → keep the public record contract in `record.rs` and put query/context persistence in the private `repo.rs` adapter, because the architecture standard controls adapter placement and does not change the task interface.

## Review
n/a until review.
