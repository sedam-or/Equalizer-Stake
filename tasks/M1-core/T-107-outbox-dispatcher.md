---
id: T-107
title: Add the transactional outbox, event envelope, and dispatcher with idempotent handlers and retries
milestone: M1
status: blocked
depends_on: [T-105]
spec_refs: [ARC-07, MOD-04, CAN-60, CAN-61, STM-80]
estimated_size: M
---

## Goal
State changes emit events in the same transaction. A background dispatcher delivers each event at least once to every registered handler, exactly once per handler in effect (receipts), with retries and a dead-letter state.

## Context
- DDL, copied verbatim from `spec/06 §4.1` into `migrations/0004_core_outbox_jobs.sql` (together with `jobs` from T-108; **create all three tables in this migration**):
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
  - When all handlers succeed, set `dispatched_at`. On failure, increment `attempts` and persist the retry due time as a versioned, PII-free marker in the existing `last_error` text, using the backoff list `[1s, 5s, 30s, 5min, 30min]` (technical const). Claim only rows whose marker is absent or due. At 5 failures, mark the row dead in `last_error`, exclude it from claims using `attempts`, and log `error!`. The schema remains exactly the `spec/06` DDL. Dead-letter attention items use OPS-08; the notification alert is delivered by notify in M5.
- The envelope: `Event { id: Uuid (v7), tenant_id, event_type: String, aggregate_id: Option<Uuid>, occurred_at, actor, request_id, payload: Value }`. `actor` reuses the shipped `EventActor` / `ActorRole` contract from T-105. A typed helper is `trait DomainEvent: Serialize { const TYPE: &'static str; fn aggregate_id(&self) -> Option<Uuid>; }`; `emit(tx, tenant, &E)` serializes it, takes actor and request id from the transaction context, and rejects a tenant that differs from that context.
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
- Outbox row states follow STM-80. Owner Retry/Discard effects are audit-backed operations; the explicit T-107 API scope exposes only `emit`, `run`, and `drain_once` and the audit migration/API is not yet a dependency, so the row mutation endpoints follow when that authorized surface exists.

## Scope
### In
- The migration with outbox, event_receipts, and jobs DDL from spec 06 §4.1 verbatim, plus the NOTIFY trigger and required permissions; no additional table or column.
- `eq_core::outbox::{Event, DomainEvent, emit, EventHandler, Dispatcher}`, plus an `EventSink` impl.
- `Dispatcher::run(shutdown: CancellationToken)` and `Dispatcher::drain_once()` for tests.
- Metrics counters through `tracing` events: dispatched, failed, dead.

### Out
- Job runner (T-108). Concrete handlers (module tasks).

## Acceptance criteria
- AC1. `emit` in a tx that rolls back leaves no outbox row. → `outbox::tests::rollback_no_event`
- AC2. Two dispatcher instances on the same DB deliver each event to a handler exactly once (count receipts and handler invocations over 500 events). → `two_dispatchers_exactly_once_per_handler`
- AC3. A handler failing twice then succeeding is called 3 times; the event ends with `dispatched_at` set and `attempts=2`. → `retry_then_success` (with a `TestClock`/fast backoff override in tests)
- AC4. After 5 failures, the row is dead (`attempts = 5` and the dead marker is stored) and no more attempts happen. → `dead_letter_after_max`
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

**Plan** — (1) Add `migrations/0004_core_outbox_jobs.sql` with the three exact DDL definitions mapped to T-107 in spec/29 §7 and a statement-level NOTIFY trigger → verify with the migration test and `scripts/erd.py --migrations`. (2) Extend `Tx` with trusted tenant and actor/request context → verify rollback and context propagation tests. (3) Add CAN-60 `Event`, typed `DomainEvent`, transaction-bound `emit`, and the T-105 `EventSink` implementation → verify the envelope and tenant-mismatch tests. (4) Add PostgreSQL dispatch with LISTEN/poll wake-up, bounded group concurrency, aggregate ordering, receipts, module gating, retry/dead state → verify the six AC tests and dispatcher lifecycle test. (5) Run Rust, SQLx, deny, guard, ERD, security-scan, and disposable PostgreSQL browser E2E checks → record each result below.

**Assumptions** — The spec/06 DDL and spec/29 schema map are exhaustive. Retry scheduling is encoded only in existing `outbox.last_error`; `attempts >= 5` is the durable dead-state predicate. CAN-60 metadata comes from trusted `Tx` context. The shipped T-105 `ActorRole` is preserved, with its role mismatch recorded for architect ratification. Payloads remain opaque to the dispatcher and are not logged. Database tests use the `eq_app` pool; browser E2E uses a fresh disposable PostgreSQL 16 database.

**Test plan** — AC1 Given an event is emitted in a transaction, when it rolls back, then no outbox row remains → `rollback_no_event`. AC2 Given two dispatchers and 500 events, when they drain concurrently, then each handler runs once and has one receipt → `two_dispatchers_exactly_once_per_handler`. AC3 Given a handler fails twice, when retry deadlines pass, then its third call succeeds with two recorded failures → `retry_then_success`. AC4 Given five handler failures, when the fifth is recorded, then the row is dead and no sixth call occurs → `dead_letter_after_max`. AC5 Given multiple events for one aggregate, when drained, then delivery follows id order → `order_within_aggregate`. AC6 Given a disabled handler module, when an event is delivered, then it is skipped and receipted → `disabled_module_handler_skipped`. Edge/abuse cases: event rollback, duplicate receipt suppression, aggregate order, bounded retry count, disabled-module gating, tenant mismatch rejection, and wake/cancel behavior. Errors use a safe handler context and ERR-71 error chain; event payloads are not log fields.

**Summary** — A prototype implemented the typed event emission and PostgreSQL dispatcher, and its Rust, schema and browser checks passed. The prototype has been reverted under WORKFLOW §4 because the task's non-generated diff exceeds CQ-101's approximate 400-line limit and that standard requires larger work to be split into tasks first. This blocked commit contains only this task report and the generated index update; no dispatcher code is shipped.

**Workflow trace** — Step 0: resumed the existing T-107 work on `milestone/m1-core`, preserved its WIP, and checked the task index. Step 1: read the task, cited specs and quality rules. Step 2: read T-105's Report and shipped contracts. Step 3: n/a (backend-only; no screen or design-system work). Step 4: Plan and test plan above. Step 5: implemented an in-scope prototype, then reverted it after the CQ-101 size finding. Step 6: exercised PostgreSQL dispatch and the server with browser E2E. Step 7: failures and fixes are recorded in Debug log. Step 8: reviewed architecture, SQL/error handling, hygiene and size; fixed the discarded handler error chain to retain ERR-71 context; no banned markers were found in the prototype's changed Rust source. Step 9: measured the prototype's queue queries below. Step 10: completed SBD-81 review below. Step 11: reran the full local checks on the prototype before reverting it. Step 12: documented the block, set `blocked`, and regenerated the index; this commit contains no implementation.

**Design reference** — n/a; the task adds no user interface.

**Spec read** — `spec/02-architecture.md` ARC-07; `spec/03-module-system.md` MOD-01/04; `spec/06-domain-model.md` §4.1; `spec/11-ops.md` OPS-08; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/5.1/8; `spec/19-error-handling.md` §§8/10; `spec/20-state-machines.md` STM-14/80/82; `spec/21-data-flows.md` FLW-03; `spec/23-contracts-and-canonical-forms.md` CAN-60/61; `spec/24-open-source-stack.md` §6; `spec/29-erd.md` core module data dictionary and §7. Read T-105's Report, shipped `Event`, `EventHandler`, `EnabledModuleSet`, and `EventSink`, plus the full M1 task workflow.

**Files** — Prototype-only paths, all reverted before this blocked commit: `Cargo.toml`, `Cargo.lock`, `crates/eq-core/Cargo.toml`, `crates/eq-core/src/{actor.rs,db.rs,outbox.rs}`, `crates/eq-core/tests/outbox.rs`, `migrations/0004_core_outbox_jobs.sql`, and seven `.sqlx/query-*.json` files. The blocked commit contains only `tasks/M1-core/T-107-outbox-dispatcher.md` and generated `tasks/INDEX.md`.

**Acceptance criteria → tests**
The passing results below belong to the temporary prototype described above; the implementation was reverted before this blocked commit.
| AC | Test | Result |
|---|---|---|
| AC1 | `rollback_no_event` | ✅ No outbox row after rollback |
| AC2 | `two_dispatchers_exactly_once_per_handler` | ✅ 500 events, exactly one call and receipt per handler |
| AC3 | `retry_then_success` | ✅ Two failures, third call succeeds, `attempts = 2` |
| AC4 | `dead_letter_after_max` | ✅ Five calls, durable dead marker, high-severity attention item, no sixth call |
| AC5 | `order_within_aggregate` | ✅ Events delivered in id order |
| AC6 | `disabled_module_handler_skipped` | ✅ Handler skipped and receipted |
| Abuse/lifecycle | `emit_rejects_tenant_mismatch`, `event_sink_preserves_actor_and_request_context`, `run_wakes_after_commit_and_stops_on_cancel` | ✅ Tenant mismatch rejected; actor/request preserved; LISTEN wake-up and cancellation work |

**Checks run** — All implementation-specific results below were run on the temporary prototype before its reversion; they do not mean this blocked commit delivers T-107. `cargo fmt --all --check` ✅. `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` ✅. `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` ✅; all workspace tests passed, including the nine outbox tests and testcontainer PostgreSQL 16/RLS tests. `cargo deny check` ✅; advisories, bans, licences and sources passed with existing duplicate `base64` and unmatched `CC0-1.0` warnings. `cargo sqlx prepare --workspace --check` ✅ against the fresh migrated PostgreSQL database. `cargo run -p eq-server -- migrate` ✅ against that database after setting the required fake test configuration. `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` ✅. `python3 scripts/erd.py --migrations` ✅ (18 migrated tables checked); `python3 scripts/erd.py --check` ✅. `scripts/tasks-index.sh --check` ✅ after block status/report update. `git diff --check` ✅. `gitleaks detect --redact --no-banner` ⚠️ reports one pre-existing false positive in `Makefile:80-85` (`curl-auth-user`); targeted `gitleaks dir --redact --no-banner` scans of the changed source, migration, metadata, manifest, task and index paths found no leaks. `pnpm -C web lint`, `typecheck`, `test`, `i18n:check`, and `make api`: n/a because no web/API contract files changed; `make e2e` did build the web app and run Playwright. Hosted GitHub Actions: not run in this unattended environment. The prototype migration and tests were deleted after the size block; ERD and Rust results above are historical prototype checks, not checks on the final blocked tree.

**Manual checks** — On the reverted prototype, `Dispatcher::run` was exercised by `run_wakes_after_commit_and_stops_on_cancel` against testcontainers PostgreSQL 16. `make e2e` launched the release server and sent real browser requests in Chromium and WebKit mobile; all six current smoke checks passed on disposable PostgreSQL 16.15 container `ritma-t107-e2e-final-20261027`, database `ritma_t107_e2e_final_20261027`, port 5438. The container was stopped and removed afterward. Exact command:
```sh
env E2E_SKIP_DB_START=1 DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:5438/ritma_t107_e2e_final_20261027 RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-suite@127.0.0.1:5438/ritma_t107_e2e_final_20261027 RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://127.0.0.1:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off RITMA_MODE=single SQLX_OFFLINE=true make e2e
```
The full persona flows JRN-01..12 are not implemented by this backend-only task; `web/e2e` currently contains only the six smoke checks above. `make dev` was not run separately because this task adds no HTTP operation to trigger the dispatcher; its runtime is exercised by the PostgreSQL integration tests and the release server by E2E.

**Debug log** — The first `cargo fmt --all --check` after preserving the handler's anyhow chain found a formatting difference; `cargo fmt --all` fixed it and the final format and clippy checks passed. Initial fresh-database migration setup lacked `RITMA_SECRET_KEY`, then `RITMA_PUBLIC_URL`; supplying the documented fake test configuration made migration succeed. The first E2E run used a fresh database without a provisioned tenant and failed four home/fallback checks with `setup_required`; after adding a synthetic tenant fixture, a final run on a newly created disposable database passed 6/6. These were test-environment setup issues, not application-code failures.

**Performance** — On a synthetic 1,000-row pending queue, the candidate-group query took about 11.56 ms and used a hash anti-join (262,950 rows removed by join filter at this test size). Testing an additional composite index in a rolled-back transaction did not improve the plan (~12.53 ms), so no schema index was added. The aggregate lock lookup used the existing `outbox_pending` index and took about 0.617 ms. These are local synthetic measurements, not production latency claims.

**Security** — Review of the reverted prototype: triggers were tenant isolation and actor/request metadata. SBD-81: (1) no new HTTP endpoint or permission surface; internal event delivery uses the registered handler set. (2) tenant came from `Tx`, `emit` rejected mismatch, and handler transactions called `begin_system` for the event tenant; integration checks used `eq_app` (`NOSUPERUSER NOBYPASSRLS`). Tables had no RLS as required by the mapped system-table DDL. (3) no new schema columns; payload was not emitted to logs, and log fields contained event/tenant/actor/request IDs, type, attempt and code. Handler error chains were retained per ERR-71 and handler messages must remain PII-free. (4) emitted event types were compile-time constants and payloads typed at the producer boundary; malformed stored envelopes failed closed to a PII-free marker. (5) no HTML/CSV output. (6) no secret or cryptographic handling added. (7) no external calls. (8) unique `(handler_id, event_id)` receipts, transaction rollback, and `SKIP LOCKED` claims protected effects against duplicate dispatch; concurrency and rollback tests passed. (9) audited owner retry/discard actions were not exposed; see the STM-80 scope deviation. (10) tenant-mismatch, rollback, two-dispatcher, disabled-module, and retry/dead abuse cases were covered. Full gitleaks scan found only the existing Makefile false positive; changed prototype paths were clean.

**Deviations** — RATIFY: (1) The task sketch asks for retry/dead timestamp columns absent from the exhaustive schema; exact DDL is kept and retry/dead state uses versioned `last_error` markers plus `attempts`. The fifth backoff is unused because the row becomes dead at five failures. (2) CAN-60 lists `admin | system`, while T-105 ships `ActorRole` with owner, admin, instructor, guardian, student and system; preserve the shipped roles rather than collapse actor attribution. (3) STM-80 names audited owner Retry/Discard actions, but T-107 exposes only `emit`, `run`, and `drain_once`, and the relevant authorized action/API contract is not in scope; those actions remain deferred. All three are detailed under Questions for architect ratification. The implementation retains the source anyhow error chain for ERR-71 and does not place the event payload in logs.

**Dependencies affected** — No prototype code or migration was shipped, so no downstream task can depend on these contracts. T-108 remains ineligible while T-107 is blocked. The prototype migration had shared the required `jobs` table, as T-107 specifies.

**Rollback plan** — No implementation commit or migration remains on the branch. The temporary prototype used only the disposable PostgreSQL container, which was stopped and removed; no shared database was changed.

**Follow-ups** — Resolve the blocking task-splitting question below, then architect ratification for the three preserved self-resolution items. The prototype's non-generated diff exceeded CQ-101; `outbox.rs` alone contributed 627 changed lines. Existing full-repository Gitleaks false positive at `Makefile:80-85` remains. OpenCode free-model audit could not run in `child-opencode` (Nemotron unavailable, DeepSeek free quota exhausted, Hy3 route refused connections); Codex completed the local self-review and checks.

## Questions
- **Self-resolved (implementer, run 5, needs architect ratification):** T-107 asks for `outbox.next_attempt_at` and `outbox.dead_letter_at`, but spec/06 §4.1 and spec/29 §7 define an exhaustive outbox schema without those columns → keep the mapped DDL exact, encode retry due state in a versioned `last_error` marker and dead state as `attempts >= 5`, because ERD-03 and the run's schema boundary forbid undeclared columns. The fifth listed backoff is unused because the row becomes dead after the fifth failure.
- **Self-resolved (implementer, run 5, needs architect ratification):** T-107's initial `Event` sketch omits CAN-60 `actor` and `request_id` and uses a borrowed event type that cannot represent a persisted row; CAN-60 lists only `admin | system`, while T-105 ships a closed `ActorRole` with the full active role set → carry actor/request context in the stored envelope, expose persisted event types as `String`, and preserve the shipped role values instead of collapsing distinct actors, because role attribution must not be changed or guessed. Architect ratification is needed for the canonical event role values.
- **Self-resolved (implementer, run 5, needs architect ratification):** STM-80 names audited owner Retry/Discard actions, but the explicit T-107 API surface is only `emit`, `Dispatcher::run`, and `drain_once`, and the audit migration/API is not a dependency yet → implement durable dispatcher states and OPS-08 attention items; defer owner mutations until the authorized action and audit contract exists, because adding an unscoped permission or bypassing audit would violate SBD-40/53.
- **Q1 (blocking):** T-107's prototype exceeds CQ-101's approximate 400 changed-line limit, which requires larger work to be split into tasks first; may the architect decompose the T-107 scope and acceptance criteria into smaller dependent tasks before implementation resumes?
  - Context: the prototype added 1,446 lines across source, tests, migration, query metadata, manifests and reports; excluding generated metadata and lockfiles, the task still exceeds 400 lines. CQ-101 says bigger work gets split into tasks first. No code has been retained in this blocked commit.
  - Options: (a) split implementation into smaller architect-authored tasks, each within CQ-101; (b) revise the task's scope/AC through an architect task update before resuming.
  - Recommendation: (a), because CQ-101 explicitly requires larger work to be split before implementation and keeps each reviewable unit narrow.
