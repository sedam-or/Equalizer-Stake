---
id: T-110a2
title: Add password and JWT primitives
milestone: M1
status: review
depends_on: [T-110a1c, T-102a, T-103]
spec_refs: [IDN-02, IDN-03, ERR-21, ERR-31, SBD-30, SBD-50, SBD-81, CQ-23, CAN-10]
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
| `crates/eq-identity/src/errors.rs` | create | Register the Identity-owned `password_weak` code. |
| `crates/eq-identity/src/password.rs`, `crates/eq-identity/src/jwt.rs` | create | Password and access-token primitives. |
| `crates/eq-identity/data/common-passwords.txt` | create | Bundled top-10k list with its MIT source notice. |
| `crates/eq-identity/src/lib.rs` | modify | Export the primitives. |
| `tasks/M1-core/T-110a2-password-and-jwt-primitives.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Task report, index and any traced corrections. |

## Interfaces
- Use Argon2 crate 0.5, password-hash, and jsonwebtoken 9 from spec 24.
- JWT claims and tenant/version checks are exactly as in Context. Do not put permissions or PII in the token.
- `validate_new_password(password, policy)` returns a typed error linked to the registered Identity-owned 422 `password_weak` code for a short or common password; login verification does not reapply the current minimum length.
- `password_weak` is an application problem code, not one of ERR-31's closed shared field-issue codes.

## Business rules
The password minimum comes from the loaded IdentityPolicy; max 128 and Argon2 parameters are technical/security constants specified by IDN-02.

## Security
- Triggers: T3, T4 (SBD-30).
- Data classes touched: S3 password and token material.
- Secret-bearing values use secrecy/zeroizing storage and handwritten redacting Debug; no Clone/Serialize/plain Debug derives (SBD-50).
- Abuse tests: reject expired and wrong-tenant tokens and ensure password debug never contains `test-secret-…` (SBD-31/73/81).

## Implementation notes
1. Check the repository `deny.toml` before adding any dependency; never add an exception.
2. Generate one random dummy hash during login-service initialization and pass it to the dummy verifier; dummy verification uses the same Argon2 parameters as real verification.
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
- [ ] All six named unit tests pass. Not run: the fixed Cargo target directory is absent and the active resource rule prohibits creating it.
- [x] `cargo fmt --all --check`, `cargo deny check`, task lint/index and applicable guards pass; build checks prohibited by resource policy are `Not run`.
- [ ] Secret formatting test confirms fake password and key material are redacted. Test is written but not run because Rust test execution is prohibited by the fixed-target restriction.
- [x] Report includes Workflow steps 0–12 and required disk/resource before/after data.
- [x] Signed commit `feat(identity): add password and JWT primitives [T-110a2]` pushed to `dev`.

## Report
**Plan** — (1) Add the Identity `password_weak` code to its ERR-21 registry → verify code and status. (2) Declare the spec-mapped Argon2, `password-hash` and JWT dependencies at workspace scope → verify the resolved lockfile and unchanged deny policy. (3) Bundle the licensed SecLists top-10k corpus with the single AC1 supplement, then implement common-password validation, Argon2id hash/verify and dummy helpers → verify password bounds, common-password rejection and m=19456 KiB/t=2/p=1 in the named tests. (4) Implement the exact HS256 wire claims and injected-clock issue/validation helpers → verify roundtrip, expiry, tenant and credential-version behavior. (5) Keep stored hashes and access tokens in secrecy-backed types with handwritten redacted `Debug`, and return JWT claims through a private validated type → verify fake credential and key material are absent from formatting tests and callers cannot construct or deserialize unvalidated claims. (6) Run formatting, dependency, metadata, guard, task and secret checks; record build and test commands as Not run because the fixed target is absent and the resource rule forbids creating it.

**Assumptions** — T-102a's shipped `Password` is NFKC-normalized, private and redacted, and enforces the 128-Unicode-scalar maximum. `IdentityPolicy.password_min_length` supplies the only new-password minimum; no composition rule is added. `password123` is the one documented corpus supplement required by AC1 because it is absent from the selected SecLists snapshot. `password_weak` is an Identity-owned 422 application code; ERR-31's shared field issue set remains unchanged. Login verification does not enforce a newer minimum. The login service constructs one random dummy hash at initialization and reuses it for unknown identities.

**Test plan** — AC1: Given the default policy and `password123`, when new-password validation runs, then the registered 422 code is returned → `common_rejected`. Given 128 and 129 Unicode-scalar passwords, when the shared parser runs, then it accepts the former and rejects the latter → `max_len_128`. Given the configured Argon2id implementation, when parameters and hash/verify/dummy helpers are exercised, then m=19456 KiB/t=2/p=1 is used and matches/mismatches/dummy work behave as specified → `argon_params`, `hash_and_dummy_verification`. AC2: Given valid claims and a fixed clock, an issued token validates and roundtrips → `roundtrip`; at expiry plus the specified leeway, validation rejects → `expired_rejected`; with a different tenant, validation rejects → `wrong_tenant_rejected`. Additional tests cover credential-version mismatch, invalid signature, and redacted token/key formatting. Edge cases considered: minimum boundary, NFKC and Unicode scalar length, malformed PHC, unknown JWT claims and timestamp overflow. Error and abuse cases: weak password uses `password_weak`; malformed stored hashes and invalid JWTs fail without exposing material; expiry, wrong tenant, stale version, and secret formatting are covered by tests. All Rust tests are written but were not executed under the target restriction.

**Summary** — Added identity-owned weak-password registration, a bundled and licensed common-password corpus, Argon2id password primitives, and HS256 access-token issue/validation helpers. Password hashes and bearer tokens use secrecy-backed wrappers with redacted formatting; JWT validation checks signature, expiry/leeway, tenant and password version, then returns claims that external callers cannot construct or deserialize. The implementation has not been Rust-built or executed in this run because the approved Cargo target is absent and creating one is prohibited.

**Spec read** — `spec/modules/identity.md` §§1 (IDN-02/03/06); `spec/24-open-source-stack.md` rows for Argon2/password-hash, JWT, secrets and common-password lists; `spec/18-security-by-design.md` §§4, 5.3 and 8 (SBD-30/49/50/81); `spec/16-code-quality.md` §§9, 12 (CQ-22/23/27); `spec/17-clean-architecture.md` §§1, 4 and 7; `spec/19-error-handling.md` §§3, 5 and 10 (ERR-21/31); `spec/23-contracts-and-canonical-forms.md` §2 (CAN-10). Also read the complete task and the dependency Reports and shipped code listed under Workflow Step 2.

**Files** —
- `Cargo.toml`, `Cargo.lock`, `crates/eq-identity/Cargo.toml` — workspace-mapped Argon2, password-hash, JWT, secrecy and random/zeroization dependencies.
- `crates/eq-identity/src/errors.rs`, `crates/eq-identity/src/lib.rs` — Identity error registry and module exports.
- `crates/eq-identity/src/password.rs`, `crates/eq-identity/src/jwt.rs` — password and access-token primitives with named tests.
- `crates/eq-identity/data/common-passwords.txt` — SecLists MIT notice and embedded top-10k corpus with the single documented supplement.
- `tasks/M1-core/T-110a2-password-and-jwt-primitives.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` — report, RATIFY decisions and generated status index.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1 common password rejection | `common_rejected` | Not run: fixed Cargo target is absent; creating it is prohibited. Test is written. |
| AC1 maximum password length | `max_len_128` | Not run for the same fixed-target restriction. Test is written. |
| AC1 Argon2id parameters | `argon_params` | Not run for the same fixed-target restriction. Test is written. |
| AC2 token issue and roundtrip | `roundtrip` | Not run for the same fixed-target restriction. Test is written. |
| AC2 expiry and leeway | `expired_rejected` | Not run for the same fixed-target restriction. Test is written. |
| AC2 tenant binding | `wrong_tenant_rejected` | Not run for the same fixed-target restriction. Test is written. |

**Checks run** —
- ✅ `rustfmt --edition 2024 --config skip_children=true crates/eq-identity/src/errors.rs crates/eq-identity/src/jwt.rs crates/eq-identity/src/lib.rs crates/eq-identity/src/password.rs` and `cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate-`base64` and unmatched-`CC0-1.0` warnings remain. No deny policy was changed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --offline --locked --format-version 1 --no-deps`.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, and `python3 -I scripts/tests/test_tasks_lint.py` — 8/8 task-lint fixtures passed.
- ✅ `python3 -I scripts/erd.py --check` — ERD matches the DDL; no migration changed.
- ✅ `git diff --cached --check` after staging only the T-110a2-owned paths.
- ✅ Per-file `gitleaks dir --redact=100 --no-banner <path>` scans on all T-110a2-owned source, data, manifest and task files — no findings.
- ⚠️ `gitleaks dir --redact=100 --no-banner .` — one existing `curl-auth-user` finding at `Makefile:80` (fingerprint `Makefile:curl-auth-user:80`), previously recorded in dependency Reports; Makefile is outside this task and unchanged.
- Not run: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`, `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace`, and the six named unit tests — `$HOME/.cache/ritma/target` is absent and the active resource rule prohibits creating a target directory.
- n/a: `make dev`, database tests, `cargo sqlx prepare --workspace` and `scripts/erd.py --migrations` — no route, SQL query or migration changed; no database used.
- n/a: web lint/typecheck/tests/i18n and `make api` — no web or OpenAPI files changed.
- n/a: `make e2e` — this primitive task adds no user journey or runtime endpoint; database used: none; E2E pass count: 0/0.
- Not run: hosted GitHub Actions; a hosted runner is unavailable in this session.

**Manual checks** — Reviewed the shipped `Password` field parser, `IdentityPolicy.password_min_length`, `Keys.jwt` secret wrapper, password/JWT claim shapes, dependency lock entries and test bodies. Confirmed the common list includes its MIT notice and one AC-required supplement; verified new production Rust files contain no `dbg!`, `todo!`, `unimplemented!`, `println!`, `eprintln!` or `#[allow]`. No server, database or browser flow was exercised.

**Debug log** — Rust formatting initially required a formatting correction; the rerun passed. The repository-wide Gitleaks scan found the pre-existing Makefile finding at line 80; the recorded fingerprint matches the prior known baseline, while per-file scans of task-owned files found no leaks. No Rust test failure was observed because tests could not be started without creating the prohibited Cargo target.

**Performance** — n/a; this task adds no request path, database query or benchmark target. Argon2id cost is fixed by IDN-02 and source-verified; runtime timing was not measured because Rust execution was unavailable.

**Security** — Triggers T3 and T4 (RATIFY TD-106). SBD-81: (1) no handler or permission chain is added; (2) no tenant input, SQL, RLS or platform access is added; (3) passwords, hashes, JWTs and signing keys are S3/auth material, held in secrecy/zeroizing wrappers where applicable, redacted in handwritten Debug, not logged or serialized; (4) the shipped Password parser enforces NFKC and the 128-scalar bound, while policy validation uses the loaded minimum; JWT claims deny unknown fields and validation checks the signed tenant/version; (5) no HTML, CSV or rendered output; (6) Argon2id, `password-hash`, HS256 and `Keys.jwt` follow IDN-02/03, with no token persistence; (7) no external calls; (8) no aggregate mutation, idempotency or versioned write; (9) no security event or audit mutation; (10) tests are written for expired/wrong-tenant/stale-version tokens and redaction, but execution is Not run under the target restriction. Targeted Gitleaks scans passed; the full-repo scan reported only the unchanged Makefile baseline finding.

**Deviations** — RATIFY TD-106: corrected security triggers to T3/T4 per SBD-30. RATIFY TD-107: register `password_weak` as the Identity-owned 422 application code and keep the shared ERR-31 field-code set closed. RATIFY TD-108: wrap password hashes and access tokens in secrecy-backed, redacted types per SBD-50. RATIFY TD-109: preserve the SecLists corpus and add the one AC1-required `password123` supplement. The private validated claims type applies CQ-23 to the task contract. The explicit `password-hash` workspace dependency follows the task Interfaces and spec 24 mapping. No route, permission, schema, migration, business rule or deny exception was added.

**Dependencies affected** — T-110a3 consumes password validation, hashing, verification and dummy-verification helpers; later login/token tasks can use the HS256 token helpers. No shipped dependency behavior changed.

**Rollback plan** — Revert the single signed T-110a2 commit. No migration or persisted data change is involved.

**Follow-ups** — Human: ratify TD-106–TD-109; review the approximately 10,937-line staged diff against CQ-101's about-400-line guidance (the task requires the bundled common-password corpus); run the identity unit tests plus workspace Clippy/tests when the fixed target restriction is lifted; review the included password corpus and the existing Makefile Gitleaks baseline. T-110a4 remains responsible for HTTP/API error translation and localized handling of `password_weak`.

**Resource report** — Before (2026-10-29T01:54:23Z): `/home` had 80G free, `$HOME/.cache/ritma/target` was absent, and the last inventory showed 8 Docker containers; no target/database/container was created or removed. After (2026-10-29T02:19:53Z): `/home` had 80G free and the fixed target remained absent. `docker system df` was queried read-only and timed out; no Docker resource was created or removed by this task.

**Workflow steps 0–12** — 0: resumed the preserved in-progress T-110a2 on `dev`, retained all unrelated WIP, confirmed dependencies are `review`, and checked the task index. 1: read the full task and cited identity, error, security, code-quality and canonical-form specs listed above. 2: read T-110a1c, T-102a and T-103 Reports (deviations, follow-ups, RATIFY items) and confirmed the shipped `IdentityPolicy`, `Password` and `Keys.jwt` interfaces. 3: n/a, backend-only task. 4: recorded the plan, assumptions, AC mapping and edge/error/abuse test plan before implementation. 5: added the error registry, password/JWT primitives, licensed corpus, dependency declarations and required tests within Scope. 6: n/a; no route or runtime endpoint is added, and Rust tests/server execution are barred by the fixed-target rule. 7: formatting was corrected and rerun green; the repository-wide Gitleaks finding was checked against the known Makefile baseline, with task-owned file scans clean. 8: reviewed the diff against spec 16 §9, spec 17 §7 and spec 19; checked secret handling, error registration, claim contents, construction/deserialization boundaries and production-code hygiene. 9: n/a; no query or measurable endpoint was added. 10: completed the SBD-81 checklist above. 11: ran all applicable static checks above; Clippy, Rust tests, `make dev` and database work remain explicitly Not run. 12: set `review`, regenerated and checked the index, made one signed task commit, and pushed it to `dev`.

## Questions
- **Self-resolved (implementer, run 7, needs architect ratification; TD-106):** this task lists T1/T3 despite adding no endpoint and implementing S3 authentication material → use T3/T4 because SBD-30 assigns T3 to S2/S3 data and T4 to auth, sessions, tokens and crypto.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-107):** the task asks for a registered `password_weak` validation error, while ERR-31's closed shared field-code set excludes it and the Files table omits the Identity registry → treat `password_weak` as the Identity-owned 422 application code already named by the identity endpoint contract, register it in `crates/eq-identity/src/errors.rs`, and keep it out of shared field issues; ERR-21/31 define this boundary.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-108):** password-hash and access-token return shapes are unspecified even though they carry credential material → store them in secrecy-backed wrappers with handwritten redacting Debug and no Clone/Serialize/plain Debug derives, as pre-authorized by SBD-50.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-109):** AC1 requires rejecting `password123` at policy minimum 10, but the exact SecLists 10k source snapshot does not contain it → retain the source corpus and add only the AC-named password as a documented supplement, because the task's stricter explicit acceptance condition must remain satisfied without adding composition rules.

## Review
n/a — awaiting maintainer review.
