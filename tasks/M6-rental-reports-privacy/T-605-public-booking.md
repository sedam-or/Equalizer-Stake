---
id: T-605
title: Add guest booking (/public/rental) with contact capture, verification and abuse limits
milestone: M6
status: draft
depends_on: [T-604]
spec_refs: [RNT-10, SEC-11, API-15]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Bands and creators can book a studio from a public page without an account, safely.

## Context
- RNT-10 in `spec/modules/rental.md` (who_can_book MembersAndGuests); guest becomes a person with no membership or a `guest` marker as the spec says; honeypot + public rate limits; email verification link before the hold is final if the spec requires it.

## Scope
### In
- Public endpoints, guest person handling, verification

### Out
- Page (T-611)

## Acceptance criteria
- AC1. Guest books and receives confirmation → `api_public_booking::guest_flow`
- AC2. Rate limit after 20/min → `rate_limited`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-rental/tests/api_public_booking.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(rental): public guest booking [T-605]`, signed off

## Report

## Questions
