---
id: T-710
title: Add the docs site (Starlight): owner guides in Bahasa Indonesia first, English second, rules help linked from the app
milestone: M7
status: draft
depends_on: [T-709]
spec_refs: [DST-43, FE-20]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Non-technical owners solve problems themselves; every rule field's '?' opens a page that explains it.

## Context
- DST-43 in `spec/14-distribution.md` §7: per channel install guides, per module 'how it works', one section per policy field with anchors matching `docs/<lang>/rules/<module>#<field>`; developer docs separate.

## Scope
### In
- `docs/` Starlight project, CI build, link checker, in-app help links

### Out
- Hosting

## Acceptance criteria
- AC1. Every policy field has a docs anchor (generated check from policy schemas) → `docs::anchors_complete`
- AC2. Link checker passes → `CI`

## Tests to write
| File | Test names |
|---|---|
| `CI` | docs build + link check |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `docs: owner and developer docs site [T-710]`, signed off

## Report

## Questions
