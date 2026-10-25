---
id: T-508
title: Add routing with fallback, retries with backoff, circuit breaker, quiet hours and per-payer aggregation
milestone: M5
status: draft
depends_on: [T-507]
spec_refs: [NTF-03, NTF-04, NTF-05, NTF-06, TST-15, PRD-13]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Critical messages get through by some channel, standard ones respect families' evenings, and a broken provider doesn't stall everything.

## Context
- NTF-03..06 in `spec/modules/notify.md`; module ACs 1–5 (fallback, critical failure event + admin in-app, breaker open after 5 failures, quiet hours deferral, sibling aggregation).
- Backoff list comes from `retry.backoff_seconds` (policy), not constants.

## Scope
### In
- Routing engine (pure where possible), breaker state, quiet hours, aggregation window

### Out
- Templates (T-509)

## Acceptance criteria
- AC1. AC1 fallback → `routing::fallback`
- AC2. AC2 critical failure → `critical_failure`
- AC3. AC3 breaker → `breaker`
- AC4. AC4 quiet hours → `quiet_hours`
- AC5. AC5 aggregation → `aggregation`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-notify/tests/routing.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(notify): routing, retries and breaker [T-508]`, signed off

## Report

## Questions
