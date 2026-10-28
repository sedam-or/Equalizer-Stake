---
id: T-109e
title: Add audit sealing and chain verification
milestone: M1
status: todo
depends_on: [T-109d]
spec_refs: [AUD-03, AUD-04, AUD-06, ARC-09, SEC-07]
estimated_size: M
---

## Goal
Compute and persist the canonical per-tenant SHA-256 chain and verify links, content, checkpoints and anonymized rows.

## Context
Use the JCS document and sequencing rules in parent T-109: sequence order, previous entry/checkpoint/zero anchor, UTC RFC 3339 microsecond timestamps, and the tenant advisory lock.

## Scope
### In
- `Sealer`, `VerifyReport`, `seal_tenant` and `verify`; offline metadata for their SQL queries.
### Out
- Database DDL/functions (T-109b), retention jobs (T-109g), user-facing audit API (T-113).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/sealer.rs`, `.sqlx/` | modify | Sealing/verification logic and offline metadata. |
| `tasks/M1-core/T-109e-audit-chain.md` | modify | Report and status. |

## Interfaces
Preserve `verify(tenant, from_seq, to_seq) -> VerifyReport { checked, anonymized, first_mismatch }` and the parent `Sealer` contract.

## Business rules
No new values; use only the parent task's chain and timestamp rules.

## Security
- Triggers: tamper detection and tenant-isolated reads/writes (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: forged or cross-tenant chain → lock by tenant, validate transaction context, compare stored link and canonical content hashes.
- Abuse tests are owned by T-109f: tampering, anonymized proof, checkpoint and concurrent sealers.

## Implementation notes
Run `cargo sqlx prepare --workspace` against the fresh disposable PostgreSQL 16 database after query changes; commit `.sqlx/` metadata.

## Acceptance criteria
- AC1. Sealing walks unsealed rows in sequence order and uses the correct anchor and canonical document.
- AC2. Verification returns a first mismatching sequence and treats anonymized hashes as authoritative only when proof exists.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | Behavioral chain tests in dependent T-109f. |

## Definition of done
- [ ] Rust format/clippy and SQLx offline metadata checks pass.
- [ ] The child tests in T-109f exercise all chain behaviors before parent T-109 is reviewed.
- [ ] Signed commit `feat(audit): add tamper-evident chain [T-109e]`.

## Report
**Plan** — Implement canonical hashing, tenant-serialized sealing and verification; validate SQL metadata against a fresh disposable PostgreSQL 16 database.
**Assumptions** — Parent T-109 fully defines fields, anchors and hash serialization.
**Test plan** — T-109f owns exact database assertions for AC3–AC5 and AC7.
**Summary** — Pending implementation.
**Spec read** — `spec/06-domain-model.md` §4.9/§4.11; `spec/18-security-by-design.md` §8; `spec/23-contracts-and-canonical-forms.md`; parent T-109.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — Database cases are owned by T-109f.
**Debug log** — Pending.
**Performance** — No measurement unless query plans are measured.
**Security** — SBD-81 pending.
**Deviations** — None beyond parent T-109 RATIFY items.
**Dependencies affected** — T-109f and audit job parts.
**Rollback plan** — Revert this part commit; no schema changes.
**Follow-ups** — None identified.

## Questions
None.

## Review
n/a until review.
