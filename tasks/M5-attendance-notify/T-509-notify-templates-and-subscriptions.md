---
id: T-509
title: Add message templates (en/id × channel) and subscribe notify to the event catalog; add the in-app inbox
milestone: M5
status: draft
depends_on: [T-508]
spec_refs: [NTF-07, TST-09]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Every event that should reach a person produces a warm, correct, localised message on the right channel.

## Context
- NTF-07 templates in `spec/modules/notify.md`; the consumer column of the event catalog in `spec/03-module-system.md` §6 lists which events notify handles.
- Voice: parent messages warm and polite (Bapak/Ibu in Indonesian), per `brand/BRAND.md` §6. Money and dates via server-side ICU formatting with tenant locale.
- TST-09: every template exists for en and id × routed channels.
- In-app inbox API `GET /notifications`, `POST /notifications/{id}/read`, unread count.

## Scope
### In
- Templates, event handlers, inbox API, identity invite/reset delivery (raw token from the event payload), template completeness test

### Out
- UI (T-512)

## Acceptance criteria
- AC1. Invite event → email with the invite URL → `subscriptions::invite_sent`
- AC2. Invoice issued → WhatsApp then email per routing → `invoice_issued_routed`
- AC3. Template completeness → `templates::complete`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-notify/tests/subscriptions.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(notify): templates and event subscriptions [T-509]`, signed off

## Report

## Questions
