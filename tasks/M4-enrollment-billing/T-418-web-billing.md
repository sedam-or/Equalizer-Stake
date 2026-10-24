---
id: T-418
title: Add the billing web pages: invoices, payment verification queue, payers, guardian invoices with payment instructions, and the frozen banner
milestone: M4
status: draft
depends_on: [T-416, T-417]
spec_refs: [BIL-07, BIL-08, BIL-11, FE-32, FE-33]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Admins collect money with less chasing, and parents always know what they owe and how to pay.

## Context
- Routes `/billing/invoices`, `/billing/invoices/:id`, `/billing/payments`, `/billing/payers` (spec 08 §6).
- FE-32 frozen banner from `/me.payer_state`; disabled actions explain why with a link to the invoice.
- Guardian invoice page shows the unique code prominently, copy buttons, payment instructions (I18nText), and proof upload.

## Scope
### In
- Pages, hooks, StatusBadge domain `invoice`/`payer`, locale strings
- A test-only clock endpoint `POST /api/v1/__test/clock {now}` that sets the shared `TestClock`, compiled only with the cargo feature `test-clock` (never in release builds; CI asserts the release binary has no such route)

### Out
- Gateway checkout (v1.x)

## Acceptance criteria
- AC1. E2E: issue cycle (time travel API in test mode), guardian uploads proof, admin verifies, invoice paid → `web/e2e/billing.spec.ts › collect`
- AC2. Frozen guardian sees the banner and disabled booking actions → `frozen banner`
- AC3. axe clean → `a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/billing.spec.ts` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): billing pages [T-418]`, signed off

## Report

## Questions
