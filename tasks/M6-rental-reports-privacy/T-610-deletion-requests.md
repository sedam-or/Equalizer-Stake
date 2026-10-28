---
id: T-610
title: Add deletion requests (right to erasure) with SLA, open-invoice block, verification and the erase job
milestone: M6
status: todo
depends_on: [T-109, T-609]
spec_refs: [AUD-11, AUD-12, AUD-05]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A family can ask to be forgotten, and the school can comply on time without breaking its accounting records.

## Context
- DDL `deletion_requests` (`spec/06-domain-model.md` §4.9) verbatim into `migrations/0064_audit_deletion_requests.sql`; RLS.
- AUD-11 steps 1–8 in `spec/modules/audit.md` §2; business-day SLA excluding closures; attention item 3 business days before due; module AC5.
- Each module contributes erase steps through a `SubjectErase` port; audit anonymisation scoped to the subject (T-109 function).
- AUD-12 consents table writes at registration approval and terms acceptance.

## Scope
### In
- Port, impls, endpoints, job, SLA attention

### Out
- UI (T-612)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/tests/privacy_erase.rs` | test | Task-listed verification file. |
| `migrations/0064_audit_deletion_requests.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T3, T4 (SBD-30).
- Data classes touched: S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `erase_flow` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. AC5 blocked with open invoices; erase after verify keeps invoices with anonymised name → `privacy::erase_flow`
- AC2. SLA attention item appears 3 business days before → `sla_attention`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/privacy_erase.rs` | AC names |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(audit): deletion requests [T-610]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
