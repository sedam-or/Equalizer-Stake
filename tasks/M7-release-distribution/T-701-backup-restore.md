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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-server/tests/backup.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T4, T5 (SBD-30).
- Data classes touched: S1/S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- Abuse-case acceptance criterion: `restore_guard` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. TST-13 round trip → `backup::round_trip`
- AC2. Wrong key → clear error, nothing restored → `wrong_key`
- AC3. Restore refuses a non-empty DB without --force → `restore_guard`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/backup.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(ops): encrypted backup and restore [T-701]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
