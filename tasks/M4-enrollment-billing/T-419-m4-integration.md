---
id: T-419
title: M4 integration — the music-school month: register, approve, schedule, issue, pay, overdue, freeze, unfreeze
milestone: M4
status: draft
depends_on: [T-418]
spec_refs: [TST-07, TST-15, PRD-06]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
One automated scenario proves the v0 business rules now work as configurable policy, end to end, through a full billing month.

## Context
- Music preset; clock starts 2026-01-05 (TST-20). Scenario: registration → approval → series materialised → invoice on the 5th → one family pays 70% then rest → one family doesn't pay → late fee and freeze on the 11th → sessions blocked → admin records payment → unfreeze → sessions unblocked.
- Also assert the summary text for billing equals the PRD-06 example sentence in `id` and `en`.

## Scope
### In
- `crates/eq-server/tests/m4_month.rs`, an E2E variant in Playwright using the `test-clock` endpoint from T-418

### Out
- —

## Acceptance criteria
- AC1. The scenario passes → `m4_month::music_school_month`
- AC2. PRD-06 summary sentence matches → `billing_summary_sentence`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/m4_month.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `test(billing): M4 month scenario [T-419]`, signed off

## Report

## Questions
