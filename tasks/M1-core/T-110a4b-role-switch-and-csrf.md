---
id: T-110a4b
title: Add cookie role switching and CSRF checks
milestone: M1
status: review
depends_on: [T-110a4a3]
spec_refs: [IDN-05, IDN-09, IDN-10, AUTH-03, CAN-30, CAN-34, STM-22, SBD-40, SBD-53]
estimated_size: M
---

## Goal
Implement `POST /api/v1/auth/switch-role` using the existing refresh cookie, issue an access token for the selected active role, and retain the existing refresh-session family. Require the exact CSRF header and matching configured Origin.

## Context
IDN-09 allows switching only to an active membership role and requires the new token to carry that role while retaining the same refresh family. IDN-05 and `spec/25` §3.3 require both `X-EQ-CSRF: 1` and an `Origin` equal to the configured public URL origin. `spec/modules/identity.md` defines the request `{role}` and login-response body; `security.role_switched` is the mapped audit action. The refresh session's columns are fixed by spec 06 §4.2 and ERD-04; no schema change is allowed.

## Scope
### In
- Add the role-switch HTTP route with request validation, refresh-cookie authentication and exact CSRF/Origin validation.
- Verify requested role is an active membership; update only the existing refresh session's `active_role`, keep its `family_id` and refresh token unchanged, and issue a new role-bound access token.
- Record `security.role_switched` through the shipped audit API in the same transaction.
- Add the named role-switch API tests and abuse tests.
### Out
- Refresh rotation, logout, password-change invalidation (T-110b), people APIs (T-112), new schema, new event names or new permission grants.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/module.rs` | modify | Register the role-switch route in the Identity module and retain its route-surface assertion. |
| `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/service.rs`, `crates/eq-identity/src/repo.rs`, `crates/eq-identity/src/jwt.rs`, `crates/eq-api/src/health.rs` | modify | Cookie auth, role-switch transaction/repository, token issue, response, and pre-tenant CSRF guard. |
| `crates/eq-identity/tests/api_auth.rs` | modify | Role-switch, CSRF and inactive-role acceptance/abuse tests. |
| `.sqlx/` | modify | Generated metadata for changed SQL macros. |
| `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Aggregate report, split decision and generated status index. |

## Interfaces
- `POST /api/v1/auth/switch-role` accepts a JSON object containing exactly the canonical `Role` and returns the existing login-response body shape.
- Cookie: `eq_rt`; required headers: `X-EQ-CSRF: 1`, `Origin: <RITMA_PUBLIC_URL origin>`.
- Invalid/missing session → existing `unauthenticated`; invalid CSRF or inactive target role → existing `forbidden`.

## Business rules
1. Role must be an active membership of the cookie session's person and tenant (IDN-09).
2. The selected role is the only permission-grant source (IDN-10); no role grant is added by switching.
3. The same `refresh_sessions` row and family remain active; only `active_role` changes.

## Security
- Triggers: T1, T3, T4, T8 (SBD-30).
- Data classes touched: S1 role, S2 person/tenant ids, S3 refresh token and access token.
- Spoofing → hash and compare the opaque cookie credential; require the role to be an active membership (IDN-04/09).
- CSRF → require both exact header and matching Origin; missing or malformed values fail closed (IDN-05/AUTH-05).
- Tampering/replay → update session and audit in one tenant transaction; do not rotate or change its family (STM-22/SBD-53).
- Abuse acceptance: `switch_role_requires_csrf`, `switch_role_rejects_inactive_role`, and `switch_role_keeps_refresh_family`.

## Implementation notes
1. Parse the `eq_rt` cookie as the existing canonical base64url refresh token and hash the decoded 32 bytes as T-110 IDN-04 does.
2. Lock the matching active session row; compare expiry and role membership before changing its role.
3. Keep the audit write in the transaction and use the exact `security.role_switched` action from the parent contract.
4. Build the response with the already-shipped login-response fields and the new access token; preserve the existing cookie/family.

## Acceptance criteria
- AC1. Given active admin and guardian memberships, a valid session can switch to guardian; response token role and permissions are guardian-only, `active_role` is updated, and the same refresh family remains → `switch_role`.
- AC2. Missing/wrong CSRF header or absent/mismatched Origin is rejected before DB mutation → `switch_role_requires_csrf`.
- AC3. A requested inactive/nonmember role is forbidden and leaves session state unchanged → `switch_role_rejects_inactive_role`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `switch_role`, `switch_role_requires_csrf`, `switch_role_rejects_inactive_role`, `switch_role_keeps_refresh_family` |

## Definition of done
- [x] Named role-switch and abuse tests pass on a disposable PostgreSQL 16 task database; drop the task database after checks.
- [x] Real HTTP request verifies success plus CSRF and inactive-role error paths.
- [x] `scripts/erd.py --migrations` is not applicable because no migration changes; SQLx metadata is regenerated for query changes.
- [x] `cargo fmt --all --check`, focused Clippy, task lint/index and required guard scripts pass.
- [x] No new schema, permission grant, event or error code; audit is transaction-bound.
- [x] Report includes Workflow steps 0–12, resource snapshots, and exact database/test pass count.
- [x] Signed task-report commit `docs(identity): report role-switch aggregate [T-110a4b]` pushed to `dev` after all split parts are reviewed.

## Split
The implementation diff exceeded CQ-101's approximately 400-line review guide. It is split into three ordered parts so each commit has one small responsibility:

1. `T-110a4a1` owns the refresh-session repository and role-switch service transaction.
2. `T-110a4a2` owns cookie parsing, CSRF/Origin checks, the HTTP handler, and route registration.
3. `T-110a4a3` owns the four named localhost HTTP integration tests.

This aggregate remained `draft` until all three parts reached `review`; it was then promoted and its report summarizes their combined acceptance results.

## Report
**Plan** — (1) Read the three split parts' Reports and shipped service, HTTP and test code → verify the aggregate contract and exact named tests are covered. (2) Run the complete auth API test suite over localhost TCP HTTP on a fresh task database → verify success, CSRF/inactive/expired/revoked rejection, session-family preservation and audit. (3) Run fmt, focused Clippy, dependency policy, task/index/guard and secret checks → verify the report and shipped parts remain green. (4) Aggregate the prior Playwright evidence and host limitation, capture resources, drop the task database, and complete this signed report task.
**Assumptions** — The preliminary session lookup is read-only and only supplies the person id needed to begin the final transaction with the correct audit actor; the final transaction locks and revalidates the session before any write. The existing `unauthenticated` and `forbidden` codes cover the specified failures. The already-shipped `has_active_role` identity port will be used to enumerate the five spec-defined roles in IDN-09 order for the response, avoiding cross-module SQL or a new port.
**Test plan** — AC1: given active admin and guardian memberships, when the existing cookie switches to guardian, then response token/permissions and `active_role` are guardian-only while row id, family id and token hash remain constant → `switch_role`, `switch_role_keeps_refresh_family`. AC2: given a missing/wrong header or absent/mismatched Origin, when switching is requested, then it returns 403 `forbidden` before database mutation → `switch_role_requires_csrf`. AC3: given an inactive/nonmember target role, when switching is requested, then it returns 403 and leaves session state unchanged → `switch_role_rejects_inactive_role`. Edge/error cases: missing, duplicate, malformed or unknown cookie; expired/revoked session; malformed or unknown role; permission snapshot remains limited to the selected role. No schema, permission, event or error-code addition.
**Summary** — The three reviewed parts provide the cookie-authenticated role-switch route, strict CSRF/Origin validation before tenant lookup, transactional active-role updates with audit, and six real-HTTP acceptance/abuse tests. The auth API suite passed 16/16 on a fresh PostgreSQL 16.15 task database. The child E2E run passed 4/6 browser checks; the two WebKit page launches are recorded as Not run because this host lacks their required system libraries.
**Spec read** — `spec/modules/identity.md` §§1–3; `spec/06-domain-model.md` §4.2; `spec/18-security-by-design.md` §§1, 3–5, 7–8; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7; `spec/19-error-handling.md` §§1–5/8–10; `spec/20-state-machines.md` STM-22; `spec/23-contracts-and-canonical-forms.md` CAN-30/CAN-34; `spec/25-auth-and-access-control.md` §§3.3/4.4. Read the Reports and relevant shipped service, repository, handler, middleware and integration tests from T-110a4a1, T-110a4a2 and T-110a4a3.
**Files** — Updated this aggregate task's status and Report; regenerated `tasks/INDEX.md`. No production source, migration, query metadata or web file changed. `RUN_LOG.md` is append-only and remains untracked.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 — active membership switches to the selected role; token/permissions match and refresh family remains unchanged | `switch_role`; `switch_role_keeps_refresh_family` | Passed as part of the 16-test Identity API suite on PostgreSQL 16.15. |
| AC2 — missing/wrong CSRF or Origin fails before mutation | `switch_role_requires_csrf`; `role_switch_csrf_guard_runs_before_tenant_resolution` | Passed; database-backed HTTP tests verified no state/audit mutation. |
| AC3 — inactive role is rejected without mutation | `switch_role_rejects_inactive_role` | Passed. |
| Expired/revoked session abuse cases fail closed | `switch_role_rejects_expired_session`; `switch_role_rejects_revoked_session` | Passed. |
**Checks run** —
- ✅ `DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4b_aggregate' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4b_aggregate' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='https://example.test' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` — 16 passed, 0 failed; real localhost HTTP. The disposable database was PostgreSQL 16.15 in the existing `ritma-db-1` container and was dropped after checks.
- ✅ Child T-110a4a3 ran the full listed command: `E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4a3' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4a3' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' RITMA_LISTEN='127.0.0.1:8080' E2E_BASE_URL='http://127.0.0.1:8080' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true make e2e` — 4/6 passed (Chromium 3/3; WebKit health 1/1) on PostgreSQL 16.15, `ritma_test_T-110a4a3`; WebKit `loads home` and `spa fallback` are **Not run** because Athena OS lacks `libWPEWebKit-2.0.so.1`, ICU 74 and Flite.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline -p eq-identity --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate-`base64` warning only.
- ✅ `scripts/tasks-index.sh --check`; `python3 -I scripts/tasks-lint.py`; `python3 -I -B scripts/tests/test_tasks_lint.py` (8/8).
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && python3 -I -B scripts/erd.py --check && git diff --check`.
- ✅ `gitleaks dir --redact --no-banner tasks/M1-core/T-110a4b-role-switch-and-csrf.md` — no leaks.
- Not run: `cargo test --workspace`; workspace integration tests start Testcontainers resources, prohibited by the disk-hygiene rule. The focused database-backed API suite above passed.
- Not run: `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check`; this aggregate task changed no web source. `pnpm -C web build` passed in the child `make e2e` run.
- Not run: the two WebKit page tests named above; the host libraries needed to launch WebKit are absent. The full `make e2e` command exited 1 after 4/6 checks.
- n/a: `scripts/erd.py --migrations` (no migration changed); SQLx prepare and `make api` (no aggregate query/API change); hosted GitHub Actions (not available locally).
**Manual checks** — The focused API test suite bound a localhost listener and exercised the production router, checking role-bound response permissions, exact CSRF/Origin behavior, unchanged session id/family/digest, inactive/expired/revoked rejection and the audit row. PostgreSQL was 16.15 in the existing `ritma-db-1`. `ritma_test_T-110a4b_aggregate` was created from `template0`, used for the 16/16 suite, then dropped after confirming zero active connections. Child A3 separately used and dropped fresh `ritma_test_T-110a4a3` for its 16/16 API suite and 4/6 `make e2e` result.
**Debug log** — No new failure occurred in this aggregate report task. Child A3's initial database quoting, browser installation, single-tenant fixture and unavailable WebKit library issues are recorded in its Report; the final API suite passed, and the two unavailable WebKit launches remain Not run.
**Performance** — n/a; report-only aggregate task, no application runtime code changed.
**Security** — Triggers: cookie authentication, role authorization, CSRF and security audit (SBD-30). The shipped parts hash and redact refresh credentials, require exact CSRF/Origin before tenant lookup, verify active membership, mutate only the selected role, preserve the refresh family, and record audit in the transaction. The named CSRF, inactive-role, expired-session and revoked-session abuse tests passed; no new permissions, schema, event or error code were introduced. No secret or PII was found by Gitleaks.
**Deviations** — RATIFY inherited decisions TD-125 (route registry file), TD-126 (documented cookie-session/CSRF/membership authorization), TD-127 (split to meet CQ-101 review size), and TD-128 (CSRF guard before tenant resolution). No new self-resolution was added for this aggregate. The two WebKit page tests are Not run for the host-library reason above.
**Dependencies affected** — T-110a5 can proceed after this aggregate is reviewed; T-110b continues to own refresh rotation and token invalidation. No shipped interface changed.
**Rollback plan** — Revert the signed report commit; no code, migration or persistent database change from this aggregate remains.
**Follow-ups** — Maintainer should ratify TD-125–TD-128 and review the WebKit host limitation. The child reports contain each split part's individual follow-ups.
**Resources** — Before: `du -sh "$HOME/.cache/ritma/target"` → 13G; `docker system df` → Images 20 total/7 active/15.5GB (2.073GB reclaimable), Containers 22 total/14 active/10.51MB, Local Volumes 448 total/19 active/75.1GB (68.81GB reclaimable), Build Cache 0; filesystem had 54G available. After: target 13G; Docker → Images 20/7/15.5GB, Containers 22/14/10.51MB, Local Volumes 448/19/75.09GB, Build Cache 0; filesystem had 54G available. Created and removed database `ritma_test_T-110a4b_aggregate` inside existing `ritma-db-1`; no container, image, volume or Cargo target directory was created or removed. Pre-existing `ritma_test_T-110a4b` was left unchanged. Child A3's Report records its separate database and the exact E2E resource use.
**Workflow steps 0–12** — 0: resumed the existing aggregate WIP on `dev`, fetched origin, preserved unrelated audit/Python WIP, and verified the three-part dependency set is reviewed. 1: reread this task and cited identity, auth, canonical-form, security, state-machine and quality requirements. 2: read all three child Reports and the shipped service, repository, route/guard and integration tests. 3: n/a, backend-only and no UI design reference applies. 4: recorded the aggregate plan, assumptions and AC-to-test mapping above. 5: n/a, implementation shipped in the three scoped child tasks; this task aggregates their evidence. 6: the child HTTP tests exercised the production router over localhost TCP; child A3 also ran `make e2e` on its disposable PostgreSQL 16 database. 7: no new retry was needed; the child A3 Debug log records earlier attempts and the remaining host limitation. 8: reviewed shipped behavior, test coverage, transaction/audit boundary and all inherited deviations. 9: n/a, no runtime code or performance target changed. 10: completed the SBD-81 aggregate review above. 11: ran fmt, focused Clippy, cargo deny, task index/lint and 8 unit tests, four guards, ERD check, diff and Gitleaks; recorded workspace, web and WebKit checks as Not run where required. 12: completed this Report, set `review`, regenerated the index, and prepared the exact signed task-report commit for `dev`.

## Questions
- **Self-resolved (implementer, run 10, needs architect ratification; TD-125):** the task requires a new Identity HTTP route but omits `crates/eq-identity/src/module.rs` from `## Files` → add that route registry file because the handler would otherwise be unreachable; registering the requested route is required by the task's Scope and does not broaden it.
- **Self-resolved (implementer, run 10, needs architect ratification; TD-126):** SBD-40 generically requires `actor.require(perm)` for non-public handlers, but no self-role-switch permission is specified and this endpoint has cookie authentication before an Actor exists → authorize with a valid refresh session and the requested active membership, because IDN-05/IDN-09 and `spec/25` §4.4 define those controls and adding an unrelated permission would change the contract.
- **Self-resolved (implementer, run 11, needs architect ratification; TD-127):** the role-switch implementation exceeded CQ-101's approximately 400-line review guide → split it into ordered service/persistence, HTTP boundary, and HTTP-test tasks while preserving the original aggregate contract and all four named tests.

## Review
Ready for maintainer review. All three split parts are `review`; the six named role-switch tests passed. The child E2E run is 4/6 with two WebKit checks Not run because the host lacks required libraries. Ratify TD-125–TD-128.
