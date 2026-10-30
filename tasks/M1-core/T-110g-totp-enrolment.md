---
id: T-110g
title: Implement TOTP enrollment and recovery codes
milestone: M1
status: todo
depends_on: [T-110f]
spec_refs: [IDN-12, STM-25, SEC-12, ERD-03]
estimated_size: M
---

## Goal
Allow a person to set up and confirm TOTP, show ten one-time recovery codes, and disable TOTP only under the state-machine guards.

## Context
- T-111 defines TOTP setup, confirmation and disable. Setup stores totp_secret_enc encrypted with totp_enabled false; confirm verifies a 6-digit code and enables TOTP, then shows ten 10-character recovery codes whose argon2 hashes are stored in totp_recovery_codes.
- Migration 0008 must exactly match spec/06 §4.12 and the T-111 row in spec/29 §7, with RLS.
- STM-25 t4 requires disable to check both current password and TOTP code, clear secret/recovery codes, audit security.totp.disabled and revoke other sessions. It refuses disable under RequiredForOwnerAdmin for owner/admin.
- totp-rs 6 is the mapped dependency; its qr feature provides QR PNG output.

## Scope
### In
- Implement setup/confirm/disable, recovery-code generation/storage, and migration 0008 exactly as mapped.
- Return the declared otpauth_url, qr_svg and secret_base32 fields.
### Out
- Accepting recovery codes during login (T-111f).

## Files
| Path | Action | Purpose |
|---|---|---|
| migrations/0008_identity_totp_recovery.sql; .sqlx/ | create/modify | Exact ERD-mapped recovery-code table/RLS and offline query metadata. |
| crates/eq-identity/src/api_me.rs | modify | TOTP request/response DTOs and handlers. |
| crates/eq-identity/src/service.rs | modify | TOTP enrollment and disable service behavior. |
| crates/eq-identity/src/repo.rs | modify | Recovery-code persistence. |
| crates/eq-identity/src/module.rs | modify | Register TOTP routes. |
| Cargo.toml and Cargo.lock | modify | Enable the mapped totp-rs qr feature. |
| crates/eq-identity/tests/api_me.rs | modify | TOTP enrollment/disable tests. |
| tasks/M1-core/T-110g-totp-enrolment.md | modify | Plan and final Report. |

## Interfaces
- POST /api/v1/me/totp/setup → {otpauth_url, qr_svg, secret_base32}.
- POST /api/v1/me/totp/confirm {code} → {recovery_codes}.
- DELETE /api/v1/me/totp accepts current password and the current 6-digit code per STM-25; required-policy denial uses policy_violation rule identity.totp.

## Business rules
TOTP policy is IdentityPolicy.totp. The code time window and duration follow totp-rs/spec/25. Recovery-code count and character length are exactly the task's ten codes of ten characters.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 encrypted TOTP secret and Argon2 recovery-code hashes.
- S/E: unauthorized setup/disable or cross-person access → Actor ownership and policy/state-machine checks (SBD-40/41/42, STM-25).
- I: seed/recovery-code leakage → encrypt seed with Keyring, use secret-safe Debug, show recovery codes once, persist only Argon2 hashes and never log secrets (SBD-10/21/23/50).
- Abuse cases: reject incorrect code, replay a used recovery code in T-111f, and reject disable when policy requires TOTP.

## Implementation notes
Use exact schema from spec/06 §4.12; run scripts/erd.py --migrations. Enable only the totp-rs qr feature named in spec/24; no new dependency or migration object.

## Acceptance criteria
- AC1. Setup stores a pending encrypted seed and returns an otpauth URL, QR SVG and base32 seed → totp_setup_confirm.
- AC2. Correct confirmation enables TOTP and returns ten 10-character codes once; disable requires the current password and TOTP code and obeys STM-25 → totp_setup_confirm and totp_disable_requires_code.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_me.rs | totp_setup_confirm, totp_disable_requires_code |

## Definition of done
- [ ] scripts/erd.py --migrations and --check pass.
- [ ] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [ ] SQLx offline metadata is refreshed and checked; OpenAPI generation is attempted and T-114 placeholder recorded as Not run.
- [ ] Signed-off commit and complete Step 12 Report.

## Report
_Filled when this task starts, using the complete Step 12 structure from WORKFLOW.md._

## Questions
None known. The parent T-111 task records the STM-25 input correction and QR representation decision.

## Review
n/a — pending review.
