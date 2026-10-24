---
id: T-507
title: Add the email (SMTP) and WhatsApp channels, channel settings with encrypted secrets, and test send
milestone: M5
status: draft
depends_on: [T-506]
spec_refs: [NTF-08, SEC-23, SEC-21]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A school connects its own email and WhatsApp in a settings screen, and secrets never leak.

## Context
- NTF-08 in `spec/modules/notify.md`: SMTP via `lettre` (rustls); WhatsApp through a `WhatsAppProvider` trait with a first implementation per the spec (Cloud API) plus a mock for tests.
- SEC-23: credentials encrypted (T-103), returned redacted; changing them requires the current password for non-owners (step-up). Module AC7: GET never returns secrets; logs never contain addresses.
- `POST /settings/channels/{channel}/test` sends a test message to the caller.

## Scope
### In
- Channels, settings API, test send, MockSmtp/MockWhatsApp in testkit wired

### Out
- Routing (T-508)

## Acceptance criteria
- AC1. AC7 no secrets leak → `api_channels::redacted`
- AC2. Test send with MockSmtp records one message → `test_send`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-notify/tests/api_channels.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(notify): email and WhatsApp channels [T-507]`, signed off

## Report

## Questions
