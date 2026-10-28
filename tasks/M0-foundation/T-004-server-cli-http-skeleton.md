---
id: T-004
title: Build the ritma CLI and HTTP skeleton (serve, migrate, version, healthz/readyz, tracing, graceful shutdown)
milestone: M0
status: review
depends_on: [T-003]
spec_refs: [ARC-04, OPS-02, OPS-03, OPS-06, API-14, CAN-30]
estimated_size: M
---

## Goal
`ritma serve` loads the config, runs migrations, and serves `/healthz` and `/readyz` through the standard tower layer stack, with request ids and structured logs. SIGTERM shuts it down gracefully.

## Context
- OPS-02 CLI: `serve` (default), `migrate`, `openapi`, `backup`, `restore`, `doctor`, `create-owner`, `rotate-keys`, `seed-demo`, `version`. This task implements `serve`, `migrate`, `version` and `healthcheck` (exit 0 if `GET http://127.0.0.1:<port>/healthz` returns 200 within 2 s, else 1; used by the container `HEALTHCHECK`, spec 26 CTR-05). The others print "not implemented yet" and exit with code 2.
- ARC-04 layers, in order:
  1. request id (UUIDv7) → `TraceLayer` with span fields request_id, tenant_id, actor_id (later), method, path, status, latency
  2. compression
  3. security headers (stub now, implemented in T-115)
  4. body limit 1 MiB
  5. `CatchPanicLayer` (tower-http): a panic becomes a 500 `internal` problem and is logged once at `error` with the request id (ERR-42)
- The request id is put into `eq_core::error::REQUEST_ID` (task-local) for the duration of the request, and returned as the `X-Request-Id` header.
- OPS-06: `/healthz` returns 200 `{"status":"ok"}` once the process is up. `/readyz` returns 200 when a DB ping (`SELECT 1`) completes in ≤ 500 ms **and** no migrations are pending; otherwise 503 with `{"status":"not_ready","reason":…}`. Neither is under `/api/v1`.
- OPS-03: graceful shutdown on SIGTERM/SIGINT, waiting ≤ 25 s for in-flight requests.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - The request-id layer **ignores** any client `X-Request-Id` and always generates its own UUIDv7 (CAN-30, D-09); the response still returns `X-Request-Id`.

## Scope
### In
- `eq-server/src/main.rs`: clap CLI with subcommands. `serve` does: tracing init (RITMA_LOG filter, json or pretty) → config → `run_migrations` → pool → `eq_api::router(state)` → `axum::serve` with graceful shutdown.
- `eq-api/src/lib.rs`: `pub struct AppState { pub pool: PgPool, pub config: Arc<Config>, pub clock: Arc<dyn Clock> }` (Clone) and `pub fn router(state: AppState) -> axum::Router` with health routes and the layer stack.
- A request-id middleware that sets the task-local and the response header.

### Out
- SPA embedding (T-005), auth and tenant layers (M1), the security-headers content (T-115).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-server/src/main.rs` | edit | CLI |
| `crates/eq-server/src/telemetry.rs` | create | tracing init |
| `crates/eq-api/src/lib.rs`, `src/health.rs`, `src/middleware/request_id.rs` | create | router |
| `crates/eq-core/src/db.rs` | edit | let the app role read the SQLx migration status after migrations run |
| `Cargo.toml`, `crates/eq-api/Cargo.toml`, `crates/eq-server/Cargo.toml` | edit | direct runtime and test dependencies required by the interfaces |
| `crates/eq-api/tests/health.rs`, `crates/eq-server/tests/cli.rs` | create/edit | acceptance and CLI integration tests |

## Interfaces
```
GET /healthz → 200 {"status":"ok","version":"0.1.0"}
GET /readyz  → 200 {"status":"ready"} | 503 {"status":"not_ready","reason":"db_unreachable"|"migrations_pending"}
```

## Business rules
None.

## Implementation notes
- To detect pending migrations, compare `sqlx::migrate!().migrations` versions with the `_sqlx_migrations` table. Cache the result for 10 s.
- Use `tokio::signal` for ctrl_c and the unix terminate signal.
- Don't log request bodies or query strings (SEC-21). Log the path template, not the raw URI, when available (`MatchedPath`).

## Acceptance criteria
- AC1. With a DB available, `GET /healthz` → 200 and the `X-Request-Id` header is present (UUID). → `eq-api/tests/health.rs::healthz_ok`
- AC2. With the DB stopped, `GET /readyz` → 503 `db_unreachable` within 1 s. → `readyz_db_down`
- AC3. `ritma migrate` against an empty DB applies 0001 and exits 0. → `eq-server/tests/cli.rs::migrate_applies`
- AC3b. `ritma healthcheck` exits 0 while the server is up and 1 when nothing listens on the port. → `eq-server/tests/cli.rs::healthcheck_exit_codes`
- AC4. A Problem body produced inside a request contains the same `request_id` as the header. → `problem_has_request_id` (add a test-only route)

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/health.rs` | `healthz_ok`, `readyz_db_down`, `readyz_migrations_pending`, `readyz_ready`, `problem_has_request_id` |
| `crates/eq-server/tests/cli.rs` | `prints_version`, `version_subcommand_reports_build_metadata`, `migrate_applies`, `unimplemented_subcommand_exits_2`, `healthcheck_exit_codes` |

## Definition of done
- [x] Rust checks pass
- [x] Manual: `DATABASE_URL=… RITMA_SECRET_KEY=… RITMA_PUBLIC_URL=http://localhost:8080 cargo run -p eq-server -- serve`, then `curl localhost:8080/healthz` (paste the output in the Report)
- [x] Commit `feat(server): cli and http skeleton [T-004]`, signed off

## Questions
- **Self-resolved (implementer, run 5, needs architect ratification):** the task's Interfaces, test table and acceptance criteria require direct framework/runtime dependencies and the listed integration-test files, but `## Files` omits their manifests and test paths → add only the three relevant Cargo manifests and the two named test files because these paths are required to compile and verify the task's specified interfaces. [TD-007]
- **Self-resolved (implementer, run 5, needs architect ratification):** SQLx creates `_sqlx_migrations` before `0001_core_bootstrap.sql` sets default privileges, so `eq_app` cannot read migration status and `/readyz` reports `migrations_pending` after a successful migration → after migrations, grant `eq_app` only `SELECT` on the SQLx metadata table using the already specified DML grant boundary; this exposes migration versions only and is required by OPS-06. [TD-008]
- **Self-resolved (implementer, run 5, needs architect ratification):** the task's startup sequence initializes tracing before Config, but the repository's Config boundary forbids reading environment variables in telemetry (`cargo clippy` `disallowed_methods`, OPS-01) → load and validate Config first, then initialize tracing from `Config.log` and `Config.log_format`, preserving the single configuration source required by the workspace lint. [TD-009]

## Report

**Plan** — (1) Add only the direct dependencies and CLI/telemetry wiring required by the task → verify `cargo check -p eq-server`. (2) Load validated `Config`, initialize tracing from its fields, and add request ID context plus the ordered HTTP middleware stack → verify the exact health API tests and error correlation test. (3) Ensure the app role can read only SQLx migration metadata → verify pending and current migration states on separate PostgreSQL 16 containers. (4) Add CLI commands, migration and healthcheck behavior, tracing, and bounded signal shutdown → verify the exact CLI tests. (5) Run the server against the local PostgreSQL 16 database and exercise `/healthz`/`/readyz` with real HTTP requests. (6) Run full Rust, guard, secret, and E2E checks; record unavailable checks explicitly and rerun after fixes.

**Assumptions** — Use `Config`'s validated listen address and log settings. `healthcheck` checks the configured port on loopback only, as CTR-05 specifies. `readyz` checks the DB on each request with the stated 500 ms bound and caches the migration version check for the stated 10 seconds. `eq_app` receives read-only access to SQLx migration version metadata so readiness can compare applied and compiled versions. The not-implemented commands return the exact specified exit status without loading configuration.

**Test plan** —
- Given the process is up, when `GET /healthz` arrives with a forged `X-Request-Id`, then it returns the canonical health body and a server-generated UUIDv7 response id → `healthz_ok`.
- Given PostgreSQL is unreachable, when `GET /readyz` is requested, then it returns `503` with `db_unreachable` within one second → `readyz_db_down`.
- Given a migrated database is connected as `eq_app`, when `GET /readyz` is requested, then it returns `200` with `ready` → `readyz_ready`.
- Given a connected but unmigrated database, when `GET /readyz` is requested, then it returns `503` with `migrations_pending` → `readyz_migrations_pending`.
- Given an empty database, when `ritma migrate` runs, then migration `0001` is applied and the process exits successfully → `migrate_applies`.
- Given an implemented and an unimplemented command, when each is invoked, then the former completes and the latter prints its placeholder and exits `2` → `unimplemented_subcommand_exits_2`.
- Given a health server is listening or absent on the configured loopback port, when `ritma healthcheck` runs, then it exits `0` or `1` within two seconds → `healthcheck_exit_codes`.
- Given a handler returns `AppError` inside a request, when the response is rendered, then its `request_id` equals `X-Request-Id` → `problem_has_request_id`.
- Edge/security: client-supplied correlation IDs are ignored; query strings and bodies are never included in spans/logs; readiness distinguishes DB-down from migrations-pending; a caught panic returns `internal` and is logged once with its request id; SIGTERM and SIGINT stop accepting requests and drain for at most 25 seconds.

**Step 0 — Start** — On `milestone/m0-foundation` after T-003 commit `f2ede1a`; the tree was clean and `scripts/tasks-index.sh --check` passed. T-003 is `review`, T-004 was the lowest eligible M0 task, and its status was changed to `in_progress` through the task file followed by `scripts/tasks-index.sh`.

**Step 1 — Task/spec** — Read the whole T-004 task. Consulted `spec/02-architecture.md` §4 (ARC-04), `spec/07-api-conventions.md` API-14, `spec/11-ops.md` §§2–3 (OPS-02/03/06), `spec/19-error-handling.md` ERR-42 and §10, `spec/23-contracts-and-canonical-forms.md` CAN-30, `spec/10-security-privacy.md` SEC-21, `spec/26-containers-and-caching.md` CTR-05/08, plus `spec/16-code-quality.md` §§9/12, `spec/17-clean-architecture.md` §§1/4 and `spec/18-security-by-design.md` §§4/5/8. No business values or domain tables are added.

**Step 2 — Dependencies** — Read T-002's and T-003's Reports, including Deviations and Follow-ups, and inspected their shipped Config, AppError, `run_migrations`, `connect`, and `Clock` APIs. T-002's SQLx 0.9 baseline and validated `Config` secret fields are inherited. T-003 provides tenant-safe pool and migration APIs; its bootstrap creates SQLx's migration table before the default privileges for `eq_app` are installed, which required the read-only metadata grant recorded as a self-resolution below.

**Step 3 — Design reference** — n/a; T-004 is a server/API infrastructure task and adds no UI.

**Step 4 — Plan and test plan** — Recorded above before implementation. The three mechanical adaptations are listed under Questions for architect ratification.

**Summary** — Added the `ritma` serve/migrate/version/healthcheck commands, explicit placeholders for later commands, structured telemetry, graceful SIGTERM/SIGINT shutdown, and public liveness/readiness endpoints. The router generates a UUIDv7 request ID, correlates problem responses, avoids query-string logging, and reports database/migration readiness within bounded timeouts.

**Spec read** — `spec/02-architecture.md` §4; `spec/07-api-conventions.md` API-14; `spec/10-security-privacy.md` SEC-21; `spec/11-ops.md` §§2–3; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/5/8; `spec/19-error-handling.md` ERR-42/§10; `spec/23-contracts-and-canonical-forms.md` CAN-30; `spec/26-containers-and-caching.md` CTR-05/08.

**Files** —
- Modified `crates/eq-server/src/main.rs`; created `crates/eq-server/src/telemetry.rs`.
- Modified `crates/eq-api/src/lib.rs`; created `crates/eq-api/src/health.rs` and `crates/eq-api/src/middleware/{mod.rs,request_id.rs}`.
- Modified `crates/eq-core/src/db.rs` to grant `eq_app` read-only access to SQLx migration version metadata after the owner applies migrations.
- Modified workspace/API/server Cargo manifests and `Cargo.lock` for direct runtime and test dependencies, including tower-http panic catching.
- Created `crates/eq-api/tests/health.rs`; expanded `crates/eq-server/tests/cli.rs`.
- Updated this task's file list, tests, Questions and Report; regenerated `tasks/INDEX.md` through `scripts/tasks-index.sh`.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `healthz_ok` | Passed; HTTP 200, canonical body and server UUIDv7 `X-Request-Id`; forged client ID ignored. |
| AC2 | `readyz_db_down` | Passed; 503 `db_unreachable` within one second. |
| AC3 | `migrate_applies` | Passed; CLI applied migration 0001 to a fresh PostgreSQL 16 container. |
| AC3b | `healthcheck_exit_codes` | Passed; exit 0 with a loopback health server and exit 1 with no listener. |
| AC4 | `problem_has_request_id` | Passed; problem body ID equals the response header. |
| Readiness interface | `readyz_migrations_pending`, `readyz_ready` | Passed; a fresh database reports pending, and a migrated database accessed as `eq_app` reports ready. |
| CLI metadata/placeholders | `prints_version`, `version_subcommand_reports_build_metadata`, `unimplemented_subcommand_exits_2` | Passed. |

**Checks run** —
- ✅ `cargo fmt --all --check`.
- ✅ `cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `cargo test --workspace` — all workspace unit, API, CLI, Testcontainers integration and doc tests passed.
- ✅ `cargo audit` — no vulnerable dependencies.
- ✅ `cargo deny check advisories` — `advisories ok` (the repository does not yet have a deny config, so Cargo Deny used its default config).
- ✅ `scripts/check-deps.sh`.
- ✅ `scripts/tasks-index.sh --check`, `git diff --check`, and `python3 scripts/erd.py --check`.
- ✅ `gitleaks detect --redact --no-banner` — no leaks found.
- Not run: `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh` do not exist yet; T-006 introduces them.
- Not run: `make dev` and `make e2e`; there is no Makefile/runtime E2E harness yet. The task's equivalent direct `cargo run -p eq-server -- serve` smoke test and real HTTP requests were run successfully. M0 phase E2E is scheduled in T-008 after T-005/006/007.
- Not run: `cargo sqlx prepare --workspace`; T-004 added no `sqlx::query!` macros (its DB reads are dynamic SQL). No migration file changed, so `scripts/erd.py --migrations` is n/a for T-004.
- Not run: web checks; T-004 changes no web files.
- Not run: hosted GitHub Actions; unavailable locally.

**Manual checks** — Started the final `ritma serve` binary with the local PostgreSQL 16 dev database on `127.0.0.1:18081`, then sent real requests. `GET /healthz` returned `HTTP/1.1 200 OK`, `X-Request-Id: <UUIDv7>`, and `{"status":"ok","version":"0.1.0"}`. `GET /readyz` returned `HTTP/1.1 200 OK` and `{"status":"ready"}`. `ritma healthcheck` exited 0. The `ritma version` subcommand printed version, `commit=unknown` (no build commit variable is configured), and enabled feature names. Sent SIGTERM; the server logged `graceful shutdown started` and exited.

**Debug log** —
- Initial router compilation rejected a hidden TraceLayer service bound; inlined the configured TraceLayer into router assembly and rechecked.
- Initial CLI compilation found the serve future needed `IntoFuture`; converted it before polling. Test compile then found a helper-name collision and Tokio server use requiring the `Serve` future to be awaited in a task; corrected both.
- The first readiness test helper used the supplied database URL in Config but accidentally connected its pool to a fixed closed port; changed it to use the supplied URL and reran all readiness tests.
- The first healthcheck test blocked the single-thread Tokio test runtime while waiting for a child process; moved the process wait to `spawn_blocking`. The server-up/down test then passed.
- A real local smoke test initially returned `migrations_pending` after the migration succeeded. Inspection showed `_sqlx_migrations` existed but was unreadable to `eq_app`, because SQLx created it before migration 0001's default privileges. Added the minimal SELECT grant after migrations and added fresh/current database tests; both states and the subsequent real smoke test passed.
- Clippy rejected direct `std::env::var` use in telemetry (`disallowed_methods`). Initialized tracing from the validated Config object and reran the full Rust checks.

**Performance** — No endpoint performance baseline was requested. Readiness enforces the specified 500 ms DB ping and migration-query bounds, caches migration state for 10 seconds, and the DB-down acceptance test confirms its response stays below one second.

**Security** — Triggers T1 and T4: public liveness/readiness endpoints and DB/migration-role handling. Only the specified status/reason/version values are exposed. Request IDs are generated server-side as UUIDv7 and client IDs are ignored (CAN-30). Trace logs include method, matched path, status, latency and request ID, never query strings or bodies. DB connectivity and migration checks are bounded. `eq_app` receives only SELECT on the SQLx migration metadata table, consistent with the specified DML-only role; tenant tables and role bypass settings are unchanged. Healthcheck disables proxies and calls loopback only. Panic responses use the registered `internal` problem renderer and log once through `AppError::Internal`. SBD-81 auth, DTO, PII, output-encoding and external-call controls do not apply to these two public health routes; the abuse tests cover forged request IDs and DB/migration states. `gitleaks` passed.

**Deviations** — RATIFY (matching Questions): (1) Added the missing manifest and test paths named by the task's interfaces/AC. (2) Granted `eq_app` SELECT on SQLx migration metadata because the bootstrap default privileges are applied after SQLx creates that metadata table; required to make OPS-06 readiness work without granting access to domain data. (3) Initialized tracing after Config parsing, using its validated fields, because project lint OPS-01 forbids reading environment variables directly outside `eq_core::config`. Decision register references: TD-007, TD-008, TD-009 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).

**Dependencies affected** — T-005 can add the SPA to the existing router and share request-ID/trace middleware. T-006 should exercise the provided serve, migrate and healthcheck CLI through its Makefile/CI. T-007 can rely on `run_migrations` making migration status readable by `eq_app`. T-008 owns the phase E2E smoke test.

**Rollback plan** — Reverting this task removes the new CLI/routes/middleware and the post-migration grant. The already applied `GRANT SELECT ON _sqlx_migrations TO eq_app` persists after a code revert; revoke it with `REVOKE SELECT ON TABLE _sqlx_migrations FROM eq_app` if removing the feature, noting a later startup will reapply it. No application tables or tenant data were changed.

**Follow-ups** — T-115 should replace the no-op security-header stub with the SEC-20 policy. T-005 should preserve the request ID middleware around the embedded SPA and API. T-006 should add local dev and e2e targets; T-008 should run the M0 Playwright smoke test.

**Step 5 — Implement** — Implemented the specified CLI, server, health routes, request-ID and trace middleware, timeout limits, panic conversion, and signal shutdown; no feature beyond Scope → In.

**Step 6 — Run locally** — Ran the final binary with the local PostgreSQL 16 server and exercised real HTTP liveness/readiness requests, healthcheck, version and SIGTERM shutdown as described above.

**Step 7 — Debug loop** — Addressed compile/test failures and the live readiness privilege issue; each fix was followed by a rerun. Final complete suite is green.

**Step 8 — Self code review** — Reviewed the diff against CQ-16 §9, CA-01/30 and ERR-42. No query string/body logging, credentials in logs, unregistered error code, `#[allow]`, `unwrap`, `todo!` or unrelated refactor was introduced. The Config Debug surface remains redacted; AppState Debug also redacts Config.

**Step 9 — Optimize** — No query/list hot path introduced. Timeout and cache choices follow task-defined technical bounds; no speculative optimization was added.

**Step 10 — Security audit** — Completed the applicable T1/T4 SBD-81 review, forged-request-id test, migration state/role tests, bounded loopback healthcheck and secret scan.

**Step 11 — Revise and verify** — After fixing compile, test, lint and readiness findings, reran the full Rust workspace format/clippy/test commands, audit tools, available guards, ERD check and gitleaks. Missing pre-T-006 scripts and E2E harness are accurately marked not run.

**Step 12 — Report, commit** — Report completed; the task status will be `review` in the task commit and `tasks/INDEX.md` is regenerated through the script.

## Questions
