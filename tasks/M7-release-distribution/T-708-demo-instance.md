---
id: T-708
title: Add demo mode: seed-demo data, role login buttons, nightly reset
milestone: M7
status: draft
depends_on: [T-707]
spec_refs: [DST-44, OPS-02]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Anyone can try the product in 10 seconds from the README.

## Context
- DST-44: `RITMA_DEMO=1`, `seed-demo` (music preset, 3 instructors, 40 students, 2 months of history), 'Try as owner/admin/teacher/parent' buttons (demo only, no passwords), nightly reset job; outbound messages always suppressed in demo.

## Scope
### In
- CLI, seed generator (deterministic), login buttons, reset job

### Out
- Hosting the demo

## Acceptance criteria
- AC1. seed-demo is deterministic (same counts and ids with the same seed) → `demo::deterministic`
- AC2. Demo mode refuses outside `RITMA_DEMO=1` → `guarded`
- AC3. No notifications leave in demo → `suppressed`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/demo.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(ops): demo mode [T-708]`, signed off

## Report

## Questions
