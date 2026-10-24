---
id: T-415
title: Add invoice cancel/adjust, manual and one-off invoices, credit notes, and HTML/PDF documents
milestone: M4
status: draft
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

## Acceptance criteria
- AC1. Cancel issued invoice as owner → cancelled, audit, no deletion → `api_adjust::owner_cancel`
- AC2. Credit note reduces balance and appears on the document → `credit_note`
- AC3. PDF renders with the IDR amount formatted as in the HTML → `document_pdf_matches`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-billing/tests/api_adjust.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(billing): adjustments and documents [T-415]`, signed off

## Report

## Questions
