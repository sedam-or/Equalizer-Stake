---
id: T-807
title: Add the signed auto-updater with pre-update backup, and installer signing
milestone: M8
status: draft
depends_on: [T-806]
spec_refs: [DST-24, DST-05]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Schools stay up to date without thinking about it, safely.

## Context
- DST-24 updater flow; DST-05 step 3 signing (unsigned builds documented until certificates exist).

## Scope
### In
- Updater config, release workflow job for desktop bundles

### Out
- —

## Acceptance criteria
- AC1. Update from vN to vN+1 in CI with a local update server; DB backed up first → `desktop::update_flow`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): updater and signing [T-807]`, signed off

## Report

## Questions
