---
id: T-608
title: Add feedback: progress notes with publish modes, ratings with limits and anonymity, and moderation
milestone: M6
status: draft
depends_on: [T-113b, T-607]
spec_refs: [FBK-01, FBK-02, FBK-03, FBK-04]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Teachers share progress with families, and families rate lessons fairly, under the school's chosen rules.

## Context
- `spec/modules/feedback.md` FBK-01..04 and module ACs 1–4; DDL §4.10 + rating hidden columns (§4.11) verbatim into `migrations/0062_feedback.sql`; `FeedbackPolicy` §6.11.
- Permissions (D-63, `feedback.md` §2, spec 25 §6): `feedback.read`, `feedback.notes.write`, `feedback.notes.publish`, `feedback.ratings.write`, `feedback.ratings.moderate`, declared through `PermissionDef` with the role grants in that table. The module's `PermissionDef`s must match the spec 25 matrix test (`rbac_matrix_matches_spec`, T-113).

## Scope
### In
- Migration, policy, endpoints, events `feedback.progress_note_published.v1`

### Out
- UI (T-612)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-feedback/tests/api_feedback.rs` | test | Task-listed verification file. |
| `migrations/0062_feedback.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T3 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `monthly_limit` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. AC1 eligibility to rate → `api_feedback::eligibility`
- AC2. AC2 monthly limit → `monthly_limit`
- AC3. AC3 anonymity → `anonymity`
- AC4. AC4 review flow → `review_flow`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-feedback/tests/api_feedback.rs` | AC names |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(feedback): notes and ratings [T-608]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
