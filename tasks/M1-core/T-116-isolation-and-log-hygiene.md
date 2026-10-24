---
id: T-116
title: Add the tenant-isolation test suite (all tenant tables, automatic) and the log-hygiene layer and tests
milestone: M1
status: todo
depends_on: [T-115]
spec_refs: [TST-06, TST-16, SEC-05, SEC-21]
estimated_size: S
---

## Goal
It is proven, and stays proven as tables are added, that one tenant can never read or write another tenant's rows. PII never reaches logs.

## Context
- TST-06: two tenants are seeded. For **every** table that has a `tenant_id` column and RLS enabled (discovered from `pg_class`/`information_schema.columns`, **not** a hand-written list):
  1. As tenant A (via `db::begin`), `SELECT count(*) FROM t WHERE tenant_id = B` = 0.
  2. With `app.tenant_id` unset (a plain tx as `eq_app`), `SELECT count(*) FROM t` = 0.
  3. `INSERT` of a row with B's `tenant_id` under A's setting fails (the RLS `WITH CHECK` violation). Generate a minimal insert per table by introspecting NOT NULL columns without defaults, filling typed dummy values. Tables whose insert generation fails go into an explicit allow-list file `crates/eq-testkit/isolation-skip.txt`, **each line with a reason**. The test fails if a listed table no longer exists.
  4. Assert that the expected system tables (outbox, event_receipts, jobs, cron_state, tenant_keys, audit.checkpoints, tenants) are the **only** tenant-ish tables without RLS.
- SEC-21 / TST-16:
  - A `tracing` layer `PiiGuardLayer` inspects event and span field **names**: `email, phone, password, token, legal_name, address, birth_date, refresh_token, access_token, secret`.
  - In `cfg(debug_assertions)` it panics. In release it replaces the value with `"[redacted]"`. Installed in `eq-server` telemetry init.
  - Request bodies are never logged (assert the TraceLayer config logs only method, path template, status and latency).
- Log capture test: run the identity API flows (login, refresh, invite, forgot, people create with email and phone) with a capture subscriber, then assert no captured line contains the seeded email, phone, password or raw tokens.

## Scope
### In
- `crates/eq-testkit/src/isolation.rs` (reusable `assert_tenant_isolation(pool)`), the test `crates/eq-server/tests/tenant_isolation.rs`, `PiiGuardLayer` in `eq-core::telemetry`, and `crates/eq-server/tests/log_hygiene.rs`.

### Out
- Platform-role tests (M9).

## Acceptance criteria
- AC1. `tenant_isolation::all_tables_isolated` passes and prints the table count (≥ 20 at this point).
- AC2. Creating a throwaway table with `tenant_id` and **no** RLS in a test migration makes `rls_coverage_complete` fail (demonstrated in a unit test using a temp schema). → `detects_missing_rls`
- AC3. `log_hygiene::no_pii_in_logs` passes. A debug-build `tracing::info!(email = %x)` panics. → `pii_field_panics_in_debug`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/tenant_isolation.rs` | `all_tables_isolated`, `rls_coverage_complete`, `detects_missing_rls` |
| `crates/eq-server/tests/log_hygiene.rs` | `no_pii_in_logs`, `pii_field_panics_in_debug` |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `test(core): tenant isolation and log hygiene [T-116]`, signed off

## Report

## Questions
