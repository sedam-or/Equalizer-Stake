---
id: T-702
title: Add 'Export everything' (ZIP of CSV per entity + manifest + files) in an open format
milestone: M7
status: draft
depends_on: [T-701]
spec_refs: [DST-42, PRD-01]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
No lock-in, provably: the owner downloads all school data in files any spreadsheet can open.

## Context
- DST-42 in `spec/14-distribution.md` §7; reuses the `SubjectExport`-style per-module exporters at tenant scope; audited; owner only with step-up.

## Scope
### In
- Endpoint, job, settings button, docs page

### Out
- —

## Acceptance criteria
- AC1. The ZIP contains one CSV per entity listed in the manifest and opens in LibreOffice headless conversion in CI → `export::all_entities`
- AC2. Audited → `export_audited`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/export_all.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(ops): export everything [T-702]`, signed off

## Report

## Questions
