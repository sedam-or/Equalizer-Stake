---
id: T-601
title: Add the rental schema, RentalPolicy (booking types, payment, who can book) and the RentalModule wiring
milestone: M6
status: draft
depends_on: [T-513]
spec_refs: [RNT-01, POL-01, MOD-03]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Studio rental becomes a module with typed booking types and payment terms, sharing the conflict ledger with lessons.

## Context
- DDL `spec/06-domain-model.md` §4.7 (rate_cards, reservations) verbatim into `migrations/0060_rental.sql`; RLS.
- Policy `RentalPolicy` exactly as `spec/04-policy-engine.md` §6.7; the `studio_rental` preset patch (T-301) must now validate — remove the `TODO(M6)` skip in the preset test.
- Implements `seed_from_preset` for `catalog.rate_cards` (spec 03 §3).
- On disable (MOD-08): new reservations blocked, confirmed ones keep holds.

## Scope
### In
- Migration, policy, module wiring, preset seeding, permissions/nav

### Out
- Booking logic (T-602+)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-rental/src/policy.rs` | create or modify | Path named in this task's contract. |
| `migrations/0060_rental.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4, T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `studio_rental_seeds_rates` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. studio_rental preset validates fully → `presets::all_presets_valid`
- AC2. Setup with studio_rental seeds rate cards → `api_setup::studio_rental_seeds_rates`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-rental/src/policy.rs` | `summary_snapshot` |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): schema and policy [T-601]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
