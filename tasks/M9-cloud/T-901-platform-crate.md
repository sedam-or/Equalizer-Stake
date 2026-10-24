---
id: T-901
title: Add the eq-platform crate (feature `platform`) and the platform schema with its role
milestone: M9
status: draft
depends_on: [T-808]
spec_refs: [CLD-11, SEC-05, ARC-06]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Cloud-only concerns live in an open, separate crate with its own database role, so tenant code never bypasses RLS.

## Context
- CLD-11 tables in `spec/15-cloud.md` §2 under schema `platform`, role `eq_platform` with BYPASSRLS used only by platform jobs (SEC-05).

## Scope
### In
- Crate, migrations, role grants, isolation test extension

### Out
- Signup (T-903)

## Acceptance criteria
- AC1. Request handlers never use the platform role (test inspects pool usage) → `platform::role_isolation`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): platform crate and schema [T-901]`, signed off

## Report

## Questions
