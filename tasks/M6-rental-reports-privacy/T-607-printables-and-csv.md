---
id: T-607
title: Add printable reports and CSV exports for every list
milestone: M6
status: draft
depends_on: [T-606]
spec_refs: [RPT-04, RPT-05, AUD-01]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Schools can print monthly reports and take any list into a spreadsheet.

## Context
- RPT-04 printable HTML with a print stylesheet; RPT-05 CSV for students, sessions, invoices, payments, marks — the export is audited (AUD-01) and PII columns follow `read_pii` permission.

## Scope
### In
- Endpoints, CSV streaming, audit

### Out
- —

## Acceptance criteria
- AC1. CSV of 1,000 sessions streams and matches the list filter → `exports::sessions_csv`
- AC2. Export audited → `export_audited`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-reports/tests/exports.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(reports): printables and CSV [T-607]`, signed off

## Report

## Questions
