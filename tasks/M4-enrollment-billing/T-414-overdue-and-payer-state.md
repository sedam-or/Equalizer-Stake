---
id: T-414
title: Add overdue processing, late fees, and payer restrict/freeze/unfreeze with events
milestone: M4
status: todo
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-billing/tests/overdue.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `disable_unfreezes` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
