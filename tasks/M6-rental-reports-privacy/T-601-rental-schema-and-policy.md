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
- DDL `spec/06-domain-model.md` §4.7 (rate_cards, reservations) verbatim; RLS.
- Policy `RentalPolicy` exactly as `spec/04-policy-engine.md` §6.7; the `studio_rental` preset patch (T-301) must now validate — remove the `TODO(M6)` skip in the preset test.
- Implements `seed_from_preset` for `catalog.rate_cards` (spec 03 §3).
- On disable (MOD-08): new reservations blocked, confirmed ones keep holds.

## Scope
### In
- Migration, policy, module wiring, preset seeding, permissions/nav

### Out
- Booking logic (T-602+)

## Acceptance criteria
- AC1. studio_rental preset validates fully → `presets::all_presets_valid`
- AC2. Setup with studio_rental seeds rate cards → `api_setup::studio_rental_seeds_rates`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-rental/src/policy.rs` | `summary_snapshot` |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): schema and policy [T-601]`, signed off

## Report

## Questions
