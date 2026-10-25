---
id: T-510
title: Add the scheduling reminders job (minutes-before windows from NotifyPolicy)
milestone: M5
status: draft
depends_on: [T-509]
spec_refs: [NTF-01, MOD-01]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Families and instructors are reminded before lessons at the times the school chose.

## Context
- `spec/modules/scheduling.md` §4 jobs: `scheduling.reminders` every 5 min, windows from `notify.session_reminders_minutes_before`, dedupe `reminder:{session}:{minutes}`; planned only if notify is enabled.
- Scheduling must not import notify: it reads the windows through a `ReminderWindows` port implemented by notify (default: empty).

## Scope
### In
- Job in eq-scheduling, port in eq-core, impl in eq-notify

### Out
- —

## Acceptance criteria
- AC1. A session at 15:00 with window 60 → one request at 14:00–14:05, never twice → `reminders::once_per_window`
- AC2. Notify disabled → no requests → `notify_off`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/reminders.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(scheduling): session reminders [T-510]`, signed off

## Report

## Questions
