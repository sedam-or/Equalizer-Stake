---
id: T-004
title: Build the ritma CLI and HTTP skeleton (serve, migrate, version, healthz/readyz, tracing, graceful shutdown)
milestone: M0
status: todo
depends_on: [T-003]
spec_refs: [ARC-04, OPS-02, OPS-03, OPS-06, API-14, CAN-30]
estimated_size: M
---

## Goal
`ritma serve` loads the config, runs migrations, and serves `/healthz` and `/readyz` through the standard tower layer stack, with request ids and structured logs. SIGTERM shuts it down gracefully.

## Context
- OPS-02 CLI: `serve` (default), `migrate`, `openapi`, `backup`, `restore`, `doctor`, `create-owner`, `rotate-keys`, `seed-demo`, `version`. This task implements `serve`, `migrate` and `version`. The others print "not implemented yet" and exit with code 2.
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
- AC4. A Problem body produced inside a request contains the same `request_id` as the header. → `problem_has_request_id` (add a test-only route)

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/health.rs` | `healthz_ok`, `readyz_db_down`, `problem_has_request_id` |
| `crates/eq-server/tests/cli.rs` | `migrate_applies`, `unimplemented_subcommand_exits_2` |

## Definition of done
- [ ] Rust checks pass
- [ ] Manual: `DATABASE_URL=… RITMA_SECRET_KEY=… RITMA_PUBLIC_URL=http://localhost:8080 cargo run -p eq-server -- serve`, then `curl localhost:8080/healthz` (paste the output in the Report)
- [ ] Commit `feat(server): cli and http skeleton [T-004]`, signed off

## Report

## Questions
