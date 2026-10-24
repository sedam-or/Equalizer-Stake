---
id: T-606
title: Add ReportsPolicy and module-aware dashboard widgets with caching and access rules
milestone: M6
status: draft
depends_on: [T-605]
spec_refs: [RPT-01, RPT-02, RPT-03, RPT-06, FE-16]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Owners see the health of their school at a glance, computed from live data, showing only what their enabled modules can support.

## Context
- `spec/modules/reports.md`: the widget catalog with `requires` modules, aggregates-only revenue (RPT-02), per-widget endpoint cached 60 s (RPT-03), access (RPT-06), `min_sessions_for_rate` marking (module AC2), utilisation formula (AC3).
- Policy `ReportsPolicy` `spec/04-policy-engine.md` §6.10.

## Scope
### In
- Policy, `/reports/widgets`, `/reports/widgets/{key}`, the widget queries

### Out
- UI (T-612)

## Acceptance criteria
- AC1. AC1 module-aware → `api_reports::module_aware`
- AC2. AC2 incomplete data → `incomplete_marked`
- AC3. AC3 utilisation → `utilisation_formula`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-reports/tests/api_reports.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(reports): dashboard widgets [T-606]`, signed off

## Report

## Questions
