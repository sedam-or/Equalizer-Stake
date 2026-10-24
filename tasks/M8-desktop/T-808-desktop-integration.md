---
id: T-808
title: M8 integration — install, setup, phone check-in over the tunnel, backup, update, move to cloud export
milestone: M8
status: draft
depends_on: [T-807]
spec_refs: [DST-02, DST-25]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
The desktop channel is proven end to end on all three OSes.

## Context
- DST-25 'Move to the cloud' export step (import side is M9 T-906).
- Scripted smoke on CI runners; manual checklist for camera on a real phone documented in the Report.

## Scope
### In
- CI matrix job, manual checklist

### Out
- —

## Acceptance criteria
- AC1. CI smoke passes on Windows, macOS and Linux → `desktop::e2e_matrix`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | e2e_matrix |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `test(desktop): M8 integration [T-808]`, signed off

## Report

## Questions
