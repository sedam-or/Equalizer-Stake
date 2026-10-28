---
id: T-107a
title: Add outbox schema and transaction-bound event emission
milestone: M1
status: review
depends_on: [T-105]
spec_refs: [ARC-07, MOD-04, CAN-60, CAN-61, ERD-03, ERD-06]
estimated_size: M
---

## Goal
Persist typed events in the same transaction as their state change. Add the mapped outbox tables and transaction-bound event envelope used by later dispatcher parts.

## Context
The single migration `0004_core_outbox_jobs.sql` is assigned to T-107 in `spec/29-erd.md` §7 and creates the exact `outbox`, `event_receipts`, and `jobs` DDL in `spec/06-domain-model.md` §4.1. These system tables have no RLS. An after-insert statement trigger sends `pg_notify('eq_outbox', '')`. CAN-60 defines the delivered envelope; CAN-61 requires tolerant payload readers.

## Scope
### In
- Create migration `0004_core_outbox_jobs.sql` with the three mapped table definitions verbatim, their mapped indexes, and the statement-level outbox notification trigger.
- Preserve trusted tenant, actor role/id, and request id in `Tx` for event creation without exposing transaction internals in `Debug`.
- Add `EventActor`, `Event`, `DomainEvent`, `emit`, and `OutboxEventSink` in `eq_core::outbox`.
- Reject a tenant or actor supplied to the event sink when it differs from the transaction context.
### Out
- Handler registration, dispatcher claims, delivery, receipts, retries, dead-letter processing and the worker loop (T-107b/c).
- Job processing (T-108).

## Files
| Path | Action | Purpose |
|---|---|---|
| `migrations/0004_core_outbox_jobs.sql` | create | Exact mapped schema and insert notification |
| `crates/eq-core/src/{actor.rs,db.rs,outbox.rs}` | modify | Preserve trusted transaction context and define event emission |
| `crates/eq-core/tests/outbox.rs` | create | Producer rollback, context and redaction tests |
| `Cargo.toml`, `Cargo.lock`, `crates/eq-core/Cargo.toml` | modify | Only if UUIDv7 or direct test dependencies require manifest updates |
| `.sqlx/` | modify | Refresh metadata for changed SQLx queries |
| `tasks/M1-core/T-107a-event-emission.md`, `tasks/M1-core/T-107-outbox-dispatcher.md`, `tasks/M1-core/T-108-jobs-and-cron.md`, `tasks/M1-core/T-116a-contract-tests-and-api-fuzzing.md`, `tasks/INDEX.md` | modify | Record this part, split ownership and the schema/contract dependency edges |
| `tasks/DECISIONS.md` | modify | Record self-resolutions TD-044 and TD-055 |

## Interfaces
```rust
pub struct EventActor { pub id: Option<PersonId>, pub role: ActorRole }
pub struct Event {
    pub id: Uuid,
    pub event_type: &'static str,
    pub tenant_id: TenantId,
    pub aggregate_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub actor: EventActor,
    pub request_id: Option<Uuid>,
    pub payload: serde_json::Value,
}
pub trait DomainEvent: Serialize {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Option<Uuid>;
}
pub async fn emit<E: DomainEvent>(tx: &mut Tx, tenant: TenantId, event: &E)
    -> Result<(), AppError>;
```
The shipped T-105 `Event` uses `&'static str` for `event_type`; the persisted `outbox.event_type` remains PostgreSQL `text`. Actor and request metadata are stored alongside the versioned payload JSON. UUIDs are v7. `OutboxEventSink` implements the shipped T-105 `EventSink` contract.

## Business rules
No business value is added. Event type names remain constants owned by each emitter; payloads contain only ids and minimal facts per CAN-60.

## Security
- Triggers: tenant isolation and event actor/request metadata (SBD-30).
- Data classes: outbox and event fields are S1 per ERD-04; payloads must not contain S2/S3 values (CAN-60).
- Tenant and actor context come from `Tx`; no IP address or raw request data is copied into an event.
- Tables stay no-RLS per the DDL. Event payloads are never logged; hand-written `Debug` for `Event` must redact the payload.
- Abuse cases: rollback leaves no row; a mismatched tenant or actor is rejected; `Debug` does not reveal a fake payload value.

## Implementation notes
Keep SQL in the core-owned outbox adapter and use parameterized `sqlx::query!`. Run `python3 -I scripts/erd.py --migrations` because this part creates a migration. T-108 depends on this part because it creates the shared `jobs` table.

## Acceptance criteria
- AC1. Emitting inside a transaction and rolling it back leaves no outbox row → `outbox::tests::rollback_no_event`.
- AC2. Emission rejects a tenant that differs from `Tx` → `emit_rejects_tenant_mismatch`.
- AC3. `OutboxEventSink` preserves the transaction actor role/id and request id → `event_sink_preserves_actor_and_request_context`.
- AC4. Formatting an `Event` does not reveal its payload → `event_debug_redacts_payload`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/outbox.rs` | `outbox::tests::rollback_no_event`, `emit_rejects_tenant_mismatch`, `outbox_insert_notifies_listener_after_commit`, `event_sink_rejects_actor_mismatch`, `event_sink_preserves_actor_and_request_context`, `policy_changed_event_preserves_metadata`, `event_debug_redacts_payload` |

## Definition of done
- [x] `scripts/erd.py --migrations` and `python3 -I scripts/erd.py --check` pass.
- [x] Rust format, Clippy and workspace tests pass, including all six PostgreSQL integration tests on disposable PostgreSQL 16.
- [x] `make e2e` passes on a fresh disposable PostgreSQL 16 database; exact command, database and pass count are recorded.
- [x] Security and guard scripts pass; SQLx metadata is current and `cargo deny` passes unchanged.
- [x] Report is complete, status is `review`, and this task has one signed-off commit.

## Report
**Plan** — (1) Add only the exact `0004` tables and notification trigger → verify the ERD migration check and migrations on fresh PostgreSQL 16. (2) Retain minimal transaction tenant/actor context and implement the CAN-60 envelope and `DomainEvent` emission → verify the rollback and context tests. (3) Add the T-105 outbox sink adapter and payload-redacting `Debug` → verify actor/request preservation and redaction. (4) Prepare SQLx metadata, run the required Rust/security/guard/E2E checks, then complete the Step 12 report.

**Assumptions** — `spec/06` and the T-107 migration row in `spec/29` are exhaustive. UUIDv7 is already enabled in the workspace. The sink actor must equal the actor captured when `Tx` began; the emitted actor and request metadata come from `Tx`, not a caller-selected payload. No new table or column is needed.

**Test plan** — AC1 `outbox::tests::rollback_no_event`; AC2 `emit_rejects_tenant_mismatch`; AC3 `event_sink_preserves_actor_and_request_context`; AC4 `event_debug_redacts_payload`. Also verify the statement notification after commit with `outbox_insert_notifies_listener_after_commit`, the Scope-required actor mismatch with `event_sink_rejects_actor_mismatch`, and the T-106 policy sink with `policy_changed_event_preserves_metadata`. Exercise all database cases on disposable PostgreSQL 16 with the `eq_app` connection for application behavior.

**Summary** — Added the mapped outbox/receipt/jobs migration and statement-level `eq_outbox` notification trigger. `Tx` now retains its trusted tenant and actor context; typed event emission stores actor/request metadata with the versioned payload in the caller's transaction, and the shipped event sink writes module-toggle and policy-change events. The producer rejects tenant and actor mismatches, and `Event` debug output redacts its payload.
**Spec read** — `spec/02-architecture.md` ARC-07; `spec/03-module-system.md` MOD-04; `spec/06-domain-model.md` §4.1; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/8; `spec/19-error-handling.md` ERR-71; `spec/20-state-machines.md` STM-80/82; `spec/21-data-flows.md` FLW-03; `spec/23-contracts-and-canonical-forms.md` CAN-60/61; `spec/29-erd.md` core module and §7.
**Files** —
- `migrations/0004_core_outbox_jobs.sql` — added the exact mapped core tables/indexes and the after-insert statement notification trigger.
- `crates/eq-core/src/actor.rs` — made the shipped actor roles serde-compatible using snake-case values.
- `crates/eq-core/src/db.rs` — retained tenant/actor transaction context and added explicit rollback.
- `crates/eq-core/src/outbox.rs` — added `DomainEvent`, transaction-bound `emit`, `OutboxEventSink`, and a redacting `Event` formatter.
- `crates/eq-core/tests/outbox.rs` — added six PostgreSQL integration tests and the event payload redaction test.
- `.sqlx/query-88c72b9177af7b052a9436bb57f57e4d4563e60d13344b0065164cdd88ad8092.json` — generated metadata for the new outbox insert query.
- `tasks/M1-core/T-107a-event-emission.md`, `T-107-outbox-dispatcher.md`, `T-107b-dispatcher-delivery.md`, `T-107c-retry-dead-letter.md`, `T-108-jobs-and-cron.md`, `T-116a-contract-tests-and-api-fuzzing.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` — recorded the split, dependency edges, reports, and TD-055.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `outbox::tests::rollback_no_event` | Passed; rollback leaves no outbox row. |
| AC2 | `emit_rejects_tenant_mismatch` | Passed; mismatched tenant is rejected without a row. |
| AC3 | `event_sink_preserves_actor_and_request_context` | Passed; persisted envelope retains person id, role, and request id. |
| AC4 | `event_debug_redacts_payload` | Passed; the fake payload token is absent from formatted output. |
| Schema notification | `outbox_insert_notifies_listener_after_commit` | Passed; a listener receives the empty `eq_outbox` notification after commit. |
| Scope actor check | `event_sink_rejects_actor_mismatch` | Passed; mismatched actor context is rejected without a row. |
| Shipped T-106 event sink | `policy_changed_event_preserves_metadata` | Passed; the policy event type and module/version/instant payload are persisted. |
**Checks run** —
- ✅ `cargo fmt --all --check`.
- ✅ `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test -p eq-core --test outbox -- --test-threads=1` — 7/7 passed (six database tests and one formatter test).
- ✅ `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` — 182 passed, 0 failed; all database tests used disposable PostgreSQL 16 Testcontainers databases.
- ✅ `env SQLX_OFFLINE=false DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:55436/ritma cargo sqlx prepare --workspace -- --all-targets` — generated query metadata against disposable PostgreSQL 16.15.
- ✅ `env SQLX_OFFLINE=false DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:55436/ritma cargo sqlx prepare --workspace --check -- --all-targets` — metadata current against container `ritma-t107a-sqlx-20261028`.
- ✅ `env SQLX_OFFLINE=true DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:55436/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@127.0.0.1:55436/ritma RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://127.0.0.1:8080 cargo run -p eq-server -- migrate` — completed successfully.
- ✅ `cargo deny check` — advisories, bans, licenses, and sources passed; existing duplicate-`base64` and unmatched `CC0-1.0` warnings remain; `deny.toml` was unchanged.
- ✅ `python3 -I scripts/erd.py --migrations` — 18 migrated tables checked against the mapped DDL; 47 mapped tables remain for later tasks.
- ✅ `python3 -I scripts/erd.py --check` — ERD matches the DDL.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && scripts/tasks-index.sh --check && git diff --check`.
- ✅ Targeted `gitleaks dir --redact --no-banner` scans on every task source, test, migration, SQLx metadata, task, and decision-register path listed under Files — no leaks found.
- ✅ `make e2e` — 6/6 browser smoke checks on a fresh disposable PostgreSQL 16 database; exact command and container are in Manual checks.
- Not run: hosted GitHub Actions, which requires a hosted runner.
- n/a: standalone web lint/typecheck/unit checks; this backend task changed no web source, and `make e2e` built the SPA and exercised the browser smoke suite.
**Manual checks** — The integration suite used `TestDb::new()` with `RITMA_TEST_DATABASE_URL` unset; TestDb started disposable `postgres:16-alpine` and cloned an isolated database for each test. The schema and SQLx checks used a separate, fresh PostgreSQL 16.15 container named `ritma-t107a-sqlx-20261028`, database `ritma`, port 55436. `env DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:55436/ritma cargo sqlx migrate run --source migrations` applied migrations 0001–0004, 0009 and 0011; the app role was then set to `LOGIN NOSUPERUSER NOBYPASSRLS`. `cargo run -p eq-server -- migrate` also completed successfully with SQLx offline mode enabled.

The exact browser command was:
```sh
env E2E_SKIP_DB_START=1 DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:55437/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@127.0.0.1:55437/ritma RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://127.0.0.1:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off E2E_BASE_URL=http://127.0.0.1:8080 SQLX_OFFLINE=true make e2e
```
It used a newly created PostgreSQL 16 `postgres:16-alpine` container `ritma-t107a-e2e-20261028`, database `ritma`, with one synthetic tenant, and passed 6/6 Chromium/mobile WebKit checks. The E2E container was removed afterward. The task adds no HTTP operation that directly emits an outbox event; the PostgreSQL integration tests exercise the real producer and notification behavior.
**Debug log** — The first compile found no cached metadata for the new `query!`; migrated a disposable DB and ran `cargo sqlx prepare`. The first full suite then found three pre-existing policy-test queries absent because the initial prepare omitted test targets; reran `cargo sqlx prepare --workspace -- --all-targets`, after which the all-targets metadata check passed. The first policy-event assertion compared chrono's `Z` serialization with `to_rfc3339()`'s `+00:00`; it now compares serde values. Clippy rejected wall-clock calls in two test fixtures, then rejected `expect()` in the fixed-time helper; replacing it with `DateTime::UNIX_EPOCH` made the final formatter and Clippy checks green. No unresolved failures remain.
**Performance** — n/a; no performance target is listed for the event producer.
**Security** — Triggers: tenant isolation and trusted actor/request metadata (SBD-30). `emit` takes tenant, person id, role and request id from the captured `Tx` and rejects a supplied tenant or sink actor that differs; the IP address is never copied to the envelope. The three system tables and trigger match the mapped no-RLS DDL; SQL executes in the caller transaction through the parameterized query. Payloads contain only the module/policy facts and ids, are never logged, and `Event` debug output redacts the payload. The rollback, tenant/actor mismatch, app-role context, notification-after-commit, and redaction tests passed. No new error code or permission was introduced.
**Deviations** — RATIFY TD-044: serialize the shipped `ActorRole` values instead of collapsing them to CAN-60's `admin | system`. RATIFY TD-055: retain T-105's `Event.event_type: &'static str` shape and persist the event type in the mapped PostgreSQL `text` column. The checkout was already dirty when this task resumed because earlier task-split and audit edits were preserved; the task commit stages only the paths listed in the T-107a Files table. TD-044 and TD-055 are recorded in Questions and `tasks/DECISIONS.md`.
**Dependencies affected** — T-107b depends on this producer and schema; T-108 depends on this part for the shared `jobs` table; T-107c follows T-107b; T-116a depends on T-107c for the complete dispatcher contract.
**Rollback plan** — Revert the source commit if it has not been deployed. Migration `0004` is forward-only; if applied to a persistent database, do not drop tables containing event/job data. Any rollback must be a compensating migration that preserves existing rows.
**Follow-ups** — T-107b adds bounded delivery and receipts; T-107c adds retry/dead-letter state and the worker loop. T-116a should include the core module-toggle and policy-change payload schemas in its event contract snapshots. Event owner actions remain outside this part.

**Step 0 — Start** — Resumed T-107a in progress on `milestone/m1-core` with existing task-split and audit edits preserved. Verified the branch and task dependencies; the worktree was already dirty, so only T-107a-authorized files were included in its commit.
**Step 1 — Task and specs** — Read the full T-107a contract and the cited ARC-07, MOD-04, CAN-60/61, STM-80/82, FLW-03, security, error, code-quality, clean-architecture, and ERD sections, including the core data dictionary and migration row.
**Step 2 — Dependencies** — Read T-105 and T-106 Reports, deviations/follow-ups, and the shipped `Event`, `EventActor`, `EventHandler`, `ActorRole`, `Tx`, `EventSink`, and policy event sink contracts.
**Step 3 — Design reference** — n/a; backend-only task with no UI.
**Step 4 — Plan and test plan** — Recorded above before implementation. Tests cover mapped DDL and notification, same-transaction rollback, tenant and actor mismatch, event context, policy metadata, and payload redaction.
**Step 5 — Implement** — Added exact migration DDL and trigger, captured transaction context, typed event serialization and sink adapters, SQLx metadata, and the integration suite. Split ownership and downstream dependency links are recorded in the named task files.
**Step 6 — Run locally** — Ran all outbox integration tests with disposable Testcontainers PostgreSQL 16, the full workspace tests, and `make e2e` with real Chromium/WebKit requests against a fresh PostgreSQL 16 container. Exact E2E command and database are recorded above.
**Step 7 — Debug** — Resolved SQLx cache omissions, canonical timestamp serialization, and test-only wall-clock/expect Clippy findings as detailed in Debug log; final checks are green.
**Step 8 — Self-review** — Confirmed SQL is confined to the core-owned outbox adapter; table/column DDL matches spec; event data uses the transaction's tenant/actor/request; no IP/payload logging was added; custom `Debug` redacts payload; no `#[allow]` or `deny.toml` change.
**Step 9 — Optimize** — n/a; no performance target or measurement was requested.
**Step 10 — Security audit** — Completed SBD-81 review for the SBD-30 tenant/actor trigger. Verified `eq_app` transactions, rollback and mismatch abuse tests, no RLS changes, no direct cross-module SQL, and redacted payload formatting.
**Step 11 — Revise and verify** — Reran formatter, Clippy, the serial workspace suite, SQLx metadata check, dependency gate, ERD checks, all four guard scripts, task-index/diff checks, and targeted Gitleaks after the final test changes. The fresh-database E2E passed after production-code implementation; subsequent changes affected only tests and the Report. Hosted GitHub Actions was not run.
**Step 12 — Report** — Completed this Report, set `review`, regenerated and checked `tasks/INDEX.md`, then created one signed-off task commit and pushed `milestone/m1-core`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** CAN-60 lists event roles as `admin | system`, while T-105 ships a closed `ActorRole` with owner, admin, instructor, guardian, student, system and platform → serialize the shipped role values in the event envelope rather than collapse distinct actors, because the role set is already a shipped contract and the mapping would lose attribution. [TD-044]
- **Self-resolved (implementer, run 6, needs architect ratification):** The T-107a interface sketch uses `String` for `Event.event_type`, while T-105 ships `&'static str` → retain the shipped field shape and keep persisted `outbox.event_type` as PostgreSQL `text`, because this is a mechanical shape difference in the already-shipped event-handler contract. [TD-055]

## Review
n/a until reviewer feedback.
