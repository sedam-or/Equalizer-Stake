---
id: T-410
title: Add the billing schema, BillingPolicy and the BillingModule wiring
milestone: M4
status: draft
depends_on: [T-408]
spec_refs: [BIL-01, POL-01, POL-12, MOD-03]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Billing exists as a module with its tables, its typed rules and summaries (the 'invoice on the 5th, due the 10th, late fee on the 11th' sentence), permissions and nav.

## Context
- DDL `spec/06-domain-model.md` §4.5 + `pending_charges` (§4.11) verbatim into `migrations/0030_billing.sql`, RLS everywhere; `payments` append-only except status columns via function (SEC-07).
- Policy `BillingPolicy` exactly as `spec/04-policy-engine.md` §6.5 (cycle, grouping, first_cycle, payment_reference, partial_payment, late_fee, overdue, reminders, payment_methods, tax if present), externally tagged enums.
- Implements `CurrencyLockCheck` (T-113): locked once any invoice exists (POL-12).
- `clippy::float_arithmetic` denied in this crate (ARC-11). Requires_any enrollment | rental.

## Scope
### In
- Migration, policy, module wiring, permissions/nav from `spec/modules/billing.md` §3, currency lock, policy i18n

### Out
- Services (T-411+)

## Acceptance criteria
- AC1. Every preset's billing patch validates → `presets::all_presets_valid`
- AC2. `due_day < issue_day` → pointer error → `billing::policy::tests::due_after_issue`
- AC3. Summary snapshot reads like PRD-06 for the music preset → `summary_snapshot`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/src/policy.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): schema and policy [T-410]`, signed off

## Report

## Questions
