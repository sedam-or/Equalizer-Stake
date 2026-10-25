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

## Questions
