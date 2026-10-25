---
id: T-107
title: Add the transactional outbox, event envelope, and dispatcher with idempotent handlers and retries
milestone: M1
status: todo
depends_on: [T-105]
spec_refs: [ARC-07, MOD-04, CAN-60, CAN-61, STM-80]
estimated_size: M
---

## Goal
State changes emit events in the same transaction. A background dispatcher delivers each event at least once to every registered handler, exactly once per handler in effect (receipts), with retries and a dead-letter state.

## Context
- DDL, copied verbatim into `migrations/0004_core_outbox_jobs.sql` (together with `jobs` from T-108; **create both tables in this migration**):
```sql
CREATE TABLE outbox (
  id uuid PRIMARY KEY, tenant_id uuid NOT NULL, event_type text NOT NULL, aggregate_id uuid,
  payload jsonb NOT NULL, occurred_at timestamptz NOT NULL, dispatched_at timestamptz,
  attempts int NOT NULL DEFAULT 0, last_error text
);
CREATE INDEX outbox_pending ON outbox (id) WHERE dispatched_at IS NULL;
CREATE TABLE event_receipts (handler_id text NOT NULL, event_id uuid NOT NULL, processed_at timestamptz NOT NULL DEFAULT now(), PRIMARY KEY (handler_id, event_id));
-- jobs table: see T-108 (spec/06 §4.1), created here too
```
  These are system tables **without RLS**. The dispatcher sets `app.tenant_id` per event before calling handler code (it calls `db::begin_system`).
- An `AFTER INSERT` trigger on `outbox` runs `pg_notify('eq_outbox', '')`, once per statement (`FOR EACH STATEMENT`).
- Dispatcher loop:
  - `LISTEN eq_outbox`, with a poll every 2 s as fallback (technical const).
  - Claim up to 100 pending rows with `FOR UPDATE SKIP LOCKED` ordered by `id`.
  - Group by `(tenant_id, aggregate_id)` and process groups concurrently (max 8, technical const), with events inside a group sequential.
  - For each handler subscribed to the `event_type`: in a new tx, `INSERT INTO event_receipts … ON CONFLICT DO NOTHING RETURNING`. If no row comes back, skip. Otherwise run the handler in the same tx, then commit.
  - When all handlers succeed, set `dispatched_at`. On failure: `attempts += 1`, `last_error`. Set `next_attempt_at = now + backoff[attempts-1]` with the backoff list `[1s, 5s, 30s, 5min, 30min]` (technical const). The claim query only takes rows with `next_attempt_at IS NULL OR next_attempt_at <= now`. After 5 failures, set `dead_letter_at` and log `error!`. **Add the columns `next_attempt_at timestamptz` and `dead_letter_at timestamptz` to `outbox` in this migration** (an addition to the spec DDL), and exclude dead rows from `outbox_pending` (`WHERE dispatched_at IS NULL AND dead_letter_at IS NULL`). The admin alert comes from notify in M5.
- The envelope: `Event { id: Uuid (v7), tenant_id, event_type: &'static str, aggregate_id: Option<Uuid>, payload: Value, occurred_at }`. A typed helper is `trait DomainEvent: Serialize { const TYPE: &'static str; fn aggregate_id(&self) -> Option<Uuid>; }`, and `emit(tx, tenant, &E)` serializes it.
- Handler trait:
```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn id(&self) -> &'static str;                 // stable, e.g. "notify.on_invoice_issued"
    fn module(&self) -> ModuleId;                 // used for runtime gating
    fn subscribes(&self) -> &'static [&'static str];
    async fn handle(&self, tx: &mut Tx, event: &Event) -> anyhow::Result<()>;
}
```
- Runtime gating: if the handler's module is disabled for the event's tenant, the receipt is written and the handler is **skipped** (MOD-01: "jobs and event handlers skip the tenant").

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Envelope delivered to handlers is CAN-60 (`id, type, tenant_id, aggregate_id, occurred_at, actor {id, role}, request_id, payload`). Payload structs derive `JsonSchema` and do **not** use `deny_unknown_fields` (tolerant reader, CAN-61); T-116a snapshots them.
  - Outbox row states and the owner Retry/Discard actions follow STM-80.

## Scope
### In
- The migration (outbox, event_receipts, jobs DDL from spec 06 §4.1 verbatim, plus the two extra columns noted above).
- `eq_core::outbox::{Event, DomainEvent, emit, EventHandler, Dispatcher}`, plus an `EventSink` impl.
- `Dispatcher::run(shutdown: CancellationToken)` and `Dispatcher::drain_once()` for tests.
- Metrics counters through `tracing` events: dispatched, failed, dead.

### Out
- Job runner (T-108). Concrete handlers (module tasks).

## Acceptance criteria
- AC1. `emit` in a tx that rolls back leaves no outbox row. → `outbox::tests::rollback_no_event`
- AC2. Two dispatcher instances on the same DB deliver each event to a handler exactly once (count receipts and handler invocations over 500 events). → `two_dispatchers_exactly_once_per_handler`
- AC3. A handler failing twice then succeeding is called 3 times; the event ends with `dispatched_at` set and `attempts=2`. → `retry_then_success` (with a `TestClock`/fast backoff override in tests)
- AC4. After 5 failures, `dead_letter_at` is set and no more attempts happen. → `dead_letter_after_max`
- AC5. Events for the same aggregate are handled in id order. → `order_within_aggregate`
- AC6. A handler of a disabled module is skipped and receipted. → `disabled_module_handler_skipped`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/outbox.rs` | the AC names above |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(core): transactional outbox and dispatcher [T-107]`, signed off

## Report

## Questions
