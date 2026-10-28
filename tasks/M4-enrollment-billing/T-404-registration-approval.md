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
- `POST /registrations/{id}/approve|reject`, list and detail for admins (decrypted applicant only for `enrollment.registration.read_pii`, granted to owner and admin, `null` for everyone else; D-62, spec 25 §6), auto-approval path

### Out
- UI (T-417)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/tests/api_approval.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T3, T7 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `existing_guardian_matched` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
