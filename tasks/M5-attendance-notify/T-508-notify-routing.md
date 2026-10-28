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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-notify/tests/routing.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T6 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- S/T/I: forged or replayed external events → verify signatures in constant time before parsing, deduplicate provider event IDs, and never log raw bodies (SBD-63).
- Abuse-case acceptance criterion: `aggregation` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
