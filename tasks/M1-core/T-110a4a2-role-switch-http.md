---
id: T-110a4a2
title: Expose role-switch HTTP route with CSRF checks
milestone: M1
status: review
depends_on: [T-110a4a1]
spec_refs: [IDN-05, IDN-09, AUTH-03, CAN-30, CAN-34, SBD-40, SBD-50, SBD-53]
estimated_size: S
---

## Goal
Expose the role-switch service through `POST /api/v1/auth/switch-role`, authenticating with the existing refresh cookie and enforcing the exact CSRF and Origin contract.

## Context
T-110a4a1 supplies the service. IDN-05 and `spec/25` §3.3 require both `X-EQ-CSRF: 1` and an `Origin` equal to the configured public URL origin. The route returns the existing login-response shape and must preserve the current cookie.

## Scope
### In
- Parse one canonical `eq_rt` cookie into secret-safe storage and compute its canonical digest.
- Require exactly one CSRF header with value `1` and exactly one matching Origin before database access.
- Add a path-specific API guard that checks CSRF and Origin before the global tenant resolver.
- Register the handler and return a no-store login-shaped response without setting a replacement cookie.
- Add unit and route-surface tests.
### Out
- Session persistence/service logic, new schema, role grants, token rotation, and database-backed HTTP acceptance tests (T-110a4a3).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_auth.rs` | modify | Request boundary, cookie/CSRF validation and response construction. |
| `crates/eq-identity/src/module.rs` | modify | Register the endpoint in the Identity module. |
| `crates/eq-api/src/health.rs` | modify | Place the switch-role CSRF guard outside the tenant resolver. |
| `tasks/M1-core/T-110a4a2-role-switch-http.md`, `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Part report, aggregate split record and task index. |

## Interfaces
- `POST /api/v1/auth/switch-role` accepts exactly `{ "role": <canonical Role> }`.
- Headers: `Cookie: eq_rt=…`, `X-EQ-CSRF: 1`, and `Origin` equal to `RITMA_PUBLIC_URL`'s origin.
- Errors reuse `unauthenticated` for missing/malformed credentials and `forbidden` for CSRF or inactive target membership.

## Business rules
1. The target role and response use canonical `Role` values.
2. A successful response keeps the browser's refresh cookie and existing session family unchanged.
3. The endpoint adds no permission key or role grant.

## Security
- Triggers: cookie authentication, CSRF, refresh/access tokens (SBD-30).
- Require exact CSRF/Origin values and reject duplicate headers; perform these checks before any database query.
- Keep the cookie credential in `SecretString` with redacted Debug output; accept only canonical base64url encoding of the expected digest input.
- The control is a valid refresh session plus active target membership defined in IDN-05/09; do not invent an `actor.require` permission.

## Implementation notes
1. Reuse the shared login state, permission snapshot and response types where their contracts match.
2. Validate exact CSRF/Origin headers in the API layer before tenant resolution, then recheck in the handler before reading the cookie or opening a database transaction.
3. Preserve the existing refresh cookie by omitting `Set-Cookie` on success.

## Acceptance criteria
- AC1. The module exposes the documented POST path and the handler returns the existing login response shape → `defaults_schema_and_summary_match_contract`.
- AC2. Missing, duplicate, malformed or mismatched CSRF/Origin values fail closed before database access → `role_switch_csrf_requires_exact_header_and_origin`, `role_switch_csrf_guard_runs_before_tenant_resolution`.
- AC3. Missing or malformed refresh cookie never exposes its value and canonical digest validation rejects invalid encodings → `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/api_auth.rs` | `role_switch_csrf_requires_exact_header_and_origin`, `role_switch_csrf_guard_runs_before_tenant_resolution`, `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required` |
| `crates/eq-identity/src/module.rs` | `defaults_schema_and_summary_match_contract` |

## Definition of done
- [x] The named unit, middleware-order and route-surface tests pass.
- [x] Focused Clippy, fmt, task lint/index and required guard scripts pass.
- [x] No schema, permission, event, cookie rotation or new error code is introduced.
- [x] Report contains Workflow steps 0–12 and disk/resource snapshots.
- [x] Signed commit `feat(identity): expose role-switch endpoint [T-110a4a2]` pushed to `dev`.

## Report
**Plan** — (1) Put the role-switch CSRF/Origin guard outside the API tenant resolver, then retain the handler check → verify invalid headers stop before tenant lookup. (2) Parse one `eq_rt` cookie into `SecretString` and require its canonical digest → verify missing, malformed and duplicate credentials fail closed. (3) Connect the service and existing login response shape, and register the Identity route → verify the documented path and OpenAPI response shape. (4) Run focused Rust, task, guard and secret checks; report database-backed HTTP work under T-110a4a3.
**Assumptions** — `RITMA_PUBLIC_URL` is the source of the expected Origin; the shared API router applies the role-switch guard before its existing tenant guard; the RoleSwitchService owns final session validation, active membership, mutation and audit.
**Test plan** — AC1: Given the Identity module is built, when routes are inspected, then the POST path exists → `defaults_schema_and_summary_match_contract`. AC2: Given missing, duplicate or wrong CSRF/Origin values, when the request passes the pre-tenant guard, then it returns `forbidden` before the tenant-resolution layer → `role_switch_csrf_requires_exact_header_and_origin`, `role_switch_csrf_guard_runs_before_tenant_resolution`. AC3: Given a missing, malformed or duplicate refresh cookie, when parsing and canonical digest validation run, then no credential is exposed and invalid input is rejected → `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required`. Database-backed success and active-membership rejection are covered by T-110a4a3.

**Summary** — Added `POST /api/v1/auth/switch-role` to Identity, with a closed role request DTO, a canonical redacted refresh-cookie parser, exact single CSRF and Origin checks, a no-store login-shaped response and no replacement cookie. A path-specific CSRF guard runs outside the global tenant resolver, which otherwise could query PostgreSQL before the handler checks those headers.

**Spec read** — `spec/modules/identity.md` IDN-05/09; `spec/25-auth-and-access-control.md` §§3.3 and 4.4; `spec/23-contracts-and-canonical-forms.md` CAN-30/31/34; `spec/18-security-by-design.md` SBD-50/53/81; `spec/19-error-handling.md` unauthenticated/forbidden contracts; `spec/16-code-quality.md` §§9/12 and `spec/17-clean-architecture.md` §7. Read T-110a4a1's Report and the shipped RoleSwitchService, repository and canonical digest implementation.

**Files** — Modified `crates/eq-identity/src/api_auth.rs` (DTO, handler, cookie/CSRF checks, guard and unit tests), `crates/eq-identity/src/module.rs` (route registration and route-surface test), `crates/eq-api/src/health.rs` (guard ordering), `tasks/DECISIONS.md` (TD-128), this task, its aggregate `T-110a4b` file, and generated `tasks/INDEX.md`. The uncommitted `crates/eq-identity/tests/api_auth.rs` changes belong to T-110a4a3 and are excluded from this part's commit.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 — route and response schema are exposed | `defaults_schema_and_summary_match_contract` | Passed |
| AC2 — exact, single CSRF/Origin checks precede tenant resolution | `role_switch_csrf_requires_exact_header_and_origin`; `role_switch_csrf_guard_runs_before_tenant_resolution` | Passed |
| AC3 — cookie is redacted and only canonical digest input is accepted | `refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required` | Passed |

**Checks run** — `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check` passed. `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity -p eq-api --lib -- --test-threads=1` passed 31 tests (30 identity, 1 API). `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline -p eq-identity -p eq-api --all-targets -- -D warnings` passed. `cargo deny check` passed advisories, bans, licences and sources with the existing duplicate-`base64` warning. `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, the four guard scripts, `python3 -I -B scripts/erd.py --check`, `git diff --check`, and `gitleaks protect --staged --redact` passed; Gitleaks found no leaks. `scripts/erd.py --migrations` is n/a because no migration changed; SQLx prepare is n/a because no query changed. **Not run:** workspace runtime tests (the workspace harness starts Testcontainers, prohibited by the disk rule); database-backed HTTP success/abuse tests and `make dev` (owned by T-110a4a3); web/i18n/Playwright checks (no web files or UI journey in this task); `make e2e` (not the M1 integration task); hosted CI.

**Manual checks** — `role_switch_csrf_guard_runs_before_tenant_resolution` sent an invalid Origin through an Axum router with a downstream tenant-resolution marker; it returned 403 and the downstream marker did not run. This was an in-process HTTP service check, not a TCP server or database check. The real localhost TCP requests and PostgreSQL 16 assertions are assigned to T-110a4a3 and will be run there.

**Debug log** — The first format check requested rustfmt on the new middleware test; formatted only `api_auth.rs` and reran fmt successfully. A focused test compile first exposed a missing test-scope Origin import plus unnecessary-qualified-name warnings; used the fully qualified Origin constant and narrowed imports, then the test passed. Clippy flagged a collapsible nested `if`; collapsed it and reran all-target Clippy successfully. A later fmt check caught one line wrap after removing an unrelated repo-import formatting change; rustfmt fixed the line and the final format check passed. No security or behavior check was weakened.

**Performance** — n/a; no query or endpoint latency optimization was measured in this boundary task.

**Security** — Triggers: cookie authentication and CSRF (SBD-30). SBD-81: (1) cookie mode uses the refresh-session service and TD-126's specified active-membership control, with no invented permission. (2) tenant remains derived from the API host resolver; the new guard rejects invalid CSRF/Origin before that resolver. (3) no field or schema was added; refresh token is `SecretString`, redacted in Debug, and only its canonical SHA-256 digest reaches repository lookup. (4) the DTO is a closed `Role` enum; duplicate/missing CSRF and Origin headers fail closed; empty, missing, malformed or duplicate `eq_rt` cookies fail closed. (5) the response is JSON with `Cache-Control: no-store`, reuses the login response and does not emit `Set-Cookie`; errors contain no cookie value. (6) the existing canonical URL-safe no-padding digest validator hashes exactly 32 decoded bytes and zeroizes the decoded token. (7) no external call. (8) no schema or aggregate invariant changed; the service handles the transactional role update. (9) `security.role_switched` remains in T-110a4a1's transaction. (10) the CSRF, guard-order and redaction/canonical-input abuse tests passed; the database abuse tests remain assigned to T-110a4a3. No secret or PII was logged.

**Deviations** — RATIFY TD-125: register the specified route in the Identity module. RATIFY TD-126: use the documented refresh session, CSRF and active-membership controls without inventing a role-switch permission. RATIFY TD-128: place the CSRF/Origin guard before the global tenant resolver because its shipped lookup can query PostgreSQL first. No schema, event, role grant, error code, cookie rotation or business rule was added.

**Dependencies affected** — T-110a4a3 now has the HTTP route and pre-tenant guard to verify. T-110a4b remains the aggregate report task after all three parts are reviewed. T-110a4a1's service/repository contract is unchanged.

**Rollback plan** — Revert this task's signed commit on `dev`; no migration or external resource needs reversal.

**Follow-ups** — Run all six database-backed localhost tests on a clean `ritma_test_T-110a4a3` in the existing PostgreSQL 16 container, then finish the T-110a4b aggregate report. Architect ratification is needed for TD-125, TD-126 and TD-128.

**Resources** — Before: fixed target `11G`; filesystem 57G available; Docker system df: 20 images/15.5GB, 17 containers/10.49MB, 443 volumes/74.56GB, build cache 0. After current focused checks: fixed target `12G`; filesystem 57G available; Docker system df: 20 images/15.5GB, 22 containers/10.51MB, 448 volumes/75.09GB, build cache 0. This task created no container, image, volume, database or target directory. Five PostgreSQL 16 containers (`interesting_mccarthy`, `compassionate_kapitsa`, `gifted_austin`, `unruffled_fermi`, `suspicious_antonelli`) and five volumes appeared during the run; no command in this task created them, ownership is unclear, and they were left untouched. The existing `ritma-db-1` was not used for this task. The shared target grew while compiling; `cargo clean` is forbidden by the supplied disk rule, so it was not run. This task could not ensure the no-net-disk-growth invariant because of shared build-cache growth and concurrent Docker resource changes.

**Workflow steps 0–12** — 0: resumed the existing WIP on `dev`, confirmed no task was in progress, claimed T-110a4a2 and regenerated the task index while preserving other WIP. 1: read the task and cited identity, auth, canonical-form, security and quality specs. 2: read T-110a4a1's Report and shipped RoleSwitchService/repository/digest code. 3: n/a, backend-only. 4: recorded the plan and AC-to-test mapping above before final checks. 5: implemented the route, strict request parsing, pre-tenant CSRF layer, response construction and named unit tests. 6: exercised the middleware ordering with a local Axum service; TCP/database behavior is the next split task. 7: recorded and fixed the formatting, compile and Clippy findings above. 8: reviewed the diff against CQ-16 §9, CA-17 §7, CAN-30/34 and SBD-81; no out-of-scope product changes remain. 9: n/a, no performance target was measured. 10: completed the SBD-81 review above. 11: reran final fmt, unit tests, all-target focused Clippy and the listed task/guard checks; the stated workspace/database/web/e2e checks were not run for the reasons above. 12: set status to review, regenerated the index, then make one signed task commit and push it to `dev`.

## Questions
- **Self-resolved (implementer, run 10, needs architect ratification; TD-125):** the parent task omitted the route registry from its Files table → register the handler in `crates/eq-identity/src/module.rs`, the existing composition point required by Scope.
- **Self-resolved (implementer, run 10, needs architect ratification; TD-126):** generic SBD-40 says `actor.require(perm)`, but this cookie-auth endpoint has no Actor before refresh authentication and no self-switch permission is specified → enforce valid refresh session, exact CSRF/Origin and active membership per IDN-05/09 and `spec/25` §4.4; add no permission.
- **Self-resolved (implementer, run 12, needs architect ratification; TD-128):** `health::router` runs `tenant_guard` before module handlers and the tenant resolver can query PostgreSQL → place the path-specific CSRF/Origin guard outside tenant resolution because this task requires rejecting invalid headers before database access (ARC-04, IDN-05, `spec/25` §3.3).

## Review
Self-review complete against CQ-16 §9, CA-17 §7, CAN-30/34, ERR-19 and SBD-81. The route is registered and documented, exact CSRF/Origin validation runs before tenant resolution and is rechecked in the handler, refresh credentials are redacted and canonically hashed, and success does not rotate the cookie. Named unit and route-surface tests plus focused Clippy and guard checks passed. Maintainer review should ratify TD-125, TD-126 and TD-128; the TCP/database acceptance checks are in T-110a4a3.
