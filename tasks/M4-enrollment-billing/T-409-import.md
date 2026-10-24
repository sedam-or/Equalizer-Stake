---
id: T-409
title: Add the student/guardian import (CSV/XLSX) with preview and atomic commit
milestone: M4
status: draft
depends_on: [T-408]
spec_refs: [ENR-11, DST-41, SEC-13]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A school moves from its spreadsheet in minutes. Migration is the biggest adoption barrier, so the import must be forgiving in preview and strict in commit.

## Context
- ENR-11 columns and behaviour in `spec/modules/enrollment.md` §1; module AC7 (100 rows with 1 invalid → preview 99 ok + 1 error; commit refused until 0 errors).
- Upload via `FileService` purpose `import` (T-303). XLSX via `calamine`. A localised template file (en/id) downloadable from the endpoint (DST-41).
- Commit runs in one transaction; persons matched by email blind index.

## Scope
### In
- `POST /import/students` (preview), `POST /import/{id}/commit`, template download

### Out
- Wizard UI (T-417)

## Acceptance criteria
- AC1. Module AC7 → `api_import::preview_and_commit`
- AC2. Duplicate emails inside the file are reported per row → `duplicate_rows_reported`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/api_import.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): spreadsheet import [T-409]`, signed off

## Report

## Questions
