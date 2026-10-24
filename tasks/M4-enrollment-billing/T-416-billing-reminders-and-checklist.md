---
id: T-416
title: Add billing reminders, checklist items, has_data and the remaining on_disable effects
milestone: M4
status: draft
depends_on: [T-415]
spec_refs: [BIL-05, MOD-08, SET-13]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Families are reminded before and on the due date per the school's rule, and billing plugs cleanly into setup and module toggling.

## Context
- BIL-05: daily cron 09:00 tenant TZ, offsets `reminders_days_from_due` (negative = before), dedupe per invoice+offset; requests go through the `Notifier` port (default no-op until M5).
- Checklist: `set_payment_instructions` if bank transfer is enabled with the template text unchanged.
- MOD-08 billing row: cron skips the tenant; open invoices readable via read-only route.

## Scope
### In
- Reminder job, `Notifier` port definition with no-op default in `eq_core::ports`, checklist, has_data, on_disable

### Out
- Message sending (M5)

## Acceptance criteria
- AC1. Offsets [-5, 0] produce two reminder requests at the right local days (time travel) → `reminders::offsets`
- AC2. Disabled billing → no jobs run for the tenant → `disabled_skips`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/tests/reminders.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): reminders and module integration [T-416]`, signed off

## Report

## Questions
