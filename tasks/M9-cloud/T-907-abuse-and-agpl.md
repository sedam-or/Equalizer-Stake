---
id: T-907
title: Add abuse controls (signup limits, disposable email block, suspend switch) and the AGPL source link
milestone: M9
status: draft
depends_on: [T-906]
spec_refs: [CLD-35, CLD-40]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
The cloud stays safe and compliant with its own licence.

## Context
- CLD-35 controls; CLD-40 footer 'Source code' link to the exact running tag from `meta/app.commit`.

## Scope
### In
- Limits, suspend admin action, footer link

### Out
- —

## Acceptance criteria
- AC1. Suspended tenant is read-only with a banner → `platform::suspend`
- AC2. Footer link points to the running commit → `agpl_link`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): abuse controls and source link [T-907]`, signed off

## Report

## Questions
