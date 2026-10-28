---
id: T-109f
title: Verify tampering, anonymization, purge and concurrent sealing
milestone: M1
status: review
depends_on: [T-109e]
spec_refs: [AUD-03, AUD-04, AUD-05, ERD-05, SEC-07]
estimated_size: S
---

## Goal
Exercise chain integrity and privacy-preserving cleanup against the schema and Rust implementation from T-109b/T-109e.

## Context
Preserve the exact acceptance test names in parent T-109. All database cases use fresh PostgreSQL 16 testcontainers and an `eq_app` pool for tenant isolation.

## Scope
### In
- Tests for content tampering, nested anonymization, purge checkpoints, concurrent sealers and cross-tenant function arguments.
### Out
- Production implementation, which is owned by T-109b/T-109e.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/tests/audit.rs` | modify | Chain and tenant-abuse tests. |
| `tasks/M1-core/T-109f-audit-chain-tests.md` | modify | Report and status. |

## Interfaces
Test only the public `Sealer`/`verify` API and the exact SQL procedures from parent T-109.

## Business rules
No new cutoff or policy values; use explicit test fixtures from parent T-109.

## Security
- Triggers: tamper, tenant isolation and sensitive free text (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: altered chain, cross-tenant procedure call or leaked free text → assert mismatch, permission denial, recursive redaction and proof-backed verification.
- Abuse tests: `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint`, `concurrent_sealers`, `audit_functions_reject_cross_tenant_arguments`.

## Implementation notes
Use only fake personal values and disposable databases. Do not test RLS using the superuser pool.

## Acceptance criteria
- AC3. Tampered content reports its sequence → `tamper_detected`.
- AC4. Nested PII is redacted and the anonymized chain verifies → `anonymize_keeps_chain_valid`.
- AC5. Purged rows leave a valid checkpoint → `purge_with_checkpoint`.
- AC7. Concurrent sealers preserve one chain → `concurrent_sealers`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint`, `concurrent_sealers`, `audit_functions_reject_cross_tenant_arguments` |

## Definition of done
- [ ] All five integration tests pass on fresh disposable PostgreSQL 16 instances.
- [x] No fake secret or free-text fixture is logged.
- [x] Signed commit `test(audit): verify audit chain controls [T-109f]`.

## Report
**Plan** — (1) Recheck parent T-109 and T-109b's mapped functions, T-109e's report and shipped `Sealer`/repository code, plus the existing `eq_app` test harness → preserve the exact parent signatures, tenant guards, hash, redaction and checkpoint behavior. (2) Extend only `crates/eq-audit/tests/audit.rs` with the five named cases → seed via the owner pool, exercise sealer and functions through transactions on `eq_app`, and inspect stored results without logging fixture content. (3) Run format, dependency, task, guard, ERD and secret checks; run the database tests only if the already available fixed cache and allowed PostgreSQL resource can be reused → record unavailable checks as Not run under the disk rule.
**Assumptions** — The parent contract and mapped T-109b migration define the test-visible functions and semantics; T-109e supplies the public chain API. `TestDb` supplies an owner pool for fixtures/inspection and a separate non-superuser `eq_app` pool for all audited calls. No migration or production code change is required.
**Test plan** — AC3 → `tamper_detected`: seal a row, tamper with its content using the owner setup pool, and assert the exact sequence is the first mismatch. AC4 → `anonymize_keeps_chain_valid`: seal nested fake PII, call anonymize as `eq_app`, verify recursive redaction, null IP, a proof, and a valid chain. AC5 → `purge_with_checkpoint`: seal before/after-cutoff rows, purge as `eq_app`, and verify the remaining row against its checkpoint. AC7 → `concurrent_sealers`: run two `Sealer`s for one tenant concurrently and verify one valid chain. Abuse → `audit_functions_reject_cross_tenant_arguments`: call each controlled procedure with another tenant id as `eq_app` and require SQLSTATE 42501. No test prints or logs fixture data.
**Summary** — Added the five named integration tests for tampering, recursive anonymization, purge checkpoints, concurrent sealers, and cross-tenant procedure arguments. Every sealer/procedure operation uses the `eq_app` pool and transaction; the owner pool is limited to fixture setup, deliberate tampering, and result inspection. Tests were written but not executed under the active disk rule.
**Spec read** — `spec/06-domain-model.md` §§4.9/4.11; `spec/18-security-by-design.md` §§2/5.4/8; `spec/29-erd.md` §§3.9/4/5/7; parent T-109; T-109b migration/report; T-109e report and shipped API/repository code.
**Files** — Modified `crates/eq-audit/tests/audit.rs`, this task Report/status, and generated `tasks/INDEX.md`.
**Acceptance criteria → tests** —
| AC / abuse case | Test | Result |
|---|---|---|
| AC3 | `tamper_detected` | Not run: requires Rust compilation and a disposable PostgreSQL database; the fixed target cache is absent and the disk rule forbids creating a target or test resources. |
| AC4 | `anonymize_keeps_chain_valid` | Not run for the same reason. |
| AC5 | `purge_with_checkpoint` | Not run for the same reason. |
| AC7 | `concurrent_sealers` | Not run for the same reason. |
| Cross-tenant procedure abuse | `audit_functions_reject_cross_tenant_arguments` | Not run for the same reason. |

Database used: none; task integration pass count: 0/5 executed (not a test failure). The five exact names are present in `crates/eq-audit/tests/audit.rs`. `make e2e`: n/a because this test-only backend task adds no UI or milestone journey.
**Checks run** — `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check` ✅ after `cargo fmt --all`; `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version/unmatched-CC0 warnings remain); `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` ✅; `python3 -I scripts/erd.py --check` ✅; `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8); `scripts/tasks-index.sh --check` ✅; `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --locked --format-version 1 --no-deps` ✅; `git diff --check` ✅; `gitleaks dir --redact --no-banner crates/eq-audit/tests/audit.rs` and the same command for this task file ✅ (no leaks). Not run: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`, `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-audit --test audit -- --test-threads=1`, and `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace`; the fixed target cache is absent, and these would create a forbidden target and (without an external test DB) a new PostgreSQL test resource. `cargo sqlx prepare --workspace` and `scripts/erd.py --migrations` are n/a (no SQL macros or migrations changed); `make e2e` is n/a (no UI journey).
**Manual checks** — Confirmed all five test names and parameterized tenant/function arguments. `db::begin` and `db.app_pool` are used for sealer and procedure calls; the cross-tenant test asserts SQLSTATE 42501 for `audit.seal`, `audit.anonymize`, and `audit.purge_expired`. The owner pool only seeds tenant rows, mutates content for the tamper case, and inspects database state; it is not used to assert RLS behavior. Fake nested `display_name`, `phone`, and `note` values are checked for absence after anonymization; IP is checked null and proof non-null. Test code does not print or log fixtures.
**Debug log** — The initial `cargo fmt --all --check` reported only rustfmt line wrapping in the new test code. Ran `cargo fmt --all`; the final format check passed. No database test failure loop was entered because the target/database prerequisites could not be run under the disk rule.
**Performance** — n/a; no production behavior or query plan changed.
**Security** — SBD-30 T4 trigger. SBD-81: (1) endpoint authentication/permission checks n/a (no endpoint); (2) all sealer/function calls use `eq_app` in a tenant transaction and the abuse test rejects cross-tenant ids; (3) S1/S2 fixture values are fake and never logged; the owner role is restricted to test setup, tampering and inspection; (4) no HTTP input/DTO; (5) no export; (6) no secrets; (7) no external calls; (8) tampering, anonymization proof and checkpoint behavior have named tests; (9) no new event; (10) changed-path Gitleaks scans pass. Runtime behavior is Not run as stated above.
**Deviations** — No new self-resolution or business/security/schema decision. Parent T-109 and dependency Reports' TD-071–086 were read and remain recorded in their owning task files. The required database tests are Not run under the disk rule; no result is claimed as passed.
**Dependencies affected** — T-109g and T-109h.
**Rollback plan** — Revert this test-only part; no schema changes.
**Follow-ups** — Run the exact five integration tests on the permitted PostgreSQL 16 test resource when the approved fixed Cargo target is available; current environment has no `$HOME/.cache/ritma/target`, and creating it is prohibited.

**Disk hygiene** — Before implementation, `$HOME/.cache/ritma/target` was absent (`du -sh "$HOME/.cache/ritma/target" 2>/dev/null`: no output, exit 1); the repository `target/` was absent. `docker system df` returned no output during the initial 10-second query wait; the bounded `timeout 5s docker system df` returned no output and exit 124. After the checks, both target paths were still absent and the bounded Docker usage query again returned no output/exit 124. The pre/post container inventory was unchanged; no container, image, volume, database, or target was created or removed. Existing `eqs-postgres` was not used by this task.

**Workflow steps 0–12** — Step 0: selected T-109f after T-109e reached `review`, set only this task `in_progress`, and regenerated the index; the unrelated pre-existing WIP remained untouched. Step 1: read the full task, parent T-109 contract, mapped audit DDL, security and ERD excerpts. Step 2: read T-109e's Report and shipped sealer/repository code, T-109b's migration/Report, T-109d's Report/record code, and the `TestDb` app-pool contract. Step 3: n/a, backend-only. Step 4: wrote the Plan, assumptions and exact AC/abuse test plan before implementation. Step 5: added all five named tests in the task's file. Step 6: database runtime checks are Not run because the approved target/resource is unavailable; no route or browser behavior is in scope. Step 7: corrected rustfmt output and reran format successfully; no runtime test failure occurred. Step 8: reviewed tenant boundaries, function SQLSTATE assertions, fixtures and owner/app pool separation against CQ, CA, ERD and ERR rules. Step 9: n/a, no measured optimization. Step 10: completed SBD-81 and scanned changed paths. Step 11: reran all available format, dependency, guard, ERD, task, metadata, diff and secret checks; build/database checks remain Not run. Step 12: completed this Report, set `review`, regenerated the index, and prepared one signed commit/push.

## Questions
None.

## Review
n/a — awaiting human review.
