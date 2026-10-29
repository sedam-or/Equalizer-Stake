---
id: T-110a4a3
title: Verify role-switch HTTP behavior
milestone: M1
status: todo
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
- [ ] All six role-switch tests pass against a clean task-named PostgreSQL 16 database in the reused container.
- [ ] HTTP requests reach a real localhost listener; success and both specified error classes are verified.
- [ ] Drop the task database after checks; create no container, image, volume or extra Cargo target.
- [ ] Report contains Workflow steps 0–12, exact command/database/pass count and disk/resource snapshots.
- [ ] Signed commit `test(identity): verify role-switch HTTP behavior [T-110a4a3]` pushed to `dev`.

## Report
**Plan** — (1) Extend the API fixture to start the production router on an ephemeral localhost port → verify real HTTP requests. (2) Add success, CSRF, inactive-role and family/audit assertions → verify each criterion under its exact name. (3) Run on a clean task-named PostgreSQL 16 database in the existing fixed container → record database and pass count. (4) Run fmt, focused Clippy, task/guard checks and diff/security scans → verify the test-only change.
**Assumptions** — Fixture tenant/person/session IDs are generated per test. The existing fixed container can host the task database; no test container is created.
**Test plan** — AC1: Given active admin and guardian memberships, when switching to guardian, then the response token and permission snapshot carry guardian only → `switch_role`. AC2: Given missing/wrong CSRF or Origin, when the request is sent, then it returns `forbidden` without changing the session or writing an audit event → `switch_role_requires_csrf`. AC3: Given no active guardian membership, when switching to guardian, then it returns `forbidden` and state is unchanged → `switch_role_rejects_inactive_role`. AC4: Given a valid active session, when switching succeeds, then id, family and digest are identical, active role changes and one audit row exists → `switch_role_keeps_refresh_family`. Expiry and revocation fail closed → `switch_role_rejects_expired_session`, `switch_role_rejects_revoked_session`. Other edge cases include missing/malformed cookies, invalid headers, cross-role permission isolation and no cookie rotation.

## Questions
- None.

## Review
Pending implementation checks.
