---
id: T-612
title: Add reports dashboard and printables, feedback pages, the audit log page with verify, and /privacy
milestone: M6
status: draft
depends_on: [T-611]
spec_refs: [RPT-01, FBK-01, AUD-07, AUD-10, AUD-11]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Owners understand the school, families see progress, and privacy rights are a button away.

## Context
- Routes `/reports`, `/reports/print/:report`, `/feedback`, `/audit`, `/privacy` (spec 08 §6). Charts with Recharts (lazy), palette tokens `--chart-*`.

## Scope
### In
- Pages, dashboard switching to widgets when reports is enabled (FE-16)

### Out
- —

## Acceptance criteria
- AC1. Owner dashboard renders widgets for enabled modules only → `web/e2e/reports.spec.ts › widgets`
- AC2. Audit verify shows OK → `audit verify`
- AC3. Guardian requests export and downloads it → `privacy export`
- AC4. axe clean → `a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/reports.spec.ts` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): reports, feedback, audit and privacy pages [T-612]`, signed off

## Report

## Questions
