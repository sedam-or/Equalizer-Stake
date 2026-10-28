---
id: T-107b
title: Add bounded outbox delivery with receipts and module gating
milestone: M1
status: draft
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
| `crates/eq-core/tests/outbox.rs` | `two_dispatchers_exactly_once_per_handler`, `order_within_aggregate`, `disabled_module_handler_skipped` |

## Definition of done
- [ ] Rust format, Clippy and workspace tests pass, including all three named PostgreSQL tests on disposable PostgreSQL 16.
- [ ] `cargo sqlx prepare --workspace --check -- --all-targets`, `cargo deny check` and all guard scripts pass.
- [ ] `make e2e` passes on a fresh disposable PostgreSQL 16 database; record exact command, database and pass count.
- [ ] Security/report/index are complete and one signed-off commit is created.

## Report
**Plan** — (1) Add handler registration and bounded dispatcher claim/drain using T-107a's types → verify receipt, ordering, batch and tenant behavior. (2) Add aggregate groups, handler transactions, module gate and receipts → run the three assigned AC tests on a fresh disposable PostgreSQL 16 database. (3) Run Rust, SQLx, deny, guards, security review and browser E2E; record exact results.
**Assumptions** — Handler receipt and effect share one transaction. A failure before T-107c exists returns an error and leaves the event pending; this part does not run background retries. The task's batch/group limits are technical constants already specified by T-107.
**Test plan** — AC1 `two_dispatchers_exactly_once_per_handler` with 500 events and receipt/call counts; AC2 `order_within_aggregate`; AC3 `disabled_module_handler_skipped` with a receipt and no handler call.
**Summary** — Pending implementation.
**Spec read** — `spec/03-module-system.md` MOD-01/04; `spec/06-domain-model.md` §4.1; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/8; `spec/19-error-handling.md` ERR-71; `spec/20-state-machines.md` STM-82; `spec/21-data-flows.md` FLW-03; `spec/23-contracts-and-canonical-forms.md` CAN-60/61; `spec/29-erd.md` core module.
**Files** — Pending implementation.
**Acceptance criteria → tests** — Pending implementation.
**Checks run** — Pending implementation; report exact commands and results.
**Manual checks** — Fresh database and exact `make e2e` command/database/pass count pending implementation.
**Debug log** — None yet.
**Performance** — n/a; the task's bounded batch and concurrency are implementation limits, not an optimization claim.
**Security** — SBD-81 review and abuse-case results pending implementation.
**Deviations** — None currently.
**Dependencies affected** — T-107c and T-116a depend on this delivery contract.
**Rollback plan** — Revert this task commit; no schema change is owned here.
**Follow-ups** — T-107c adds failure backoff and the long-running worker loop.

## Questions
n/a before implementation.

## Review
n/a until reviewer feedback.
