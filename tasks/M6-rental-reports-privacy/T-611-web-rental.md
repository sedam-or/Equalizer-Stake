---
id: T-611
title: Add the rental web pages: book a room, my reservations, admin reservations and rate cards, and the public /book page
milestone: M6
status: draft
depends_on: [T-610, T-315]
spec_refs: [RNT-04, RNT-10, FE-31]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Booking a studio feels like booking a table at a café: pick a time, see the price, confirm.

## Context
- Routes `/rental`, `/rental/reservations`, `/book` (spec 08 §6). SlotPicker from T-312; price quote from the server; ConflictPanel on 409.

## Scope
### In
- Pages, locale strings

### Out
- —

## Acceptance criteria
- AC1. E2E member booking with deposit flow → `web/e2e/rental.spec.ts › member`
- AC2. Guest booking on mobile → `guest`
- AC3. axe clean → `a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/rental.spec.ts` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): rental pages [T-611]`, signed off

## Report

## Questions
