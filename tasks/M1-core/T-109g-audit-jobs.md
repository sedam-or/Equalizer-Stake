---
id: T-109g
title: Add audit retention and recent verification jobs
milestone: M1
status: todo
depends_on: [T-109f]
spec_refs: [AUD-04, AUD-07, ARC-09, OPS-08, SEC-07]
estimated_size: S
---

## Goal
Add tenant-scoped retention/anonymization and nightly recent-chain verification jobs, using the shared `PolicyStore` and raising the specified attention item on mismatch.

## Context
Parent T-109 defines job kinds, local cron schedules, policy calculations and attention item fields. T-108's `JobCtx` may be extended additively to carry the shared policy store.

## Scope
### In
- `audit.retention` and `audit.verify_recent` handlers, typed retry/permanent failures, and job behavior tests.
- Pass the shared optional `PolicyStore` through `JobCtx` without changing existing job callers.
### Out
- Module registration and cron schedule wiring (T-109h), server runtime loop (T-117a).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/src/jobs.rs`, `crates/eq-audit/src/lib.rs`, `crates/eq-core/src/jobs.rs` | modify/create | Job handlers and shared context. |
| `crates/eq-audit/tests/audit.rs` | modify | Job behavior tests. |
| `tasks/M1-core/T-109g-audit-jobs.md` | modify | Report and status. |

## Interfaces
Preserve kinds `audit.retention` and `audit.verify_recent`, and use `AuditPolicy`/`Sealer` APIs from prior parts.

## Business rules
Retention intervals come only from `AuditPolicy`; verify the last 24 hours as specified by the parent task.

## Security
- Triggers: tenant PII retention and chain integrity (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: wrong tenant, policy bypass or silent mismatch → use tenant `JobCtx`, shared `PolicyStore`, `eq_app`, and a high-severity audit mismatch attention item.
- Abuse tests: `audit_retention_job_uses_policy_store`, `audit_verify_job_raises_attention_on_mismatch`.

## Implementation notes
Keep `JobCtx` backward compatible for existing callers; never log payloads, PII or secret values.

## Acceptance criteria
- AC1. Retention uses the shared policy and calls anonymize then purge → `audit_retention_job_uses_policy_store`.
- AC2. Recent verification creates the specified attention item for a mismatch → `audit_verify_job_raises_attention_on_mismatch`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `audit_retention_job_uses_policy_store`, `audit_verify_job_raises_attention_on_mismatch` |

## Definition of done
- [ ] Both job tests pass on fresh disposable PostgreSQL 16 testcontainers.
- [ ] Rust checks and guard scripts pass without weakening job or tenant controls.
- [ ] Signed commit `feat(audit): add audit maintenance jobs [T-109g]`.

## Report
**Plan** — Implement policy-backed retention and recent verification, then test both handlers through their public job interfaces.
**Assumptions** — T-109 parent supplies all schedule and attention values; T-117a supplies runtime tenant scheduling.
**Test plan** — AC1/AC2 map to the two named integration tests using disposable DBs.
**Summary** — Pending implementation.
**Spec read** — `spec/18-security-by-design.md` §8; `spec/29-erd.md` attention-items row; parent T-109.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — Database tests exercise handler calls; runtime schedule wiring is T-109h/T-117a.
**Debug log** — Pending.
**Performance** — n/a unless measured.
**Security** — SBD-81 pending.
**Deviations** — RATIFY TD-073 from the parent task.
**Dependencies affected** — T-109h and M1 integration tasks.
**Rollback plan** — Revert this part; existing JobCtx defaults remain available.
**Follow-ups** — None identified.

## Questions
None.

## Review
n/a until review.
