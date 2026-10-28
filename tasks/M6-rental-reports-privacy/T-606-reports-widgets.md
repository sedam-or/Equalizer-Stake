---
id: T-606
title: Add ReportsPolicy and module-aware dashboard widgets with caching and access rules
milestone: M6
status: todo
depends_on: [T-605]
spec_refs: [RPT-01, RPT-02, RPT-03, RPT-06, FE-16]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Owners see the health of their school at a glance, computed from live data, showing only what their enabled modules can support.

## Context
- `spec/modules/reports.md`: the widget catalog with `requires` modules, aggregates-only revenue (RPT-02), per-widget endpoint cached 60 s (RPT-03), access (RPT-06), `min_sessions_for_rate` marking (module AC2), utilisation formula (AC3). **Data access (spec 17 CA-30):** reports owns no tables and never queries other modules' tables. It reads only through ports or read views named `rv_<owner>_<name>`. Each view is created by a new migration tagged with the **owning** module (`migrations/0066_<owner>_report_views.sql` onward, `spec/29-erd.md` §7), and listed in that module's spec as a public contract.
- Policy `ReportsPolicy` `spec/04-policy-engine.md` §6.10.

## Scope
### In
- Policy, `/reports/widgets`, `/reports/widgets/{key}`, the widget queries

### Out
- UI (T-612)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-reports/tests/api_reports.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. AC1 module-aware → `api_reports::module_aware`
- AC2. AC2 incomplete data → `incomplete_marked`
- AC3. AC3 utilisation → `utilisation_formula`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-reports/tests/api_reports.rs` | AC names |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(reports): dashboard widgets [T-606]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
