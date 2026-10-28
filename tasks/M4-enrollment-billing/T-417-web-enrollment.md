---
id: T-417
title: Add the enrollment web pages: /join, registrations review, student profile, guardian student switcher, leave, credits and import wizard
milestone: M4
status: draft
depends_on: [T-409, T-315]
spec_refs: [ENR-03, ENR-04, ENR-07, ENR-08, ENR-10, ENR-11, FE-30, FE-34]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Parents register in a few taps on a phone, admins approve families in one screen, and everyone sees a student's whole picture in one place.

## Context
- Routes from `spec/08-frontend.md` §6: `/join`, `/registrations`, `/registrations/:id`, `/students/:id`, `/import`.
- ENR-10 guardian portal: student switcher (tabs on mobile) with status badges.
- Design: spec 09 components only; terminology via `useTerm`; empty/loading/error states; drafts for the registration review form.

## Scope
### In
- Pages, API hooks, locale strings (en/id)

### Out
- Billing pages (T-418)

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/e2e/enrollment.spec.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `a11y` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. E2E: a parent submits /join for 2 children on a 390 px viewport; the admin approves; the guardian accepts the invite and sees both children → `web/e2e/enrollment.spec.ts › family flow`
- AC2. Import wizard shows row errors and blocks commit until fixed → `import wizard`
- AC3. axe: 0 serious/critical on every page → `a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/enrollment.spec.ts` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): enrollment pages [T-417]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
