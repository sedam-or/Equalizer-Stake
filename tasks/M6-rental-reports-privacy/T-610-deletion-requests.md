---
id: T-610
title: Add deletion requests (right to erasure) with SLA, open-invoice block, verification and the erase job
milestone: M6
status: draft
depends_on: [T-609]
spec_refs: [AUD-11, AUD-12, AUD-05]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A family can ask to be forgotten, and the school can comply on time without breaking its accounting records.

## Context
- AUD-11 steps 1–8 in `spec/modules/audit.md` §2; business-day SLA excluding closures; attention item 3 business days before due; module AC5.
- Each module contributes erase steps through a `SubjectErase` port; audit anonymisation scoped to the subject (T-109 function).
- AUD-12 consents table writes at registration approval and terms acceptance.

## Scope
### In
- Port, impls, endpoints, job, SLA attention

### Out
- UI (T-612)

## Acceptance criteria
- AC1. AC5 blocked with open invoices; erase after verify keeps invoices with anonymised name → `privacy::erase_flow`
- AC2. SLA attention item appears 3 business days before → `sla_attention`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/privacy_erase.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(audit): deletion requests [T-610]`, signed off

## Report

## Questions
