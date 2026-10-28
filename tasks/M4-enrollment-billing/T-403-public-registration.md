---
id: T-403
title: Add the public registration API (/public/registrations) with consent, honeypot, encrypted applicant data and payment proof
milestone: M4
status: draft
depends_on: [T-303, T-402]
spec_refs: [ENR-03, SEC-03, SEC-13, AUD-12, API-08]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Parents can register their children from a public link without an account, and the school receives a complete, private, reviewable application.

## Context
- ENR-03 in `spec/modules/enrollment.md` §1: fields controlled by `registration.collect_*`, multi-child, preferred slots, consent text version hash, honeypot + optional captcha (SEC-11).
- `registrations.applicant_enc` is the encrypted JSON of contact + students (SEC-03); only `applicant_display` is plaintext.
- Payment proof upload through `FileService` purpose `registration_doc`/`payment_proof` (T-303).
- Public route under `/api/v1/public/registrations`, `Idempotency-Key` required (API-08), rate limited as public (API-15).
- Emits `enrollment.registration_submitted.v1`. Consent recorded in `consents` (AUD-12) at approval time for the created guardian.

## Scope
### In
- Endpoint, service, encryption, validation per policy, honeypot, event

### Out
- Approval (T-404)
- The `/join` page (T-417)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/tests/api_registration.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3, T5 (SBD-30).
- Data classes touched: S0/S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- Abuse-case acceptance criterion: `sbd_403_abuse` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. A valid submission with 2 children → 201, encrypted at rest (no plaintext email in the row) → `api_registration::submit_encrypted`
- AC2. Honeypot filled → 202 but nothing stored → `honeypot_silent`
- AC3. `public_form=false` → 404 `module_disabled`-style `not_found` → `public_form_off`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/api_registration.rs` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): public registration [T-403]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
