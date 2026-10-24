---
id: T-906
title: Add per-tenant backup/restore and import of desktop or self-host backups into the cloud
milestone: M9
status: draft
depends_on: [T-905]
spec_refs: [CLD-33, DST-25]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Moving to the cloud (or support restores) works per school without touching others.

## Context
- CLD-33 tenant-scoped dump and restore with id remapping; DST-25 `.eqbak` import with the user-provided key, discarded after import.

## Scope
### In
- Tenant export/import, signup 'Import from file' step

### Out
- —

## Acceptance criteria
- AC1. Desktop backup imports into a new cloud tenant with identical counts → `platform::import_eqbak`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): tenant backup and import [T-906]`, signed off

## Report

## Questions
