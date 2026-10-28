---
id: T-418
title: Add the billing web pages: invoices, payment verification queue, payers, guardian invoices with payment instructions, and the frozen banner
milestone: M4
status: draft
depends_on: [T-416, T-417]
spec_refs: [BIL-07, BIL-08, BIL-11, FE-32, FE-33]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Admins collect money with less chasing, and parents always know what they owe and how to pay.

## Context
- Routes `/billing/invoices`, `/billing/invoices/:id`, `/billing/payments`, `/billing/payers` (spec 08 §6).
- FE-32 frozen banner from `/me.payer_state`; disabled actions explain why with a link to the invoice.
- Guardian invoice page shows the unique code prominently, copy buttons, payment instructions (I18nText), and proof upload.

## Scope
### In
- Pages, hooks, StatusBadge domain `invoice`/`payer`, locale strings
- A test-only clock endpoint `POST /api/v1/__test/clock {now}` that sets the shared `TestClock`, compiled only with the cargo feature `test-clock` (never in release builds; CI asserts the release binary has no such route)

### Out
- Gateway checkout (v1.x)

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/e2e/billing.spec.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T3, T5, T7 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `a11y` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. E2E: issue cycle (time travel API in test mode), guardian uploads proof, admin verifies, invoice paid → `web/e2e/billing.spec.ts › collect`
- AC2. Frozen guardian sees the banner and disabled booking actions → `frozen banner`
- AC3. axe clean → `a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/billing.spec.ts` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(web): billing pages [T-418]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
