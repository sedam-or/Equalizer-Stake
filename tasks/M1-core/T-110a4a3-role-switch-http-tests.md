---
id: T-110a4a3
title: Verify role-switch HTTP behavior
milestone: M1
status: review
depends_on: [T-110a4a2]
spec_refs: [IDN-05, IDN-09, IDN-10, CAN-30, CAN-34, STM-22, SBD-40, SBD-53, SBD-81]
estimated_size: S
---

## Goal
Verify the complete role-switch endpoint over real localhost HTTP requests using PostgreSQL 16.

## Context
T-110a4a1 supplies the session/service transaction and T-110a4a2 supplies the HTTP route and CSRF boundary. The parent contract requires active membership, role-bound permissions, refresh-family preservation and transaction-bound audit.

## Scope
### In
- Add the four parent-named integration tests for success, CSRF rejection, inactive-role rejection and refresh-family preservation, plus explicit expired/revoked-session tests.
- Verify error codes, response token role and permission snapshot, unchanged session id/family/token digest, and the audit row.
- Use the existing PostgreSQL 16 container and a clean task database; do not create a container.
### Out
- Production handler/service/schema changes, browser UI, external calls and unrelated auth behavior.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/tests/api_auth.rs` | modify | Real HTTP role-switch acceptance and abuse tests. |
| `tasks/M1-core/T-110a4a3-role-switch-http-tests.md`, `tasks/M1-core/T-110a4b-role-switch-and-csrf.md`, `tasks/INDEX.md` | modify/generate | Test report, aggregate split record and task index. |

## Interfaces
- Tests send real TCP HTTP requests to the production Axum router on localhost.
- PostgreSQL 16 is reached through the existing fixed `ritma-db-1` container.

## Business rules
1. Only active membership roles can be selected.
2. The successful response token and permission snapshot are for the selected role.
3. The existing refresh row, family and token digest remain unchanged, and the audit event commits with the role mutation.

## Security
- Triggers: cookie authentication, CSRF, role authorization and audit integrity (SBD-30).
- Negative CSRF and inactive-membership paths must return `forbidden`, preserve state and write no audit row.
- Success must preserve refresh family/digest and create exactly one `security.role_switched` audit entry.
- Use obvious fake credentials; do not print tokens, passwords, PII or raw request headers.

## Implementation notes
1. Reuse the existing auth fixture and test its server through a bound localhost TCP listener.
2. Capture the session id, family id, token digest and active role before and after switching.
3. Query the audit row through the owner connection while the application request uses the app role.

## Acceptance criteria
- AC1. Active admin and guardian memberships permit switching to guardian with a guardian-only token/permission snapshot → `switch_role`.
- AC2. Missing/wrong CSRF or missing/mismatched Origin returns `forbidden` before state mutation → `switch_role_requires_csrf`.
- AC3. An inactive/nonmember target is forbidden with no session or audit change → `switch_role_rejects_inactive_role`.
- AC4. A successful switch retains session id, family id and refresh digest while changing only `active_role` and writing the audit event → `switch_role_keeps_refresh_family`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_auth.rs` | `switch_role`, `switch_role_requires_csrf`, `switch_role_rejects_inactive_role`, `switch_role_keeps_refresh_family`, `switch_role_rejects_expired_session`, `switch_role_rejects_revoked_session` |

## Definition of done
- [x] All six role-switch tests pass against a clean task-named PostgreSQL 16 database in the reused container.
- [x] HTTP requests reach a real localhost listener; success and both specified error classes are verified.
- [x] Drop the task database after checks; create no container, image, volume or extra Cargo target.
- [x] Report contains Workflow steps 0–12, exact command/database/pass count and disk/resource snapshots.
- [x] Signed commit `test(identity): verify role-switch HTTP behavior [T-110a4a3]` pushed to `dev`.

## Report
**Plan** — (1) Extend the existing fixture to serve Identity routes through a bound localhost TCP listener → verify requests use real HTTP. (2) Add success, CSRF, inactive-role, expiry, revocation, and refresh-family assertions under the six exact test names → verify every parent acceptance case and abuse case. (3) Create a clean `ritma_test_T-110a4a3` database from `template0`, run migrations, and connect the application pool as `eq_app` → verify the DB role is non-superuser and does not bypass RLS. (4) Run the auth integration suite, `make e2e` on the same disposable PostgreSQL 16 container, and the applicable Rust/task/guard/security checks → record command, pass count, and environment limits.
**Assumptions** — The already-shipped Identity route/service and test fixture are authoritative. Test rows use generated tenant/person/session ids and the test database is dropped after checks. The host's WebKit browser dependencies are not required by the six role-switch tests; the E2E report will state any browser project the host cannot run.
**Test plan** — AC1: Given active admin and guardian memberships, when switching to guardian over TCP HTTP, then the response token and permission snapshot use guardian only → `switch_role`. AC2: Given missing/wrong CSRF or absent/mismatched Origin, when switching is requested, then it returns 403 `forbidden` without session or audit mutation → `switch_role_requires_csrf`. AC3: Given no active guardian membership, when switching to guardian, then it returns 403 `forbidden` without session or audit mutation → `switch_role_rejects_inactive_role`. AC4: Given a valid session, when switching succeeds, then its id, family and digest remain unchanged, `active_role` changes, no replacement cookie is sent, and one audit row commits → `switch_role_keeps_refresh_family`. Expired and revoked sessions return 401 `unauthenticated` without mutation → `switch_role_rejects_expired_session`, `switch_role_rejects_revoked_session`. Edge cases include missing and malformed cookies plus each missing/malformed CSRF or Origin header; security abuse cases include inactive membership, expiry, revocation, cross-role permission isolation and no cookie rotation.
**Summary** — Added six database-backed role-switch acceptance and abuse tests over a real localhost TCP listener. All 16 tests in the Identity auth API suite passed on a fresh PostgreSQL 16.15 task database; the database was dropped after the checks. Chromium passed all three existing browser smoke tests, while two WebKit page tests could not launch because the Athena OS host lacks the Ubuntu browser libraries Playwright requires.
**Spec read** — `spec/modules/identity.md` §§1–2 (IDN-05/09/10 and endpoint response); `spec/18-security-by-design.md` §§1, 3–5, 7–8 (SBD-30/40/53/81); `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7; `spec/19-error-handling.md` §§1–5/8–10. Read T-110a4a2's full Report, its shipped route/CSRF guard, and T-110a4a1's report and service/repository behavior.
**Files** — Modified `crates/eq-identity/tests/api_auth.rs` with a real TCP role-switch request helper, response/session snapshots, and six named tests; modified this task with the Report/status; regenerated `tasks/INDEX.md`. No production code, migration, SQLx metadata, web source, or other task file changed.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 — active roles switch and response token/permissions match guardian | `switch_role` | Passed |
| AC2 — CSRF and Origin failures return `forbidden` before mutation | `switch_role_requires_csrf` | Passed |
| AC3 — inactive target is rejected without session/audit change | `switch_role_rejects_inactive_role` | Passed |
| AC4 — same session/family/digest, new role and one audit row | `switch_role_keeps_refresh_family` | Passed |
| Expired session fails closed | `switch_role_rejects_expired_session` | Passed |
| Revoked session fails closed | `switch_role_rejects_revoked_session` | Passed |
**Checks run** —
- ✅ `DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4a3' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4a3' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='https://example.test' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_auth -- --test-threads=1` — 16 passed, 0 failed; includes all six named role-switch tests.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate-`base64` warning only.
- ✅ `scripts/tasks-index.sh --check`; `python3 -I scripts/tasks-lint.py`; `python3 -I -B scripts/tests/test_tasks_lint.py` (8 passed); `scripts/check-deps.sh`; `scripts/check-literals.sh`; `scripts/check-web-boundaries.sh`; `scripts/check-layers.sh`; `python3 -I -B scripts/erd.py --check`; `git diff --check`.
- ✅ `gitleaks dir --redact --no-banner crates/eq-identity/tests/api_auth.rs` and the same command for this task file — no leaks.
- ✅ `pnpm -C web exec playwright install chromium webkit` — browser binaries installed under the user cache; Playwright reported missing host libraries for WebKit.
- ⚠️ `make e2e` — exact invocation: `E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-110a4a3' RITMA_DATABASE_OWNER_URL='postgres://eq:test-secret-owner@localhost:5433/ritma_test_T-110a4a3' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' RITMA_LISTEN='127.0.0.1:8080' E2E_BASE_URL='http://127.0.0.1:8080' CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true make e2e` — database `ritma_test_T-110a4a3`, PostgreSQL 16.15 in existing `ritma-db-1`; final run passed 4/6: Chromium 3/3, including home and SPA fallback, and the WebKit project's health request. WebKit `loads home` and `spa fallback` could not launch because Playwright requires `libWPEWebKit-2.0.so.1`, ICU 74 and Flite libraries absent from Athena OS. The full target exited 1. These two WebKit browser checks are **Not run** for that host dependency reason. The command used a temporary symlink `target -> $HOME/.cache/ritma/target`, removed on exit. The initial run also exposed missing browser binaries and, after the auth suite had seeded multiple tenants, the single-tenant root route returned 500; browser binaries were installed and the database was recreated with one E2E tenant before the final run.
- Not run: `cargo test --workspace`; workspace integration tests start Testcontainers containers, which this run forbids. The task's full database-backed auth API suite was run against the existing PostgreSQL container.
- Not run: `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check`; no web source was changed. `pnpm -C web build` did pass inside `make e2e`.
- n/a: `scripts/erd.py --migrations` (no migration changed); `cargo sqlx prepare --workspace --check` and `make api` (no `query!` metadata or API contract changed); hosted GitHub Actions (cannot be run locally).
**Manual checks** — The auth suite started a TCP listener on an ephemeral localhost port and sent real HTTP login, role-switch, and bearer-protected requests against the production Identity routes. Database: freshly created `ritma_test_T-110a4a3`, PostgreSQL 16.15 in the existing healthy `ritma-db-1` container; migrated from `template0`, application connection was `eq_app` (`NOSUPERUSER`, `NOBYPASSRLS`), owner connection checked session and audit state. Result: 16/16 tests passed. Before browser E2E, the same named database was dropped/recreated from `template0` and seeded with one fake E2E tenant so the single-tenant server route had an unambiguous tenant; `make e2e` reused it with `E2E_SKIP_DB_START=1`. The temporary repository `target` symlink pointed to the pre-existing `$HOME/.cache/ritma/target` and was removed on exit; no container, image, volume, or second Cargo target was created. The task database was dropped after checks.
**Debug log** — The first `CREATE DATABASE` attempt left the database untouched because PostgreSQL requires quoting its hyphenated identifier; the quoted command succeeded. The first `make e2e` run found no Playwright browser binaries, so Chromium and WebKit were installed locally. After those binaries were present, the root and SPA fallback requests returned 500 because the preceding auth tests had seeded multiple tenants; recreated the task database from `template0`, seeded one fake tenant, and reran. Chromium then passed all three smoke tests. WebKit's two page tests still could not launch because Athena OS lacks the browser's Ubuntu 24.04 shared libraries; installing those requires system-level package changes, so they remain Not run. No role-switch test failed.
**Performance** — n/a; test-only change, no new application query or hot-path implementation.
**Security** — Triggers: cookie authentication, role authorization, CSRF and security audit (SBD-30). SBD-81: (1) the tests hit the existing cookie-authenticated Identity route and assert exact `unauthenticated`/`forbidden` codes; (2) the app pool is verified as `eq_app` with neither superuser nor RLS-bypass privileges, and tenant ids are generated in the fixture; (3) no production field changes; test email/password are fake, and tokens/cookies are not printed; (4) target role and CSRF/Origin cases exercise the existing boundary validation; (5) responses are JSON and success emits no `Set-Cookie`; (6) the route's shipped parser hashes the refresh credential and keeps it redacted; (7) no external calls; (8) session id/family/digest are asserted unchanged; (9) the success test requires exactly one `security.role_switched` audit row, while rejected paths require none; (10) the named CSRF, inactive, expired and revoked abuse tests passed. No security control was weakened.
**Deviations** — None in the role-switch contract or production code. The full browser `make e2e` suite is not green on this host: its two WebKit page tests are recorded as Not run above. No new RATIFY item; inherited parent decisions TD-125, TD-126, TD-127 and TD-128 remain for the aggregate task's review.
**Dependencies affected** — T-110a4b is the aggregate report task and now has all three split parts at `review`; its own status/report remains a separate follow-up. No implementation interface changed.
**Rollback plan** — Revert this signed task commit; no migration or persistent database resource remains.
**Follow-ups** — Maintainer review of the six HTTP tests; parent aggregate report/promotion when its task workflow permits; ratify TD-125/TD-126/TD-127/TD-128 in the aggregate. The WebKit browser checks need a runner with Playwright's supported browser libraries or an approved local system dependency installation.
**Resources** — Before: fixed Cargo target 12G; filesystem 57G available; latest completed `docker system df` showed 20 images/15.5GB, 22 containers/10.51MB (14 active), 448 volumes/75.12GB (19 active), build cache 0. Existing `ritma-db-1` was healthy. After: fixed target 13G; Playwright user cache 973M; filesystem 54G available; quick Docker counts remained 20 images, 22 total/14 running containers and 448 volumes. The task database was about 9.8MB before it was dropped. A post-run `docker system df` exceeded 30 seconds and its read-only client was stopped; no Docker cleanup was run. The release build and Playwright binaries grew existing caches; no Cargo target directory/container/image/volume was created. Temporary `target` symlink was removed.
**Workflow steps 0–12** — 0: resumed the existing A3 WIP on `dev`, verified the task index, preserved unrelated audit/Python WIP, claimed this task and regenerated the index. 1: read the full task and identity, security, quality, architecture and error-handling references. 2: read T-110a4a2's Report and shipped route/CSRF guard, plus T-110a4a1's service/repository Report and code. 3: n/a, backend integration task. 4: recorded the plan, assumptions and AC-to-test mapping above before final implementation/checks. 5: completed the six named localhost HTTP tests and fixture helpers. 6: exercised success and failure paths against real TCP HTTP; ran the server and browser smoke journey through `make e2e` on the disposable database. 7: recorded the database/browser issues and corrective attempts above; the only remaining unavailable checks are the two WebKit launches. 8: reviewed the test diff against CQ-16 §9, CA-17 §7, ERR-19 and the task scope; no product or unrelated changes remain. 9: n/a, no application performance change. 10: completed the SBD-81 review above. 11: reran applicable format, workspace Clippy, auth DB tests, task/guard, ERD, cargo-deny and secret checks; recorded workspace tests and WebKit checks as Not run with reasons. 12: completed this Report, set the task to `review`, regenerated the index, and made one signed task commit pushed to `dev`.

## Questions
- None.

## Review
Ready for maintainer review; all six role-switch tests pass. Two WebKit browser checks remain Not run on this host as recorded under Checks run.
