---
id: T-112b
title: PII audit, normalized email, deactivation and pagination
milestone: M1
status: review
depends_on: [T-112a]
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
- Implement only AC 6, 7, 8, 9 and the tests assigned to T-112b in this split.
- Use the task's inline context and canonical interface definitions for those criteria.
### Out
- AC and behavior owned by the sibling split part(s) listed in T-112.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/api_people.rs` | modify | Add the assigned PII detail, duplicate-email, deactivation and cursor-page HTTP behavior. |
| `crates/eq-identity/src/people.rs` | modify | Orchestrate scoped PII reads, person creation, deactivation, audit and paginated directory reads. |
| `crates/eq-identity/src/errors.rs` | modify | Register the task-defined `email_taken` conflict code. |
| `crates/eq-identity/src/module.rs` | modify | Register the task-defined people creation and deactivation routes. |
| `crates/eq-core/src/auth/directory.rs` | modify | Extend the Core-owned people port/adapter for PII details, person creation, deactivation and cursor pages on Core tables. |
| `crates/eq-audit/src/record.rs` | modify | Expose the audit module's task-scoped daily-deduplicated record operation. |
| `crates/eq-audit/src/repo.rs` | modify | Persist the daily-deduplicated audit event on the audit-owned table. |
| `crates/eq-identity/tests/api_people.rs` | modify | Add the five named AC6–AC9 and security-matrix integration tests. |
| `.sqlx/` | update | Add offline metadata for the new compile-time SQL queries. |
| `tasks/DECISIONS.md` | modify | Record implementation, time-source, event-contract, sizing and generator decisions TD-173–TD-177. |
| `tasks/INDEX.md` | generated | Regenerate from this task's status. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T3, T4 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `people_endpoints_role_matrix` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC6. An admin reading another person's PII twice in a day → exactly one `identity.person.pii_read` audit row. → `pii_read_audited_once_per_day`
- AC7. A duplicate email (case and whitespace different) → 409 `email_taken`. → `email_taken_normalized`
- AC8. Deactivate → the person's refresh token no longer refreshes (401). → `deactivate_revokes_sessions`
- AC9. Pagination over 120 people with limit 50 returns 50/50/20 with no duplicates. → `cursor_pagination`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/tests/api_people.rs` | `pii_read_audited_once_per_day`, `email_taken_normalized`, `deactivate_revokes_sessions`, `cursor_pagination`, `people_endpoints_role_matrix` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass
- [ ] `make api` regenerated — Not run: the Makefile target is a T-114 placeholder (TD-177).
- [x] Commit `feat(identity): people, roles and guardian links [T-112b]`, signed off

## Report
**Start (Step 0)** — Resumed the existing T-112b `in_progress` work on `dev`; no task branch was created because the operator directs all task commits to `dev`. T-112a is `review` and is the only dependency.

**Plan** —
1. Extend the existing Identity detail/create/deactivate/list handlers and service flow for AC6–AC9 → verify the named request-level tests.
2. Keep people SQL in Core and the daily-deduplicated audit insert in Audit → verify the module boundary guards and tenant-scoped integration suite.
3. Use only the ERD-mapped existing columns and refresh SQLx offline metadata → verify SQLx prepare/check and ERD drift checks.
4. Run focused Rust checks and browser E2E against a fresh `ritma_test_T-112b` database in `ritma-db-1` → record exact pass counts and environment limitations.
5. Audit the diff and security paths, write this Report, run task-index/lint checks, and commit one signed-off task commit to `dev`.

**Assumptions** — `spec/29-erd.md` §7 maps `persons` to `0002_core_tenancy.sql` (T-101) and `audit.entries` to `0006_audit.sql` (T-109); the existing `spec/06-domain-model.md` DDL supplies every needed column, so no migration is needed. Core's `CoreIdentityDirectory` port owns people-table SQL, and Audit owns `audit.entries` SQL. Only the existing `identity.people.read_pii` permission is used. No new outbox event contract is inferred (TD-176).

**Dependency review (Step 2)** — Read T-112a's Report, especially TD-167/168/171/172 and the API-15, T-114 and T-306 follow-ups. Its shipped Core directory boundary, Identity route/service layout and transactional audit calls match this task's implementation; the shared rate limiter remains deferred under TD-152.

**Test plan** —
| Given / When / Then | Test |
|---|---|
| Given an authorized admin and another in-scope person with encrypted PII, when the admin reads the detail twice on one tenant-local day, then one compliance audit row exists and the permitted PII is returned. | `pii_read_audited_once_per_day` |
| Given an existing normalized email, when a second person is created with different case and surrounding whitespace, then the API returns 409 `email_taken`. | `email_taken_normalized` |
| Given a person with a live refresh session, when an authorized admin deactivates them, then refresh returns 401 and the session is revoked. | `deactivate_revokes_sessions` |
| Given 120 people and limit 50, when the client follows each cursor, then pages contain 50, 50 and 20 unique people. | `cursor_pagination` |
| Given callers with and without the required permission/scope, when they attempt the people endpoints, then forbidden or cross-tenant access is rejected before PII disclosure. | `people_endpoints_role_matrix` |

Edge/error/abuse cases: empty and malformed cursors, missing/out-of-scope person, normalized duplicate blind-index conflict, final-owner protection, RLS under non-superuser `eq_app`, denied PII reads and audit snapshots without personal fields.

**Design reference (Step 3)** — n/a; this is a backend/API task and changes no UI.

**Summary** — Added scoped person detail and creation, normalized-email conflict handling, tenant-local-day PII-read audit deduplication, deactivation with refresh-session and invite revocation, and cursor-paginated directory reads. Core and Audit retain ownership of their tables, with no schema or permission additions.

**Spec read** — `spec/06-domain-model.md` §§4.1, 4.2, 4.9; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§4, 8; `spec/19-error-handling.md` §10; `spec/20-state-machines.md` STM-20/26/27; `spec/21-data-flows.md` FLW-02/03; `spec/22-validation-and-sanitization.md` VAL-25/51; `spec/25-auth-and-access-control.md` AUTH-04/05; and `spec/29-erd.md` Core and Audit module dictionaries, §5 data classes and §7 migration/task map.

**Files** —
- Modified `crates/eq-identity/src/api_people.rs`, `people.rs`, `errors.rs`, and `module.rs` for the assigned HTTP and service behavior.
- Modified `crates/eq-core/src/auth/directory.rs` for Core-owned people reads/writes and page queries.
- Modified `crates/eq-audit/src/lib.rs`, `record.rs`, and `repo.rs` for daily-deduplicated audit writes.
- Modified `crates/eq-identity/tests/api_people.rs` with the five named acceptance/security tests.
- Updated `.sqlx/` offline metadata for changed compile-time queries; no migration changed.
- Updated this task, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md`.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC6 | `pii_read_audited_once_per_day` | Pass |
| AC7 | `email_taken_normalized` | Pass |
| AC8 | `deactivate_revokes_sessions` | Pass |
| AC9 | `cursor_pagination` | Pass |
| Security matrix | `people_endpoints_role_matrix` | Pass |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo fmt --all --check`
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --workspace --all-targets -- -D warnings`
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo test --locked --offline -p eq-identity --test api_people -- --test-threads=1` — 11/11, using `ritma_test_T-112b` in existing PostgreSQL 16 `ritma-db-1` as `eq_app` (`NOSUPERUSER`, `NOBYPASSRLS`).
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo test --locked --offline -p eq-audit --lib` — 6/6.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo test --locked --offline -p eq-identity --lib` — 43/43.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo test --locked --offline -p eq-core --lib new_directory_person_debug_redacts_secret_fields` — 1/1.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo sqlx prepare --workspace -- --all-targets`; ✅ `cargo sqlx prepare --check --workspace -- --all-targets` — metadata prepared and checked against the task database.
- ✅ `cargo deny check` — advisories, bans, licences and sources; no deny policy changes.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check`; migration check n/a because no migration changed.
- ✅ `scripts/tasks-index.sh --check`; ✅ `python3 -I scripts/tasks-lint.py`; ✅ `git diff --cached --check`; ✅ `gitleaks protect --staged --redact` — no leaks found.
- ✅ `cargo run --locked --offline -p eq-server -- migrate` — applied the existing migrations to the fresh task database.
- ⚠ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=0 make e2e` — 4/6 passed against `ritma_test_T-112b` in `ritma-db-1`; two WebKit page checks were Not run because host libraries `libicu74` and `libflite1` are unavailable.
- ❌ `make api` attempted; the target exits intentionally with “OpenAPI generation is added by T-114” (TD-177). No generated API artifacts were changed.
- Not run: `cargo test --workspace`; its Testcontainers suite would start another PostgreSQL container, outside the one-container resource limit.
- Not run: `make dev`; the focused reqwest integration suite exercised the real HTTP routes against the disposable task database without starting the shared dev stack.
- Not run: web lint/typecheck/test/i18n suites; this task changed no web source. The web build in `make e2e` passed.
- Not run: hosted GitHub Actions; unavailable from this local run.

**Manual checks (Step 6)** — The focused `api_people` suite sent real HTTP requests through the Identity router, covering success and error responses. Browser E2E used a fresh PostgreSQL 16.15 database, `ritma_test_T-112b`, inside the existing `ritma-db-1`; it passed 4/6 checks, with two WebKit page launches unavailable due to missing host libraries. The database was dropped after testing. Resource accounting: fixed Cargo target was 8.6G before task work and 8.9G after required builds; Docker snapshot changed from 8 images/9 containers/6 volumes to 8 images/10 containers/7 volumes, with no container, image or volume created or removed by this task. An unrelated `awesome_banzai` container was present in the final inventory; this task did not create it, and its owner/timing is unknown, so it and all `kinevo*`, `infrastructure*` and `eqs-*` resources were left untouched. The target grew due to required builds; no cache or unowned resource was removed.

**Debug log (Step 7)** — First API integration attempt used the wrong fake `eq_app` password and failed 0/11; role configuration confirmed `test-secret-app`, after which the same suite passed 11/11. First E2E attempt used a fresh database without the fake tenant seed and passed 2/6; after seeding the fixture, the rerun passed 4/6. The remaining two WebKit checks could not launch because `libicu74` and `libflite1` are absent. The `make api` attempt confirmed the Makefile placeholder owned by T-114; this is recorded in TD-177. The first database-drop command failed because it left the hyphenated database name unquoted; the quoted retry dropped only `ritma_test_T-112b`. No failure was hidden or marked green.

**Self code review (Step 8)** — Reviewed the diff against `spec/16` §9 and `spec/17` §7: Identity holds orchestration and typed invariants, Core queries only Core-owned tables, Audit writes only Audit-owned tables, SQL remains parameterized, no `SELECT *` or new `#[allow]` was added, and public Rust APIs carry spec references. `git diff --numstat -- crates/eq-*` measures 1,723 Rust source/test additions and 84 deletions, excluding generated SQLx metadata (CQ-101 overage recorded in TD-175).

**Performance (Step 9)** — Ran `EXPLAIN ANALYZE` on page, role, detail and two link queries with three fake people in the task database; measured plans were below approximately 0.12 ms on this small fixture. No production-scale latency claim is made; no schema/index change was authorized by this task.

**Security (Step 10)** — Triggers T2/T3/T4. SBD-81 review: permission checks precede data access; scope and tenant filters remain in SQL and RLS was exercised as `eq_app`; S2 fields are encrypted and disclosed only under `identity.people.read_pii`; PII audit snapshots are empty and no PII/secret logging was added; input uses the T-102a field kinds and unknown fields are rejected; SQL is parameterized; the role/scope abuse path is covered by `people_endpoints_role_matrix`. Shared API-15 rate limiting remains an existing cross-cutting follow-up TD-152. The task's state transitions audit in the caller transaction; no outbox event contract is invented (TD-176).

**Deviations** — RATIFY: inherited TD-063 (split the oversized source task), TD-066 (spec-backed Security section), and TD-067 (required report structure); TD-173 (add only mechanically omitted implementation paths); TD-174 (use injected Clock time for the daily PII audit); TD-175 (disclose CQ-101 review-size overage); TD-176 (do not infer an outbox event contract); TD-177 (defer API artifact generation to T-114). `make api` remains ungenerated until T-114 implements its target.

**Dependencies affected** — T-114 remains responsible for generated OpenAPI/client artifacts; T-306 remains responsible for web translations. No dependency interface or schema was changed.

**Rollback plan** — Revert the single T-112b task commit. No migration or schema compensation is needed; the disposable task database was dropped.

**Follow-ups** — Ratify TD-173–TD-177 and review the CQ-101 split sizing; wire shared API-15 limits per TD-152; generate OpenAPI/client artifacts in T-114; add the API error translations in T-306; run the two unavailable WebKit cases in an environment with the required libraries. No further task was started.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-173):** T-112b lists only its integration test although AC6–AC9 require HTTP handlers, Core-owned person/PII/pagination/deactivation operations, audit-owned daily deduplication and SQLx metadata → add only those existing module-boundary paths named in Files, because the task itself specifies each behavior and `spec/29-erd.md` maps every needed column to existing migrations; do not add a permission, event or schema object.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-174):** the PII audit deduplication window uses the injected Clock but ordinary audit insertion uses database statement time → supply the injected instant to the dedicated daily-deduplicated insert, because AGENTS.md §4.3 requires domain time to come from the injected Clock and the stored event must fall inside the same tenant-local-day window used for deduplication.
- **Decision (TD-175, needs architect ratification):** the measured non-generated implementation diff exceeds CQ-101's approximate review guidance → keep the assigned ACs together and disclose the overage because the omitted Files paths are required to implement this task's explicit contract.
- **Decision (TD-176, needs architect ratification):** create/deactivate transitions name audit actions but no outbox event contract → record the specified audit in the same transaction and do not derive an event name from the audit action, because events are cross-module contracts and TD-167 establishes the same boundary.
- **Decision (TD-177, needs architect ratification):** `make api` is an explicit T-114 placeholder and generated artifacts are outside T-112b Files → report the command as unavailable and leave generation to T-114, following TD-164.
