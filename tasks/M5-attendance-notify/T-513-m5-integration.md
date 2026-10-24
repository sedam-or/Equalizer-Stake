---
id: T-513
title: M5 integration — a teaching day: reminders, check-in, marks, no-show credit and notifications with channel failure
milestone: M5
status: draft
depends_on: [T-512]
spec_refs: [TST-07, TST-15, TST-16]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Prove a full teaching day works across scheduling, attendance, enrollment and notify, including a provider outage.

## Context
- Scenario: reminders sent → instructor checks in (GPS pass) → second instructor fails GPS → admin reviews → marks with one absent → no-show handled per policy → WhatsApp mock fails 5× → breaker opens → email fallback.
- TST-16 log capture: no PII during the scenario.

## Scope
### In
- `crates/eq-server/tests/m5_day.rs`

### Out
- —

## Acceptance criteria
- AC1. Scenario passes → `m5_day::teaching_day`
- AC2. No PII in logs → `m5_day::no_pii`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/m5_day.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `test(attendance): M5 teaching day [T-513]`, signed off

## Report

## Questions
