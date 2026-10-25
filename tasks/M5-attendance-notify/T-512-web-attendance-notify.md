---
id: T-512
title: Add attendance review and marks UI, the notifications inbox and bell, channel settings and notification preferences
milestone: M5
status: draft
depends_on: [T-511]
spec_refs: [ATT-05, ATT-10, NTF-08, NTF-10, FE-33]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Admins clear the review queue quickly, instructors mark attendance in seconds, and everyone controls their notifications.

## Context
- Routes `/attendance/review`, `/sessions/:id` marks tab, `/notifications`, `/settings/channels`, `/me` notification prefs (spec 08 §6).
- Marks use optimistic UI (FE-33). Channel settings show redacted secrets and a test-send button.

## Scope
### In
- Pages, bell with unread count in the shell, locale strings

### Out
- —

## Acceptance criteria
- AC1. E2E: pending check-in reviewed by admin → `web/e2e/attendance.spec.ts › review`
- AC2. Marks lock message after the window → `marks locked`
- AC3. Channel test send shows success → `channels`
- AC4. axe clean → `a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/attendance.spec.ts` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): attendance review, marks and notifications [T-512]`, signed off

## Report

## Questions
