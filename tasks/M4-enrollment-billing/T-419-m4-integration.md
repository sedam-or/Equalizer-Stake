---
id: T-419
title: M4 integration — the music-school month: register, approve, schedule, issue, pay, overdue, freeze, unfreeze
milestone: M4
status: todo
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-server/tests/m4_month.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T3, T7 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `billing_summary_sentence` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
