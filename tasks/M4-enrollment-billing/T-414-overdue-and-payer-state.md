---
id: T-414
title: Add overdue processing, late fees, and payer restrict/freeze/unfreeze with events
milestone: M4
status: draft
depends_on: [T-413]
spec_refs: [BIL-06, BIL-08, SCH-13, MOD-08]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
The school's own late-payment rules apply automatically and fairly, and a frozen family is blocked consistently across scheduling and rental.

## Context
- BIL-06/BIL-08 in `spec/modules/billing.md`, using the snapshotted terms. Module AC3 (Rp100.000 late fee day after due + freeze, admin-manual unfreeze) and AC6 (unfreeze refused while overdue unpaid under AdminManual).
- Emits `billing.invoice_overdue.v1`, `billing.payer_frozen.v1`, `billing.payer_unfrozen.v1` (structs in `eq_core::events`, consumed by scheduling T-212).
- `/me` gains `payer_state` for the frozen banner (FE-32).
- On module disable: unfreeze everyone with audit reason `module-disabled` (MOD-08).

## Scope
### In
- Daily overdue job, late fee line, payer state machine + endpoints (unfreeze override), events, `/me.payer_state`, `on_disable`

### Out
- UI (T-418)

## Acceptance criteria
- AC1. Module AC3 (time travel) → `overdue::late_fee_and_freeze`
- AC2. Module AC6 → `unfreeze_rule`
- AC3. Frozen → scheduling participations blocked end to end → `frozen_blocks_sessions`
- AC4. Disabling billing unfreezes → `disable_unfreezes`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/tests/overdue.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): overdue, late fees and freeze [T-414]`, signed off

## Report

## Questions
