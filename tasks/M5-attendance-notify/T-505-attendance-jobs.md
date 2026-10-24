---
id: T-505
title: Add attendance jobs: review SLA reminders and escalation, and photo retention purge
milestone: M5
status: draft
depends_on: [T-504]
spec_refs: [ATT-05, ATT-08, TST-15]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Pending reviews never rot, and photos of people are deleted on schedule.

## Context
- ATT-05: remind after `remind_after_minutes`, escalate to owner once after `sla_minutes` (`attendance.review_overdue.v1`). Module AC6.
- ATT-08: daily purge of check-in photos older than `CorePolicy.privacy.photo_retention_days`; files row `deleted_at` set, bytes removed (AC8).

## Scope
### In
- Two jobs with time-travel tests

### Out
- —

## Acceptance criteria
- AC1. AC6 escalation once → `jobs::escalate_once`
- AC2. AC8 purge → `photo_purge`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-attendance/tests/jobs.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(attendance): review SLA and photo purge [T-505]`, signed off

## Report

## Questions
