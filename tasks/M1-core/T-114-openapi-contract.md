---
id: T-114
title: Merge per-module OpenAPI, add `ritma openapi` output, the committed web/openapi.json and the contract-drift CI job
milestone: M1
status: review
depends_on: [T-004, T-005, T-006, T-102a, T-113b]
spec_refs: [API-02, MOD-04, TST-10, API-01, API-03, API-13, CAN-03, CAN-20, VAL-20]
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

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Field-kind components (`DisplayName`, `Email`, …) from T-102a are registered in the merged document with their `x-sanitize` and `x-field-kind` extensions; DTO names follow CAN-20 (`Xxx`, `XxxSummary`, `XxxCreate`, `XxxPatch`, action bodies, `Page_Xxx`).
  - The drift check also covers the generated client files of T-306a once they exist (`web/src/api/*.gen.ts`).

## Scope
### In
- `eq-api::openapi::build() -> utoipa::openapi::OpenApi`, the CLI command, the committed `web/openapi.json` and `web/src/api/schema.d.ts`, and the CI job.
- A test that every route registered in the axum router has an OpenAPI operation (walk both lists).
- ERR-21: merge every compiled crate's `errors::CODES` into the document as the top-level extension `x-error-codes: [{code, status, title}]`, sorted by code. Add a test `error_codes_unique_and_valid` that fails on a duplicate code or a code failing `validate_code`.

### Out
- The SPA client wrapper (M3).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-api/src/openapi.rs` | create | Build the merged OpenAPI document, schemas, security schemes and error registry. |
| `crates/eq-api/src/lib.rs` | modify | Export the OpenAPI builder. |
| `crates/eq-api/src/modules.rs` | modify | Share route composition between runtime and OpenAPI generation. |
| `crates/eq-api/src/health.rs` | modify | Document and share the health/readiness route declarations with the runtime. |
| `crates/eq-api/tests/openapi.rs` | test | Task-listed verification file. |
| `crates/eq-identity/src/api_people.rs` | verify | Temporary DTO-only change used to prove generated-contract drift detection; restored before commit. |
| `crates/eq-server/src/main.rs` | modify | Implement `ritma openapi` output. |
| `crates/eq-core/src/errors.rs` | modify | Expose the Core `errors::CODES` registry required by ERR-21. |
| `crates/eq-identity/src/errors.rs` | modify | Expose the Identity `errors::CODES` registry required by ERR-21. |
| `crates/eq-audit/src/errors.rs` and `crates/eq-audit/src/lib.rs` | create/modify | Expose the compiled Audit crate's empty code registry. |
| `crates/eq-scheduling/src/errors.rs` and `crates/eq-scheduling/src/lib.rs` | create/modify | Expose the compiled Scheduling crate's empty code registry. |
| `crates/eq-enrollment/src/errors.rs` and `crates/eq-enrollment/src/lib.rs` | create/modify | Expose the compiled Enrollment crate's empty code registry. |
| `crates/eq-billing/src/errors.rs` and `crates/eq-billing/src/lib.rs` | create/modify | Expose the compiled Billing crate's empty code registry. |
| `crates/eq-attendance/src/errors.rs` and `crates/eq-attendance/src/lib.rs` | create/modify | Expose the compiled Attendance crate's empty code registry. |
| `crates/eq-rental/src/errors.rs` and `crates/eq-rental/src/lib.rs` | create/modify | Expose the compiled Rental crate's empty code registry. |
| `crates/eq-notify/src/errors.rs` and `crates/eq-notify/src/lib.rs` | create/modify | Expose the compiled Notify crate's empty code registry. |
| `crates/eq-reports/src/errors.rs` and `crates/eq-reports/src/lib.rs` | create/modify | Expose the compiled Reports crate's empty code registry. |
| `crates/eq-feedback/src/errors.rs` and `crates/eq-feedback/src/lib.rs` | create/modify | Expose the compiled Feedback crate's empty code registry. |
| `Makefile` | modify | Make `api` generate the checked OpenAPI and TypeScript contract. |
| `web/package.json` | modify | Add the pinned OpenAPI TypeScript generator and `gen:api` script. |
| `web/pnpm-lock.yaml` | modify | Pin the OpenAPI TypeScript generator dependency. |
| `.github/workflows/ci.yml` | modify | Normalize generated JSON formatting before the existing contract-drift check. |
| `tasks/DECISIONS.md` | modify | Record mechanical T-114 decisions TD-194–TD-196. |
| `tasks/INDEX.md` | regenerate | Keep the task index in sync via `scripts/tasks-index.sh`. |
| `web/openapi.json` | create or modify | Path named in this task's contract. |
| `web/src/api/schema.d.ts` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. `openapi` output is byte-identical across two runs. → `openapi::tests::deterministic`
- AC2. Every axum route (method + path) has an operation, and vice versa. → `routes_match_openapi`
- AC3. The CI `contract` job passes on a clean tree and fails after changing a DTO field without regenerating (documented in the Report with the failing output). → manual check

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/openapi.rs` | `deterministic`, `routes_match_openapi`, `problem_schema_present` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass; local contract job equivalent is green (hosted GitHub Actions not run)
- [x] Commit `build(api): OpenAPI contract and drift check [T-114]`, signed off

## Report

**Plan** — (1) Confirm the dependency reports and route/error/schema contracts, and record the task-file path drift → verify the dependency code and exact route composition. (2) Factor route composition so the runtime router and OpenAPI builder use the same compiled module routes → verify the route parity test. (3) Build the OpenAPI 3.1 document, Problem schema, security schemes, field-kind schemas and sorted `x-error-codes` → verify focused OpenAPI tests. (4) Implement the CLI and Make target → verify repeated CLI output is byte-identical. (5) Add the pinned TypeScript generator and generate both committed contract files → verify `make api` and web typecheck. (6) Exercise contract drift with a temporary DTO-only change, then restore it → verify the contract diff fails before regeneration and passes after regeneration. (7) Run the task checks, review the full diff and record results → verify task index and task lint before the single signed-off commit.

**Assumptions** — The shipped `OpenApiRouter` route declarations are the source of truth; refactoring their shared composition preserves runtime behavior. ERR-30's listed Problem extension members, including `missing`, take precedence over the task's abbreviated list. To avoid inventing route-specific error codes, the merged OpenAPI document will advertise one shared `default` `application/problem+json` response for each operation. `errors::CODES` is an ERR-21 per-crate contract; crates with no registered module errors expose an empty registry. Decisions TD-194–TD-196 document the mechanical task/spec alignment.

**Test plan** — Given the same compiled module routes, serialize the OpenAPI document twice and assert byte equality (`openapi::tests::deterministic`). Given runtime route composition and the generated paths, compare every method/path in both directions (`routes_match_openapi`). Given the built document, assert Problem RFC 9457 fields and optional extensions, its problem media type, bearer/cookie schemes, server prefix and registered field-kind validation extensions (`problem_schema_present`). Given all compiled crate registries, reject duplicate or malformed codes using `validate_code` (`error_codes_unique_and_valid`). Given an unregenerated contract after a temporary DTO field change, require the documented CI diff command to fail; after restoration and regeneration, require it to pass (AC3 manual check). Edge cases include empty module error registries, optional compile-time modules and route paths under `/api/v1`; error behavior is a shared default Problem response without inventing per-route codes. No database, money, tenant state or new security permission is involved.

**Summary** — `ritma openapi` now emits a deterministic OpenAPI 3.1 contract built from the same composed module routes used at runtime. The document includes field-kind schemas, security schemes, RFC 9457 Problem schemas and the registered error-code extension; the web TypeScript schema is generated and checked for drift.

**Spec read** — Task-inline API-02, MOD-04, TST-10, API-01/03/13, CAN-03/20 and VAL-20; `spec/16-code-quality.md` §§9,12; `spec/17-clean-architecture.md` §§1,4,7; `spec/18-security-by-design.md` §5; `spec/19-error-handling.md` §§3,4,10; `spec/22-validation-and-sanitization.md` §4; `spec/23-contracts-and-canonical-forms.md` §§3,10. UI design reference: n/a. User journeys: n/a.

**Files** —
- `.github/workflows/ci.yml` — normalize OpenAPI JSON and compare regenerated OpenAPI/TypeScript outputs.
- `Makefile` — add the API contract generation target.
- `crates/eq-api/src/health.rs` — share and document health routes and response schemas.
- `crates/eq-api/src/lib.rs` — export `openapi`.
- `crates/eq-api/src/modules.rs` — share compiled module route composition.
- `crates/eq-api/src/openapi.rs` — build deterministic OpenAPI, schemas, security definitions and error registry.
- `crates/eq-api/tests/openapi.rs` — route parity, Problem schema, code registry and determinism tests.
- `crates/eq-attendance/src/errors.rs`, `crates/eq-attendance/src/lib.rs` — expose the empty error registry.
- `crates/eq-audit/src/errors.rs`, `crates/eq-audit/src/lib.rs` — expose the empty error registry.
- `crates/eq-billing/src/errors.rs`, `crates/eq-billing/src/lib.rs` — expose the empty error registry.
- `crates/eq-core/src/errors.rs` — expose Core's code registry.
- `crates/eq-enrollment/src/errors.rs`, `crates/eq-enrollment/src/lib.rs` — expose the empty error registry.
- `crates/eq-feedback/src/errors.rs`, `crates/eq-feedback/src/lib.rs` — expose the empty error registry.
- `crates/eq-identity/src/errors.rs` — expose Identity's code registry.
- `crates/eq-notify/src/errors.rs`, `crates/eq-notify/src/lib.rs` — expose the empty error registry.
- `crates/eq-rental/src/errors.rs`, `crates/eq-rental/src/lib.rs` — expose the empty error registry.
- `crates/eq-reports/src/errors.rs`, `crates/eq-reports/src/lib.rs` — expose the empty error registry.
- `crates/eq-scheduling/src/errors.rs`, `crates/eq-scheduling/src/lib.rs` — expose the empty error registry.
- `crates/eq-server/src/main.rs` — implement the `openapi` CLI subcommand.
- `tasks/DECISIONS.md`, `tasks/INDEX.md`, `tasks/M1-core/T-114-openapi-contract.md` — decision records, regenerated index and this task Report.
- `web/openapi.json`, `web/src/api/schema.d.ts` — committed generated contract and TypeScript types.
- `web/package.json`, `web/pnpm-lock.yaml` — pin `openapi-typescript` and its generation script.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `openapi::tests::deterministic`; two CLI SHA-256 outputs | Passed; both CLI outputs were `c28ce2bf774c88a1ef97677f833777fa8f2811d5a5b86e7b1631bffd2b386c71`. |
| AC2 | `routes_match_openapi` | Passed; all route method/path pairs match in both directions. |
| AC3 | Temporary DTO-only schema change, then restore and regenerate | Passed: the CI generation step produced diffs in both committed contract files; after reverting the DTO probe, the same generation step passed `git diff --exit-code`. |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked -p eq-api --test openapi` — 4 passed.
- Not run: `cargo test --workspace` — this workspace suite uses Testcontainers; this task needs no database tests and the approved resource set keeps only the existing `ritma-db-1` container.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true make api` — CLI, frozen install, JSON formatting and TypeScript generation completed.
- ✅ `pnpm -C web lint`, `pnpm -C web typecheck`, `pnpm -C web test` — passed; web test 1/1 and production bundle 98.93 KB gzip.
- Not run: `pnpm -C web i18n:check` — attempted, but `web/package.json` has no `i18n:check` script (`ERR_PNPM_RECURSIVE_EXEC_FIRST_FAIL: Command "web" not found`); this task adds no UI strings.
- ✅ `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check` — ERD matches DDL; migration check n/a because no migration changed.
- ✅ `cargo deny check` — advisories, bans, licenses and sources pass; existing duplicate-`base64` warning remains unchanged.
- Not run: `cargo sqlx prepare --workspace` — no SQL query or migration changed.
- Not run: hosted GitHub Actions contract job — local equivalent is exercised under Manual checks.
- Not run: `make dev` and `make e2e` — no HTTP endpoint or UI journey behavior changed; E2E would require a separate database/container outside the approved task resource set.
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, and `git diff --check`.
- ✅ Staged `gitleaks protect --staged --redact` — no leaks found.
- ✅ AC3 contract-drift probe — temporary `#[schema(example = "test-value")]` on `CreatePersonBody.display_name` changed both generated files and made `git diff --exit-code -- web/openapi.json web/src/api` fail; after removing the attribute, the exact generator/formatter/client-generation sequence passed the same diff check.

**Manual checks** — `make api` executed `ritma openapi`, formatted `web/openapi.json`, and generated `web/src/api/schema.d.ts`; two independent CLI invocations produced identical SHA-256 hashes. The temporary DTO probe changed both generated files, the contract diff rejected the stale files, and after restoring the DTO the contract generation and diff passed. No database was needed; no HTTP endpoint was introduced, so a dev server request was not applicable.

**Debug log** — Initial compile review found an invalid `Extensions` import, use of `.fold` on a slice, an `Info` generic mismatch and a missing `PartialSchema` import; these were corrected and the full workspace Clippy check and focused OpenAPI tests now pass. The first web lint found that Biome normalized generated JSON differently; the Make and CI generation steps now apply the same formatter before drift comparison, and final web lint passes.

**Performance** — Vite production build reports 98.93 KB gzip JavaScript, below FE-02's 200 KB limit. No endpoint latency or query optimization applies.

**Security** — No SBD-30 triggers. OpenAPI security schemes mirror the existing bearer and refresh-cookie auth contract; route composition exposes only compiled routes. JSON output is generated through serde and contains no secret values. Staged Gitleaks scan passed with no leaks.

**Deviations** — RATIFY: TD-063, dependency references now target the final split part that provides the referenced contract.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.
**Deviations** — RATIFY: TD-194–TD-197, see the matching self-resolution/decision entries under Questions and `tasks/DECISIONS.md`.

**Dependencies affected** — T-306a consumes the generated API schema; future module tasks must populate their own `errors::CODES` when they register error codes. T-116a can add its fuzz script to the existing conditional contract job.

**Rollback plan** — Revert the single T-114 commit. No database migration or persistent runtime data was added.

**Follow-ups** — Architect ratification of TD-194–TD-197; maintainer review should decide whether CQ-101 warrants a follow-up split/refactor. Owning module tasks should register their error codes and document operation-specific failure responses when those contracts are defined. The SPA client wrapper remains out of scope per this task.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** this task depended on the original parent contract after its CQ-101 split → depend on the final implementation part that provides the contract, because the parent now owns only the split map.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 26, needs architect ratification; TD-194):** Scope requires the OpenAPI builder, CLI, route integration, package generator and Make target, but the Files table names only tests and generated outputs → add only those direct implementation paths to this task's Files table because they are required by its existing Scope and acceptance criteria.
- **Self-resolved (implementer, run 26, needs architect ratification; TD-195):** ERR-21 requires every compiled crate to expose `errors::CODES`, while only Core and Identity currently publish registries → expose the existing Core and Identity registries at that path and add empty registries to compiled modules with no registered codes; this adds no error code or wire behavior.
- **Self-resolved (implementer, run 26, needs architect ratification; TD-196):** the task's abbreviated Problem field list omits `missing` defined by ERR-30, and existing route declarations document success only → include every ERR-30 Problem member and add a common default problem+json response in the generated OpenAPI for each operation, without inventing per-route error codes.
- **Self-resolved (implementer, run 26, needs architect ratification; TD-197):** the required T-114 contract exceeds CQ-101's approximate review-size guide → preserve the task's acceptance criteria and report 929 changed lines excluding generated OpenAPI/TypeScript and the lockfile, because splitting requires edits to other task contracts or omission of required behavior.

## Review
n/a — awaiting maintainer review.
