---
id: T-110a4B
title: Register the spec-defined Identity write permission
milestone: M1
status: todo
depends_on: [T-110a4A]
spec_refs: [IDN-10, IDN-11, SBD-40, SBD-42]
estimated_size: S
---

## Goal
Expose the existing identity.people.write permission with only its spec-defined Admin grant.

## Context
The permission key and Admin grant already appear in spec/modules/identity.md §3. The login permission snapshot and owner-without-admin fallback shipped in T-110a4 and are covered by TD-118.

## Scope
### In
- Register identity.people.write for ActorRole::Admin only.
- Verify the Identity module's registered permission list and the owner fallback when no active admin exists.
- Make the auth integration fixture accept the task-named ritma_test_<TASK_ID> database.
### Out
- New permission keys/grants, permission calculation changes, bearer middleware and role switching.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/module.rs | modify | Register the exact spec-defined permission and Admin grant. |
| crates/eq-identity/tests/api_auth.rs | modify | Owner/admin acceptance test and task-named DB guard/fixture. |
| tasks/M1-core/T-110a4B-identity-permissions.md, tasks/DECISIONS.md, tasks/INDEX.md | read/modify/generate | Reference TD-118 and update task state. |

## Interfaces
- Use Module::permissions and PermissionDef.
- Reuse the shipped LoginHttpState permission snapshot; owner fallback follows IDN-11.

## Business rules
identity.people.write is Admin-only in the Identity module. The owner receives that permission only under IDN-11's no-active-admin condition.

## Security
- Triggers: T1 and T8 (SBD-30).
- Register no additional grant; disabled modules continue to fail closed.
- Abuse test: owner permission is absent when an active tenant admin exists.

## Implementation notes
Preserve the existing permission calculation and test both sides of the owner fallback. Use only the task-specific disposable test database.

## Acceptance criteria
- AC1. IdentityModule exposes identity.people.write for Admin only → defaults_schema_and_summary_match_contract.
- AC2. Owner receives the Admin set only when no active Admin exists → owner_gets_admin_perms_without_admin.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/src/module.rs | defaults_schema_and_summary_match_contract |
| crates/eq-identity/tests/api_auth.rs | owner_gets_admin_perms_without_admin |

## Definition of done
- [ ] Both named tests pass on a fresh ritma_test_T-110a4B database in the existing PostgreSQL 16 container; drop it afterward.
- [ ] fmt, focused Clippy, task lint/index, cargo deny and applicable guards pass.
- [ ] No other permission key, grant, role or schema object is introduced.
- [ ] Report includes Workflow steps 0–12 and resource snapshots.
- [ ] Signed commit feat(identity): register the spec-defined write permission [T-110a4B] is pushed to dev.

## Report
**Plan** — (1) Confirm the key/grant in the module spec → verify exact Admin role. (2) Register it and keep the test fixture task-database-safe → verify module and integration tests. (3) Exercise owner with/without an active admin, then run focused guards.
**Assumptions** — The owner fallback already shipped in T-110a4; this part only registers its spec-defined permission and verifies it.
**Test plan** — AC1: module list has one Admin-only write permission → defaults_schema_and_summary_match_contract. AC2: owner fallback present/absent across membership states → owner_gets_admin_perms_without_admin. Errors: none added.
**Spec read** — spec/modules/identity.md §3, IDN-10/11, SBD-40/42, spec 16 §9/12 and spec 19 §10.
**Security** — SBD-81 review planned; no new permission grant beyond the mapped Admin grant.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification; TD-118):** T-110a5 prohibits adding grants while AC2 requires identity.people.write → register only the already-specified Admin grant and use IDN-11's owner fallback; no permission is invented.
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** this ordered child uses uppercase suffix B so it sorts before lowercase aggregate T-110a4a under LC_ALL=C without changing the DAG guard.

## Review
n/a — awaiting maintainer review.
