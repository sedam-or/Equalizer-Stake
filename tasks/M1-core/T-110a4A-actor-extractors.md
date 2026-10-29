---
id: T-110a4A
title: Add shared Actor request extractors
milestone: M1
status: review
depends_on: [T-110a4, T-110a2]
spec_refs: [IDN-10, AUTH-01, SBD-40, SBD-42, SBD-50]
estimated_size: S
---

## Goal
Expose the verified request Actor and an optional Actor extractor from eq-core.

## Context
T-110a4 supplies the tenant-aware Identity HTTP boundary; T-110a2 supplies the signed token contract. This part owns only the shared request type and extraction behavior.

## Scope
### In
- Add Actor with private identity/context fields, typed getters, has/require, and redacted Debug.
- Add required Actor and optional MaybeActor FromRequestParts extractors.
- Add exact unit tests for missing verified identity, registered/absent permission and optional extraction.
### Out
- Bearer parsing or validation, permission calculation, IdentityModule grants, database reads and cookie role switching.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-core/src/auth.rs | modify | Actor, MaybeActor, request extraction and unit tests. |
| tasks/M1-core/T-110a4a-actor-auth-and-permissions.md, T-110a4A-actor-extractors.md, T-110a4B-identity-permissions.md, T-110a4C-bearer-authentication.md, T-110a4b-role-switch-and-csrf.md, T-110a5-actor-permissions-and-role-switch.md, tasks/DECISIONS.md, tasks/INDEX.md | modify/create/generate | Record the CQ-101 split, decisions and ordered task graph. |

## Interfaces
- Actor fields remain private; Identity constructs it only after verification.
- Actor::has(&str) checks the effective set; Actor::require(&'static str) returns the existing forbidden error.
- MaybeActor(pub Option<Actor>) is None only when no verified Actor extension is present.

## Business rules
No new permission, role or business value is introduced.

## Security
- Triggers: T1, T3 and T8 (SBD-30).
- Keep tenant/person/session identifiers and ActorCtx redacted in Debug; no bearer token is stored in Actor.
- A missing required Actor fails with unauthenticated; MaybeActor alone treats an absent extension as optional.
- Abuse tests: missing required extension, absent optional extension, and missing permission.

## Implementation notes
Use only the verified Actor extension inserted by Identity middleware. Do not parse request headers in eq-core.

## Acceptance criteria
- AC1. Actor extraction rejects a missing verified Actor → actor_requires_authenticated_extension.
- AC2. A present grant succeeds and an absent grant returns forbidden → actor_requires_registered_permission.
- AC3. MaybeActor returns None without an extension and Some with one → maybe_actor_is_optional.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-core/src/auth.rs | actor_requires_authenticated_extension, actor_requires_registered_permission, maybe_actor_is_optional |

## Definition of done
- [x] All three named tests pass.
- [x] cargo fmt, focused eq-core Clippy/tests, task lint/index and applicable guards pass.
- [x] No bearer or other secret is stored on Actor; no new permission or error code is added.
- [x] Report records Workflow steps 0–12 and disk/resource before/after data.
- [x] Signed commit feat(core): add shared Actor request extractors [T-110a4A] is pushed to dev.

## Report
**Plan** — (1) Add the required and optional extractors → verify exact unit tests. (2) Check field privacy and redacted Debug → verify source review and hygiene scan. (3) Run focused Rust, task and guard checks using the fixed shared target.
**Assumptions** — Only Identity middleware inserts the verified Actor extension; no business or permission rule is added here.
**Test plan** — AC1: missing extension → actor_requires_authenticated_extension. AC2: present/missing permission → actor_requires_registered_permission. AC3: absent/present extension → maybe_actor_is_optional. Error: unauthenticated only.
**Summary** — Added a typed `Actor`, required and optional request extractors, permission checks, and redacted Debug output in `eq-core`. All three focused unit tests pass, including the missing authenticated-extension and absent-grant cases.
**Spec read** — `spec/18-security-by-design.md` §§5.4, 8 (SBD-81), and 9 (SBD-50); `spec/25-auth-and-access-control.md` §§1–2; `spec/modules/identity.md` §§1 and 3; `spec/16-code-quality.md` §§9 and 12; `spec/17-clean-architecture.md` §§1, 4 and 7; `spec/19-error-handling.md` §10. Dependency Reports and shipped code for T-110a4 and T-110a2 were read before implementation.
**Files** — Modified `crates/eq-core/src/auth.rs`; created the ordered T-110a4A/B/C task files and the retained T-110a4a/T-110a4b aggregate/part task files; updated `tasks/M1-core/T-110a5-actor-permissions-and-role-switch.md`, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md` to record the CQ-101 split and dependency order.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1: required extraction rejects a missing verified Actor | `actor_requires_authenticated_extension` | ✅ |
| AC2: present grant succeeds and absent grant is forbidden | `actor_requires_registered_permission` | ✅ |
| AC3: optional extraction returns None/Some according to extension presence | `maybe_actor_is_optional` | ✅ |
**Checks run** — ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`; ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked -p eq-core auth::tests -- --test-threads=1` (3 passed); ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked -p eq-core --all-targets -- -D warnings`; ✅ `scripts/tasks-index.sh --check`; ✅ `python3 -I -B scripts/tasks-lint.py`; ✅ `python3 -I -B scripts/tests/test_tasks_lint.py` (8 passed); ✅ `scripts/check-deps.sh`; ✅ `scripts/check-literals.sh`; ✅ `scripts/check-web-boundaries.sh`; ✅ `scripts/check-layers.sh`; ✅ `python3 -I -B scripts/erd.py --check`; ✅ `git diff --check`; ✅ `gitleaks dir --redact=100 --no-banner crates/eq-core/src/auth.rs` (no leaks). Not run: workspace-wide Rust checks and E2E (this part changes no database or runtime endpoint; those checks are outside its DoD and would exercise unrelated tasks); database and migrations (n/a, none changed); hosted CI (not available locally).
**Manual checks** — n/a; no endpoint is owned by this part.
**Debug log** — None.
**Performance** — n/a; no query or request-path lookup is added.
**Security** — Triggers T1/T3/T8. SBD-81: auth mode/permission placement and SQL scope are owned by the API boundary and remain unchanged; no request-supplied tenant is read; Actor identity/context fields are private and redacted in Debug; no DTO, secret, crypto operation, external call, state mutation or audit event is added. Abuse tests cover missing identity and absent permission. No audit finding required a code change. Gitleaks found no leaks.
**Deviations** — RATIFY TD-122: the CQ-101 split required child IDs that sort before the retained lowercase aggregate; uppercase A/B/C suffixes satisfy the existing bytewise task-index ordering without changing its guard.
**Dependencies affected** — T-110a4B and T-110a4C use Actor.
**Rollback plan** — Revert this part's signed commit; no persisted state changes.
**Follow-ups** — Architect ratification of TD-122; review CQ-101 split sizing across the aggregate's three ordered parts.
**Workflow steps 0–12** — 0: resumed the existing in-progress task and checked the generated index; 1: read this task and cited specs; 2: read T-110a4/T-110a2 Reports and shipped interfaces; 3: n/a (backend-only); 4: recorded plan, assumptions and AC-to-test plan before implementation; 5: implemented the Actor/extractors and named tests; 6: n/a (no endpoint/runtime behavior owned by this part); 7: corrected one focused-test compile failure, then the rerun passed; 8: reviewed visibility, permission/error use, cloning and Debug redaction against the task/spec; 9: n/a (no measured optimization target); 10: completed SBD-81 audit above; 11: reran formatting, focused tests/Clippy, task checks, guards, ERD check, diff check and secret scan; 12: recorded this report, set status to review, regenerated the index, and prepared the single signed task commit.

## Questions
- **Self-resolved (implementer, run 9, needs architect ratification; TD-122):** the split parts must sort below the original aggregate → use uppercase suffix A/B/C because scripts/tasks-index.sh fixes LC_ALL=C and compares IDs bytewise; no task-index rule is weakened.

## Review
n/a — awaiting maintainer review.
