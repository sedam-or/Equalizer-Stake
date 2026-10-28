---
id: T-107b
title: Add bounded outbox delivery with receipts and module gating
milestone: M1
status: review
depends_on: [T-107a]
spec_refs: [ARC-07, MOD-01, MOD-04, CAN-60, CAN-61, STM-82]
estimated_size: M
---

## Goal
Deliver persisted events to registered handlers in a bounded batch, preserving per-aggregate order and preventing duplicate handler effects.

## Context
T-107a provides the CAN-60 envelope and the `outbox`, `event_receipts` and `jobs` tables. The system tables have no RLS. `spec/21` FLW-03 requires claim batches, `SKIP LOCKED`, sequential delivery per aggregate, and handler transactions with tenant context. STM-82 defines `(handler_id, event_id)` receipt semantics. T-105 provides `ModuleId` and the `EnabledModuleSet` port.

## Scope
### In
- Add the `EventHandler` trait and `Dispatcher::new` / `Dispatcher::drain_once`.
- Claim at most 100 due rows with `FOR UPDATE SKIP LOCKED`; group by tenant/aggregate and process aggregate events in id order, with at most eight groups concurrently.
- Insert one receipt in the handler's transaction; skip a duplicate receipt; commit the receipt without calling a handler when its module is disabled.
- Mark an event dispatched only after all subscribed handlers succeed. On handler error, roll back the handler transaction and return the error with the event still pending; T-107c adds durable retry state.
### Out
- Long-running LISTEN/poll loop, retry/dead-letter transitions and OPS-08 attention item (T-107c).
- Concrete module handlers.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/outbox.rs` | modify | Add handler and bounded delivery contracts |
| `crates/eq-core/tests/outbox.rs` | modify | Verify exactly-once effects, ordering and disabled modules |
| `.sqlx/` | modify | Add offline metadata for delivery queries |
| `tasks/M1-core/T-107b-dispatcher-delivery.md`, `tasks/M1-core/T-107-outbox-dispatcher.md`, `tasks/INDEX.md` | modify | Report part and parent status |
| `tasks/DECISIONS.md` | modify | Record any task-specific self-resolution |

## Interfaces
```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    fn subscribes(&self) -> &'static [&'static str];
    async fn handle(&self, tx: &mut Tx, event: &Event) -> anyhow::Result<()>;
}
pub struct Dispatcher { /* pool, handlers, module gate */ }
impl Dispatcher {
    pub fn new(pool: PgPool, handlers: Vec<Box<dyn EventHandler>>,
               enabled: Arc<dyn EnabledModuleSet>) -> Self;
    pub async fn drain_once(&self) -> anyhow::Result<usize>;
}
```

## Business rules
No new business value is introduced. Batch size 100 and group concurrency 8 are the technical limits specified by T-107.

## Security
- Triggers: tenant isolation and handler side effects (SBD-30).
- Event and receipt fields are S1 (ERD-04); event payloads contain ids/minimal facts per CAN-60.
- Use the event row's tenant to create each handler `Tx`; never join across module tables or pass an untrusted tenant from a handler.
- Receipt and handler effects share the handler transaction. A failed handler leaves neither committed receipt nor committed handler effect.
- Abuse cases: concurrent dispatchers over 500 events; duplicate receipt; disabled handler; aggregate ordering; cross-tenant event context.
- Do not log payloads or raw handler error messages.

## Implementation notes
Use `SKIP LOCKED` on the event row, preserve id order within each `(tenant_id, aggregate_id)` group, and keep SQL in the core outbox adapter. `drain_once` returns without spinning; retry schedule and wake-up lifecycle belong to T-107c.

## Acceptance criteria
- AC1. Two dispatchers deliver each of 500 events exactly once per handler → `two_dispatchers_exactly_once_per_handler`.
- AC2. Events for the same aggregate are delivered in id order → `order_within_aggregate`.
- AC3. A disabled-module handler is skipped but receipted and the event is marked dispatched → `disabled_module_handler_skipped`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/outbox.rs` | `two_dispatchers_exactly_once_per_handler`, `order_within_aggregate`, `disabled_module_handler_skipped`, `duplicate_receipt_skips_handler`, `handler_failure_rolls_back_receipt_and_keeps_event_pending` |

## Definition of done
- [x] Rust format, Clippy and workspace tests pass, including all three named PostgreSQL tests on disposable PostgreSQL 16.
- [x] `cargo sqlx prepare --workspace --check -- --all-targets`, `cargo deny check` and all guard scripts pass.
- [x] `make e2e` passes on a fresh disposable PostgreSQL 16 database; record exact command, database and pass count.
- [x] Security/report/index are complete and one signed-off commit is created.

## Report
**Plan** —
1. Add bounded claim/query and persisted-envelope decoding using the shipped T-107a types → verify SQLx metadata against fresh PostgreSQL 16.
2. Lock up to 100 pending rows with `FOR UPDATE SKIP LOCKED`; for non-null aggregates, expose only the oldest pending event per tenant/aggregate → verify two dispatchers cannot deliver across the 100-row boundary out of order while null-aggregate events remain independently claimable.
3. Run each handler in a tenant-scoped transaction with a unique receipt and runtime module gate → verify duplicate, disabled-module and tenant-context behavior.
4. Process each group sequentially and no more than eight groups concurrently; mark only fully handled events dispatched → verify aggregate order and failure leaves the event pending.
5. Run all outbox PostgreSQL tests on fresh test databases and exercise dispatcher behavior locally → record the exact `make e2e` command, disposable database and pass count.
6. Run formatting, Clippy, workspace tests, SQLx, dependency, ERD, guard and secret checks; complete SBD-81 review and this Report.
**Assumptions** — Handler receipt and effect share one transaction. A failure before T-107c exists returns an error and leaves that event pending; this part does not run background retries. The 100-row batch and eight-group concurrency are technical bounds already specified by the T-107 parent.
**Test plan** — AC1 Given 500 events and two dispatchers, when both drain repeatedly, then each event has one handler call and one receipt → `two_dispatchers_exactly_once_per_handler`. AC2 Given 150 events for one aggregate (more than one claim batch), when one dispatcher pauses on the oldest event and a second dispatcher drains, then no later event is delivered until the oldest completes and the handler observes increasing ids → `order_within_aggregate`. AC3 Given a disabled module, when its handler is selected, then it is not called but its receipt and event dispatch marker commit → `disabled_module_handler_skipped`. Abuse/edge cases: a pre-existing receipt suppresses a duplicate effect; separate tenant groups receive the event tenant in `app.tenant_id`; handler failure rolls back its receipt and leaves that event pending; no-subscriber events are handled per TD-056.
**Summary** — Added bounded outbox claims, aggregate-head locking across concurrent dispatchers, tenant-scoped receipt transactions, enabled-module gating and successful handling of events with no subscribers.
**Spec read** — `spec/03-module-system.md` MOD-01/04; `spec/06-domain-model.md` §4.1; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/5/8; `spec/19-error-handling.md` §§8/10 (ERR-71); `spec/20-state-machines.md` STM-82; `spec/21-data-flows.md` FLW-03; `spec/23-contracts-and-canonical-forms.md` CAN-60/61; `spec/29-erd.md` core module.
**Files** —
- `crates/eq-core/src/outbox.rs` — dispatcher claim, grouping, handler transactions, receipts and module gating.
- `crates/eq-core/tests/outbox.rs` — five requested dispatcher tests and shared fixtures.
- `.sqlx/query-1c7e36006d47a298ad38f0a97744e3e1cee4dc122b9722f6809e6a1951f8d9a9.json`, `.sqlx/query-a1ccadfd37af26be54434c1a59dba3449842f6396a2763175937c98a00482c68.json`, `.sqlx/query-d9e6b4c4a84e0a0a3995f1f50486b48fa7e05469512b7bd61b5cb95e5e872148.json` — generated offline query metadata.
- `tasks/M1-core/T-107b-dispatcher-delivery.md` — implementation plan, test plan, results and status.
- `tasks/M1-core/T-107-outbox-dispatcher.md` — split-table status.
- `tasks/DECISIONS.md` — TD-057 decision and register.
- `tasks/INDEX.md` — regenerated by `scripts/tasks-index.sh`.
**Acceptance criteria → tests** —

| AC | Test | Result |
|---|---|---|
| AC1 | `two_dispatchers_exactly_once_per_handler` | ✅ Two dispatchers drained 500 events; exactly one call and receipt per handler/event. |
| AC2 | `order_within_aggregate` | ✅ A paused oldest event among 150 same-aggregate rows blocked a second dispatcher from delivering later rows; ids remained ordered and a matching aggregate in another tenant retained its own context. |
| AC3 | `disabled_module_handler_skipped` | ✅ Disabled handler was not called, but its receipt and dispatch marker committed; unhandled events completed. |
| Abuse | `duplicate_receipt_skips_handler` | ✅ Existing receipt suppressed another handler call. |
| Abuse | `handler_failure_rolls_back_receipt_and_keeps_event_pending` | ✅ Failed handler left no receipt or dispatch marker. |
**Checks run** —
- `cargo fmt --all --check` — ✅
- `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` — ✅
- `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` — ✅ 187 passed, 0 failed; this includes all 12 tests in the outbox suite; its database-backed cases use fresh testcontainers PostgreSQL 16 instances.
- `SQLX_OFFLINE=false DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:55439/ritma cargo sqlx prepare --workspace -- --all-targets` — ✅ regenerated `.sqlx/` metadata against disposable PostgreSQL 16.
- `SQLX_OFFLINE=true DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:55439/ritma cargo sqlx prepare --workspace --check -- --all-targets` — ✅.
- `cargo deny check` — ✅ advisories, bans, licenses and sources; existing duplicate-`base64` diagnostic remains.
- `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` — ✅.
- `python3 -I scripts/erd.py --check && python3 -I scripts/erd.py --migrations` — ✅; 18 migrated tables checked, 47 not migrated. No migration changed in this task.
- `scripts/tasks-index.sh --check && git diff --check` — ✅.
- Targeted `gitleaks dir --redact --no-banner <changed-file>` scans for each changed source, test, SQLx metadata and task file — ✅ no leaks.
- `rg -n 'dbg!|todo!|unwrap\(|#\[allow|console\.log|TODO' crates/eq-core/src/outbox.rs crates/eq-core/tests/outbox.rs` — ✅ no matches (rg exit 1 means none found).
- Final E2E command below — ✅ 6/6.
- Web lint/typecheck/unit/i18n checks — n/a; no `web/` source changed. `make e2e` built the SPA.
- `make api` — n/a; API/OpenAPI contract files did not change.
- Hosted GitHub Actions — Not run: requires the hosted runner.
**Manual checks** — Dispatcher behavior was exercised in PostgreSQL integration tests using fresh testcontainers databases. `make e2e` ran the release server against disposable PostgreSQL 16 (`postgres:16-alpine`) container `ritma-t107b-e2e-final-20261028`, database `ritma`; the only seed was a synthetic setup tenant. Chromium and mobile WebKit completed 6/6 smoke tests. The container was removed after the run. Exact command:
```sh
env E2E_SKIP_DB_START=1 DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:55438/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@127.0.0.1:55438/ritma RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://127.0.0.1:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off RITMA_MODE=single E2E_BASE_URL=http://127.0.0.1:8080 SQLX_OFFLINE=true make e2e
```
`make dev` was n/a: this backend task adds no HTTP operation that triggers the dispatcher; direct runtime behavior is covered by the PostgreSQL integration tests.
**Debug log** — Initial compile exposed missing SQLx offline metadata and an omitted `anyhow::Context` import; generated metadata against disposable PostgreSQL and added the import. The next compile found the JSON payload was already a `JsonValue`, so removed a redundant conversion. Clippy flagged `drain_once` over the CQ-101 line guide; extracted claim/group/mark/delivery helpers. Initial browser E2E had no provisioned tenant and failed the setup fallback checks; seeded one fake tenant and reran 6/6. The first outbox run had one failure: applying the aggregate-head predicate to NULL grouped unassociated events and made the disabled-module test expect more work than was claimable; changed the predicate so null-aggregate rows remain independently claimable, then all 12 tests passed. Post-review found a concurrency ordering race when an aggregate has more pending events than one claim batch; changed the query to select only each non-null aggregate's oldest pending event and expanded the ordering test to pause the oldest event while a second dispatcher drains.
**Performance** — `EXPLAIN (ANALYZE, BUFFERS)` on the claim query against 10,000 synthetic pending events across 100 aggregates used the existing `outbox_pending` index for candidate ordering and a materialized `DISTINCT ON` aggregate-head scan. The first correlated anti-join plan took 146.108 ms and 62,585 shared buffer hits; the selected head-CTE plan took 9.195 ms and 751 shared buffer hits. This is a local synthetic measurement, not a production latency claim; no index or schema change was added.
**Security** — Triggers: tenant isolation and handler side effects. SBD-81: (1) no HTTP route or permission surface was added; module handlers are registered inside the application. (2) the tenant id is read from the persisted outbox event, and each handler transaction uses `begin_system` with that tenant; no client tenant is accepted. The outbox and receipts are the existing mapped system tables without RLS; integration checks use the `eq_app` role with `NOSUPERUSER NOBYPASSRLS`. (3) no database fields or data classes were added; the existing payload is marked `[REDACTED]` in `Event::Debug` and is not logged. (4) event types are matched against the closed registered subscription list; stored envelopes are deserialized before invocation. (5) no HTML, CSV or URL output is produced. (6) no secrets or cryptographic operations are added. (7) no external call is made. (8) the unique `(handler_id, event_id)` receipt and transaction rollback protect duplicate effects; dispatch markers commit only for fully completed events. (9) no security event or audit permission change is introduced. (10) concurrency ordering, duplicate receipt, disabled module, tenant context and failure rollback are covered by tests. Targeted Gitleaks scans of every changed file found no leaks; raw payloads and handler messages are not logged.
**Deviations** — **RATIFY TD-056:** FLW-03 and this task do not define the zero-subscriber case → when no registered handler subscribes to an event type, the empty handler set succeeds and the event is marked dispatched; see Questions and `tasks/DECISIONS.md`. **RATIFY TD-057:** FLW-03 requires sequential delivery per non-null aggregate but the specified `SKIP LOCKED` claim alone permits a second dispatcher to take rows beyond the first 100-row batch → only claim the oldest pending row per non-null `(tenant_id, aggregate_id)` until it is marked dispatched, preserving order across dispatchers without a schema change; keep null-aggregate events independently claimable because they have no aggregate sequence to preserve.
**Dependencies affected** — T-107c and T-116a depend on this delivery contract.
**Rollback plan** — Revert this task commit; no schema change is owned here.
**Follow-ups** — T-107c adds due-time filtering, failure backoff and the long-running worker loop; the claim query currently treats every undispatched row as pending because T-107b precedes retry state. Human review should check the final task diff against CQ-101's approximate 400-line guide: dispatcher source, tests and task edits exceed that guide even after the original parent was split into T-107a/b/c.
**Workflow trace** — Step 0: resumed the existing `in_progress` T-107b WIP on `milestone/m1-core`, verified the task index and preserved the task-owned changes. Step 1: read the full task and cited specs; batch 100 and concurrency 8 are technical limits from the parent. Step 2: read T-107a's Report and shipped event/outbox code. Step 3: n/a (backend-only). Step 4: recorded the plan and AC-to-test plan before implementation. Step 5: implemented dispatcher, transactions, receipts, module gating and named tests. Step 6: ran disposable PostgreSQL integration tests and fresh-database browser E2E. Step 7: captured and fixed SQLx setup, compile, lint, null-aggregate and initial empty-tenant E2E failures; the final reruns passed. Step 8: reviewed CQ-101, CA boundaries, ERR-71, SQL parameterization, tenancy and debug redaction; found and fixed cross-dispatch ordering. Step 9: measured the candidate and materialized-head plans with EXPLAIN; selected the 9.195 ms plan. Step 10: completed the SBD-81 audit above. Step 11: reran the Rust, SQLx, deny, ERD, guard, secret and E2E checks below. Step 12: completed this Report, set `review`, regenerated the index and created one signed commit.
**Design reference** — n/a; backend-only task with no UI changes.

## Questions
- **Self-resolved (implementer, run 5, needs architect ratification):** The outbox task defines dispatch completion as all subscribed handlers succeeding but does not state what happens when the registered handler set is empty → treat zero subscribers as success and mark the event dispatched, because FLW-03 says to mark after all handlers succeed and an empty subscription set has no effects to deliver. [TD-056]
- **Self-resolved (implementer, run 5, needs architect ratification):** FLW-03 requires sequential delivery per aggregate, but `FOR UPDATE SKIP LOCKED` over a bounded batch can let a second dispatcher claim later rows beyond the first 100 rows → claim only the oldest pending event per non-null `(tenant_id, aggregate_id)` until that event is marked dispatched, while leaving null-aggregate rows independently claimable because they have no aggregate sequence to preserve. This preserves FLW-03 ordering across dispatchers without changing the mapped schema. [TD-057]

## Review
n/a until reviewer feedback.
