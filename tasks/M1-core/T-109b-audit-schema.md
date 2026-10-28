---
id: T-109b
title: Add audit tables, grants and controlled database functions
milestone: M1
status: todo
depends_on: [T-109a]
spec_refs: [AUD-01, AUD-03, AUD-05, ERD-03, ERD-04, ERD-05, SEC-07]
estimated_size: S
---

## Goal
Create the mapped audit schema, immutable app-role entry table, checkpoints, row policies and controlled functions. The SQL must match spec 06 §4.9/§4.11 and the T-109 row in ERD §7.

## Context
Implement only `migrations/0006_audit.sql`. The parent T-109 Context contains the complete mapped DDL, grants, `seal`, `anonymize` and `purge_expired` contracts. `audit.entries` is tenant-scoped with RLS; checkpoints are system rows without RLS. SECURITY DEFINER functions use `search_path = pg_catalog, audit` and explicitly reject a tenant different from `app.tenant_id`.

## Scope
### In
- Exact audit tables/indexes, least-privilege grants, RLS policies, trigger-free controlled functions from parent T-109.
- Verify that `eq_app` cannot UPDATE/DELETE entries and cannot pass another tenant to the functions.
### Out
- Rust record/seal/policy/job behavior (T-109c–T-109h).

## Files
| Path | Action | Purpose |
|---|---|---|
| `migrations/0006_audit.sql` | create | Exact mapped audit schema and functions. |
| `crates/eq-audit/tests/audit.rs` | modify | App-role privilege test. |
| `tasks/M1-core/T-109b-audit-schema.md` | modify | Report and status. |

## Interfaces
Use the table DDL and function signatures in parent T-109 Context. Do not add columns, tables, roles, policies or permission names.

## Business rules
None beyond the parent task's exact cutoff and anonymization semantics.

## Security
- Triggers: tenant isolation and immutable audit storage (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05; free-text `reason` follows ERD-05 anonymization requirements.
- Threat: cross-tenant mutation or procedure calls → RLS, revoked UPDATE/DELETE and explicit tenant-context checks.
- Abuse tests: `app_role_cannot_mutate`; cross-tenant procedure rejection is retained in T-109f.

## Implementation notes
Read the audit module section and T-109 migration mapping in `spec/29-erd.md`; run both ERD checks after the migration change.

## Acceptance criteria
- AC1. `eq_app` cannot UPDATE or DELETE audit entries → `app_role_cannot_mutate`.
- AC2. Migration matches the complete spec 06 DDL and ERD mapping → `scripts/erd.py --migrations`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `app_role_cannot_mutate` |

## Definition of done
- [ ] `scripts/erd.py --migrations` and `scripts/erd.py --check` pass.
- [ ] The app-role test uses `eq_app`, not a superuser connection.
- [ ] Signed commit `feat(audit): add audit schema and functions [T-109b]`.

## Report
**Plan** — Implement mapped DDL and least-privilege controls; test app-role mutation rejection and run ERD validation.
**Assumptions** — Parent T-109 is the source for exact signatures and values; the spec DDL is normative.
**Test plan** — AC1→`app_role_cannot_mutate`; AC2→`scripts/erd.py --migrations`.
**Summary** — Pending implementation.
**Spec read** — `spec/06-domain-model.md` §§4.9/4.11; `spec/18-security-by-design.md` §§4/8; `spec/29-erd.md` §§4/5/7.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — n/a; database test is the runtime check.
**Debug log** — Pending.
**Performance** — n/a.
**Security** — SBD-81 pending.
**Deviations** — None beyond parent T-109 RATIFY items.
**Dependencies affected** — T-109c; schema consumers use the parent chain.
**Rollback plan** — Forward-only migration; no compensating migration is needed before merge because it is not applied to shared databases.
**Follow-ups** — None identified.

## Questions
None.

## Review
n/a until review.
