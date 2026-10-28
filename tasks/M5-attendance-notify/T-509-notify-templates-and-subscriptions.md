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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-notify/tests/subscriptions.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T4, T6 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- S/T/I: forged or replayed external events → verify signatures in constant time before parsing, deduplicate provider event IDs, and never log raw bodies (SBD-63).
- Abuse-case acceptance criterion: `complete` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Invite event → email with the invite URL → `subscriptions::invite_sent`
- AC2. Invoice issued → WhatsApp then email per routing → `invoice_issued_routed`
- AC3. Template completeness → `templates::complete`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-notify/tests/subscriptions.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(notify): templates and event subscriptions [T-509]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
