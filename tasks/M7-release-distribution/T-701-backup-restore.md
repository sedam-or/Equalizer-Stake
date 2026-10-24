---
id: T-701
title: Add encrypted backup and restore (CLI + scheduled + UI trigger) with the round-trip test
milestone: M7
status: draft
depends_on: [T-613]
spec_refs: [OPS-05, SEC-31, TST-13, PRD-16]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A school can always get its data back: daily encrypted backups that are proven to restore.

## Context
- OPS-05 in `spec/11-ops.md` (format `.eqbak`: pg_dump custom format + files tree + manifest, streamed through XChaCha20-Poly1305 with the backup key, SEC-31), retention, `backup --out` / `restore --in [--force]`.
- TST-13: seed → backup → wipe → restore → row counts and file hashes equal (docker in CI).
- Owner UI: Settings → Backups: last backup, 'Back up now', download (owner, step-up).

## Scope
### In
- CLI, scheduled job, API, settings page, TST-13

### Out
- Desktop scheduling (M8)

## Acceptance criteria
- AC1. TST-13 round trip → `backup::round_trip`
- AC2. Wrong key → clear error, nothing restored → `wrong_key`
- AC3. Restore refuses a non-empty DB without --force → `restore_guard`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/backup.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(ops): encrypted backup and restore [T-701]`, signed off

## Report

## Questions
