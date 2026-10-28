---
id: T-512
title: Add attendance review and marks UI, the notifications inbox and bell, channel settings and notification preferences
milestone: M5
status: todo
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/e2e/attendance.spec.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
