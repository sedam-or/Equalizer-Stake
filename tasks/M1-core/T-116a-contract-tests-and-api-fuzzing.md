---
id: T-116a
title: Add contract tests (API fuzzing with Schemathesis, event schema snapshots, field-kind coverage)
milestone: M1
status: todo
depends_on: [T-102a, T-107c, T-114, T-115]
spec_refs: [CAN-03, CAN-60, CAN-61, CAN-95, CAN-96, CAN-97, VAL-02, VAL-51, VAL-53, TST-10, TST-17, TST-18, ERR-21, OSS-20, OSS-21]
estimated_size: M
---

## Goal
The published contracts can't drift or crash silently. CI fuzzes every API endpoint from the OpenAPI document and fails on any 5xx, unregistered error code or response that doesn't match its schema; event payload schemas are snapshot-tested; and every request DTO field is proven to use a field kind.

## Context
- CAN-96 / TST-18: Schemathesis runs against a seeded server in CI and asserts: no 5xx; every error is `application/problem+json` with a `code` listed in `x-error-codes`; every 2xx body validates against its response schema; unknown fields are rejected.
- CAN-60/61/97: event payload structs derive `JsonSchema`. Their schemas are written to `contracts/events/<event_type>.schema.json` and compared in a test. A diff means either an additive change (same version) or a new `.vN` (CAN-61); the PR must say which.
- VAL-02: every request DTO field that carries a value references a field kind (`$ref` to a component with `x-field-kind`) or is a closed enum, boolean or nested DTO. Plain `{"type":"string"}` without bounds in a request schema is a bug.
- The CI job `contract` in `.github/workflows/ci.yml` already runs `scripts/ci/api-fuzz.sh` when the file exists (OSS-21). This task creates it.

## Scope
### In
- `crates/eq-testkit/src/bin/seed_ci.rs`: creates a fresh database from migrations, runs the real setup service with the `music_school` preset, creates one person per role, and prints JSON `{ "base_url", "tokens": { "owner", "admin", "instructor", "guardian", "student" } }` to stdout. Refuses to run unless `RITMA_ENV=ci` (SBD-61).
- `scripts/ci/api-fuzz.sh`: starts Postgres (`docker compose -f deploy/dev/compose.yaml up -d db`), builds and starts `ritma serve` in the background with CI env, waits for `/readyz`, runs `seed_ci`, then runs Schemathesis (via `uvx schemathesis`) against `web/openapi.json` once **unauthenticated** and once per role token, with all checks enabled, a fixed seed, and `--max-examples` from an env var (default 25). Collects the report as a CI artifact. Exits non-zero on any failure.
- `crates/eq-api/tests/contract_error_codes.rs`: every `code` seen in Schemathesis responses is in `x-error-codes` (Schemathesis custom check, written as a small Python hook file `scripts/ci/schemathesis_hooks.py` registered with `--hooks`, or as a Rust test that replays the recorded cassette; pick the simpler and document it).
- `crates/eq-api/tests/contract_events.rs`: iterates the event registry (all compiled modules), writes `contracts/events/*.schema.json` when `UPDATE_CONTRACTS=1`, and otherwise fails on any difference.
- `crates/eq-api/tests/contract_fields.rs`: walks every request body and query schema in the generated OpenAPI and fails on a value property that has no `x-field-kind`, enum, boolean, `$ref` to a DTO, or documented exemption list (`contracts/field-exemptions.txt`, each line with a reason).

### Out
- Web-side parity runner (T-306a).
- Load testing (k6, M7).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-testkit/src/bin/seed_ci.rs` | create | CI seed |
| `scripts/ci/api-fuzz.sh`, `scripts/ci/schemathesis_hooks.py` | create | fuzzing |
| `crates/eq-api/tests/contract_events.rs`, `contract_fields.rs`, `contract_error_codes.rs` | create | contract tests |
| `contracts/events/*.schema.json`, `contracts/field-exemptions.txt` | create | snapshots |

## Interfaces
- `seed_ci` stdout JSON as above. Env: `DATABASE_URL`, `RITMA_SECRET_KEY` (a fixed `test-secret-…` value), `RITMA_PUBLIC_URL=http://localhost:8080`, `RITMA_ENV=ci`.
- `scripts/ci/api-fuzz.sh` env: `FUZZ_MAX_EXAMPLES` (default 25), `FUZZ_SEED` (default 42).

## Business rules
None.

## Security
- Triggers: T1 (fuzzes public and cookie endpoints), T4 (tokens for CI roles).
- Threats → controls: **I**nformation disclosure by a crash path → fuzzing finds 5xx before release; **E**levation via a role-confused endpoint → per-role runs assert that forbidden responses use registered codes. CI tokens are short-lived, generated per run, never committed (SBD-51).
- Abuse-case acceptance criteria: a deliberately broken handler (test-only route behind `cfg(test)` is not enough; use a feature `fuzz-canary` compiled only in this test) that panics on a long string makes `api-fuzz.sh` fail → described in the Report with the run link.

## Implementation notes
- Keep the Schemathesis version pinned in the script (`uvx schemathesis@<version>`), and print it.
- Stateful sequences (Schemathesis links) are optional; start with per-operation fuzzing.
- Make the fuzz run deterministic (fixed seed) so failures reproduce locally with `scripts/ci/api-fuzz.sh`.

## Acceptance criteria
- AC1. `scripts/ci/api-fuzz.sh` passes locally and in the `contract` CI job. → `manual: CI run link in Report`
- AC2. Changing an event payload field name without bumping its version fails `contract_events`. → `contract_events::detects_payload_change`
- AC3. Adding a request field typed as plain `String` fails `contract_fields`. → `contract_fields::rejects_unbounded_string`
- AC4. A response with an unregistered `code` fails the hook. → `manual + hook unit test`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/contract_events.rs` | `event_schemas_match_snapshots`, `detects_payload_change` |
| `crates/eq-api/tests/contract_fields.rs` | `every_request_field_has_a_kind`, `rejects_unbounded_string` |
| `crates/eq-api/tests/contract_error_codes.rs` | `codes_in_responses_are_registered` |

## Definition of done
- [ ] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` pass
- [ ] `scripts/ci/api-fuzz.sh` passes locally and in CI
- [ ] Guard scripts pass
- [ ] Commit `test(api): add contract tests and API fuzzing [T-116a]`, signed off

## Report

## Questions

## Review
