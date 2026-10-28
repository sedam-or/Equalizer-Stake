---
id: T-107c
title: Add outbox retry, dead-letter and worker lifecycle
milestone: M1
status: review
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
- Classify event-handler failures as retryable or permanent per ERR-52; permanent failures go directly to dead-letter.
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
| `Cargo.toml`, `crates/eq-core/Cargo.toml` | modify | Add Tokio's cancellation-token support required by the task interface |
| `.sqlx/` | modify | Add offline metadata for retry/attention queries |
| `tasks/M1-core/T-107c-retry-dead-letter.md`, `tasks/M1-core/T-107-outbox-dispatcher.md`, `tasks/INDEX.md` | modify | Report the final part and parent status |
| `tasks/DECISIONS.md` | modify | Maintain TD-043 and TD-045 decisions |

## Interfaces
```rust
impl Dispatcher {
    pub fn new_with_clock(pool: PgPool, handlers: Vec<Box<dyn EventHandler>>, enabled: Arc<dyn EnabledModuleSet>, clock: Arc<dyn Clock>) -> Self;
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
- AC4. A permanent handler failure is dead-lettered without waiting for retry attempts → `permanent_handler_failure_dead_letters_without_retry`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/outbox.rs` | `retry_then_success`, `dead_letter_after_max`, `run_wakes_after_commit_and_stops_on_cancel`, `permanent_handler_failure_dead_letters_without_retry`, `unknown_failure_marker_fails_closed` |

## Definition of done
- [x] Rust format, Clippy and workspace tests pass, including all three named PostgreSQL tests on disposable PostgreSQL 16.
- [x] `cargo sqlx prepare --workspace --check -- --all-targets`, `cargo deny check`, `python3 -I scripts/erd.py --check` and all guard scripts pass.
- [x] `make e2e` passes on a fresh disposable PostgreSQL 16 database; record exact command, database and pass count.
- [x] Security/report/index are complete and one signed-off commit is created.

## Report
**Plan** — (1) Add the ERR-52 retryable/permanent handler result and Tokio cancellation support → compile existing handlers and pass `cargo deny check`. (2) Encode retry/dead state in the mapped `outbox.last_error`, select only due retries, and release aggregate order after terminal dead-letter → verify exact backoff boundaries and terminal handling with a controllable clock and PostgreSQL. (3) Upsert one tenant-scoped OPS-08 attention item for a dead event and expose process-local attempt/dispatched/failed/retry/dead counters → verify safe logs, counter totals, dedupe and no raw error persistence. (4) Add LISTEN plus poll fallback and cancellation → verify notification wake and clean exit. (5) Run full Rust, SQLx metadata, dependency, ERD, guard, secret-scan and fresh-database E2E checks.
**Assumptions** — Task-specified delays and the five-retryable-failure threshold are authoritative. ERR-52's permanent/retryable split takes precedence over treating every handler failure as retryable. `last_error` remains PII-free. Error causes are not formatted into logs; operational records use ids and stable failure codes. Owner actions are deferred under TD-045.
**Test plan** — AC1 Given a retryable handler fails twice and then succeeds, when the injected clock passes the first two due instants, then it is called three times and records two attempts → `retry_then_success`. AC2 Given five retryable failures, when each specified retry becomes due, then the fifth failure is terminal and only one high-severity attention row exists → `dead_letter_after_max`. AC3 Given a dispatcher is listening, when a committed event notifies `eq_outbox`, then it drains it and stops after cancellation → `run_wakes_after_commit_and_stops_on_cancel`. AC4 Given a permanent handler failure, when drained, then it is dead-lettered without retry and the next event for the same aggregate proceeds → `permanent_handler_failure_dead_letters_without_retry`. Edge/abuse checks cover a not-yet-due marker, malformed/unknown markers failing closed, duplicate attention dedupe, RLS through `eq_app`, dispatched/failed counters, and fake handler text absent from persisted markers and logs.
**Summary** — The dispatcher now classifies handler failures according to ERR-52, schedules retryable failures with the task's bounded backoff, and dead-letters permanent or exhausted events using the mapped columns. Dead-letter attention is tenant-scoped and deduplicated; dead events no longer block later events for the same aggregate. The worker listens for committed inserts, retains its polling fallback, stops on cancellation, and records safe process-local counters and structured log fields.
**Workflow trace** — Step 0: resumed the authorized T-107c work on `milestone/m1-core`, checked the task index and kept WIP at one. Step 1: read the full task and its cited specification sections, including the mapped core schema. Step 2: read T-107b's Report and inspected its shipped dispatcher and aggregate-ordering query. Step 3: n/a; backend-only task. Step 4: recorded the Plan, assumptions and Given/When/Then test plan before implementation. Step 5: implemented the typed handler error, retry/dead marker handling, attention upsert, cancellation worker and counter/log fields, plus the named PostgreSQL tests. Step 6: exercised real PostgreSQL dispatch, retry timing, tenant-scoped dead-letter attention and LISTEN/cancel behavior; `make e2e` made real HTTP requests against the release server and built the SPA. Step 7: corrected an initial Clippy `too_many_lines` finding by extracting marker persistence and post-commit counter/log recording; the first fresh-database E2E fixture lacked a tenant and returned `setup_required`, so a new fresh database with a synthetic tenant was used for the final green run. Step 8: reviewed SQL parameterization, aggregate order, error mapping, layer boundaries, hygiene and log redaction; no schema or unrelated files changed. Step 9: n/a for new query/index optimization; this task only adds a terminal-marker predicate to the existing T-107b claim query and adds no index. Step 10: completed the SBD-81 review below and scanned changed paths for secrets. Step 11: reran Rust format, Clippy, the full workspace tests, SQLx metadata check, cargo deny, ERD, guard and task-index checks; final E2E passed 6/6. Step 12: set `review`, regenerated the index and prepared this single signed-off task commit.
**Spec read** — `spec/06-domain-model.md` §§4.1/4.11; `spec/11-ops.md` OPS-03/08; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/8; `spec/19-error-handling.md` ERR-50/51/52/71; `spec/20-state-machines.md` STM-14/80/82; `spec/21-data-flows.md` FLW-03; `spec/23-contracts-and-canonical-forms.md` CAN-60/61; `spec/24-open-source-stack.md` §6; `spec/29-erd.md` core module data dictionary and §7.
**Files** — Modified `Cargo.toml`, `Cargo.lock`, `crates/eq-core/Cargo.toml`, `crates/eq-core/src/outbox.rs`, `crates/eq-core/tests/outbox.rs`, and regenerated `.sqlx/` query metadata. Updated `tasks/DECISIONS.md`, this task Report/status, its parent `tasks/M1-core/T-107-outbox-dispatcher.md`, and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `retry_then_success` | ✅ 3 handler calls; two persisted retry attempts; exact 1s then 5s delays controlled by the injected clock; retry/dispatched counters match. |
| AC2 | `dead_letter_after_max` | ✅ Five retryable failures, terminal marker, no sixth attempt, one high-severity tenant-scoped attention item, expected counters. |
| AC3 | `run_wakes_after_commit_and_stops_on_cancel` | ✅ Listener receives committed insert notification, handler runs, and cancellation stops the worker. |
| AC4 | `permanent_handler_failure_dead_letters_without_retry` | ✅ Permanent failure dead-letters on first attempt; the later event for the same aggregate dispatches. |
| Abuse/edge | `unknown_failure_marker_fails_closed` and remaining outbox integration suite | ✅ Unknown marker is not claimed; all 18 outbox tests pass on disposable PostgreSQL 16. |
**Checks run** — `cargo fmt --all --check` ✅. `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` ✅. `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` ✅; 193 tests passed, 0 failed, including 18 outbox tests against testcontainers PostgreSQL 16. `SQLX_OFFLINE=false DATABASE_URL=postgres://eq:test-secret-owner@localhost:55432/ritma cargo sqlx prepare --workspace --check -- --all-targets` ✅. `cargo deny check` ✅; advisories, bans, licences and sources pass, with existing duplicate-`base64` and unmatched `CC0-1.0` warnings. `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` ✅. `scripts/tasks-index.sh --check`, `python3 -I scripts/erd.py --check`, and `git diff --check` ✅. Migration DDL is unchanged, so `python3 -I scripts/erd.py --migrations` is n/a. Per-file `gitleaks dir --redact --no-banner --verbose` scans on changed Rust, test, `.sqlx`, Cargo manifests/lock and task/decision paths found no leaks; a broad scan also surfaced the previously documented Makefile baseline false positive at `Makefile:80-85`. Web lint/typecheck/unit/i18n checks are n/a because no web source changed; E2E built the SPA. Hosted GitHub Actions: Not run, requires the hosted runner.
**Manual checks** — PostgreSQL integration tests used fresh testcontainers based on `postgres:16-alpine`; `run_wakes_after_commit_and_stops_on_cancel` exercised LISTEN wake and shutdown against a real database. `make dev` was not run separately because the server does not yet wire this dispatcher into a production runtime; the worker lifecycle is exercised directly by its integration test. E2E used disposable PostgreSQL 16.15 container `ritma-t107c-db-run6` and newly created database `ritma_t107c_e2e_final2_20261028`, migrated from empty and seeded with one synthetic completed tenant. Exact command:
```sh
env E2E_SKIP_DB_START=1 DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:55432/ritma_t107c_e2e_final2_20261028 RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@127.0.0.1:55432/ritma_t107c_e2e_final2_20261028 RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://127.0.0.1:18080 RITMA_LISTEN=127.0.0.1:18080 RITMA_BACKUP_SCHEDULE=off RITMA_MODE=single E2E_BASE_URL=http://127.0.0.1:18080 SQLX_OFFLINE=true E2E_PLAYWRIGHT_CONTAINER=1 make e2e
```
Result: 6/6 Playwright smoke tests passed across Chromium and WebKit mobile. The owned container was stopped and removed after the run.
**Debug log** — The first `cargo fmt --all --check` found formatting differences; `cargo fmt --all` fixed them. The first Clippy run found `drain_once` exceeded `too_many_lines`; marker persistence and post-commit metrics/logging were extracted into helpers, then Clippy passed. An initial workspace-test invocation inherited an incompatible `RITMA_TEST_DATABASE_URL`; clearing that variable made the full suite pass. SQLx metadata preparation with the app role failed on a `policy_store` access restriction; using the migration owner URL (the documented metadata-preparation path) succeeded. The first E2E attempt used an empty fresh database and failed 4/6 home/fallback checks with `setup_required`; a second fresh migrated database with a synthetic tenant passed 6/6. No test failure remained.
**Performance** — n/a for new lookup/list optimization. The claim query is the existing T-107b query with an exact terminal-marker exclusion in aggregate-head selection; no schema or index was added.
**Security** — SBD-81: (1) no new HTTP endpoint or permission surface; handler registry remains the internal effect boundary. (2) Tenant ID comes from the stored event and is applied through `db::begin_system`; attention upserts use the event tenant, and isolation tests use `eq_app` with `NOSUPERUSER NOBYPASSRLS`. (3) Retry and terminal state contain only versioned timestamps/stable codes; no raw handler text or payload is persisted or logged. Structured logs use event/tenant IDs, stable failure code and outcome. (4) Event JSON is handled by the existing CAN-60 envelope decoder and invalid marker formats fail closed. (5) No HTML/CSV output. (6) No new secrets or cryptographic handling. (7) No external calls. (8) Handler receipts, row locks, `SKIP LOCKED`, and deduplicated attention upserts protect duplicate effects; tests cover concurrent dispatch, marker behavior, tenant scope and attention dedupe. (9) Owner Retry/Discard actions are not exposed because no authorized audited contract is in scope. (10) Abuse tests cover transient and permanent failure, retry exhaustion, unknown marker, aggregate successor progress, and the absence of handler error text from durable state; changed-path secret scans were clean.
**Deviations** — RATIFY TD-043: encode retry/dead state with the exact existing columns and versioned marker; the fifth retryable failure is terminal. RATIFY TD-045: raise an event-id-only OPS-08 attention item and defer audited owner actions. RATIFY TD-058: apply ERR-52's retryable/permanent classification to event handlers; permanent handler failures go directly to dead-letter. Details remain in Questions and `tasks/DECISIONS.md`.
**Dependencies affected** — T-116a can proceed after this final dispatcher part is `review`.
**Rollback plan** — Revert this task commit; no schema changes are made here.
**Follow-ups** — M5 delivers the notification alert. Owner Retry/Discard stays deferred until its authorized audited contract is assigned. The 616-line non-generated task diff exceeds CQ-101's approximate 400-line PR guidance; the run requires one PR per milestone, and the reviewer should consider whether a further architect-owned split is needed before merging.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** the task requires retry/dead state but the mapped DDL has no `next_attempt_at` or `dead_letter_at` → use the existing `last_error` marker `ritma-retry-v1:<due_unix_ms>:handler_failed` and `ritma-dead-v1:handler_failed`, with `attempts >= 5` terminal; the fifth backoff is unreachable by the specified five-failure threshold. This keeps the ERD exact. [TD-043]
- **Self-resolved (implementer, run 6, needs architect ratification):** STM-80 names owner Retry/Discard effects but the task API lacks an authorized permission and audit contract → create the required OPS-08 attention item with kind `outbox_dead_letter`, high severity, a stable event-id dedupe key and event-id-only payload, while deferring owner mutations to a task with the authorized audited surface. [TD-045]
- **Self-resolved (implementer, run 6, needs architect ratification):** T-107c described all handler failures as retryable while ERR-52 requires the retryable/permanent split → use a typed `EventHandlerError`, apply the specified backoff only to retryable failures, and dead-letter permanent failures immediately because ERR-51/52 define that behavior. [TD-058]

## Review
n/a until reviewer feedback.
