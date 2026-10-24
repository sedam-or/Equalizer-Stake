---
id: T-709
title: Add observability: structured logs, Prometheus metrics, dead-letter alerts, clock-skew check
milestone: M7
status: draft
depends_on: [T-708]
spec_refs: [OPS-06, OPS-07, OPS-08, OPS-09]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Self-hosters and the cloud team can see health and fix problems before schools notice.

## Context
- OPS-06..09 in `spec/11-ops.md`: `/metrics` (opt-in, token), dead letters → attention items, clock comparison with DB, resource budget checks.

## Scope
### In
- Metrics endpoint, alerts, startup checks

### Out
- Dashboards (cloud)

## Acceptance criteria
- AC1. Dead-lettered job creates an attention item → `obs::dead_letter_attention`
- AC2. Clock skew > 5 s logs a warning and shows a banner → `clock_skew`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/observability.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(ops): observability [T-709]`, signed off

## Report

## Questions
