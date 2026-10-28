---
id: T-107c
title: Add outbox retry, dead-letter and worker lifecycle
milestone: M1
status: draft
depends_on: [T-107b]
spec_refs: [ARC-07, OPS-08, ERR-71, STM-14, STM-80, STM-82]
estimated_size: M
---

## Goal
Add bounded retry scheduling, terminal dead-letter handling and a cancelable outbox worker loop without changing the mapped schema.

## Context
T-107a/b provide the event schema and successful delivery path. The schema is exhaustive: `outbox` has only `attempts` and `last_error` for failure state. The task's five-failure threshold and backoff list are specified; STM-80 also requires high-severity attention items (OPS-08).

## Scope
### In
- Add `Dispatcher::run(shutdown: CancellationToken)` using `LISTEN eq_outbox` plus the task's two-second poll fallback.
- Persist retry due time in a versioned, PII-free `last_error` marker; claim only absent/due markers.
- Increment `attempts` after handler failures; after five failures write the dead marker and exclude the event from future claims.
- Create/update one high-severity OPS-08 attention item for a dead event, deduped by event id.
- Emit counters with safe identifiers and stable failure codes. Do not log payloads or raw handler error text.
### Out
- Owner Retry/Discard operations, because their authorized audited endpoint contract is not in this scope; see TD-045.
- Notification delivery, owned by M5.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/outbox.rs` | modify | Retry markers, dead handling, attention item and run loop |
| `crates/eq-core/tests/outbox.rs` | modify | Verify retry, dead state and worker wake/cancel |
| `.sqlx/` | modify | Add offline metadata for retry/attention queries |
| `tasks/M1-core/T-107c-retry-dead-letter.md`, `tasks/M1-core/T-107-outbox-dispatcher.md`, `tasks/INDEX.md` | modify | Report the final part and parent status |
| `tasks/DECISIONS.md` | modify | Maintain TD-043 and TD-045 decisions |

## Interfaces
```rust
impl Dispatcher {
    pub async fn run(&self, shutdown: CancellationToken) -> anyhow::Result<()>;
    pub async fn drain_once(&self) -> anyhow::Result<usize>;
}
```
`drain_once` handles failures durably and returns the number of rows attempted. The database schema stays identical to `spec/06` §4.1.

## Business rules
No new business rule is introduced. Use the specified retry delays and dead-letter threshold. The fifth failure is terminal, so the final listed backoff remains unused as recorded in TD-043.

## Security
- Triggers: tenant isolation, error logging and operational attention payloads (SBD-30).
- Data classes: all touched columns are S1; attention payload contains only an event id (ERD-04).
- Store only a versioned due-time marker or a stable dead code in `last_error`; never persist or log arbitrary handler error text, payloads or PII.
- Dead-letter attention rows use the event tenant, high severity, an event-id-only payload and a stable dedupe key. No new permission or owner action is added.
- Abuse cases: repeated handler failure becomes dead at five, never attempts again, creates one attention row, and does not expose fake handler PII in `last_error` or logs.

## Implementation notes
Use `attempts >= 5` as the durable dead predicate. The marker is `ritma-retry-v1:<due_unix_ms>:handler_failed`; the terminal marker is `ritma-dead-v1:handler_failed`. These strings are technical internal state encodings, not business values. Use the injected/system `Clock` consistently and keep error chain text out of logs.

## Acceptance criteria
- AC1. A handler that fails twice then succeeds is called three times; the row ends dispatched with two attempts → `retry_then_success`.
- AC2. After five failures the event has a dead marker, is never attempted again, and has one high-severity attention item → `dead_letter_after_max`.
- AC3. `run` wakes after a committed notification and exits after cancellation → `run_wakes_after_commit_and_stops_on_cancel`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/outbox.rs` | `retry_then_success`, `dead_letter_after_max`, `run_wakes_after_commit_and_stops_on_cancel` |

## Definition of done
- [ ] Rust format, Clippy and workspace tests pass, including all three named PostgreSQL tests on disposable PostgreSQL 16.
- [ ] `cargo sqlx prepare --workspace --check -- --all-targets`, `cargo deny check`, `python3 -I scripts/erd.py --check` and all guard scripts pass.
- [ ] `make e2e` passes on a fresh disposable PostgreSQL 16 database; record exact command, database and pass count.
- [ ] Security/report/index are complete and one signed-off commit is created.

## Report
**Plan** — (1) Add versioned retry/due marker parsing and use only the mapped `last_error`/`attempts` fields → verify exact retry timing with an injected fast test backoff. (2) Add terminal dead marker and OPS-08 attention upsert → verify one attention row and no sixth attempt. (3) Add LISTEN/poll/cancel lifecycle → verify wake-up and shutdown. (4) Run Rust, SQLx, deny, guards, secret scans and fresh-database E2E; finish the report.
**Assumptions** — Task-specified delays and five failures are authoritative. `last_error` remains PII-free. Error causes are not formatted into logs; operational logs use ids and stable failure codes. Owner actions are deferred under TD-045.
**Test plan** — AC1 `retry_then_success`; AC2 `dead_letter_after_max`; AC3 `run_wakes_after_commit_and_stops_on_cancel`; additionally inspect fake handler text for absence from persisted marker and emitted log fields.
**Summary** — Pending implementation.
**Spec read** — `spec/06-domain-model.md` §§4.1/4.11; `spec/11-ops.md` OPS-03/08; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/8; `spec/19-error-handling.md` ERR-50/71; `spec/20-state-machines.md` STM-14/80/82; `spec/21-data-flows.md` FLW-03; `spec/29-erd.md` core module and §7.
**Files** — Pending implementation.
**Acceptance criteria → tests** — Pending implementation.
**Checks run** — Pending implementation; report exact commands and results.
**Manual checks** — Fresh database and exact `make e2e` command/database/pass count pending implementation.
**Debug log** — None yet.
**Performance** — n/a; the bounded schedule is specified and no production benchmark is requested.
**Security** — SBD-81 review and failure redaction abuse results pending implementation.
**Deviations** — RATIFY TD-043: encode retry/dead state with the exact existing columns and versioned marker; the fifth failure is terminal. RATIFY TD-045: raise an event-id-only OPS-08 attention item and defer audited owner actions. Details remain in Questions and `tasks/DECISIONS.md`.
**Dependencies affected** — T-116a can proceed after this final dispatcher part is `review`.
**Rollback plan** — Revert this task commit; no schema changes are made here.
**Follow-ups** — M5 delivers the notification alert. Owner retry/discard stays deferred until its authorized audited contract is assigned.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** the task requires retry/dead state but the mapped DDL has no `next_attempt_at` or `dead_letter_at` → use the existing `last_error` marker `ritma-retry-v1:<due_unix_ms>:handler_failed` and `ritma-dead-v1:handler_failed`, with `attempts >= 5` terminal; the fifth backoff is unreachable by the specified five-failure threshold. This keeps the ERD exact. [TD-043]
- **Self-resolved (implementer, run 6, needs architect ratification):** STM-80 names owner Retry/Discard effects but the task API lacks an authorized permission and audit contract → create the required OPS-08 attention item with kind `outbox_dead_letter`, high severity, a stable event-id dedupe key and event-id-only payload, while deferring owner mutations to a task with the authorized audited surface. [TD-045]

## Review
n/a until reviewer feedback.
