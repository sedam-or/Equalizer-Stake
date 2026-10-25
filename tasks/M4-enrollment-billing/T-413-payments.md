---
id: T-413
title: Add payments: record (staff), submit with proof (payer), verify/reject, partial payments and paid transitions
milestone: M4
status: draft
depends_on: [T-412]
spec_refs: [BIL-07, BIL-12, SEC-07, SEC-13, API-08]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Payments are recorded once, verified by a human where needed, and invoices move to paid with an auditable trail.

## Context
- BIL-07 in `spec/modules/billing.md`; module AC7 (min first payment 7000 bps of 600.000 → 300.000 rejected, 420.000 accepted).
- Payer submission uploads a proof (`payment_proof`, T-303) and creates a `pending_verification` payment; admin verifies or rejects.
- `payments` is append-only except verify/status through a SECURITY DEFINER function (SEC-07). Idempotency-Key required on create (API-08).
- Events `billing.payment_recorded.v1`, `billing.invoice_paid.v1`.
- BIL-12: define the `PaymentProvider` trait (seam only, no implementation in v1) so a gateway can be added in v1.x without touching payment recording.

## Scope
### In
- Endpoints, services, verification queue list, events

### Out
- UI (T-418)

## Acceptance criteria
- AC1. Module AC7 → `api_payments::partial_rules`
- AC2. Full payment → invoice paid + event → `full_payment_paid`
- AC3. Direct UPDATE of payments.amount as eq_app fails → `payments_append_only`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/tests/api_payments.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): payments [T-413]`, signed off

## Report

## Questions
