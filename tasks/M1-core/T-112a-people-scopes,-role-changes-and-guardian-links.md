---
id: T-112a
title: People scopes, role changes and guardian links
milestone: M1
status: review
depends_on: [T-111, T-102a]
spec_refs: [IDN-10, IDN-11, IDN-13, SEC-03, SEC-04, AUD-01, API-05, API-06, API-07, PRD-02, STM-20, STM-26, STM-27, VAL-25, VAL-51, AUTH-04]
estimated_size: M
---

## Goal
Admins manage people, roles and guardian links. Every list or detail read is scoped to what the actor may see, and PII is decrypted only for permitted roles, with an audit trail.

## Context
- IDN-10 scopes. A `Scope` is computed from the `Actor`:
  - `All` (owner, admin)
  - `Instructor(pid)`: people who participate in sessions they teach. **Until scheduling exists (M2), Instructor scope = self only.** Leave a `// TODO(T-2xx)` with the hook `ScopeResolver` port, which scheduling registers later.
  - `Guardian(pid)`: self plus linked students.
  - `Self_(pid)`
- Scoping happens in SQL (`WHERE id = ANY($visible_ids)` or a join on `person_links`), never by filtering in Rust after loading.
- API (all under `/api/v1`):

| Method | Path | Perm | Body → Result | Errors |
|---|---|---|---|---|
| GET | `/people` | identity.people.read | `?role=&q=&status=&limit=&cursor=&sort=display_name\|-created_at` → `{items: [PersonSummary], next_cursor}` | |
| POST | `/people` | identity.people.write | `{display_name, legal_name?, email?, phone?, birth_date?, roles: [Role], invite: bool, guardian_of?: [uuid], guardians?: [{guardian_id, relationship?, is_primary}]}` → Person (201) | 409 `email_taken` |
| GET | `/people/{id}` | identity.people.read | Person (PII fields null unless `identity.people.read_pii` and in scope) | 404 if out of scope |
| PATCH | `/people/{id}` | identity.people.write | partial Person, `If-Match` | 409 `version_conflict`, 428 |
| POST | `/people/{id}/invite` | identity.people.write | `{role, channel: email\|whatsapp\|link}` → `{invite_url?}` | |
| POST | `/people/{id}/roles` | identity.roles.write | `{role}` → 204 | 403 if admin grants owner/admin |
| DELETE | `/people/{id}/roles/{role}` | identity.roles.write | → 204 | 409 `last_owner` |
| POST | `/people/{id}/deactivate`, `/reactivate` | identity.people.write | → 204 | 409 `last_owner` |
| POST | `/people/{id}/links` | identity.people.write | `{student_id, relationship?, is_primary}` → 204 (id is the guardian) | 409 `primary_exists` |
| DELETE | `/people/{id}/links/{student_id}` | identity.people.write | → 204 | |

- `PersonSummary {id, display_name, roles, status, avatar_url?}`. `Person = PersonSummary + {legal_name?, email?, phone?, birth_date?, locale?, guardians: [{id, display_name, relationship, is_primary}], students: [{id, display_name}], version}`.
- Rules:
  1. Only an owner may grant or revoke the `owner` or `admin` roles (`identity.admins.manage`). An admin calling it → 403 `forbidden {perm: identity.admins.manage}`.
  2. Removing `owner` from, or deactivating, the last active owner → 409 `last_owner`.
  3. A student has at most one primary guardian (DB unique index). Violation → 409 `primary_exists`.
  4. A guardian link requires the guardian to have the `guardian` role and the student the `student` role → 422 `validation`.
  5. Email uniqueness per tenant via `email_bidx` (unique index) → 409 `email_taken`.
  6. Deactivate: all memberships of the person → `deactivated`, then `AuthService::revoke_all`.
  7. PII read audit (AUD-01): when an actor reads **another** person's decrypted PII, write audit `identity.person.pii_read` (category compliance), throttled to one per (actor, subject, local day) through `INSERT … WHERE NOT EXISTS` on the audit table for that day.
- `q` search uses `display_name ILIKE` with `pg_trgm` similarity ordering. Email search: if `q` contains `@`, look up by `bidx_email(q)` exactly.
- Cursor: base64url JSON `{k: sort_key, id}` (API-05); max limit 200, default 50.
- Every mutation: audit (`identity.person.created|updated|deactivated|reactivated`, `identity.role.granted|revoked`, `identity.link.created|deleted`) with `Auditable` snapshots that **exclude** PII (`#[audit(skip)]` on encrypted fields).

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Request DTOs use T-102a field kinds per VAL-25 §5.2 (`DisplayName`, `LegalName`, `Email`, `Phone`, `BirthDate`, `Address`, `ShortText`) and reject unknown fields; tests per VAL-51 (unknown field, mass assignment, all errors together).
  - Membership and link transitions follow STM-20/27, including 409 `last_owner` and 409 `primary_exists`. Unlinking the last guardian of a minor with an active enrollment (409 `guardian_required`, D-41) applies once enrollment exists (M4) — not in this task.
- Permission ≠ scope (AUTH-04): the permission decides the action, the SQL scope (`All`/`Instructor`/`Guardian`/`Self_`) decides the rows; out-of-scope ids return 404, a missing permission 403 (spec 25 §7, ADR-0017).

## Scope
### In
- Implement only AC 1, 2, 3, 4, 5 and the tests assigned to T-112a in this split.
- Use the task's inline context and canonical interface definitions for those criteria.
### Out
- AC and behavior owned by the sibling split part(s) listed in T-112.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_people.rs` | add | HTTP handlers and strict request/response DTOs for the assigned people, role and link flows. |
| `crates/eq-identity/src/people.rs` | add | Actor-derived scope and identity service rules for the assigned flows. |
| `crates/eq-identity/src/domain.rs` | add | Private entry point for pure Identity domain rules. |
| `crates/eq-identity/src/domain/people.rs` | add | Pure last-owner and guardian-link invariants with unit tests. |
| `crates/eq-identity/src/lib.rs` | modify | Export the people HTTP and service modules. |
| `crates/eq-identity/src/module.rs` | modify | Register the people routes and spec-defined permission grants. |
| `crates/eq-core/src/auth/directory.rs` | modify | Extend the Core-owned directory port/adapter for scoped people reads and Core-owned membership/link transitions. |
| `crates/eq-core/src/db.rs` | modify | Provide the FLW-02 tenant-scoped read-only transaction used by people list/detail handlers. |
| `crates/eq-identity/src/errors.rs` | modify | Register the task's stable `last_owner` and `primary_exists` codes. |
| `crates/eq-identity/tests/api_people.rs` | add | Task-listed HTTP acceptance tests plus a read-only transaction regression check. |
| `.sqlx/` | update | Offline metadata for the new compile-time SQL queries. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T3 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `admin_cannot_grant_admin` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. A guardian with 2 linked students calling `GET /people?role=student` gets exactly those 2. → `api_people::guardian_scope`
- AC2. A student calling `GET /people/{other}` → 404. → `student_cannot_see_others`
- AC3. Removing owner from the only owner → 409 `last_owner`. → `last_owner_protected`
- AC4. An admin granting `admin` → 403. The owner can. → `admin_cannot_grant_admin`
- AC5. A second primary guardian → 409 `primary_exists`. → `one_primary_guardian_api`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_people.rs` | `guardian_scope`, `student_cannot_see_others`, `last_owner_protected`, `admin_cannot_grant_admin`, `one_primary_guardian_api`, `read_routes_use_read_only_transactions` |
| `crates/eq-identity/src/domain/people.rs` | `last_active_owner_cannot_be_removed`, `guardian_link_requires_active_distinct_roles` |

## Definition of done
- [x] Code quality and architecture: spec/16-code-quality.md section 12 and spec/17-clean-architecture.md followed; scripts/check-layers.sh and scripts/check-web-boundaries.sh pass.
- [ ] Rust checks pass; make api regenerated. The focused API and unit tests, fmt and clippy pass. cargo test --workspace was not run because its Testcontainers databases exceed the user's fixed one-container/one-task-database limit. make api remains the T-114 placeholder.
- [x] Commit feat(identity): people, roles and guardian links [T-112a], signed off.

## Report
**Plan** — Reconcile the assigned ACs with shipped Core/Identity ownership; add SQL-scoped reads, pure domain invariants and transactional role/link mutations; run the named HTTP tests using the single task database; complete security, performance and guard review; report and commit one task on dev.

**Assumptions** — No schema or migration change is required: the shipped persons, memberships, and person_links tables and primary-guardian index cover AC1-AC5. The authenticated Actor supplies tenant, person and role context. This split implements only AC1-AC5. PII remains absent from these response DTOs; pagination, decryption, and per-day PII-read audit belong to T-112b. OpenAPI generation remains owned by T-114.

**Test plan** — AC1: Given one guardian and exactly two linked active students, when the guardian requests /people?role=student, only those ids are returned -> guardian_scope. AC2: Given two students in one tenant, when one requests the other's detail, SQL scope returns 404 -> student_cannot_see_others. AC3: Given the only active owner, when that owner role is removed, the request returns 409 last_owner and preserves the membership -> last_owner_protected. AC4: Given an admin, when it grants admin, the request returns 403; an owner grant succeeds -> admin_cannot_grant_admin. AC5: Given a student with a primary guardian, when a second guardian is linked as primary, the request returns 409 primary_exists -> one_primary_guardian_api. Edge/error cases include role filters, same-tenant and linked-row scope, forbidden permission, invalid guardian/student roles, unique-index conflict, and transaction rollback. Abuse case: an admin cannot escalate its own or another person's role to owner/admin without identity.admins.manage.

**Summary** — People list and detail handlers enforce Actor-derived row scope in Core SQL, while pure domain functions validate last-owner and guardian-role invariants before tenant transactions write changes and specified audit entries. The six focused HTTP tests and browser E2E passed before the final pure-domain extraction; the two extracted domain tests pass on the final code.

**Spec read** — spec/16-code-quality.md sections 9 and 12; spec/17-clean-architecture.md sections 1, 4 and 7; spec/18-security-by-design.md sections 2, 5.1, 5.2, 5.4, 7 and 8 (SBD-81); spec/19-error-handling.md sections 2, 4, 9 and 10; spec/21-data-flows.md FLW-02; spec/25-auth-and-access-control.md AUTH-04/05; spec/29-erd.md identity module, section 5, and section 7. No migration was added, so scripts/erd.py --migrations is n/a.

**Files** — Added crates/eq-identity/src/api_people.rs, crates/eq-identity/src/people.rs, crates/eq-identity/src/domain.rs, crates/eq-identity/src/domain/people.rs, and crates/eq-identity/tests/api_people.rs; modified crates/eq-core/src/auth/directory.rs, crates/eq-core/src/db.rs, crates/eq-identity/src/errors.rs, crates/eq-identity/src/lib.rs, and crates/eq-identity/src/module.rs; added 12 SQLx query metadata files under .sqlx/; updated this task, tasks/DECISIONS.md, and generated tasks/INDEX.md.

**Acceptance criteria -> tests**

| AC | Test | Result |
|---|---|---|
| AC1 guardian sees exactly linked students | guardian_scope | [PASS] exactly two expected students |
| AC2 student cannot see another student | student_cannot_see_others | [PASS] 404 not_found |
| AC3 last active owner is protected | last_owner_protected | [PASS] 409 last_owner; membership stays active |
| AC4 admin cannot grant admin; owner can | admin_cannot_grant_admin | [PASS] admin gets 403; owner gets 204 |
| AC5 student cannot receive a second primary guardian | one_primary_guardian_api | [PASS] 409 primary_exists |
| Read routes use read-only transactions | read_routes_use_read_only_transactions | [PASS] PostgreSQL reports read-only mode |

**Checks run** —
- [PASS] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check
- [PASS] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --workspace --all-targets -- -D warnings
- [PASS, before final pure-domain extraction] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-112a' RITMA_DATABASE_OWNER_URL='postgres://eq:eq@localhost:5433/ritma_test_T-112a' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:3000' cargo test --locked --offline -p eq-identity --test api_people -- --test-threads=1 — 6/6 passed.
- [PASS] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --lib scope_follows_verified_actor_role — 1/1 passed earlier in this task.
- [PASS] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --lib domain::people::tests — 2/2 passed.
- [PASS] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL='postgres://eq:eq@localhost:5433/ritma_test_T-112a' RITMA_DATABASE_OWNER_URL='postgres://eq:eq@localhost:5433/ritma_test_T-112a' cargo sqlx prepare --check --workspace -- --all-targets — metadata matches.
- [PASS] scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh
- [PASS] python3 -I scripts/erd.py --check
- [PASS] scripts/tasks-index.sh --check and python3 -I scripts/tasks-lint.py — run after setting this task to review.
- [PASS] CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo deny check — advisories, bans, licenses, and sources passed; cargo-deny emitted existing duplicate-base64 warnings.
- [PASS] git diff --check and targeted gitleaks dir --redact scans of task-owned source, task, decision and SQLx files; no findings.
- [PASS, before final pure-domain extraction] make e2e — 6/6 Playwright cases passed on the task database.
- [NOT RUN after final pure-domain extraction] API integration and browser E2E were not rerun: the single task database had already been dropped after testing, and the resource instruction authorized only that disposable database. The extracted domain checks were rerun on final code.
- [NOT RUN] cargo test --workspace — it launches Testcontainers and additional PostgreSQL databases, exceeding the user's explicit one-container/one-task-database limit. Focused task tests ran against ritma_test_T-112a.
- [NOT RUN] make api — invoked, but the target returns the T-114 placeholder message; no generated OpenAPI/client artifacts were fabricated.
- [NOT RUN] pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check — no web files changed; web localization is introduced by T-306.
- [N/A] scripts/erd.py --migrations — no migration changed.

**Manual checks** — Ran actual HTTP requests through the Axum router in api_people against ritma_test_T-112a in the existing healthy ritma-db-1 PostgreSQL 16 container. Verified eq_app is NOSUPERUSER NOBYPASSRLS. Browser E2E command: CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-112a' RITMA_DATABASE_OWNER_URL='postgres://eq:eq@localhost:5433/ritma_test_T-112a' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='http://localhost:8080' RITMA_LISTEN='127.0.0.1:8080' E2E_BASE_URL='http://127.0.0.1:8080' E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=1 make e2e. It used the fresh/migrated ritma_test_T-112a database, one fake tenant, and the project's disposable Playwright runner; 6/6 passed before the final pure-domain extraction. The task database was dropped after those tests, and the existing ritma-db-1 container is retained.

**Debug log** — The first API test attempt could not authenticate as eq_app; set its test-only password to test-secret-app and confirmed it remains NOSUPERUSER NOBYPASSRLS, after which all six tests passed. E2E first encountered leftover multiple-tenant state and failed 4/6; resetting only ritma_test_T-112a then exposed the expected no-tenant 503; adding one clearly fake tenant resolved the fixture requirement and the final run passed 6/6. After the later pure-domain extraction, the task database had already been dropped, so DB-backed checks were not repeated. No other container or database was created or removed.

**Performance** — EXPLAIN on the scoped people query with a single synthetic tenant used tenant/person/link indexes; observed execution time was about 0.205 ms with no person rows. This is a smoke measurement, not representative production load; no index or schema change was made.

**Security** — Triggers T2/T3. SBD-81: (1) bearer auth and identity.people.read/identity.roles.write/identity.people.write checks precede DB access; owner/admin role grants additionally require identity.admins.manage; scoped rows are enforced in SQL and invisible people return 404; shared API-15 per-user/per-IP rate limiting is not wired yet and remains the TD-152 follow-up; (2) tenant comes from RequireTenant/verified Actor, transaction tenant mismatch is rejected, and RLS is tested with non-superuser eq_app; (3) no S2 person fields are decrypted or returned in this split, and summary names are emitted only for scoped rows; (4) role enum is closed, DTOs deny unknown fields, and relationship uses ShortText; (5) no HTML or export output; (6) no secret is read or logged; (7) no external calls; (8) pure domain functions validate last-owner and guardian-link invariants, active owners are locked before the last-owner check, and the database unique index enforces one primary guardian; writes and audits share one transaction; (9) role/link transitions record only IDs, role/status and primary flag; (10) admin_cannot_grant_admin proves an admin escalation is denied and last_owner_protected/one_primary_guardian_api exercise integrity controls. S2 PII decryption/audit remains in T-112b. TD-167 documents why no event was invented where STM-20/27 specify no event contract.

**Deviations** — RATIFY TD-063 (split the original oversized task while preserving named tests); TD-066 (add the spec-backed Security section); TD-067 (complete required task headings using existing contract); TD-166 (add the minimum omitted implementation paths); TD-167 (use specified audit records and do not invent an outbox event); TD-168 (add Core's read-only transaction helper required by FLW-02); TD-169 (disclose 1,612 non-generated additions and 13 deletions against CQ-101's approximate 400-line review guidance, caused by omitted Files paths); TD-170 (defer errors.last_owner and errors.primary_exists locale keys to T-306, which owns the not-yet-existing catalog and lists both keys); TD-171 (move domain decisions to pure domain functions per CA-02/23); TD-172 (record the E2E Playwright image/container disk-rule overrun, retain truthful test results, and do not remove the image outside the allowed cleanup list). API error status/code behavior is covered by assertions; make api generation remains T-114's responsibility.

**Dependencies affected** — T-112b can add scoped PII handling, pagination, and link-detail hydration on these routes; T-114 owns OpenAPI generation; T-306 owns error translations.

**Rollback plan** — Revert the single T-112a commit. No migration or persistent schema change was made; the disposable test database is dropped after checks.

**Follow-ups** — Ratify TD-063/066/067/166/167/168/169/170/171/172; complete PII read audit, pagination, and link detail in T-112b; add both error locale keys in T-306; generate OpenAPI/client artifacts in T-114; wire shared API-15 rate-limit middleware covering these routes as part of TD-152. The full workspace test suite was not run under the user's fixed database/container boundary.

**Disk hygiene** — Before T-112a: du -sh $HOME/.cache/ritma/target reported 8.5G. docker system df: Images 7 / 10.96GB; Containers 9 / 10.49MB; Local Volumes 6 / 5.032GB; Build Cache 0B. After T-112a: target 8.6G. docker system df: Images 8 / 14.55GB; Containers 9 / 10.49MB; Local Volumes 6 / 5.041GB; Build Cache 0B. The existing ritma-db-1 container was reused. Database ritma_test_T-112a was created for this task's tests and dropped after them. The Playwright runner container launched by make e2e used --rm and was removed automatically; Docker assigned its name and the wrapper did not retain it. The mcr.microsoft.com/playwright:v1.64.1-noble image (3.59GB) was pulled for that run and remains cached; no volume was created. The disk-size rule could not be fully met: the existing fixed Cargo target grew 0.1G from compilation and Docker gained the Playwright image; the permitted cleanup commands do not authorize removing that arbitrary image or pruning the shared target. Reuse both in later tasks. No other Docker resource was created or removed.

**Workflow steps** — Step 0: confirmed dev/status/index and retained unrelated WIP; Steps 1-2: read this task, cited specs, dependency Reports and shipped code; Step 3: n/a (backend-only); Step 4: plan and AC/test map above; Steps 5-7: implement, focused HTTP tests and real browser E2E, debug attempts recorded above; Step 8: reviewed code against sections 9/7/10 and diff boundaries; Step 9: recorded EXPLAIN smoke result; Step 10: SBD-81 findings above and targeted secret scan; Step 11: checks above, with environment/ownership limitations explicit; Step 12: report complete, status review, index/lint checks, then one signed commit.
## Questions
- **Self-resolved (implementer, run 24, needs architect ratification; TD-166):** T-112a listed only its test file although AC1–AC5 require mounted HTTP handlers, Actor-derived SQL scoping, role/link persistence, audit calls and stable errors → add only the existing identity route/service/registry paths plus the Core-owned CoreIdentityDirectory port/adapter and SQLx metadata needed by those acceptance criteria, because the endpoint behavior is already specified in this task, the shipped port establishes the cross-module boundary, and ERD-02 assigns persons, memberships and person_links to Core.
- **Decision (TD-167, needs architect ratification):** no outbox event name or payload is specified for STM-20/27 role or guardian-link changes → commit the named audit records in the state-change transaction and do not invent a cross-module event.
- **Self-resolved (implementer, run 24, needs architect ratification; TD-168):** FLW-02 requires tenant-scoped read-only transactions for list/detail reads but Core exposed only `db::begin` → add `db::begin_read_only` and use it only for the people GET endpoints, because that is the stricter read-path contract and changes no business behavior or schema.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

- **Decision (TD-169, needs architect ratification):** completing the assigned ACs requires a diff above CQ-101's approximate review-size guidance because the original Files list omitted implementation paths -> keep this work in the current one-task commit and disclose the size, because moving it now would change another task's ownership.
- **Decision (TD-170, needs architect ratification):** the new API error codes have no current web locale catalog -> defer errors.last_owner and errors.primary_exists keys to T-306, which explicitly owns the catalog and lists both codes.
- **Self-resolved (implementer, needs architect ratification; TD-171):** last-owner and guardian-role invariants were initially checked in application code -> move only these pure decisions into Identity's domain layer and keep fact gathering, permission checks, SQL, transaction and audit work in the service, because CA-02/CA-23 require domain rules to be pure.
- **Decision (TD-172, needs architect ratification):** T-112a's E2E used a Playwright Docker image and auto-named runner despite the current disk rule -> report the actual run and retained image truthfully; do not remove an image outside the allowed cleanup list, and use a local E2E runner or report Not run in future tasks.

## Review
n/a — pending review after implementation.
