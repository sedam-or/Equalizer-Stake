---
id: T-602
title: Add the pure booking validation and pricing (15-minute slices, peak rates)
milestone: M6
status: draft
depends_on: [T-601]
spec_refs: [RNT-02, RNT-03, TST-02]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Rental rules and prices are pure, tested functions, so quotes and charges always agree.

## Context
- RNT-02 validation per booking type (duration bounds, step, notice, days ahead, opening hours via `domain::slot` reuse through eq-scheduling's public domain API — allowed: rental may depend on always-on scheduling, ARC-02).
- RNT-03 price = Σ 15-minute slices of `hourly_minor / 4` with peak windows; module AC3 (100k base, 150k peak 18–21, 17:00–19:00 booking).

## Scope
### In
- `eq_rental::domain::{booking, pricing}` with property tests

### Out
- API (T-603)

## Acceptance criteria
- AC1. AC3 peak pricing → `pricing::tests::peak_example`
- AC2. Property: price additive over adjacent bookings → `proptest price_additive`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-rental/src/domain/*.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): booking validation and pricing [T-602]`, signed off

## Report

## Questions
