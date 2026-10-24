---
id: T-904
title: Add plans, limits and daily usage metering with graceful over-limit behaviour
milestone: M9
status: draft
depends_on: [T-903]
spec_refs: [CLD-12]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Fair plans by active students, with limits that never delete data or stop running operations abruptly.

## Context
- `spec/15-cloud.md` §3 plans and the 'active student' definition; CLD-12 trial and downgrade behaviour; banners in the app.

## Scope
### In
- usage_snapshots job, limit checks on new enrollments, banners

### Out
- Payments (T-905)

## Acceptance criteria
- AC1. Over the free limit → new enrollments blocked with a clear message; existing operations continue → `platform::over_limit`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): plans and metering [T-904]`, signed off

## Report

## Questions
