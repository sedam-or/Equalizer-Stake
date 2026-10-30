---
id: T-110h
title: Accept one-time TOTP recovery codes at login
milestone: M1
status: todo
depends_on: [T-110g]
spec_refs: [IDN-12, STM-25, AUTH-06, ERD-03]
estimated_size: S
---

## Goal
Allow an enrolled person to use one unused recovery code instead of a TOTP code exactly once.

## Context
- T-111 AC7 requires TOTP login without a code → 401 totp_required; valid TOTP → 200; a recovery code → 200; reuse of that code → 401.
- T-111e stores each recovery code as an Argon2 hash in the ERD-mapped identity.totp_recovery_codes table with used_at.
- Reuse the existing LoginService, Clock, tenant transaction and session creation path.

## Scope
### In
- Verify a submitted recovery code against unused hashes, mark exactly one matching row used in the login transaction, and continue through normal login success/audit/session behavior.
- Implement the original named totp_flow acceptance test.
### Out
- Generating or displaying recovery codes (T-111e).

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/service.rs | modify | Consume one recovery code during TOTP verification. |
| crates/eq-identity/src/repo.rs | modify | Atomically match and mark one unused hash. |
| crates/eq-identity/tests/api_me.rs | modify | End-to-end TOTP/recovery-code login flow. |
| tasks/M1-core/T-110h-recovery-code-login.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
The existing login request accepts the TOTP input field; when TOTP is enabled, a valid unused recovery code may be supplied in its place. Reuse returns the existing 401 totp_required/invalid-credentials behavior defined by Identity.

## Business rules
One successful recovery-code login consumes one code in the same transaction. No new policy field or retry count is introduced.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 recovery-code hashes and submitted secret.
- S/E: replay or cross-tenant code use → tenant/person-scoped atomic consume within the authentication transaction (SBD-40/41/42, STM-25).
- I: code disclosure → Argon2 verification, no raw code persistence or logging, redacted request Debug (SBD-10/21/50).
- Abuse-case acceptance criterion: totp_flow proves first use succeeds and replay fails.

## Implementation notes
Do not change the event or error-code catalog. A submitted code is accepted only when it verifies against an unused hash belonging to the authenticated tenant/person.

## Acceptance criteria
- AC1. TOTP remains required without a valid TOTP or recovery code; a valid recovery code succeeds once and replay fails → totp_flow.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_me.rs | totp_flow |

## Definition of done
- [ ] totp_flow passes on the task DB in the existing PostgreSQL 16 container.
- [ ] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [ ] SQLx metadata is refreshed/checked after query changes; OpenAPI generation is attempted.
- [ ] Signed-off commit and complete Step 12 Report.

## Report
_Filled when this task starts, using the complete Step 12 structure from WORKFLOW.md._

## Questions
None known. Preserve the TOTP policy and schema decisions in T-111/T-111e.

## Review
n/a — pending review.
