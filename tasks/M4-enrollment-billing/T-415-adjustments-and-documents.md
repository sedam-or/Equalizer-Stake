---
id: T-415
title: Add invoice cancel/adjust, manual and one-off invoices, credit notes, and HTML/PDF documents
milestone: M4
status: todo
depends_on: [T-414]
spec_refs: [BIL-09, BIL-10, BIL-11]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Admins can correct mistakes without deleting history, bill one-off items, and hand families a proper invoice or receipt.

## Context
- BIL-09/10/11 in `spec/modules/billing.md`. Cancelling an issued invoice is an owner override (audited); adjustments are new lines or credit notes, never edits of issued lines.
- Documents: HTML with a print stylesheet; PDF via a headless-free renderer (`typst` crate or `printpdf`), using tenant branding, locale and currency formatting. Pick one, justify it in the Report (dependency licence must pass cargo-deny).

## Scope
### In
- Endpoints, services, document renderer, consumers for rental confirm/cancel (one-off invoice, credit note) — gated to compile only with the rental feature present via events, not imports

### Out
- UI (T-418)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-billing/tests/api_adjust.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T5 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- Abuse-case acceptance criterion: `credit_note` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Cancel issued invoice as owner → cancelled, audit, no deletion → `api_adjust::owner_cancel`
- AC2. Credit note reduces balance and appears on the document → `credit_note`
- AC3. PDF renders with the IDR amount formatted as in the HTML → `document_pdf_matches`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/tests/api_adjust.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): adjustments and documents [T-415]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
