---
id: T-806
title: Add desktop backups: daily schedule, missed-run catch-up, folder choice, retention, tray warning
milestone: M8
status: draft
depends_on: [T-805]
spec_refs: [DST-23, OPS-05]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A PC that dies doesn't take the school with it.

## Context
- DST-23 rules in `spec/14-distribution.md` §5 using T-701 backups.

## Scope
### In
- Scheduler, settings, restore-from-file

### Out
- —

## Acceptance criteria
- AC1. Backup taken at next start when the PC was off at 03:00 → `desktop::catch_up`
- AC2. Keeps last 14 → `retention`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): backups [T-806]`, signed off

## Report

## Questions
