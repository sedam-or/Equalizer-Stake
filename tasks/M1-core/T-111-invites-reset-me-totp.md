---
id: T-111
title: Add invites, password reset, /me endpoints, TOTP enrolment and device sessions
milestone: M1
status: review
depends_on: [T-110c, T-110d, T-110e, T-110f, T-110g, T-110h]
spec_refs: [IDN-07, IDN-08, IDN-12, IDN-13, SEC-12, STM-21, STM-24, STM-25, AUTH-06]
estimated_size: S
---

## Goal
No password is ever sent. People join by invite link and recover by single-use reset link. Every person can manage their profile, password, TOTP and devices.

## Split
This aggregate owns final verification and reporting only. Its implementation is split into six ordered tasks because the original scope combines invite issuance and acceptance, password reset, profile/session management, and TOTP/recovery-code login across multiple security boundaries. After all six parts are review, promote this aggregate to todo, verify the combined acceptance criteria and reports, then set it to review. T-112 and T-115 continue to depend on this aggregate.

| Part | Scope | Acceptance tests |
|---|---|---|
| T-110c | Issue/resend invites and the service contract consumed by T-112 | resend_revokes_previous |
| T-110d | Read and accept invite links | accept_ok, accept_expired_410 |
| T-110e | Password reset and non-enumerating recovery | forgot_no_enumeration, forgot_rate_limited_silently, reset_revokes_sessions |
| T-110f | Own profile, password and device sessions | me_patch_requires_if_match |
| T-110g | TOTP setup/confirmation/disable and recovery-code issuance | totp_setup_confirm |
| T-110h | Single-use recovery-code login | totp_flow |

## Context
- IDN-07 invites:
  - `create_invite(tx, actor, person_id, role, channel)`: revokes earlier unaccepted invites for (person, role) by setting `expires_at = now`. It creates a 32-byte random token (sha256 stored) with `expires_at = now + invite_expires_days`, sets the membership `status='invited'` (insert if missing), emits `identity.user_invited.v1 {person_id, role, invite_id}`, and audits `identity.invite.created`.
  - The URL is `{RITMA_PUBLIC_URL}/{lang}/invite/{token}`, where `lang` is the person's locale, then the tenant locale.
  - Channel `link` returns `invite_url` in the response (admin copies it). `email`/`whatsapp` don't return it. `identity.user_invited.v1` contains only `{person_id, role, invite_id}`; until a secure token handoff is specified, M5 Notify cannot construct a usable link from this event and delivery remains a follow-up (TD-137, TD-163).
  - `GET /auth/invites/{token}` → `{tenant_name, display_name, role, expires_at}`. Unknown → 404. Expired → 410 `invite_expired`. Accepted → 410.
  - `POST /auth/invites/{token}/accept {password}`: validates the password per policy, sets `password_hash` and `password_changed_at`, sets membership `active` and `accepted_at`, and returns the login response plus the cookie. Audit `identity.invite.accepted`.
- IDN-08 reset:
  - `POST /auth/forgot {email}` always returns 202, in constant time. If the person exists and has an active membership, and there were fewer than `reset_max_per_hour` resets in the last hour (count `password_resets` rows), create a token with TTL `reset_token_ttl_minutes` and emit `identity.password_reset_requested.v1 {person_id, token_id}`. The raw token is not put in the event payload; a secure Notify handoff remains a follow-up (TD-143, TD-163). The outbox is a server-side table, never exposed.
  - `POST /auth/reset {token, new_password}` → 204. It sets the password, marks the token used, revokes **all** refresh sessions of the person, and audits `security.password.reset`. Invalid, expired or used token → 400 `token_invalid`. Weak password → 422 `password_weak`.
- `/me`:
  - `GET /me` → `PersonMe {id, display_name, email, phone, locale, avatar_url, roles, active_role, permissions, linked_students?}`. Email and phone are decrypted (own data, no PII audit). `linked_students` is present for guardians.
  - `PATCH /me {display_name?, phone?, locale?}`. Phone is normalised to E.164 (T-103), with a new bidx. Uses `If-Match` on `persons.version` (API-07).
  - `POST /me/password {current_password, new_password}` → 204. Revokes all **other** refresh sessions. Audit `security.password.changed`.
- IDN-12 TOTP:
  - `POST /me/totp/setup` → `{otpauth_url, qr_svg, secret_base32}`. Stores `totp_secret_enc` (pending; `totp_enabled` stays false).
  - `POST /me/totp/confirm {code}` → `{recovery_codes: [10 × 10 chars]}`. Sets `totp_enabled=true` and stores argon2 hashes of the recovery codes in a new table `totp_recovery_codes(person_id, tenant_id, code_hash, used_at)`, added in `migrations/0008_identity_totp_recovery.sql` with RLS (DDL verbatim from `spec/06-domain-model.md` §4.12; PK `(person_id, code_hash)`; `code_hash` is S3).
  - `DELETE /me/totp {password}` disables it, unless the policy is `RequiredForOwnerAdmin` and the person holds owner/admin → 422 `policy_violation` rule `identity.totp` (ERR-10; STM-25 t4).
  - Login accepts a recovery code in place of `totp` (single use).
- Sessions: `GET /me/sessions` → `[{id, created_at, last_used_at, user_agent, ip, current: bool}]` (only non-revoked, non-expired). `DELETE /me/sessions/{id}` revokes it (its family).
- IDN-13: deactivating a membership revokes all refresh sessions for that person in the tenant (T-112 calls `AuthService::revoke_all(person)`, implemented here).
- Invite, reset and TOTP flows reveal nothing about whether an account exists (AUTH-06); the full flow table is spec 25 §4.

## Scope
### In
- Final aggregate verification and reporting for T-110c–T-110h: check the exact acceptance tests and Reports against AC1–AC8, verify migration `0008` against the schema map, and record cross-part deviations and follow-ups.
- i18n-free (server returns codes only).

### Out
- Reimplementing endpoints, services, migration `0008` or DTOs already owned by T-110c–T-110h; creating a raw-token event or inventing a token-delivery handoff; sending email or WhatsApp (M5 Notify); the people admin API (T-112).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/tests/api_invites.rs` | verify | AC1–AC3 are implemented by T-110c/T-110d. |
| `crates/eq-identity/tests/api_reset.rs` | verify | AC4–AC6 are implemented by T-110e. |
| `crates/eq-identity/tests/api_me.rs` | verify | TOTP setup and profile precondition behavior are implemented by T-110f/T-110g. |
| `crates/eq-identity/tests/api_auth.rs` | verify | AC7 recovery-code login is implemented by T-110h. |
| `migrations/0008_identity_totp_recovery.sql` | verify | Check the T-110g migration against the T-111 ERD mapping. |
| `tasks/M1-core/T-111-invites-reset-me-totp.md` | modify | Aggregate plan and final Report. |
| `tasks/DECISIONS.md` | modify | Record TD-163 for the aggregate's stale event wording. |
| `tasks/INDEX.md` | generate | Regenerate task status index. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3, T4 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `forgot_rate_limited_silently` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Invite accept within the TTL → login response, membership `active`. → `api_invites::accept_ok`
- AC2. Invite older than `invite_expires_days` → 410 `invite_expired`. → `accept_expired_410`
- AC3. Resending an invite makes the old token 410. → `resend_revokes_previous`
- AC4. `/auth/forgot` for unknown and known emails → both 202, and only the known one creates an outbox event. → `forgot_no_enumeration`
- AC5. The 4th reset request in an hour (policy 3) creates no token, still 202. → `forgot_rate_limited_silently`
- AC6. A reset revokes all refresh sessions. → `reset_revokes_sessions`
- AC7. TOTP setup → confirm with a correct code → login without TOTP → 401 `totp_required`. With the code → 200. With a recovery code → 200, and the same recovery code again → 401. → `totp_flow`
- AC8. `PATCH /me` without `If-Match` → 428. With a stale one → 409 `version_conflict`. → `me_patch_requires_if_match`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_invites.rs` | `accept_ok`, `accept_expired_410`, `resend_revokes_previous` |
| `crates/eq-identity/tests/api_reset.rs` | `forgot_no_enumeration`, `forgot_rate_limited_silently`, `reset_revokes_sessions` |
| `crates/eq-identity/tests/api_me.rs` | `me_patch_requires_if_match`, `totp_setup_confirm` |
| `crates/eq-identity/tests/api_auth.rs` | `totp_flow` |

## Definition of done
- [x] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass
- [ ] Rust checks pass; `make api` regenerated — full workspace tests are not run under the fixed-container rule; `make api` remains the T-114 placeholder (TD-164).
- [x] Commit `feat(identity): invites, reset, profile and TOTP [T-111]`, signed off

## Report

**Plan** — 1. Reconcile this aggregate's Scope, Files and event wording with T-110c–T-110h and their recorded decisions → verify the contract matches the shipped IDs-only events. 2. Read every dependency Report and inspect its acceptance tests and implementation boundaries → map AC1–AC8 to the exact named tests and their recorded results. 3. Check migration `0008` against `spec/06` and `spec/29`, then run schema, Rust static, dependency and guard checks using the fixed Cargo cache → verify runnable checks and record the container limit. 4. Finish this aggregate Report and run task index/lint, diff and staged secret checks → verify only T-111 documentation and generated index files are committed.

**Assumptions** — T-111 is the final aggregate/report task defined by TD-134; implementation belongs to T-110c–T-110h. All six dependencies are `review`, and their Reports are the execution evidence for the named acceptance tests. No implementation or migration change is needed here. Re-running child database tests would exceed the one-container/resource boundary and their fixtures assert their own task DB names. T-114 owns OpenAPI generation; this aggregate does not add generated API/client files.

**Test plan** — AC1: Given a valid invite within the policy TTL, when accepted, then login succeeds and membership is active → `api_invites::accept_ok`. AC2: Given expired or accepted invite tokens, when preview/accept is attempted, then the API returns 410 `invite_expired` → `accept_expired_410`. AC3: Given a pending invite, when resent, then the earlier token is expired → `resend_revokes_previous`. AC4: Given known and unknown emails, when forgot-password runs, then both return 202 and only an eligible known account creates an IDs-only event → `forgot_no_enumeration`. AC5: Given the reset request cap has been reached, when the next request arrives, then response remains 202 and no token is created → `forgot_rate_limited_silently`. AC6: Given a valid reset token and sessions, when reset completes, then the token is consumed and all sessions are revoked → `reset_revokes_sessions`. AC7: Given TOTP enrollment, when setup, confirmation and login run, then missing factor is rejected, valid TOTP/recovery succeeds, and recovery replay fails → `totp_setup_confirm` and `totp_flow`. AC8: Given a profile version, when PATCH omits or supplies a stale ETag, then it returns 428/409 → `me_patch_requires_if_match`. Edge/error/abuse coverage is recorded in child Reports: invalid and replayed tokens, rate caps and non-enumeration, stale versions, TOTP policy, recovery replay, cross-tenant isolation, secret redaction and transaction rollback.

**Summary** — T-111 now serves as the aggregate verification record for the six identity implementation tasks. All eight acceptance criteria map to the required existing tests, and their Reports record successful runs on task-scoped databases. No identity code or schema was changed by this aggregate.

**Spec read** — `spec/03-module-system.md` event catalog; `spec/06-domain-model.md` §§4.1–4.2, §4.12; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§4, 5, 8; `spec/19-error-handling.md` §10; `spec/20-state-machines.md` STM-20–25; `spec/21-data-flows.md` FLW-12/60; `spec/22-validation-and-sanitization.md`; `spec/23-contracts-and-canonical-forms.md` CAN-60/80; `spec/25-auth-and-access-control.md` AUTH-06 and §4; `spec/29-erd.md` Identity/Core dictionaries, §5 and §7.

**Files** — Modified `tasks/M1-core/T-111-invites-reset-me-totp.md` (aggregate contract and final Report), `tasks/DECISIONS.md` (TD-163 and TD-164), and generated `tasks/INDEX.md`. No source code, migration, SQLx metadata or generated API/client file changed.

**Acceptance criteria** —

| AC | Exact test(s) | Evidence |
|---|---|---|
| AC1: invite acceptance activates membership and returns login | `api_invites::accept_ok` | Pass, 1/1; T-110d `api_invites` suite 7/7 on fresh `ritma_test_T-110d` |
| AC2: expired/accepted invite is rejected and replay does not create a second session | `api_invites::accept_expired_410` | Pass, 1/1; T-110d suite 7/7 |
| AC3: resend expires prior pending invite | `resend_revokes_previous` | Pass; T-110c `api_invites` suite 2/2 on fresh `ritma_test_T-110c` |
| AC4: forgot response does not enumerate addresses | `forgot_no_enumeration` | Pass, 1/1; T-110e `api_reset` suite 3/3 on fresh `ritma_test_T-110e` |
| AC5: capped request stays silent and creates no token | `forgot_rate_limited_silently` | Pass, 1/1; T-110e suite 3/3 |
| AC6: reset consumes token and revokes sessions | `reset_revokes_sessions` | Pass, 1/1; T-110e suite 3/3 |
| AC7: TOTP and recovery flows work; recovery replay fails | `totp_setup_confirm`, `totp_flow` | Pass, 1/1 each; T-110g used fresh `ritma_test_T-110d`, T-110h used fresh `ritma_test_T-110h` (auth suite 20/20) |
| AC8: `/me` patch enforces If-Match | `me_patch_requires_if_match` | Pass, 1/1; T-110f `api_me` suite 3/3 on fresh `ritma_test_T-110d` |

**Checks run** —

- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --lib -- --skip lockout_threshold_emits_and_audits --skip locked_wrong_password_is_generic --skip successful_login_resets_failures --skip unknown_email_runs_dummy_hash` — 36 passed, 0 failed, 4 DB-backed tests filtered out.
- ✅ The exact acceptance tests and task database suites listed above passed in their dependency Reports. They used only the existing PostgreSQL 16 container `ritma-db-1`, with each fresh task database dropped afterward.
- ✅ `cargo deny check` — advisories, bans, licenses and sources pass; existing duplicate `base64` version warnings remain.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --migrations && python3 -I scripts/erd.py --check` — schema migration validation passed for 26 tables; the T-111 ERD row maps `0008_identity_totp_recovery.sql` to `totp_recovery_codes`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo sqlx prepare --check --workspace -- --all-targets` (recorded by the dependency task Reports).
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, `git diff --check`, and staged `gitleaks protect --staged --redact` (final results are recorded after status/index regeneration and staging).
- **Not run to completion:** `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --lib`. The attempt started an automatically named Testcontainers Postgres container; its exit result was not captured. The exact container `f8791c62b950` (`keen_lederberg`) and its attached anonymous volume were verified as created by this attempt and removed. The four tests that create `TestDb` were then excluded; the remaining 36 library tests passed. Do not treat the initial attempt as a pass.
- **Not run:** `cargo test --workspace`, to avoid testcontainers creating containers outside the fixed resource rule. The focused identity library tests and task-specific HTTP suites are reported above.
- **Not run:** `make api`; Makefile still prints `OpenAPI generation is added by T-114`, and this aggregate has no generated API/client files in Scope. This limitation is recorded as TD-164. `make dev` and another `make e2e` were not run because T-111 adds no runtime/UI code; live HTTP and E2E runs are documented in its dependency Reports. Web lint/typecheck/unit/i18n checks are not applicable because no web files changed. The child Reports for T-110d/e/h record `make e2e` at 4/6, Chromium local equivalent 3/3, and two WebKit page checks not run due missing host libraries `libicu74` and `libflite1`.
- `scripts/erd.py --migrations` is required and passed; no migration was changed by T-111.

**Manual checks** — n/a for this aggregate-only task; the dependency Reports document real HTTP requests through the local Axum router with `eq_app` and the applicable fresh task database, including invite resolution/activation, reset non-enumeration and session revocation, TOTP/recovery replay, cross-tenant isolation and audit rollback paths. T-110d specifically used `ritma_test_T-110d` inside existing `ritma-db-1` and dropped only that database after testing. T-111 itself created no database.

**Debug log** — The first focused `eq-identity --lib` attempt invoked tests whose `TestDb::new()` starts Testcontainers, contrary to the fixed resource rule. The process ended without a captured exit/result; the created container and its attached anonymous volume were identified and removed exactly. The four container-backed tests were then excluded by exact test name; 36 remaining tests passed. No other Docker resource was removed.

**Performance** — n/a; this aggregate changes no runtime code or query. Child T-110f measured its session query plan and reported it in its own Report.

**Security** — No code or data-class change in T-111. Dependency Reports verify Actor-derived tenant/person scope, `eq_app` RLS checks, parameterized tenant-scoped queries, password/reset/invite token hashing, secret redaction, S3-free IDs-only events, non-enumerating recovery, TOTP recovery single use, audit transactions, and cross-tenant/replay abuse cases. Shared API-15 per-user/per-IP rate limiting is still an explicit follow-up under TD-152; it is not reported as implemented. No real secrets or personal data were added.

**Deviations** — RATIFY TD-063, TD-066 and TD-067 are historical task-file corrections. TD-134 splits the oversized original scope into six implementation tasks and makes T-111 the final aggregate. TD-135's raw-token event proposal is superseded by T-110c TD-137 and T-110e TD-143; the spec-compliant event payloads remain IDs-only. TD-136 assigns child IDs T-110c–h. RATIFY TD-163 aligns this file with CAN-60/FLW-60. RATIFY TD-164 records the T-114 OpenAPI placeholder and leaves that check explicitly not run. Rationale and changed files are in `tasks/DECISIONS.md`.

**Dependencies affected** — No implementation dependency or source file changed. T-112 and T-115 continue to depend on this aggregate; T-114 remains responsible for OpenAPI/client generation.

**Rollback plan** — Revert the single signed T-111 commit. No schema migration, database or source-code change is part of this aggregate.

**Follow-ups** — Architect ratification of TD-063, TD-066–067, TD-134–137, TD-140–162, TD-163 and TD-164; decide the shared API-15 per-user/per-IP limiter (TD-152); implement a secure Notify handoff for invite/reset tokens that does not expose S3 event data (TD-137/143); add `invite_expired` translations in T-306; generate API/client artifacts in T-114; review the documented CQ-101 task-size deviations; rerun the two WebKit page checks on a host with `libicu74` and `libflite1`; rerun the full workspace test suite in a resource-permitted environment.

**Workflow steps 0–12** — 0: resumed the existing T-111 `in_progress` task on `dev` at `8db4ad3`; verified T-110d and all six dependency tasks are `review`, preserved unrelated WIP, and passed task index/lint preflight; 1: read the full T-111 contract and cited specs; 2: read T-110c–h Reports including Deviations/Follow-ups and inspect shipped tests/code; 3: n/a (backend aggregate); 4: recorded the aggregate plan and AC-to-test map before final verification; 5: no implementation was in scope for this aggregate; 6: dependency tasks exercised real HTTP behavior and their documented E2E flows; 7: recorded the Testcontainers attempt and exact cleanup above, then the bounded library suite passed; 8: reviewed only the T-111/DECISIONS/index diff and security/event boundaries; 9: n/a (no runtime changes); 10: reviewed the dependency abuse cases and API-15 follow-up; 11: ran the applicable schema, Rust static, focused tests, cargo-deny, guard, task, diff and secret checks, and recorded every Not run result; 12: set this task to `review`, regenerated and checked the index, staged only T-111-owned files, committed one signed commit and pushed only `dev`.

**Disk and resource accounting** — At T-111 start, the fixed target was `8.5G`; Docker reported 7 images/10.96GB, 9 containers/10.49MB, 6 volumes/5.032GB and no build cache. After checks, the target remained `8.5G`; Docker reported 7 images/10.96GB, 9 containers/10.49MB, 6 volumes/5.048GB and no build cache. No resource counts changed; the reported volume size differs by 16MB, and no volume was removed. The only Docker container/anonymous volume created during this task was the Testcontainers resource described in Debug log; both were removed after verifying ownership. Existing `ritma-db-1`, `ritma_test_T-110a4b`, and all kinevo*, infrastructure*, eqs* and equalizer-stake_* resources were preserved. No T-111 database was created.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** this task depended on the original parent contract after its CQ-101 split → depend on the final implementation part that provides the contract, because the parent now owns only the split map.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-134):** the task combines six independently testable security workflows and exceeds CQ-101's approximate 400 changed-line review limit → split implementation into T-110c through T-110h, keep this task as an aggregate depending on all six, and finalize it after they are review, because CQ-101 requires larger work to be split without changing its business contracts.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-135):** the invite event is listed with person_id, role and invite_id only, but FLW-12 requires Notify to build a URL containing the token while the database stores only its hash → add a raw token field to this same event's server-side outbox payload, never expose or log it, because no other specified path can recover the URL token after issuance.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-136):** T-111 cannot depend on child IDs T-111a–f because the task index requires dependencies to sort before their dependent → use the next unused IDs T-110c–h after T-110b and keep T-111 as their aggregate, as required by tasks/README.md.
- **Self-resolved (implementer, run 22, needs architect ratification; TD-163):** the original T-111 context asks both invite and password-reset events to carry raw tokens, but T-110c/T-110e ship IDs-only events under CAN-60/FLW-60 → keep the spec-compliant IDs-only events and record secure delivery handoff as a follow-up, because S3 values must not enter events and no protected handoff is specified here.
- **Self-resolved (implementer, run 22, needs architect ratification; TD-164):** T-111's inherited DoD asks for `make api`, but the Makefile target is still T-114's explicit placeholder and generated artifacts are outside this aggregate's Files → record `make api` as not run and defer generation to T-114, because this aggregate makes no API implementation change and the placeholder cannot generate a contract.

## Review
Pending maintainer review.
