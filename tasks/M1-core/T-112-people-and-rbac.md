---
id: T-112
title: Add the people API with scoped queries, role management, guardian–student links and deactivation
milestone: M1
status: review
depends_on: [T-102a, T-110b, T-111]
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
- Track the split task contract and dependency order only.
### Out
- All implementation and tests, which are assigned to the child parts below.

## Split
| Part | Owns | Depends on | Status |
|---|---|---|---|
| T-112a | People scopes, role changes and guardian links; AC 1, 2, 3, 4, 5 | T-111, T-102a | todo |
| T-112b | PII audit, normalized email, deactivation and pagination; AC 6, 7, 8, 9 | T-112a | todo |

## Files
| Path | Action | Purpose |
|---|---|---|
| `tasks/M1-core/T-112a-*.md` | create or modify | Path named in this task's contract. |
| `tasks/M1-core/T-112b-*.md` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Every original acceptance criterion and test is assigned exactly once to a child part in the Split table.

## Tests to write
| File | Test names |
|---|---|
| n/a | Child parts own all implementation tests. |

## Definition of done
- [x] Every original criterion and test name is assigned once to a dependent child.
- [x] The task index reflects the child dependency chain.

## Report
**Plan** — 1. Compare this parent’s nine criteria with T-112a/T-112b scopes and test tables → verify each criterion and test name has exactly one owner. 2. Verify the child task frontmatter and dependency chain against the split table and generated index → align the parent’s stale status labels and record TD-165. 3. Check the task index/linter and inspect only this task’s documentation diff → confirm no implementation, schema, API or security contract was changed.

**Assumptions** — T-112 is the contract parent and contributes only one aggregate acceptance criterion; the implementation and test work belongs to T-112a and T-112b. Task frontmatter is the authoritative status source, with `tasks/INDEX.md` generated from it. The parent has no runtime behavior, migration, API artifact or UI change.

**Test plan** — AC1: Given the original T-112 context has criteria AC1–AC9 and named implementation tests, when the child scopes and test tables are compared, then AC1–AC5 belong only to T-112a and AC6–AC9 belong only to T-112b, with every test name assigned exactly once. Edge cases: ensure the five T-112a acceptance tests and five T-112b tests (including the role-matrix abuse test) are listed once and that the child dependency is ordered. Error/abuse cases: the parent adds no runtime path; the child-owned tests retain coverage for scope denial, last-owner protection, admin-role denial, unique primary guardian, PII audit, normalized email, deactivation revocation and pagination. Dependency check: T-112a depends on T-111/T-102a and T-112b depends on T-112a.

**Summary** — T-112 remains a tracking parent and assigns all nine original criteria and their named tests across the two dependent child tasks. Their frontmatter and index status are `todo`, and their dependency chain is T-112a after T-111/T-102a, then T-112b after T-112a. No endpoint, code, schema or test implementation changed.

**Spec read** — The parent’s inlined IDN-10/11/13, AUD-01, API-05/06/07, STM-20/26/27, VAL-25/51 and AUTH-04 contracts; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` §§1, 4; `spec/18-security-by-design.md` §§4–5; `spec/19-error-handling.md` §10; `spec/25-auth-and-access-control.md` §§1, 7. The task’s implementation details remain delegated to its child parts.

**Files** — Modified `tasks/M1-core/T-112-people-and-rbac.md` to align split status labels and complete this Report; modified `tasks/DECISIONS.md` to record TD-165; regenerated `tasks/INDEX.md`. T-112a and T-112b files were read but not changed.

**Acceptance criteria** —

| AC | Evidence | Result |
|---|---|---|
| AC1. Every original criterion and test is assigned exactly once to a dependent child | T-112a owns AC1–AC5 and lists `guardian_scope`, `student_cannot_see_others`, `last_owner_protected`, `admin_cannot_grant_admin`, `one_primary_guardian_api`; T-112b owns AC6–AC9 and lists `pii_read_audited_once_per_day`, `email_taken_normalized`, `deactivate_revokes_sessions`, `cursor_pagination`, `people_endpoints_role_matrix` | Pass: nine criteria and ten unique child test names have one owner each |
| Supporting check: the index reflects the child dependency chain | T-112a frontmatter depends on T-111/T-102a; T-112b frontmatter depends on T-112a | Pass: generated index matches these dependencies |

**Checks run** — `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py` pass. Manual contract comparison confirms the two child test-name lists are disjoint and cover their allocated AC ranges. `git diff --check` passes. Not run: Cargo tests, SQL tests/migrations, OpenAPI generation, web checks and E2E, because this aggregate only tracks the split and changes no runtime files. A read-only PostgreSQL catalog query confirmed the existing task database list; no test database was created. No migration changed, so `scripts/erd.py --migrations` is not applicable.

**Manual checks** — n/a; no runtime behavior is owned by T-112.

**Debug log** — The parent Split table listed both child tasks as `draft`, but their task frontmatter and generated index list them as `todo`. Following the task-frontmatter source of truth, the parent table was updated to `todo` and TD-165 was recorded. No child task status was changed.

**Performance** — n/a; no runtime code or query changed.

**Security** — No security control or runtime contract changed in this tracking task. The two implementation children retain their security sections for scoped queries, role checks, S2/S3 PII, and the named abuse-case tests.

**Deviations** — RATIFY TD-063 records the historical CQ-101 split; TD-066 and TD-067 are historical template/security-section corrections. RATIFY TD-165 aligns the parent’s split table with child task frontmatter and the generated index. No behavior, permission, role grant, error code, schema or event is added.

**Dependencies affected** — T-112a remains dependent on T-111 and T-102a; T-112b remains dependent on T-112a. T-113 continues to depend on T-112b.

**Rollback plan** — Revert the single signed T-112 documentation commit. No runtime, database or schema state is changed.

**Follow-ups** — Implement T-112a next; after it reaches `review`, proceed to T-112b. Architect ratification of the historical TD-063/066/067 and current TD-165 remains outstanding.

**Workflow steps 0–12** — 0: resumed on `dev` at `27accbb`; confirmed `dev` equals `origin/dev`, no tracked changes, preserved unrelated untracked WIP, confirmed T-112 is the lowest eligible task and its dependencies are `review`, then claimed it and regenerated the index; 1: read the full parent and child task contracts and cited specifications; 2: read T-102a, T-110b and T-111 Reports, their Deviations/Follow-ups and shipped field/auth code; 3: n/a (no UI); 4: recorded this plan and AC/test ownership check before editing the split status; 5: no implementation was in scope; 6: n/a (no runtime behavior); 7: no failures; 8: reviewed the task/decision/index diff; 9: n/a; 10: confirmed the parent adds no security behavior and preserves child security contracts; 11: ran index/lint/diff checks; 12: set the parent to `review`, regenerated and checked the index, staged only T-112-owned files, committed one signed commit and pushed only `dev`.

**Disk and resource accounting** — Before T-112, `du -sh "$HOME/.cache/ritma/target"` reported `8.5G`; `docker system df` reported Images 7/7/10.96GB/0B reclaimable, Containers 9/9/10.49MB/0B, Local Volumes 6/6/5.032GB/0B, Build Cache 0/0/0B. After task checks, the same counts and sizes were reported. Post-commit read-only snapshots reported volume size between 5.057GB and 5.068GB (ending at 5.057GB), while images, containers, volume count and build cache remained unchanged. The cause of this volume-size delta is unknown; no Docker/database resource was created or removed by T-112, and no cleanup was performed because the existing volumes were not created by this task. Existing `ritma-db-1` remained running; only pre-existing database `ritma_test_T-110a4b` was present. The disk-size rule was not confirmed for the Docker volume estimate. No Cargo command ran, and the fixed target directory was not modified. Protected kinevo*, infrastructure*, eqs* and equalizer-stake_* resources and all pre-existing untracked WIP were left untouched.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 23, needs architect ratification; TD-165):** the split table labels T-112a/T-112b `draft` while their frontmatter and generated index show `todo` → update the parent labels to the authoritative current status because both children are ready with the dependency chain verified.

## Review
Pending maintainer review.
