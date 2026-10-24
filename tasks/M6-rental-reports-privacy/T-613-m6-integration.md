---
id: T-613
title: M6 integration — studio rental school and privacy requests end to end, plus the full a11y sweep
milestone: M6
status: draft
depends_on: [T-612]
spec_refs: [TST-07, TST-11]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
The second business shape (rental-only) works from setup to paid booking, and privacy rights work end to end.

## Context
- Scenario A: setup with `studio_rental` → guest books → deposit invoice → paid → confirmed → completed.
- Scenario B: guardian requests export and deletion → blocked by open invoice → paid → verified → erased.
- Re-run TST-11 over every route that now exists.

## Scope
### In
- `crates/eq-server/tests/m6_*.rs`, Playwright a11y update

### Out
- —

## Acceptance criteria
- AC1. Scenario A → `m6_rental::studio_flow`
- AC2. Scenario B → `m6_privacy::erase_flow`
- AC3. a11y sweep passes → `web/e2e/a11y.spec.ts`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/m6_rental.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `test: M6 rental and privacy scenarios [T-613]`, signed off

## Report

## Questions
