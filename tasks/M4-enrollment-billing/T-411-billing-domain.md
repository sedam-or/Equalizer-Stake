---
id: T-411
title: Add the pure billing domain: totals, proration, unique codes, due instants, late fee and term snapshots
milestone: M4
status: todo
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-billing/src/domain/*.rs` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Module AC2 due instant in Asia/Jakarta → `domain::due::tests::end_of_due_day`
- AC2. Module AC8 proration → `proration::tests::two_of_four_remaining`
- AC3. Module AC9 property → `proptest totals_consistent`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/src/domain/*.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): pure billing domain [T-411]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
