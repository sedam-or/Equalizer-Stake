---
id: T-604
title: Add rental jobs (hold expiry, auto-complete) and payment confirmation on invoice paid
milestone: M6
status: draft
depends_on: [T-603]
spec_refs: [RNT-06, RNT-07, RNT-09, TST-15]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Unpaid holds free the room on time, and paid reservations confirm themselves.

## Context
- RNT-06 consumer of `billing.invoice_paid.v1` for invoices linked to a reservation; RNT-07 expiry every minute (module AC2); RNT-09 auto-complete.

## Scope
### In
- Two jobs, one consumer

### Out
- —

## Acceptance criteria
- AC1. AC2 hold expiry releases holds → `jobs::hold_expiry`
- AC2. Paid → confirmed → `paid_confirms`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-rental/tests/jobs.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): expiry and confirmation [T-604]`, signed off

## Report

## Questions
