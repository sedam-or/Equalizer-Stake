---
id: T-412
title: Add monthly cycle issuance (daily cron checks the policy day), grouping per payer, pending charges and invoice issuing
milestone: M4
status: draft
depends_on: [T-411]
spec_refs: [BIL-02, BIL-04, ARC-09, TST-15]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
On the school's issue day, every family gets exactly one correct invoice, even with several servers running and even if the server restarts mid-run.

## Context
- BIL-02/BIL-04 in `spec/modules/billing.md`. Daily cron `billing.daily` checks `cycle.issue_day` in tenant TZ (never a hard-coded day, ARC-09); dedupe key `billing.issue:{tenant}:{YYYY-MM}` plus per-payer idempotency.
- Lines from `BillableSource` (T-408) plus `pending_charges` (registration fee on `enrollment_activated`, leave fee on `leave_fee_charged`, rental on `reservation_confirmed` later).
- `skip_issuing_while_frozen` honoured. Emits `billing.invoice_issued.v1`.
- Module AC1: issue_day=5, 2 guardians (one with 2 students), clock at the 5th 00:05 → exactly 2 invoices; re-run → still 2.

## Scope
### In
- Job, issuance service, pending-charge consumers, invoice numbering per tenant per year

### Out
- Payments (T-413)

## Acceptance criteria
- AC1. Module AC1 → `issuance::cycle_exactly_once`
- AC2. Two planners/runners concurrently → still 2 invoices → `issuance_concurrent`
- AC3. Policy change on the 8th doesn't change the invoice issued on the 5th (module AC4) → `terms_snapshot_protects`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/tests/issuance.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): cycle issuance [T-412]`, signed off

## Report

## Questions
