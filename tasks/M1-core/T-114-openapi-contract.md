---
id: T-114
title: Merge per-module OpenAPI, add `ritma openapi` output, the committed web/openapi.json and the contract-drift CI job
milestone: M1
status: todo
depends_on: [T-113]
spec_refs: [API-02, MOD-04, TST-10, API-01, API-03, API-13]
estimated_size: S
---

## Goal
One OpenAPI 3.1 document describes every compiled endpoint and DTO, including problem responses. CI fails whenever code and the committed contract drift.

## Context
- MOD-04 step 5: every compiled module's routes appear in OpenAPI, whatever their runtime state. Build it with `utoipa-axum` `OpenApiRouter`, merging core + each module's `routes()`.
- Document-level:
  - `info.title = brand::NAME + " API"`, `info.version = CARGO_PKG_VERSION`, `info.license = {name: "AGPL-3.0-only"}`.
  - Server `/api/v1`.
  - Security scheme `bearer` (JWT) and `refreshCookie` (apiKey in cookie `eq_rt`).
- `components.schemas.Problem` = RFC 9457 + `code`, `request_id`, and optional `errors`, `conflicts`, `alternatives`, `perm`, `module`, `rule`, `until`. Each handler declares its problem responses (`(status = 409, body = Problem, description = "slot_taken")`).
- Output must be deterministic (sorted keys) for a stable diff. Serialize with `serde_json::to_string_pretty` on a `BTreeMap`-backed value.
- `ritma openapi` CLI subcommand (the CLI skeleton exists from T-004) prints to stdout.
- CI job `contract` (in `.github/workflows/ci.yml`, created in T-006): `cargo run -p eq-server -- openapi > /tmp/openapi.json && diff -u web/openapi.json /tmp/openapi.json`, then `pnpm -C web gen:api && git diff --exit-code web/src/api/schema.d.ts && pnpm -C web typecheck`.
- `web` script `gen:api` = `openapi-typescript openapi.json -o src/api/schema.d.ts` (the web scaffold from T-005 already has the dependency; add it if missing).

## Scope
### In
- `eq-api::openapi::build() -> utoipa::openapi::OpenApi`, the CLI command, the committed `web/openapi.json` and `web/src/api/schema.d.ts`, and the CI job.
- A test that every route registered in the axum router has an OpenAPI operation (walk both lists).

### Out
- The SPA client wrapper (M3).

## Acceptance criteria
- AC1. `openapi` output is byte-identical across two runs. → `openapi::tests::deterministic`
- AC2. Every axum route (method + path) has an operation, and vice versa. → `routes_match_openapi`
- AC3. The CI `contract` job passes on a clean tree and fails after changing a DTO field without regenerating (documented in the Report with the failing output). → manual check

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/openapi.rs` | `deterministic`, `routes_match_openapi`, `problem_schema_present` |

## Definition of done
- [ ] Rust checks pass; contract job green
- [ ] Commit `build(api): OpenAPI contract and drift check [T-114]`, signed off

## Report

## Questions
