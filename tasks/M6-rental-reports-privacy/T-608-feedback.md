---
id: T-608
title: Add feedback: progress notes with publish modes, ratings with limits and anonymity, and moderation
milestone: M6
status: draft
depends_on: [T-607]
spec_refs: [FBK-01, FBK-02, FBK-03, FBK-04]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Teachers share progress with families, and families rate lessons fairly, under the school's chosen rules.

## Context
- `spec/modules/feedback.md` FBK-01..04 and module ACs 1–4; DDL §4.10 + rating hidden columns (§4.11); `FeedbackPolicy` §6.11.

## Scope
### In
- Migration, policy, endpoints, events `feedback.progress_note_published.v1`

### Out
- UI (T-612)

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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(feedback): notes and ratings [T-608]`, signed off

## Report

## Questions
