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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-rental/src/domain/*.rs` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `peak_example` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
