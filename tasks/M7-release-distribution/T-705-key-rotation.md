---
id: T-705
title: Add master-key and tenant-DEK rotation (CLI) with background re-encryption
milestone: M7
status: draft
depends_on: [T-704]
spec_refs: [SEC-01, SEC-02]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Keys can be rotated without downtime if a secret leaks.

## Context
- SEC-01 `EQ_SECRET_KEY_PREVIOUS`; SEC-02 new DEK version, re-encrypt job per table/column in batches, retire old version when done (T-103 key versioning).

## Scope
### In
- CLI `rotate-keys`, job, progress in attention items

### Out
- —

## Acceptance criteria
- AC1. After rotation all ciphertexts use the new version and decrypt → `rotation::reencrypt_all`
- AC2. Interrupted job resumes → `resumable`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/rotation.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(security): key rotation [T-705]`, signed off

## Report

## Questions
