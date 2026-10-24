---
id: T-403
title: Add the public registration API (/public/registrations) with consent, honeypot, encrypted applicant data and payment proof
milestone: M4
status: draft
depends_on: [T-402]
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

## Questions
