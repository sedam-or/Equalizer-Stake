---
id: T-109f
title: Verify tampering, anonymization, purge and concurrent sealing
milestone: M1
status: todo
depends_on: [T-109e]
spec_refs: [AUD-03, AUD-04, AUD-05, ERD-05, SEC-07]
estimated_size: S
---

## Goal
Exercise chain integrity and privacy-preserving cleanup against the schema and Rust implementation from T-109b/T-109e.

## Context
Preserve the exact acceptance test names in parent T-109. All database cases use fresh PostgreSQL 16 testcontainers and an `eq_app` pool for tenant isolation.

## Scope
### In
- Tests for content tampering, nested anonymization, purge checkpoints, concurrent sealers and cross-tenant function arguments.
### Out
- Production implementation, which is owned by T-109b/T-109e.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/tests/audit.rs` | modify | Chain and tenant-abuse tests. |
| `tasks/M1-core/T-109f-audit-chain-tests.md` | modify | Report and status. |

## Interfaces
Test only the public `Sealer`/`verify` API and the exact SQL procedures from parent T-109.

## Business rules
No new cutoff or policy values; use explicit test fixtures from parent T-109.

## Security
- Triggers: tamper, tenant isolation and sensitive free text (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: altered chain, cross-tenant procedure call or leaked free text → assert mismatch, permission denial, recursive redaction and proof-backed verification.
- Abuse tests: `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint`, `concurrent_sealers`, `audit_functions_reject_cross_tenant_arguments`.

## Implementation notes
Use only fake personal values and disposable databases. Do not test RLS using the superuser pool.

## Acceptance criteria
- AC3. Tampered content reports its sequence → `tamper_detected`.
- AC4. Nested PII is redacted and the anonymized chain verifies → `anonymize_keeps_chain_valid`.
- AC5. Purged rows leave a valid checkpoint → `purge_with_checkpoint`.
- AC7. Concurrent sealers preserve one chain → `concurrent_sealers`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint`, `concurrent_sealers`, `audit_functions_reject_cross_tenant_arguments` |

## Definition of done
- [ ] All five integration tests pass on fresh disposable PostgreSQL 16 instances.
- [ ] No fake secret or free-text fixture is logged.
- [ ] Signed commit `test(audit): verify audit chain controls [T-109f]`.

## Report
**Plan** — Add exact AC3–AC5/AC7 tests plus cross-tenant function abuse coverage; run each against a fresh Testcontainers database.
**Assumptions** — The parent contract specifies the expected hash, redaction and checkpoint behavior.
**Test plan** — Each acceptance criterion maps to its same-named integration test; cross-tenant arguments must be rejected.
**Summary** — Pending implementation.
**Spec read** — `spec/18-security-by-design.md` §8; `spec/29-erd.md` §5; parent T-109.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — Each case records its disposable database and result.
**Debug log** — Pending.
**Performance** — n/a.
**Security** — SBD-81 pending.
**Deviations** — None beyond parent T-109 RATIFY items.
**Dependencies affected** — T-109g and T-109h.
**Rollback plan** — Revert this test-only part; no schema changes.
**Follow-ups** — None identified.

## Questions
None.

## Review
n/a until review.
