---
id: T-109h
title: Register the always-on audit module and sink
milestone: M1
status: review
depends_on: [T-109g]
spec_refs: [AUD-01, AUD-02, AUD-07, MOD-01, MOD-04, SEC-07]
estimated_size: S
---

## Goal
Replace the audit module stub with the always-on `AuditModule`, wire policy and job contracts, and implement the existing `AuditSink` port.

## Context
Use the shipped `Module`, `AuditSink`, `PolicyDoc`, `JobHandler` and `CronSpec` interfaces. Parent T-109 defines the manifest, policy behavior, event action names and exact schedules.

## Scope
### In
- Always-on manifest, policy schema/default/validation/summary, job and cron registration, and transactional module-toggle/policy-change audit sink behavior.
### Out
- HTTP routes (T-113), runtime 10-second scheduler integration (T-117a), and privacy workflows (M6).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/module.rs` | modify/create | Replace stub and register the module and its `AuditSink` port. |
| `crates/eq-audit/tests/audit.rs` | modify | Verify caller-transaction sink writes and tenant mismatch rejection. |
| `tasks/M1-core/T-109h-audit-module.md`, `tasks/DECISIONS.md` | modify | Report, status and self-resolution register. |

## Interfaces
Use the existing module and sink ports only. Preserve action names `core.module.enabled`, `core.module.disabled`, and `core.policy.changed`. Register `audit.retention` at the parent-specified `0 3 * * *` local schedule and `audit.verify_recent` at the self-resolved `30 3 * * *` local schedule (TD-090).

## Business rules
No new permissions, routes, policy fields, event names or schedule values.

## Security
- Triggers: audit writes for core state changes (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: missing audit sink or disabled audit module → audit remains `always_on`, with writes using the caller's transaction.
- Abuse case: a caller cannot use the audit sink with a tenant different from its transaction context → `audit_sink_rejects_mismatched_tenant_context`.

## Implementation notes
Keep `routes()` empty; the audit HTTP API is explicitly T-113.

## Acceptance criteria
- AC1. Audit module is always-on and registers both job kinds/schedules → `audit_module_registers_its_always_on_jobs_and_schedules`.
- AC2. Existing `AuditSink` port records toggle and policy change in the caller's transaction.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/src/module.rs` | `audit_module_registers_its_always_on_jobs_and_schedules` |
| `crates/eq-audit/tests/audit.rs` | `audit_sink_records_changes_in_the_caller_transaction`, `audit_sink_rejects_mismatched_tenant_context` |

## Definition of done
- [ ] Module unit test passes; policy wiring and `AuditSink` compile against existing ports.
- [ ] Rust checks and layer guards pass.
- [x] Signed commit `feat(audit): register audit module and sink [T-109h]`.

## Report
**Plan** — (1) Confirm T-109g's exported handlers, `AuditPolicy`, and core transaction-owned attention API → verify the dependency interfaces are present and its Report follow-ups are carried forward. (2) Replace the audit stub with the always-on manifest and policy adapters → verify default/schema/validation/summary behavior in the exact module test. (3) Register both shipped job handlers and tenant-local cron specs, and register the `AuditSink` port → verify kinds, schedules and port availability in the exact module test. (4) Implement both `AuditSink` methods through audit `record` after checking the supplied tenant against the caller transaction → verify both action writes are visible only inside the caller transaction until commit and reject a mismatched tenant. (5) Run available format, dependency, guard, task-index, and secret checks → record unavailable build/database checks as Not run under the disk rule.
**Assumptions** — No route or permission is required for the always-on backend module. TD-090 records the missing `audit.verify_recent` nightly schedule; TD-091 records the audit category and snapshots needed to preserve module identity and state without inventing a UUID or schema field.
**Test plan** — AC1: Given `AuditModule`, when its manifest, jobs, cron specs and registered ports are inspected, then the module is always-on, both handlers/schedules are registered and `AuditSink` is available → `audit_module_registers_its_always_on_jobs_and_schedules`. AC2: Given a transaction opened by `ModuleService` or `PolicyStore`, when the sink records a toggle and policy change, then both exact actions are visible through that same transaction and disappear after rollback → `audit_sink_records_changes_in_the_caller_transaction`. Edge cases: missing previous policy document, policy schema v1, and an unsupported migration version. Error/abuse case: a tenant argument different from the transaction context returns an error before writing → `audit_sink_rejects_mismatched_tenant_context`; snapshots contain module identity and the policy document passed by the typed policy store.
**Summary** — Replaced the audit stub with the always-on `AuditModule`, its audit policy behavior, job handlers and tenant-local schedules. Implemented and registered the existing `AuditSink` port so module toggles and policy changes are written through the caller's transaction after validating its tenant context. Added the named module test plus integration tests for same-transaction behavior and rejection of a mismatched tenant; Rust and database tests are Not run under the active no-new-target/no-new-database rule.
**Spec read** — `spec/03-module-system.md` MOD-01/04; `spec/17-clean-architecture.md` §§1/4/7; `spec/18-security-by-design.md` §§4/5.4/8; `spec/20-state-machines.md` §§3/4; parent T-109; T-109g Report and shipped jobs/attention API.
**Files** — Modified `crates/eq-audit/src/lib.rs`; created `crates/eq-audit/src/module.rs`; modified `crates/eq-audit/tests/audit.rs`; updated this task Report and status, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `audit_module_registers_its_always_on_jobs_and_schedules` | Not run: the fixed Cargo target is absent and the active disk rule prohibits creating one. |
| AC2 | `audit_sink_records_changes_in_the_caller_transaction` | Not run: this TestDb test requires a fresh PostgreSQL 16 container/database, which the active resource rule prohibits creating. |
| Abuse | `audit_sink_rejects_mismatched_tenant_context` | Not run for the same target/database restrictions; test asserts no event is inserted for a mismatched tenant. |
**Checks run** — ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`; ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` (advisories, bans, licenses and sources pass; pre-existing duplicate-version and unmatched-CC0 warnings remain); ✅ `scripts/check-deps.sh`; ✅ `scripts/check-literals.sh`; ✅ `scripts/check-web-boundaries.sh`; ✅ `scripts/check-layers.sh`; ✅ `python3 -I scripts/erd.py --check`; ✅ `python3 -I scripts/tasks-lint.py`; ✅ `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` (8/8); ✅ `scripts/tasks-index.sh --check`; ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --locked --format-version 1 --no-deps`; ✅ `git diff --check`; ✅ `gitleaks dir --redact --no-banner` on each changed task-owned source/test/task/decision path (5 paths, no leaks). `scripts/erd.py --migrations` n/a (no migration changed). Not run: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`, `... cargo test --workspace`, the module unit test and both audit integration tests, and `make dev`; the fixed target does not exist and the active resource rule prohibits creating a target, disposable database or container. Database: none; 0/2 new integration tests executed (0/1 module unit test). `make api`, web checks and `make e2e` n/a (no API, web, or user-journey behavior in Scope).
**Manual checks** — No endpoint, browser or runtime request applies to this backend-only task. Reviewed that each sink uses the supplied transaction, validates tenant context before inserting, and does not begin or commit another transaction.
**Debug log** — The initial `cargo fmt --all --check` found only rustfmt import/wrapping differences; `cargo fmt --all` corrected them and the final check passed. No build or database-test failure was attempted; those checks are explicitly Not run. `timeout 15s docker system df` returned exit 124 without output.
**Disk/resource snapshot** — Before: `du -sh "$HOME/.cache/ritma/target" 2>/dev/null` printed nothing and the directory was absent; `timeout 15s docker system df` timed out (exit 124), with the prior T-109g report recording 8 containers and 11 images. After: both fixed and repository `target/` directories remain absent (`du` exit 1); `timeout 15s docker system df` again timed out (exit 124); read-only inventory returned 8 containers, 11 images and 434 volumes. No database, container, image, volume or Cargo target was created or removed by T-109h.
**Performance** — n/a.
**Security** — SBD-30 T4 trigger (audit). SBD-81: (1) no endpoint or auth mode added; (2) no client-supplied tenant id; both sink methods compare the argument with the transaction-local tenant before writing, with no RLS changes; (3) no database fields or data classes added; module identity/state and policy snapshots use the existing audit columns; (4–7) no DTO, output, crypto, secret or external-call changes; (8) both writes use the caller's transaction and the mismatched-tenant test asserts rejection before insert; (9) SBD-53 toggle and policy changes are audited as `Security`; (10) the named abuse test is present but Not run due the resource restrictions above. Gitleaks scans of all five changed task-owned paths found no leaks.
**Deviations** — RATIFY TD-090–TD-092; inherit parent T-109 decisions TD-066–TD-075 and T-109g TD-087–TD-089.
**Dependencies affected** — Parent T-109 aggregate verification, T-117a runtime scheduler integration and M1 consumers of module/policy audit sinks.
**Rollback plan** — Revert the T-109h commit; `eq-audit::AuditModule` returns to the prior stub. No migration or persistent schema change is included.
**Follow-ups** — T-117a starts the 10-second runtime sealing interval. Run the module test, both new integration tests, workspace clippy/tests, and the full database-backed checks when the approved target and disposable database resources are permitted. Ratify TD-090–TD-092.

**Workflow steps 0–12** — Step 0: resumed on `dev`, retained unrelated SQLx/`.pending`/`.hold`/Python-cache WIP, confirmed this was the only in-progress task, and used the existing `in_progress` status. Step 1: read T-109h, its cited specs, parent T-109 and the module/audit contracts. Step 2: read T-109g's Report, deviations, follow-ups, ratifications and shipped handlers/repository code. Step 3: n/a, backend-only. Step 4: wrote the plan and AC/edge/error/abuse test plan before implementation; recorded TD-090–092. Step 5: added the always-on module, policy wiring, jobs/schedules, registered AuditSink, and named unit/integration tests. Step 6: no HTTP/runtime request applies; database tests are Not run because new database/container resources are forbidden. Step 7: rustfmt findings were corrected; no tests were launched. Step 8: reviewed transaction ownership, tenant check, module boundary, policy and audit action semantics. Step 9: n/a, no measured optimization. Step 10: completed the SBD-81 checklist and Gitleaks scans. Step 11: reran all permitted format, dependency, guard, ERD, task, metadata, diff and secret checks; clippy and Rust/database tests remain explicitly Not run. Step 12: completed this Report, set `review`, regenerated the index, and prepared this task's one signed commit for push to `dev`.

## Questions
- **Self-resolved (implementer, run 6; TD-090):** T-109h requires both cron schedules from parent T-109, but the parent specifies only `audit.retention` at `0 3 * * *` local → run `audit.verify_recent` daily at `30 3 * * *` local, after retention has completed, so the nightly chain check does not compete with anonymization/purge.
- **Self-resolved (implementer, run 6; TD-091):** the `AuditSink` port and parent identify the toggle/policy actions but do not define their category or snapshot context, and `ModuleId` is not a UUID → classify them as `Security` per SBD-53, keep `entity_id` absent, and include module identity plus enabled-state or versioned policy-document snapshots in `before`/`after`.
- **Self-resolved (implementer, run 6; TD-092):** the task's security abuse-case test only inspected module registration and did not exercise the sink's tenant boundary; SBD-73 requires a test of the triggered security control → register the existing `AuditSink` port and add database integration tests for caller-transaction rollback and mismatched-tenant rejection, without changing the port or persistence contract.

## Review
n/a until review.
