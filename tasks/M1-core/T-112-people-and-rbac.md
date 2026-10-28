---
id: T-112
title: Add the people API with scoped queries, role management, guardian–student links and deactivation
milestone: M1
status: draft
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
| T-112a | People scopes, role changes and guardian links; AC 1, 2, 3, 4, 5 | T-111, T-102a | draft |
| T-112b | PII audit, normalized email, deactivation and pagination; AC 6, 7, 8, 9 | T-112a | draft |

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
- [ ] Every original criterion and test name is assigned once to a dependent child.
- [ ] The task index reflects the child dependency chain.

## Report
**Deviations** — RATIFY: TD-063, split this CQ-101-sized scope into dependent implementation parts while retaining this file as the contract parent.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
