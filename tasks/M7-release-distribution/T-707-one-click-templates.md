---
id: T-707
title: Add 1-click deploy templates (Railway, Render, DigitalOcean, Coolify, CasaOS, Umbrel; Elestio and YunoHost docs)
milestone: M7
status: draft
depends_on: [T-706]
spec_refs: [DST-10, DST-11]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Owners with a 'tech friend' deploy with a button and land on the setup wizard.

## Context
- DST-10 table and DST-11 post-deploy message in `spec/14-distribution.md` §4; each template provisions app + Postgres 16, generates secrets and setup token, sets public URL, mounts `/data` or uses s3 files.

## Scope
### In
- Template files under `deploy/*`, README deploy buttons

### Out
- The YunoHost package repo (separate)

## Acceptance criteria
- AC1. Each template validates with the platform's CLI/linter where one exists (documented per platform) → `manual + CI lint`
- AC2. A Render blueprint deploy on a test account reaches `/setup` (documented in Report) → `manual`

## Tests to write
| File | Test names |
|---|---|
| `CI` | template lint |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(deploy): one-click templates [T-707]`, signed off

## Report

## Questions
