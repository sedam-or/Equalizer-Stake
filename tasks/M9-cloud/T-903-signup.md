---
id: T-903
title: Add cloud signup (magic link, Google OIDC), slug and region choice, then the unchanged setup wizard
milestone: M9
status: draft
depends_on: [T-902]
spec_refs: [CLD-10, SET-11, PRD-04]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Sign up and go: an owner reaches the same 3-step wizard in under a minute, no token needed.

## Context
- CLD-10 flow; `POST /setup/school` multi-mode path (501 since T-302) now implemented for authenticated platform accounts.

## Scope
### In
- Signup pages and API, OIDC, magic links, tenant provisioning

### Out
- Billing (T-905)

## Acceptance criteria
- AC1. E2E: signup → wizard → dashboard on a new subdomain → `web/e2e/cloud-signup.spec.ts`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/cloud-signup.spec.ts` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): signup [T-903]`, signed off

## Report

## Questions
