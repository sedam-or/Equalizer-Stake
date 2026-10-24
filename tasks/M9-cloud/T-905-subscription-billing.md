---
id: T-905
title: Add subscription billing providers (Xendit, Stripe), webhooks and dunning
milestone: M9
status: draft
depends_on: [T-904]
spec_refs: [CLD-20, CLD-21]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Schools pay for the cloud in the way that is normal for them (VA/QRIS/e-wallet or card).

## Context
- CLD-20/21: `SubscriptionProvider` trait, webhook verification, dunning 3/7/14 days, downgrade never deletes. Webhooks follow SBD-63: verify the signature (constant-time) before parsing, check timestamp tolerance, dedupe by provider event id, never log the raw body. Abuse-case test: `replayed_webhook_is_ignored`.

## Scope
### In
- Providers, webhooks, dunning emails

### Out
- —

## Acceptance criteria
- AC1. Webhook replay is idempotent → `platform::webhook_idempotent`
- AC2. Failed payments follow the dunning schedule (time travel) → `dunning`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): subscription billing [T-905]`, signed off

## Report

## Questions
