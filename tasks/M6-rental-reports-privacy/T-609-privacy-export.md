---
id: T-609
title: Add the data-subject export (ZIP of JSON + CSV) with signed download and expiry
milestone: M6
status: draft
depends_on: [T-608]
spec_refs: [AUD-10, SEC-15, DST-42]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Anyone can get a copy of their (or their child's) data, as UU PDP and GDPR expect.

## Context
- AUD-10 in `spec/modules/audit.md` §2: contents, permissions, files row purpose `export`, signed URL valid 24 h then deleted by job.
- Each module contributes its part through a `SubjectExport` port (core), so audit doesn't import modules.

## Scope
### In
- Port, per-module impls, job, endpoints

### Out
- UI (T-612)

## Acceptance criteria
- AC1. Export contains the expected files for a guardian with 2 children → `privacy::export_contents`
- AC2. Download after 24 h → 404 → `export_expires`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/privacy_export.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(audit): data-subject export [T-609]`, signed off

## Report

## Questions
