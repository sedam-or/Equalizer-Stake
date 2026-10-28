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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-billing/tests/api_payments.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T3, T7 (SBD-30).
- Data classes touched: S1/S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `partial_rules` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
