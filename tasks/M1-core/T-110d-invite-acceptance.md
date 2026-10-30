---
id: T-110d
title: Preview and accept identity invites
milestone: M1
status: blocked
depends_on: [T-110c]
spec_refs: [IDN-02, IDN-07, STM-20, STM-21, AUTH-06]
estimated_size: M
---

## Goal
Allow an invite holder to preview a pending invitation and accept it once by setting a policy-valid password and starting an authenticated session.

## Context
- T-111 defines GET /auth/invites/{token} → tenant_name, display_name, role, expires_at. Unknown tokens return 404; expired or accepted tokens return 410 invite_expired.
- POST /auth/invites/{token}/accept {password} validates Password against the current IdentityPolicy, marks the invite accepted, sets the credential password and timestamp, activates the invited membership through Core, creates the login response and refresh cookie, and audits identity.invite.accepted.
- Invite tokens are 32 random bytes; only their SHA-256 hash is stored. The raw token is never logged.

## Scope
### In
- Implement the public invite preview and acceptance endpoints and the acceptance transaction.
- Keep invite lookup tenant-scoped and token comparison constant-work as required by STM-21/AUTH-06.
### Out
- Invite issuance/resend (T-110c, the shipped dependency), reset, profile or TOTP behavior.

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/api_invites.rs | create | Preview and accept HTTP DTOs/handlers. |
| crates/eq-identity/src/invites.rs | modify | Accept and preview service behavior. |
| crates/eq-identity/src/repo.rs | modify | Invite lookup and single-use acceptance. |
| crates/eq-identity/src/module.rs | modify | Register routes. |
| crates/eq-identity/tests/api_invites.rs | modify | Exact acceptance tests. |
| tasks/M1-core/T-110d-invite-acceptance.md | modify | Plan and final Report. |
| tasks/INDEX.md | generate | Regenerate status index. |

## Interfaces
- GET /api/v1/auth/invites/{token} returns {tenant_name, display_name, role, expires_at}.
- POST /api/v1/auth/invites/{token}/accept accepts {password} and returns the existing login response plus the refresh cookie.
- Errors are the spec-defined 404, 410 invite_expired and 422 password_weak; add no error code.

## Business rules
Use only IdentityPolicy.password_min_length, invite_expires_days and the injected Clock. Acceptance is single-use and activates the membership in the same transaction as credentials, refresh family and audit.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 invite token hash/password hash; S2 profile fields in the existing login response.
- S/E: token guessing, replay or cross-tenant lookup → tenant-scoped hash lookup, single-use transaction and policy validation (SBD-40/41/42, STM-21).
- I: secret leakage → hash-only token persistence, redacted Debug and no token/PII logging (SBD-10/21/50).
- Abuse cases: accept_ok and accept_expired_410 cover valid, expired and replayed invite paths.

## Implementation notes
Reuse the invite repository/service shipped by T-110c and the existing LoginService success response; do not query memberships in the Identity repository.

## Acceptance criteria
- AC1. A valid invite accepted inside its TTL returns the login response and activates the membership → api_invites::accept_ok.
- AC2. Expired or accepted invite tokens return 410 invite_expired → accept_expired_410.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_invites.rs | accept_ok, accept_expired_410 |

## Definition of done
- [ ] Exact acceptance tests pass against the task database in the existing PostgreSQL 16 container.
- [ ] Rust format, Clippy, focused tests, cargo-deny and guard scripts pass.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17; no new #[allow].
- [ ] OpenAPI generation is attempted and T-114 placeholder recorded as Not run; SQLx metadata is updated if queries change.
- [ ] Signed-off commit and complete Step 12 Report.

## Report
**Plan** — (1) extend the shipped invite repository/service with tenant-scoped preview and single-use acceptance → verify the token hash, invite state and expiry through the app pool; (2) add Core-owned membership activation and the identity login-session handoff → verify the invite, credentials, membership, audit and refresh family commit together; (3) add public preview/accept DTOs and register both routes → verify the response contract and OpenAPI route inventory; (4) write `accept_ok` and `accept_expired_410` first → verify valid, expired, accepted, replay and wrong-tenant cases; (5) run the task's Rust, database, dependency, guard, OpenAPI and repository checks → verify each exact command in Checks. Work stopped at Step 4 because the required tenant display name has no approved, secure Core read path.

**Assumptions** — The invite token is parsed in its canonical base64url form and only its SHA-256 digest is used for persistence lookup; the tenant comes from the request's resolved host context; person display name is obtained through the Core identity port. No assumption was made that a tenant slug is its display name.

**Test plan** — Given a valid, pending invite and a policy-valid password, acceptance sets the password, accepts the invite, activates the membership, creates the login/refresh session and audits the transition in one transaction → `api_invites::accept_ok`. Given an expired or already accepted invite, the route returns 410 `invite_expired` and does not change credentials or membership → `api_invites::accept_expired_410`. Boundary and abuse cases: unknown token → 404; token from another tenant → 404 with no cross-tenant data; malformed/non-canonical token; replay after acceptance → 410; weak or common password → 422 `password_weak`; expired-at-now uses the injected Clock; raw token/password/profile data never appear in logs or event payloads.

**Summary** — No application implementation was made. The required `tenant_name` preview field cannot be fetched through the existing approved Core resolver: `eq_app` has no direct `tenants` table access, and the SECURITY DEFINER resolver returns only `id`, `slug`, `status`, `timezone`, `locale` and `currency`. The task is blocked pending an architect-defined Core-safe name lookup and its task/migration scope.

**Spec read** — `spec/modules/identity.md` §§1–2 (IDN-02/07); `spec/03-module-system.md` §6; `spec/06-domain-model.md` §§4.1–4.2; `spec/18-security-by-design.md` §§4–5, 8 (SBD-40–43/49/50/81); `spec/19-error-handling.md` §2 (ERR-10); `spec/20-state-machines.md` §§3.1–3.2 (STM-20/21); `spec/23-contracts-and-canonical-forms.md` §3 (CAN-Token); `spec/25-auth-and-access-control.md` §§1, 4.5; `spec/29-erd.md` Core and Identity dictionaries, §§5 and 7; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4 (CA-21/30–33).

**Files** — Modified `tasks/M1-core/T-110d-invite-acceptance.md` (plan, contract analysis, self-resolutions and blocking question); generated `tasks/INDEX.md` (blocked status). No application, migration, spec or guard file was changed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1. A valid invite is accepted and logs in atomically | `api_invites::accept_ok` | Not run: blocked at Step 4 before implementation. |
| AC2. Expired or accepted invites return 410 | `api_invites::accept_expired_410` | Not run: blocked at Step 4 before implementation. |

**Checks run** —

- ✅ `scripts/tasks-index.sh` generated `tasks/INDEX.md` with the `blocked` status.
- ❌ `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py` — both fail because `scripts/tasks-lint.py` rejects the workflow-defined `blocked` status; guard left unchanged.
- ✅ `python3 scripts/erd.py --check` — ERD matches the DDL.
- ✅ `git diff --check`.
- ✅ `gitleaks detect --source tasks/M1-core/T-110d-invite-acceptance.md --no-git --redact` — no leaks.
- ❌ `make api` — attempted; exits 2 with “OpenAPI generation is added by T-114.” OpenAPI generation was not run because the repository target is still a placeholder.
- Not run: Rust formatting, Clippy, focused tests, `cargo deny`, database acceptance tests, `make dev`, and `make e2e` — blocked before implementation; no code or migration changed.
- Not applicable: `scripts/erd.py --migrations` and SQLx prepare — no migration or query was changed.

**Manual checks** — Read `migrations/0002_core_tenancy.sql` and `migrations/0009_core_tenant_resolver.sql`: the former revokes `tenants` table access from `eq_app`; the latter's SECURITY DEFINER resolver omits `tenants.name`. `TenantInfo` contains `slug` but no name. No database was modified or contacted for this blocked task.

**Debug log** — None; the task was blocked during Step 4 contract analysis, before implementation or runtime checks.

**Performance** — n/a; no implementation or measurement.

**Security** — Triggers: T1, T3 and T4. SBD-81 review: auth mode would be public token possession; tenant must remain from resolved context; invite hash must be tenant-scoped; raw token/password must stay in `SecretString` and out of logs; acceptance, membership, credentials, refresh family and audit must share one transaction; abuse cases are listed in Test plan. Direct `SELECT tenants.name` from Identity would violate CA-30 and the explicit privilege boundary; returning `slug` as `tenant_name` would change the response meaning. No such control was bypassed.

**Deviations** — RATIFY: the task's Implementation note and Scope → Out named T-111a, but this task depends on T-110c and T-110c shipped the invite service/repository; both references now follow the dependency graph and shipped code. RATIFY: `tasks/README.md` says tasks never end `blocked`, while the status table and WORKFLOW §4 define `blocked`; this task uses the defined blocked lifecycle and run instructions. Blocking Q1 remains unresolved.

**Dependencies affected** — T-110e and later invite/reset/profile parts, plus aggregate T-111, cannot proceed until this task is resolved; unrelated eligible tasks remain available.

**Rollback plan** — No code or schema change to roll back; the blocked status/report can be resolved by the architect's task/spec update and a fresh implementation run.

**Follow-ups** — Architect must specify the Core-owned, tenant-safe way to return `tenants.name` to the public preview and authorize the exact forward-only migration/function and Files/task scope. The existing task validator also rejects the workflow-defined `blocked` status; its failure is recorded without changing the guard.

**Step 0 — Start** — Selected T-110d as the lowest eligible task after T-110c reached `review`; started on `dev` at `dc2a1c9` with preserved audit `.pending`/`.hold` and Python cache WIP untouched. Regenerated and checked the index after claiming the task.

**Step 1 — Read task and specs** — Read the full task, its acceptance/security contracts and cited Identity, domain, error, state-machine, auth, ERD and quality sections listed above. `make api` is explicitly deferred by the current repository to T-114; it does not provide the missing tenant-name interface.

**Step 2 — Dependencies** — Read T-110c's Report, including RATIFY TD-137/TD-134/TD-136 and its M5 token-handoff/T-112 permission follow-ups, and inspected the shipped `InviteService`, `repo.rs`, and Core directory port. The shipped code is the source for the mechanical T-111a → T-110c note correction.

**Step 3 — Design reference** — n/a; backend-only task.

**Step 4 — Plan and test plan** — Recorded above before implementation. The contract review found that `tenant_name` has no secure, task-authorized source. Blocked under WORKFLOW §4 with Q1; no business value, field, resolver function or migration was invented.

**Step 5 — Implement** — n/a; blocked before implementation.

**Step 6 — Run locally** — n/a; no handler was implemented.

**Step 7 — Debug loop** — n/a; this is a contract/security boundary question, not a failing implementation.

**Step 8 — Self code review** — n/a; no application diff.

**Step 9 — Optimize** — n/a; no application diff or measurement.

**Step 10 — Security audit** — Completed the design-boundary review recorded above; implementation abuse tests are not present because the task is blocked.

**Step 11 — Revise and verify** — Ran index generation, ERD check, diff check, task-file Gitleaks and the required Make API attempt. The status validator and its `--check` wrapper fail on the valid blocked status; Rust and runtime checks were not run because implementation was blocked. No check was claimed green without running it.

**Step 12 — Report and commit** — Set `status: blocked`, generated `tasks/INDEX.md`, and prepared the required signed-off block commit on `dev`.

## Questions
- **Q1 (blocking):** Which approved Core interface should provide `tenants.name` for the public invite preview without exposing direct tenant-table access or changing the response contract?
  - Context: T-110d requires `tenant_name`; ERD identifies the Core-owned `tenants.name`; `migrations/0002_core_tenancy.sql` revokes direct `tenants` access from `eq_app`; `migrations/0009_core_tenant_resolver.sql` returns only `id`, `slug`, `status`, `timezone`, `locale` and `currency`. The task Files list has no Core tenant resolver or migration, and existing migrations are forward-only.
  - Options: (a) architect authorizes a Core-owned SECURITY DEFINER resolver/port exposing `name` and supplies its task and forward-only migration scope; (b) change the preview contract to omit `tenant_name`; (c) return `slug` under `tenant_name`.
  - Recommendation: (a), preserving the specified response and existing table privilege boundary; the architect must define the exact migration/function and task ownership before implementation.
- **Self-resolved (implementer, run 16, needs architect ratification):** The Implementation note and Scope → Out name T-111a, but T-110d depends on T-110c and T-110c shipped the invite service/repository → use T-110c as the referenced issuance task and extension point because the declared dependency and shipped code define the available interface.
- **Self-resolved (implementer, run 16, needs architect ratification):** `tasks/README.md` says tasks never end `blocked`, but its status table and `WORKFLOW.md` §§2/4 define `blocked` and this run requires blocking unresolved tasks → set `status: blocked`, because the explicit workflow lifecycle and run mechanics govern this task; leave the guard unchanged and report its mismatch.

## Review
Blocked pending architect resolution of Q1.
