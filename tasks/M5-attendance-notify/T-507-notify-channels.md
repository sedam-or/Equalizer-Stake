---
id: T-507
title: Add the email (SMTP) and WhatsApp channels, channel settings with encrypted secrets, and test send
milestone: M5
status: todo
depends_on: [T-103, T-506]
spec_refs: [NTF-08, SEC-23, SEC-21]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A school connects its own email and WhatsApp in a settings screen, and secrets never leak.

## Context
- NTF-08 in `spec/modules/notify.md`: SMTP via `lettre` (rustls); WhatsApp through a `WhatsAppProvider` trait with a first implementation per the spec (Cloud API) plus a mock for tests.
- SEC-23: credentials encrypted (T-103), returned redacted; changing them requires the current password for non-owners (step-up). Module AC7: GET never returns secrets; logs never contain addresses.
- `POST /settings/channels/{channel}/test` sends a test message to the caller. **Security (spec/18):** create `eq_core::outbound`, the single shared HTTP client (rustls, TLS verification on, connect and total timeouts from config, redirects disabled, host allow-list per channel), and use it for every channel (SBD-47). Hold channel credentials as `secrecy::SecretString` after decryption (SBD-50). Never log request or response bodies.

## Scope
### In
- Channels, settings API, test send, MockSmtp/MockWhatsApp in testkit wired

### Out
- Routing (T-508)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-notify/tests/api_channels.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T4, T6 (SBD-30).
- Data classes touched: S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- S/T/I: forged or replayed external events → verify signatures in constant time before parsing, deduplicate provider event IDs, and never log raw bodies (SBD-63).
- Abuse-case acceptance criterion: `redacted` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
