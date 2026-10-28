---
id: T-109d
title: Add transaction-bound audit recording
milestone: M1
status: todo
depends_on: [T-109c]
spec_refs: [AUD-02, ARC-05, FLW-01, SEC-07]
estimated_size: S
---

## Goal
Add the typed audit event and `record` operation so an audit insert shares the caller's transaction and failure rolls the business write back.

## Context
Use the exact `AuditEvent` and `record(tx: &mut Tx, event) -> Result<(), AppError>` signature from parent T-109. `AuditSink` wiring belongs to T-109h.

## Scope
### In
- Transaction-bound insert into `audit.entries`, actor/request metadata from the supplied actor context, and the rollback acceptance test.
### Out
- Derive macro (T-109a), schema (T-109b), sealer and jobs (T-109e–T-109h).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/record.rs` | modify/create | Record API and exports. |
| `crates/eq-audit/tests/audit.rs` | modify | Transaction rollback case. |
| `tasks/M1-core/T-109d-audit-record-api.md` | modify | Report and status. |

## Interfaces
Preserve `AuditEvent<'a>` and `record(&mut Tx, AuditEvent<'_>)` exactly as stated in the parent contract. No endpoint or public error code is added.

## Business rules
Use only `Category`, actor data and event fields defined by parent T-109.

## Security
- Triggers: audit disclosure, tampering and tenant context (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: cross-tenant or unpaired audit writes → use caller `Tx`, verify tenant context, and rely on transaction rollback.
- Abuse case: forced audit insert failure leaves no business row → `audit_failure_rolls_back`.

## Implementation notes
SQL stays in the audit module's own adapter. Do not open a second transaction for the audit insert.

## Acceptance criteria
- AC1. A forced audit insert failure rolls back the business write → `audit_failure_rolls_back`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `audit_failure_rolls_back` |

## Definition of done
- [ ] The database integration test passes on a fresh PostgreSQL 16 testcontainer.
- [ ] Rust architecture/security checks pass with no new `#[allow]`.
- [ ] Signed commit `feat(audit): add transaction-bound record API [T-109d]`.

## Report
**Plan** — Add the exact event type and insert through the supplied `Tx`; test rollback after a trigger rejects the audit row.
**Assumptions** — Tenant and actor are provided by the caller's transaction context.
**Test plan** — AC1→`audit_failure_rolls_back` using `eq_app` and a disposable DB.
**Summary** — Pending implementation.
**Spec read** — `spec/02-architecture.md` ARC-05; `spec/17-clean-architecture.md` §4; `spec/18-security-by-design.md` §8; parent T-109 record contract.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — Database integration test.
**Debug log** — Pending.
**Performance** — n/a; no list query.
**Security** — SBD-81 pending.
**Deviations** — None beyond parent T-109 RATIFY items.
**Dependencies affected** — T-109e and downstream audit consumers.
**Rollback plan** — Revert this part commit; the schema remains from T-109b.
**Follow-ups** — None identified.

## Questions
None.

## Review
n/a until review.
