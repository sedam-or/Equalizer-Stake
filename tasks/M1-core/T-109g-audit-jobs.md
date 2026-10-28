---
id: T-109g
title: Add audit retention and recent verification jobs
milestone: M1
status: review
depends_on: [T-109f]
spec_refs: [AUD-04, AUD-07, ARC-09, OPS-08, SEC-07]
estimated_size: S
---

## Goal
Add tenant-scoped retention/anonymization and nightly recent-chain verification jobs, using the shared `PolicyStore` and raising the specified attention item on mismatch.

## Context
Parent T-109 defines job kinds, local cron schedules, policy calculations and attention item fields. T-108's `JobCtx` may be extended additively to carry the shared policy store.

## Scope
### In
- `audit.retention` and `audit.verify_recent` handlers, typed retry/permanent failures, and job behavior tests.
- Pass the shared optional `PolicyStore` through `JobCtx` without changing existing job callers.
### Out
- Module registration and cron schedule wiring (T-109h), server runtime loop (T-117a).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/src/jobs.rs`, `crates/eq-audit/src/lib.rs`, `crates/eq-core/src/jobs.rs` | modify/create | Job handlers and shared context. |
| `crates/eq-audit/src/repo.rs` | modify | Keep audit-sequence and maintenance-function queries in the audit repository (CA-11/CQ-24/CQ-42). |
| `crates/eq-core/src/lib.rs`, `crates/eq-core/src/attention.rs`, `crates/eq-core/src/attention/repo.rs` | modify/create | Implement the core-owned `attention::raise` API defined by STM-14. |
| `crates/eq-audit/tests/audit.rs` | modify | Job behavior tests. |
| `.sqlx/query-f3fa368eeb034f34ce02c3a483c076846a3772472f0f4c64da7c740b6ae336d2.json`, `.sqlx/query-cba1904818c014471c89c6ec422bcdc531b8f5c116714f2fac1df3e428c36bb3.json`, `.sqlx/query-84eb188f940e98368bfe6d0547b0a1f87799fdda112248bb90d05388797edc62.json`, `.sqlx/query-a9ac2cccfde6e4fa2e6d5b2eea7a9444716bc200eff1911664a10206643fa35d.json` | add | Offline metadata for the exact audit-bound, maintenance-function and core attention upsert queries. |
| `tasks/M1-core/T-109g-audit-jobs.md`, `tasks/DECISIONS.md` | modify | Report, status and ratification record. |

## Interfaces
Preserve kinds `audit.retention` and `audit.verify_recent`, and use `AuditPolicy`/`Sealer` APIs from prior parts.

## Business rules
Retention intervals come only from `AuditPolicy`; verify the last 24 hours as specified by the parent task.

## Security
- Triggers: tenant PII retention and chain integrity (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: wrong tenant, policy bypass or silent mismatch → use tenant `JobCtx`, shared `PolicyStore`, `eq_app`, and a high-severity audit mismatch attention item.
- Abuse tests: `audit_retention_job_uses_policy_store`, `audit_verify_job_raises_attention_on_mismatch`.

## Implementation notes
Keep `JobCtx` backward compatible for existing callers; never log payloads, PII or secret values.

## Acceptance criteria
- AC1. Retention uses the shared policy and calls anonymize then purge → `audit_retention_job_uses_policy_store`.
- AC2. Recent verification creates the specified attention item for a mismatch → `audit_verify_job_raises_attention_on_mismatch`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `audit_retention_job_uses_policy_store`, `audit_verify_job_raises_attention_on_mismatch` |

## Definition of done
- [ ] Both job tests pass on fresh disposable PostgreSQL 16 testcontainers.
- [ ] Rust checks and guard scripts pass without weakening job or tenant controls.
- [x] Signed commit `feat(audit): add audit maintenance jobs [T-109g]`.

## Report
**Plan** — (1) Read the parent T-109 contract, T-109f Report and shipped test/sealer code, plus the T-109c policy accessors and T-108 job context → preserve policy-only retention cutoffs, injected time, tenant transactions, and `eq_app` boundaries. (2) Add additive `JobCtx` policy-store support and two handler types; keep audit SQL in the audit repository; retention reads `AuditPolicy` in its tenant transaction and runs anonymize before purge; verification selects the tenant's entries in the specified 24-hour window, calls `Sealer::verify`, and raises a high-severity item through the core-owned `attention::raise` API specified by STM-14. (3) Add the exact two integration tests through the public handler types, using a fresh TestDb PostgreSQL 16 database, fake payload values, owner access only for fixture setup/inspection, and the app pool for handlers. (4) Run available formatting, dependency, guard, task, metadata, ERD and secret checks; report Rust compilation and PostgreSQL checks as Not run if the approved fixed Cargo target/database resources remain unavailable.
**Assumptions** — Parent T-109 supplies the retention values (default 7 years, anonymization 2 years), the 24-hour verification window, and the required high-severity attention-item behavior; T-117a supplies runtime tenant scheduling. T-109h owns module registration, so these tests instantiate the handler types directly. Core owns `attention_items` (ERD-04); the task's attention effect uses the `attention::raise` API named by STM-14, so `eq-audit` does not write the core table directly. No migration, role, permission, policy field or event is added.
**Test plan** — AC1 → `audit_retention_job_uses_policy_store`: seed one entry older than the policy anonymization threshold but within retention and one older than retention; set both timestamps before sealing; run the handler using the shared PolicyStore and `eq_app`; verify the former is anonymized with proof and the latter is purged with a checkpoint. AC2 → `audit_verify_job_raises_attention_on_mismatch`: seal a row, alter its content using the owner fixture pool, invoke the recent-verification handler through `eq_app` with an injected time covering the row, and verify the high-severity tenant attention item identifies the mismatched sequence. Both tests use a new TestDb instance (PostgreSQL 16) and do not log fixture values. If the fixed build cache remains unavailable, mark both Not run with database none and 0/2 executed; do not claim a pass.
**Summary** — Added tenant-scoped `audit.retention` and `audit.verify_recent` handlers. Retention loads the effective `AuditPolicy` through `JobCtx.policy_store`, then calls anonymize before purge in the same tenant transaction. Recent verification checks the injected 24-hour window and raises a high-severity, sequence-only attention item through core's `attention::raise`. Added the optional PolicyStore builder/context propagation and both required integration tests. The integration tests and Rust build checks are Not run under the active fixed-target/disk restriction; no test pass is claimed.
**Spec read** — `spec/06-domain-model.md` §4.11; `spec/18-security-by-design.md` §§2/5.4/8; `spec/19-error-handling.md` §5; `spec/20-state-machines.md` §3.14; `spec/29-erd.md` core `attention_items` row and §§4/5/7; parent T-109; T-109f Report/code; T-109c Report/policy; T-108 job context.
**Files** — Modified `crates/eq-core/src/jobs.rs`, `crates/eq-core/src/lib.rs`, `crates/eq-audit/src/lib.rs`, and `crates/eq-audit/src/repo.rs`; added `crates/eq-core/src/attention.rs`, `crates/eq-core/src/attention/repo.rs`, and `crates/eq-audit/src/jobs.rs`; added the two named tests to `crates/eq-audit/tests/audit.rs`; added four exact SQLx metadata files (`.sqlx/query-f3fa…d2.json`, `.sqlx/query-cba…bb3.json`, `.sqlx/query-84eb…c62.json`, `.sqlx/query-a9ac…35d.json`); updated this task, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md`. Unrelated pre-existing WIP files remain untouched.
**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `audit_retention_job_uses_policy_store` | Not run: fixed Cargo target cache is absent and the active disk rule prohibits creating a target or database/container. |
| AC2 | `audit_verify_job_raises_attention_on_mismatch` | Not run for the same reason. |

Database used: none. If run, each test creates a fresh disposable TestDb from `postgres:16-alpine`. Exact commands not run: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-audit --test audit audit_retention_job_uses_policy_store -- --exact` and `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-audit --test audit audit_verify_job_raises_attention_on_mismatch -- --exact`. Pass count: 0/2 executed; no failed test is implied. `make e2e`: n/a because this backend job task adds no web route or UI journey.
**Checks run** — `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check` ✅; same environment with `cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-license warnings remain); `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh` ✅; `python3 -I scripts/erd.py --check` ✅; `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8); `scripts/tasks-index.sh --check` ✅; `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --locked --format-version 1 --no-deps` ✅; `git diff --check` ✅; SHA-256 and JSON checks for all four new SQLx metadata files ✅; `gitleaks dir --redact --no-banner <changed task-owned path>` run on each of 15 task-owned paths ✅. Not run: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the two PostgreSQL integration tests, and `cargo sqlx prepare --workspace`; the fixed target path `$HOME/.cache/ritma/target` and repo `target/` are absent, and the active disk rule prohibits creating build targets or fresh database/container resources. `timeout 5s docker system df` exited 124 without output; no database, container, image, volume, or Cargo target was created or removed. `scripts/erd.py --migrations` n/a (no migration changed); web lint/typecheck/test/i18n and `make e2e` n/a (no web/UI scope).
**Disk/resource snapshot** — Before the available-check pass, `du -sh "$HOME/.cache/ritma/target" 2>/dev/null` produced no output (exit 1; path absent) and `timeout 15s docker system df` produced no output (exit 124). After the pass, `du -sh "$HOME/.cache/ritma/target"` reported `cannot access .../target: No such file or directory` (exit 1), and `timeout 15s docker system df` again timed out with no output (exit 124). The before/after container and image inventories were identical: 8 existing containers and 11 existing images; T-109g created or removed no container, image, volume, database or Cargo target.
**Manual checks** — Confirmed both handlers use the trusted tenant from `JobCtx`, a tenant-pinned `eq_app` transaction, and injected `ctx.now`; retention reads the effective policy and calls `anonymize` before `purge_expired` within one transaction. All SQL is in the table owner's repository: audit queries/functions in audit `repo.rs`, core `attention_items` through core `attention::raise`/`attention/repo.rs`. The core API derives tenant id from `Tx`, follows STM-14 dedupe/reopen behavior, and stores only the mismatched sequence. Test values for minor-related free text are obviously fake; neither payloads nor PII are logged. T-109h owns handler registration; T-117a must supply the optional PolicyStore through `JobRunner::with_policy_store`.
**SQLx metadata** — The four new descriptions were manually assembled from checked query strings and the mapped function/table signatures; SHA-256 and JSON structure were checked. Compile-time validation is Not run because `cargo sqlx prepare` would require the unavailable build/database resources; rerun `cargo sqlx prepare --check` when the approved target is available.
**Debug log** — Initial `cargo fmt --all --check` runs found only rustfmt line-wrapping/import-order differences; `cargo fmt --all` corrected them and the final check passed. No compile or database test was attempted because the fixed target and permitted test resources are unavailable. The bounded Docker usage query timed out with exit 124.
**Performance** — n/a; no performance target or workload measurement is in Scope.
**Security** — SBD-30 T4 trigger. SBD-81: (1) no endpoint/auth change; (2) trusted `JobCtx.tenant`, tenant-bound `begin_system`, and `eq_app`; (3) no new data fields and no PII in logs or attention payloads; (4) the policy document is validated by `PolicyStore`; (5) no export; (6) no new secrets; (7) no external calls; (8) named tests cover policy-backed retention and mismatch handling; execution is Not run as above; (9) no new event; (10) Gitleaks scans of changed paths are clean. Core owns the attention table and writes it inside the caller transaction through `attention::raise`.
**Deviations** — RATIFY TD-073 from the parent task; RATIFY TD-087 for task Files omissions and repository/offline-query boundaries; RATIFY TD-088 for the stable, idempotent mismatch attention-item identifier and payload; RATIFY TD-089 for the core-owned `attention::raise` API required by STM-14/ERD-04. No business value, table, column, role, permission or event is added.
**Dependencies affected** — T-109h consumes the exported handlers and T-117a supplies the shared `PolicyStore` through `JobRunner::with_policy_store`; M1 integration tasks consume the completed audit jobs.
**Rollback plan** — Revert this part; existing JobCtx defaults remain available.
**Follow-ups** — Run both integration tests against fresh PostgreSQL 16 testcontainers and run the full Rust/clippy plus `cargo sqlx prepare --check` when the approved fixed Cargo target is available; the prepare check must validate the manually assembled query metadata. T-109h registers the handlers; T-117a must call the `with_policy_store` builder. Architect ratification is needed for TD-087–089.

**Workflow steps 0–12** — Step 0: selected T-109g after T-109f reached `review`; confirmed `dev`, checked the task index, retained unrelated existing WIP, and kept the task-owned `JobCtx` WIP. Step 1: read the entire task, parent T-109 and cited ERD/security/error/state-machine excerpts. Step 2: read T-109f's Report/deviations/follow-ups and shipped tests/sealer code, plus T-109c policy accessors/Report and T-108 job context. Step 3: n/a, backend-only. Step 4: wrote the implementation and exact AC test plan before coding; classified the omitted repositories/metadata, attention identifier, and core table owner as TD-087–089. Step 5: added the two handlers, optional PolicyStore context propagation, core attention API, repository queries, metadata, and both named tests. Step 6: runtime requests/DB tests are Not run under the fixed-target/no-resource rule; no route or browser path is in scope. Step 7: corrected rustfmt findings and reran format successfully; no compile/test failure loop was entered. Step 8: reviewed against CQ-22/23/24/26/42, CA-11/30, ERR-51 and STM-14; verified parameterized queries, owner-only SQL, transaction/tenant boundaries and no PII logging. Step 9: n/a, no measured optimization. Step 10: completed SBD-81 review and scanned every changed task-owned path with Gitleaks. Step 11: reran the available format, dependency, guard, ERD, task, metadata, SQLx metadata hash, diff and secret checks; build/database checks are explicitly Not run. Step 12: completed this Report, set `review`, regenerated/checked the index, and prepared one signed task commit and push.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-087):** T-109g's Files table omits the repository layers and SQLx offline metadata needed by its compile-time queries → add audit query helpers to audit `repo.rs`, route core-table writes through core's `attention::raise`, and list all four exact metadata files because CA-11/CA-30/CQ-24/CQ-42 require owner-only SQL and SQLX_OFFLINE requires checked-in metadata.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-089):** T-109g's required attention item targets core-owned `attention_items`, but `eq-audit` has no core attention port → implement the `attention::raise` API named by STM-14 in `eq-core`, with its SQL in the core-owned repository and the caller's tenant transaction, because ERD-04 assigns the table to core and CA-30 forbids another module from writing it.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-088):** T-109g requires a high-severity audit mismatch attention item but does not give its stable `kind`, diagnostic payload, or dedupe shape → use module `audit`, kind `audit_chain_mismatch`, payload containing only the mismatched sequence, and `audit_chain_mismatch:<sequence>` as the tenant-scoped dedupe key because the mismatch is the sole required diagnostic and the sequence key makes retries idempotent without PII.

## Review
n/a — awaiting human review.
