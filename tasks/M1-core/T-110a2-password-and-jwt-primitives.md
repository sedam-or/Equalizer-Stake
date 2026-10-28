---
id: T-110a2
title: Add password and JWT primitives
milestone: M1
status: todo
depends_on: [T-110a1c, T-102a, T-103]
spec_refs: [IDN-02, IDN-03, SBD-50, SBD-81, CQ-23, CAN-10]
estimated_size: M
---

## Goal
Provide the password validation/hash and access-token primitives used by identity login. Use the exact Argon2id and JWT contract in spec 24, spec/modules/identity.md and T-110a.

## Context
- Argon2id parameters are `m=19456 KiB`, `t=2`, `p=1`; minimum length comes from `IdentityPolicy.password_min_length`, maximum is 128 Unicode scalar values, and common passwords use the bundled SecLists/NCSC top-10k list (MIT).
- JWT uses HS256 and `Keys.jwt`, a 600-second TTL, and claims `sub`, `tid`, `role`, `sid`, `iat`, `exp`, `ver`; `ver` is the credential password-change epoch or 0 when absent. Verify signature, expiry with 30-second leeway, tenant, and version.
- T-103 exposes `eq_core::crypto::Keys` and `bidx_email`; password values use T-102a's secrecy-backed `Password` type.

## Scope
### In
- Implement new-password validation, Argon2id hash/verify, dummy verification, and JWT issue/validation helpers.
- Preserve exact unit test names from T-110a.
### Out
- Credential persistence, login routes, refresh-token rotation and Actor permission computation.

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.toml`, `Cargo.lock`, `crates/eq-identity/Cargo.toml` | modify | Declare only spec-mapped Argon2/JWT dependencies. |
| `crates/eq-identity/src/password.rs`, `crates/eq-identity/src/jwt.rs` | create | Password and access-token primitives. |
| `crates/eq-identity/data/common-passwords.txt` | create | Bundled top-10k list with its MIT source notice. |
| `crates/eq-identity/src/lib.rs` | modify | Export the primitives. |
| `tasks/M1-core/T-110a2-password-and-jwt-primitives.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Task report, index and any traced corrections. |

## Interfaces
- Use Argon2 crate 0.5, password-hash, and jsonwebtoken 9 from spec 24.
- JWT claims and tenant/version checks are exactly as in Context. Do not put permissions or PII in the token.
- `validate_new_password(password, policy)` returns the registered `password_weak` validation error for a short or common password; login verification does not reapply the current minimum length.

## Business rules
The password minimum comes from the loaded IdentityPolicy; max 128 and Argon2 parameters are technical/security constants specified by IDN-02.

## Security
- Triggers: T1, T3 (SBD-30).
- Data classes touched: S3 password and token material.
- Secret-bearing values use secrecy/zeroizing storage and handwritten redacting Debug; no Clone/Serialize/plain Debug derives (SBD-50).
- Abuse tests: reject expired and wrong-tenant tokens and ensure password debug never contains `test-secret-…` (SBD-31/73/81).

## Implementation notes
1. Check the repository `deny.toml` before adding any dependency; never add an exception.
2. Keep dummy password verification on the same Argon2 parameters as real verification.
3. Use the injected clock in token APIs; do not call `Utc::now()` in domain code.

## Acceptance criteria
- AC1. `password123` is rejected for minimum 10 and password length/hash behavior meets IDN-02 → `common_rejected`, `max_len_128`, `argon_params`.
- AC2. JWT roundtrip works; expired and wrong-tenant claims are rejected → `roundtrip`, `expired_rejected`, `wrong_tenant_rejected`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/password.rs` | `common_rejected`, `max_len_128`, `argon_params` |
| `crates/eq-identity/src/jwt.rs` | `roundtrip`, `expired_rejected`, `wrong_tenant_rejected` |

## Definition of done
- [ ] All six named unit tests pass.
- [ ] `cargo fmt --all --check`, `cargo deny check`, task lint/index and applicable guards pass; build checks prohibited by resource policy are `Not run`.
- [ ] Secret formatting test confirms fake password and key material are redacted.
- [ ] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [ ] Signed commit `feat(identity): add password and JWT primitives [T-110a2]` pushed to `dev`.

## Report
**Plan** — Implement the exact password and JWT contracts with bounded secret handling, add the listed unit tests, and run available format/dependency/guard checks.
**Assumptions** — Password normalization is performed by the shared field pipeline; login accepts existing passwords without enforcing a newer minimum.
**Test plan** — Exercise min−1/min/max/max+1, common-password rejection, hash parameters, JWT signature, expiry and tenant mismatch.

## Questions
None.

## Review
n/a — pending implementation.
