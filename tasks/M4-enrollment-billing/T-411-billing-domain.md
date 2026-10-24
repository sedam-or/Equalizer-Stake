---
id: T-411
title: Add the pure billing domain: totals, proration, unique codes, due instants, late fee and term snapshots
milestone: M4
status: draft
depends_on: [T-410]
spec_refs: [BIL-01, BIL-03, BIL-06, ARC-11, TST-02]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Every money calculation lives in small, pure, property-tested functions, so the riskiest code in the product is the best tested.

## Context
- BIL-01 amounts, BIL-03 proration (`first_cycle` modes), BIL-06 late fee (Flat | Percent with `apply_bps`), unique code generation (`payment_reference.unique_code.digits`) and the due instant rule: due = end of `due_day` in tenant TZ (23:59:59.999) — module AC2.
- Terms snapshot structs stored on invoices: `late_fee_terms`, `overdue_terms`, `due_at` (POL-05).
- Module AC8 (proration example) and AC9 (money safety property).

## Scope
### In
- `eq_billing::domain::{totals, proration, reference, due, late_fee, terms}` with unit and property tests

### Out
- DB and jobs

## Acceptance criteria
- AC1. Module AC2 due instant in Asia/Jakarta → `domain::due::tests::end_of_due_day`
- AC2. Module AC8 proration → `proration::tests::two_of_four_remaining`
- AC3. Module AC9 property → `proptest totals_consistent`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/src/domain/*.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): pure billing domain [T-411]`, signed off

## Report

## Questions
