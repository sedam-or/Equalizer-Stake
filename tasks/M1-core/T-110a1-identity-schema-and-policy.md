---
id: T-110a1
title: Add identity schema and shared RoleMap
milestone: M1
status: review
depends_on: [T-102a, T-103, T-106, T-109d]
spec_refs: [ERD-03, ERD-04, ERD-06, SEC-05, SBD-40, SBD-42, CQ-101]
estimated_size: S
---

## Goal
Create the one identity migration assigned by the ERD and the shared role-keyed policy container.

## Context
`spec/29-erd.md` §7 maps `0007_identity.sql` to T-110. It creates only `credentials`, `refresh_sessions`, `invites`, and `password_resets` using this DDL from spec 06 §4.2; enable tenant RLS on each with the existing `eq_enable_rls` helper.
```sql
CREATE TABLE credentials (
  person_id uuid PRIMARY KEY REFERENCES persons(id),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  password_hash text,
  password_changed_at timestamptz,
  totp_secret_enc bytea,
  totp_enabled boolean NOT NULL DEFAULT false,
  failed_count int NOT NULL DEFAULT 0,
  failed_window_start timestamptz,
  locked_until timestamptz
);
CREATE TABLE refresh_sessions (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  family_id uuid NOT NULL,
  token_hash bytea NOT NULL UNIQUE,
  active_role text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  last_used_at timestamptz NOT NULL DEFAULT now(),
  idle_expires_at timestamptz NOT NULL,
  absolute_expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  user_agent text,
  ip inet
);
CREATE TABLE invites (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  role text NOT NULL,
  token_hash bytea NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  accepted_at timestamptz,
  created_by uuid REFERENCES persons(id),
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE password_resets (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  token_hash bytea NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  used_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
```
The identity ERD dictionary assigns S3 to password/token hashes and TOTP secret, S2 to refresh user-agent/IP, and S1 to other columns. Do not add columns, indexes, grants, or other schema objects.

## Scope
### In
- Add the mapped migration exactly and enable RLS on all four tables.
- Add the generic `RoleMap<T>` with owner/admin/instructor/guardian/student fields to `eq_core::policy::types`.
- Record the complete ordered identity implementation split in the parent task and task index; each successor owns only its listed behavior.
### Out
- Identity policy values, module integration, credentials, endpoints, and any other migration.

## Files
| Path | Action | Purpose |
|---|---|---|
| `migrations/0007_identity.sql` | create | Mapped identity DDL and tenant RLS. |
| `crates/eq-core/src/policy/types.rs` | modify | Export the shared role-keyed value shape. |
| `tasks/M1-core/T-110a1-identity-schema-and-policy.md`, `tasks/M1-core/T-110a1b-validated-identity-policy-model.md`, `tasks/M1-core/T-110a1c-identity-policy-module.md`, `tasks/M1-core/T-110a2-password-and-jwt-primitives.md`, `tasks/M1-core/T-110a3-login-service-and-lockout.md`, `tasks/M1-core/T-110a4-login-http-endpoint.md`, `tasks/M1-core/T-110a5-actor-permissions-and-role-switch.md`, `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Record the CQ-101 task split, reports and ordering. |

## Interfaces
Use the table names, columns, types, nullability and FK targets exactly as copied from spec 06 §4.2 and listed in the T-110 row of ERD §7. `RoleMap<T>` has exactly five public fields: `owner`, `admin`, `instructor`, `guardian`, `student`.

## Business rules
No new values or database behavior. Tenant isolation uses the existing `eq_enable_rls` helper.

## Security
- Triggers: T3, T4 (SBD-30).
- Data classes follow ERD-04: S3 credentials/tokens/TOTP, S2 refresh metadata, S1 remaining columns.
- Each table has `tenant_id` and receives forced row-level security through `eq_enable_rls`; do not add app access paths or expose secrets.
- Abuse case: `scripts/erd.py --migrations` rejects extra or mismatched columns.

## Acceptance criteria
- AC1. Migration matches the ERD map and spec 06 DDL, and enables tenant RLS on all four tables → `scripts/erd.py --migrations` plus review of all four helper calls.
- AC2. `RoleMap<T>` exposes the five canonical school roles and rejects unknown serialized keys → source and schema review.
- AC3. The identity parent and all ordered successor task files agree on dependency and test ownership, and the generated index lists each task → `scripts/tasks-index.sh --check` and task lint.

## Tests to write
| File | Test names |
|---|---|
| `scripts/erd.py` | `--migrations` (migration contract guard) |
| `crates/eq-core/src/policy/types.rs` | `role_map_rejects_unknown_role_keys` |
| `scripts/tasks-index.sh`, `scripts/tasks-lint.py` | `--check`, task lint |

## Definition of done
- [x] `python3 -I scripts/erd.py --migrations` and `python3 -I scripts/erd.py --check` pass.
- [x] Migration contains exactly the four mapped tables and RLS helper calls; no extra schema object.
- [x] `cargo fmt --all --check`, `cargo deny check`, guards, task lint/index and `git diff --check` pass.
- [x] Report includes Workflow steps 0–12 and before/after resource data; prohibited database/build checks are `Not run` with the reason.
- [x] Signed commit `feat(identity): add schema and RoleMap [T-110a1]` pushed to `dev`.

Spec 16 §12 checks for this task:
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` (Not run: fixed Cargo target is absent; creating a target is prohibited by the run's disk rule).
- [x] Web checks are n/a because no web path is touched.
- [x] Generated API/SQLx files are n/a because no API route or query changes.
- [x] No new lint exceptions or TODO markers; no new error case.
- [x] Security fields are classified and the SBD-81 controls are reviewed; the named unit test is present, with execution recorded as Not run.

## Report
**Plan** — (1) Compare the source DDL and ERD mapping → verify the exact four table names and columns. (2) Create `0007_identity.sql` and call the existing RLS helper once per table → verify DDL, table scope and helper calls. (3) Add `RoleMap<T>` → verify the five canonical fields and unknown-key rejection. (4) Record all ordered identity successor contracts and preserve one owner per original AC/test → verify the parent split, task index and task lint. (5) Run the available static, schema, dependency, guard, task and secret checks; record build/database checks as Not run when the disk rule disallows their resources.
**Assumptions** — The `spec/29-erd.md` T-110 row and spec 06 §4.2 are authoritative for migration filename, table ownership and DDL. `RoleMap<T>` has no value invariant by itself. No runtime or business behavior is added.
**Test plan** — AC1: Given the four spec 06 identity tables, when `python3 -I scripts/erd.py --migrations` runs, then their columns, types, nullability and FKs match; manually verify one `eq_enable_rls` call per table. AC2: Given a JSON `RoleMap` with the five canonical keys plus `platform`, when deserialized, then the unknown key is rejected → `role_map_rejects_unknown_role_keys` (written, execution Not run). AC3: Given the split identity task docs, when index generation and task lint run, then each listed successor exists and the dependency/test ownership graph is valid → `scripts/tasks-index.sh --check` and `python3 -I -B scripts/tasks-lint.py`. Edge case: the closed map rejects an unknown role key. Error case: serde rejects the extra field at the boundary; this task adds no API problem code. Abuse cases: the unknown `platform` key cannot extend the role set, the migration checker rejects schema drift, and task lint rejects invalid task references. No database concurrency or timezone behavior is in scope.
**Summary** — The mapped migration creates only `credentials`, `refresh_sessions`, `invites`, and `password_resets`, applying the existing forced tenant-RLS helper to each. `RoleMap<T>` exposes the five canonical role fields and denies unknown serialized fields. The parent and ordered successor task definitions are committed with the generated index so every listed task is present; the named Rust regression test is present but execution is Not run because creating a target cache is prohibited.
**Spec read** — `spec/06-domain-model.md` §§1, 4.2; `spec/29-erd.md` §§1, 3.2, 4–7; `spec/16-code-quality.md` §§1, 4, 9, 12; `spec/17-clean-architecture.md` §§1, 4; `spec/18-security-by-design.md` §§2–5, 7–8; `spec/19-error-handling.md` §10.
**Files** —
- Created `migrations/0007_identity.sql` with the mapped identity DDL and four RLS calls.
- Modified `crates/eq-core/src/policy/types.rs` to add and document `RoleMap<T>` and its unknown-key test.
- Created this task report and the successor task files `T-110a1b` and `T-110a1c`.
- Created successor task files `T-110a3`, `T-110a4` and `T-110a5` so the generated index references committed contracts.
- Modified the T-110a parent split tracker and T-110a2 dependency to preserve the ordered task graph.
- Modified `tasks/DECISIONS.md` with TD-099 and generated `tasks/INDEX.md` from task frontmatter.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `python3 -I scripts/erd.py --migrations`; manual review of four RLS helper calls | Pass: 25 migrated tables checked; the migration contains exactly the four mapped tables and calls `eq_enable_rls` once per table. Runtime RLS test Not run. |
| AC2 | `role_map_rejects_unknown_role_keys` | Test added and source review passed (`deny_unknown_fields`); Not run because Rust test execution would create the absent target directory. |
| AC3 | `scripts/tasks-index.sh --check`; `python3 -I -B scripts/tasks-lint.py` | Pass: all 156 task entries are generated from existing files; task lint passes. |
**Checks run** —
- ✅ `python3 -I scripts/erd.py --migrations` — 25 migrated tables checked; 40 remain unmigrated.
- ✅ `python3 -I scripts/erd.py --check` — ERD matches spec DDL.
- ✅ `cargo fmt --all --check`.
- ✅ `cargo deny check` — advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-CC0 warnings remain, with no policy exception added.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `scripts/tasks-index.sh --check && python3 -I -B scripts/tasks-lint.py && python3 -I -B -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` — task lint passes; 8/8 fixtures pass.
- ✅ `cargo metadata --offline --locked --format-version 1 >/dev/null`.
- ✅ `git diff --check`.
- ✅ Gitleaks loop: `gitleaks dir --redact --no-banner --log-level error --report-format json --report-path /dev/null "$f"` for each of the 12 task-owned migration, source, task, decision and index paths — passed, no findings.
- Not run: `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` — the fixed `$HOME/.cache/ritma/target` does not exist and creating any target directory is prohibited.
- Not run: `SQLX_OFFLINE=true cargo test --workspace` and `role_map_rejects_unknown_role_keys` — same target-directory restriction.
- Not run: `make dev` — this task adds no server route or runtime flow; no project dev database was started, and creating a container/database is prohibited.
- n/a: web lint/typecheck/tests/i18n, `make api`, and `cargo sqlx prepare --workspace` — no web, API route, or SQL query changed.
- n/a: `make e2e` — this schema/type task has no user journey or runtime integration flow; database used: none; E2E pass count: 0/0.
- Not run: hosted GitHub Actions; unavailable in this local environment.
**Resource report** — Before (2026-10-28T17:40:31Z): `du -sh "$HOME/.cache/ritma/target"` printed `du: cannot access '/home/kepolu/.cache/ritma/target': No such file or directory` (exit 1); `docker system df` printed no output and timed out (exit 124); Docker inventory was 8 containers, 11 images and 434 volumes; `/home` had 83G available. After checks (2026-10-28T17:44:35Z), the same `du` output/exit, Docker output/exit and counts were unchanged; `/home` still had 83G available. No Docker container, image, volume, database or Cargo target was created or removed; database used: none.
**Manual checks** — Compared the migration against spec 06 §4.2 and the identity ERD dictionary plus the T-110 migration row. Verified exactly four tables, no extra schema objects, and one call per table. Read `eq_enable_rls` in `0001_core_bootstrap.sql`; it enables and forces RLS and installs the tenant policy. Reviewed the `RoleMap<T>` field set and `deny_unknown_fields`. No `make dev` or browser journey was applicable.
**Debug log** — None; no check failure required a debug attempt.
**Performance** — n/a; this task adds no query or runtime path.
**Security** — Triggers T3/T4 (SBD-30). The ERD classes remain S3 for password/token hashes and the TOTP secret, S2 for refresh-session user-agent/IP, and S1 for the remaining columns. The S2 fields remain plaintext as specified by ERD-04, bounded by the refresh-session lifetime, and must never be logged or exported without `read_pii`. Each tenant table calls the helper that enables and forces RLS; the SQL is reviewed, but database enforcement was not executed. `RoleMap<T>` rejects unknown role keys; the abuse test is present but not run. No secrets or PII were added; targeted Gitleaks passed. Auth-chain, permission, audit and API-output controls are n/a because this task adds no endpoint or behavior.
**Deviations** — RATIFY TD-098: split schema/RoleMap from policy values and module integration to keep the implementation concerns ordered. RATIFY TD-099: removed the task's T1 classification because no endpoint is in scope; T3/T4 still apply to S3 fields and RLS. RATIFY TD-100: included the remaining successor task contracts because the parent split and generated index already referenced them; committing the files together keeps task-index validation truthful. The complete task-definition diff is 805 lines, above CQ-101's approximate 400-line guide; this is called out for maintainer review because most lines are the self-contained ordered task contracts, while the migration and RoleMap implementation stay narrowly scoped.
**Dependencies affected** — T-110a1b depends on this task; T-110a1c and T-110a2 remain ordered after the policy/module tasks. No shipped business interface changed.
**Rollback plan** — Before applying the migration, revert the signed commit. After applying it, preserve the forward-only `0007_identity.sql`; the new tables are additive and have no consumer in this task. Any later removal requires a separately reviewed forward migration.
**Follow-ups** — Review the 805-line task-definition change against CQ-101's approximate 400-line guide. When an approved shared target becomes available, run workspace clippy/tests including `role_map_rejects_unknown_role_keys`. A runtime RLS check is also deferred because no permitted disposable database was available.
**Workflow steps 0–12** — 0: resumed the existing `in_progress` task on `dev`; inspected the pre-existing WIP without discarding it; task index check passed. 1: read this task and its cited spec sections. 2: read T-102a, T-103, T-106 and T-109d Reports and shipped code; inherited follow-ups do not change this schema contract. 3: n/a, backend-only. 4: plan and AC/test mapping are above. 5: implemented only the migration, shared role map and assigned task-split metadata. 6: `make dev` not run; no runtime endpoint is in scope and required resources are unavailable under the disk rule. 7: no failures. 8: reviewed DDL, field set, layering and security against the cited standards. 9: n/a, no performance-sensitive runtime path. 10: completed the applicable SBD-81 review above. 11: ran the checks listed above and recorded build/database checks as Not run. 12: set this task to `review`, regenerated and checked the index, then created one signed commit on `dev` and pushed it.

## Questions
- **Self-resolved (implementer, run 7, needs architect ratification; TD-098):** T-110a1 combined mapped schema, RoleMap, policy values and module integration beyond CQ-101's approximate 400-line PR guidance → keep this task to migration/RoleMap and use ordered T-110a1b/T-110a1c tasks for policy model/module integration; no policy or schema behavior changes.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-099):** the original Security trigger list included T1, but this task adds no endpoint → retain only T3/T4 because it adds S3 columns and tenant RLS (SBD-30).
- **Self-resolved (implementer, run 7, needs architect ratification; TD-100):** the parent split and generated index already include T-110a3–T-110a5, but their task files were absent from this task's Files list → include those self-contained successor contracts so the committed task index cannot reference uncommitted task files (CQ-15).

## Review
n/a — awaiting maintainer review.
