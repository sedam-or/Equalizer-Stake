---
id: T-801
title: Add the Tauri 2 desktop app that runs the server in-process and opens the setup wizard
milestone: M8
status: draft
depends_on: [T-711]
spec_refs: [DST-20, DST-02]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A school with one front-desk PC installs an app and is at the setup wizard without a terminal.

## Context
- DST-20 in `spec/14-distribution.md` §5: `apps/desktop`, `eq_server::run(config)` on a tokio runtime (no sidecar), native window to `http://127.0.0.1:<port>/setup?token=…`.

## Scope
### In
- `apps/desktop` Tauri project, `eq_server::run` library entry (refactor `serve` to call it)

### Out
- Embedded Postgres (T-802)

## Acceptance criteria
- AC1. App starts against an external dev DB and shows the wizard (manual + WebDriver smoke) → `desktop::smoke`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | smoke |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): Tauri shell [T-801]`, signed off

## Report

## Questions
