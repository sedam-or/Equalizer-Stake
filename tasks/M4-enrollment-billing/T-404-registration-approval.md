---
id: T-404
title: Add registration review and the approval transaction (people, links, enrollments, memberships, invites)
milestone: M4
status: draft
depends_on: [T-403]
spec_refs: [ENR-04, IDN-07, SCH-07]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
One click approves a family: guardian and students are created or matched, enrollments start, series seats are taken, and the guardian receives an invite.

## Context
- ENR-04 in `spec/modules/enrollment.md` §1 (approval modes Auto | AdminApproval, placement Audition | Trial | Direct, matching existing persons by email blind index).
- Module AC3: 2 children → 1 guardian, 2 students, 2 enrollments, 2 series memberships, 1 invite, all in **one** transaction; any failure rolls back everything.
- Series seat conflicts or capacity → 409 with which child and which series.

## Scope
### In
- `POST /registrations/{id}/approve|reject`, list and detail for admins (decrypted applicant for `enrollment.registration.read_pii`), auto-approval path

### Out
- UI (T-417)

## Acceptance criteria
- AC1. Module AC3 approval transaction → `api_approval::family_approval_atomic`
- AC2. A matching existing guardian email links instead of duplicating → `existing_guardian_matched`
- AC3. Capacity full on one child's series → 409 and nothing created → `capacity_conflict_rolls_back`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/api_approval.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): registration approval [T-404]`, signed off

## Report

## Questions
