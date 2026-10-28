---
id: T-109h
title: Register the always-on audit module and sink
milestone: M1
status: todo
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
| `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/module.rs` | modify/create | Replace stub and register the module. |
| `tasks/M1-core/T-109h-audit-module.md` | modify | Report and status. |

## Interfaces
Use the existing module and sink ports only. Preserve action names `core.module.enabled`, `core.module.disabled`, `core.policy.changed` and schedules from parent T-109.

## Business rules
No new permissions, routes, policy fields, event names or schedule values.

## Security
- Triggers: audit writes for core state changes (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: missing audit sink or disabled audit module → audit remains `always_on`, with writes using the caller's transaction.
- Abuse case: module registration test proves always-on status and both job schedules → `audit_module_registers_its_always_on_jobs_and_schedules`.

## Implementation notes
Keep `routes()` empty; the audit HTTP API is explicitly T-113.

## Acceptance criteria
- AC1. Audit module is always-on and registers both job kinds/schedules → `audit_module_registers_its_always_on_jobs_and_schedules`.
- AC2. Existing `AuditSink` port records toggle and policy change in the caller's transaction.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/src/module.rs` | `audit_module_registers_its_always_on_jobs_and_schedules` |

## Definition of done
- [ ] Module unit test passes; policy wiring and `AuditSink` compile against existing ports.
- [ ] Rust checks and layer guards pass.
- [ ] Signed commit `feat(audit): register audit module and sink [T-109h]`.

## Report
**Plan** — Wire the already implemented policy/jobs/record contracts into the `Module` and `AuditSink` interfaces and verify manifest/schedule behavior.
**Assumptions** — No route or permission is required for the always-on backend module.
**Test plan** — AC1 maps to the exact module test; AC2 is checked through existing sink integration tests.
**Summary** — Pending implementation.
**Spec read** — `spec/03-module-system.md` MOD-01/04; `spec/17-clean-architecture.md` §4; parent T-109.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — n/a; no endpoint is in scope.
**Debug log** — Pending.
**Performance** — n/a.
**Security** — SBD-81 pending.
**Deviations** — None beyond parent T-109 RATIFY items.
**Dependencies affected** — Parent T-109 and M1 consumer tasks.
**Rollback plan** — Revert this part; the module stub returns.
**Follow-ups** — T-117a starts the 10-second runtime sealing interval.

## Questions
None.

## Review
n/a until review.
