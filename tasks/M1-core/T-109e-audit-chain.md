---
id: T-109e
title: Add audit sealing and chain verification
milestone: M1
status: review
depends_on: [T-109d]
spec_refs: [AUD-03, AUD-04, AUD-06, ARC-09, SEC-07]
estimated_size: M
---

## Goal
Compute and persist the canonical per-tenant SHA-256 chain and verify links, content, checkpoints and anonymized rows.

## Context
Use the JCS document and sequencing rules in parent T-109: sequence order, previous entry/checkpoint/zero anchor, UTC RFC 3339 microsecond timestamps, and the tenant advisory lock.

## Scope
### In
- `Sealer`, `VerifyReport`, `seal_tenant` and `verify`; offline metadata for their SQL queries.
### Out
- Database DDL/functions (T-109b), retention jobs (T-109g), user-facing audit API (T-113).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/repo.rs`, `crates/eq-audit/src/sealer.rs`, `.sqlx/` | modify | Sealing/verification logic, repository queries and offline metadata. |
| `tasks/M1-core/T-109e-audit-chain.md` | modify | Report and status. |

## Interfaces
Preserve `verify(tenant, from_seq, to_seq) -> VerifyReport { checked, anonymized, first_mismatch }` and the parent `Sealer` contract.

## Business rules
No new values; use only the parent task's chain and timestamp rules.

## Security
- Triggers: tamper detection and tenant-isolated reads/writes (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: forged or cross-tenant chain → lock by tenant, validate transaction context, compare stored link and canonical content hashes.
- Abuse tests are owned by T-109f: tampering, anonymized proof, checkpoint and concurrent sealers.

## Implementation notes
Run `cargo sqlx prepare --workspace` against the fresh disposable PostgreSQL 16 database after query changes; commit `.sqlx/` metadata.

## Acceptance criteria
- AC1. Sealing walks unsealed rows in sequence order and uses the correct anchor and canonical document.
- AC2. Verification returns a first mismatching sequence and treats anonymized hashes as authoritative only when proof exists.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/tests/audit.rs` | Behavioral chain tests in dependent T-109f. |

## Definition of done
- [ ] Rust format/clippy and SQLx offline metadata checks pass.
- [ ] The child tests in T-109f exercise all chain behaviors before parent T-109 is reviewed.
- [x] Signed commit `feat(audit): add tamper-evident chain [T-109e]`.

## Report
**Plan** — (1) Recheck the parent chain contract, T-109d's Report, audit DDL/data dictionary and migration map → verify every selected field and anchor is already mapped and no migration is needed. (2) Add the `Sealer`/`VerifyReport` API, JCS hash document and direct approved crate dependencies → verify exports and task scope. (3) Keep SQL in `repo.rs`; seal in a tenant transaction under the specified advisory lock, walk unsealed rows by `seq`, reject a later sealed/checkpoint anchor that would break order, and persist hashes through `audit.seal` → verify exact field set, UTC microsecond timestamp format, and matching offline SQL metadata. (4) Verify the requested sequence range from the preceding row/checkpoint/zero anchor, checking links, canonical content, and anonymized proof → verify first mismatch and counts are set from checked rows. (5) Review source for tenant isolation, sensitive data handling, and SQL ownership, then run available formatting, policy, task, guard, ERD, metadata, diff and secret checks → record build/database checks as Not run under the active disk/resource rule.
**Assumptions** — Parent T-109 defines the chain fields, SHA-256/JCS encoding, timestamp form, anchors, and proof requirement. T-109d's Report and shipped code leave the sealer lock to this task and provide no other interface change. No schema or migration change is required.
**Test plan** — AC1: Given unsealed audit entries, when `seal_tenant` runs, then entries are processed in ascending `seq` and their hash input is the parent contract's JCS document prefixed by the previous/checkpoint/zero hash → `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint`, and `concurrent_sealers` in dependent T-109f. AC2: Given a requested range, when `verify` checks it, then it reports the first bad sequence, validates links and content, and accepts anonymized content only with a proof → the same named T-109f tests. Edge cases: empty ranges/chains, a checkpoint anchor, an unsealed predecessor, a sealed row after the first unsealed sequence, missing stored hash, and malformed/missing anonymization proof. Error case: a supplied tenant differing from transaction-local context is rejected. Abuse cases: cross-tenant access remains bounded by the app transaction/RLS and advisory lock; tampered content or missing proof is reported as a mismatch. These integration assertions are owned by T-109f; no database tests are added in this task.
**Summary** — Added `Sealer::seal_tenant` and `Sealer::verify` with tenant-scoped transactions, the audit advisory lock, ordered chain hashing, checkpoint anchors, and proof-aware verification. Hashes use the parent contract's JCS fields and UTC microsecond timestamps; all table queries remain in `repo.rs`. Sealing fails closed when existing sealed rows would make processing unsealed entries violate sequence order. Runtime/database tests are owned by T-109f and were not run under the current disk/resource restriction.
**Spec read** — `spec/02-architecture.md` ARC-09; `spec/06-domain-model.md` §§4.9/4.11; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` §§4/5/7/8; `spec/19-error-handling.md` §10; `spec/23-contracts-and-canonical-forms.md` CAN-70; `spec/29-erd.md` §§3.9/4/5/7; parent T-109; T-109d Report and shipped API; T-109f test contract.
**Files** — Modified `Cargo.lock`, `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/repo.rs`, `tasks/DECISIONS.md`, this task Report/status, and generated `tasks/INDEX.md`; created `crates/eq-audit/src/sealer.rs` and `.sqlx/query-0500fbe9c9670536019bd60872bc39ab0d818f936ead81d64ac6824bd6903787.json`, `.sqlx/query-1fb7e83c5714a2e357a8f84e291ab2fcf9e671b0121533bf5d1bc6db233bd7bf.json`, `.sqlx/query-2d33dc852145112ba1862fb4982846814946689f2bba4991f1c6aa7b397563c7.json`, `.sqlx/query-37cbbc1e6175f0e7c586ef6209813bda0dc3df0df161457b5c0fe35e9a262dd0.json`, `.sqlx/query-a13ed521c0b8e437e31bc693c9f41dba7e377771e56b8d7eb800600fb9a8becf.json`, `.sqlx/query-562d6838f1958f96cbc45fff2d465511e387370030fea742c9a3a6aefbfa7f30.json`, and `.sqlx/query-6fb46d660b090fef76fd4677614b5f754897a5f54c1080676bf941fb21a2858e.json`. Unrelated pending files, WIP SQLx metadata, Python caches, and `RUN_LOG.md` were not staged.
**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint`, `concurrent_sealers` (T-109f) | Not run: dependent database tests are assigned to T-109f; no disposable DB or Cargo target may be created under the current resource rule. |
| AC2 | `tamper_detected`, `anonymize_keeps_chain_valid`, `purge_with_checkpoint` (T-109f) | Not run for the same reason. |

T-109f is still `todo`; its five integration tests are not yet implemented or executed. Database used: none; test pass count for this task: 0 (tests are delegated to T-109f). `make e2e`: n/a, because this backend-only task adds no route or browser journey.
**Checks run** — `cargo fmt --all --check` ✅; `cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-CC0 warnings remain); `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` ✅; `python3 -I scripts/erd.py --check` ✅; `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8); `scripts/tasks-index.sh --check` ✅; `cargo metadata --locked --format-version 1 --no-deps > /dev/null` ✅; manual SQLx query/hash comparison ✅ (9 repository query macros matched offline metadata, including the manually recorded `audit.seal` query; metadata was not regenerated by SQLx); `git diff --check` ✅; targeted `gitleaks dir --redact --no-banner <changed-path>` ✅ for each task-owned changed path. Not run: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `SQLX_OFFLINE=true cargo check -p eq-audit`, `cargo sqlx prepare --workspace`, `SQLX_OFFLINE=true cargo test -p eq-audit --test audit`, and `make dev`; the fixed Cargo target cache is absent, no task database/container may be created, and SQLx prepare requires the unavailable database. `make e2e` n/a (no route/UI journey); `scripts/erd.py --migrations` n/a (no migration changed).
**Manual checks** — Compared the queried audit columns to spec 06 §§4.9/4.11 and the T-109 row in ERD §7; checked the JCS fields and UTC microsecond formatting against T-109; confirmed the tenant context check, app-role transaction, RLS-filtered `entries` query, tenant filter on the no-RLS checkpoint table, advisory lock key, and use of `audit.seal` instead of direct row updates. Confirmed the sealer uses the row before the first unsealed entry as its anchor, refuses an already sealed later sequence rather than extending the chain out of order, anonymized rows require a non-null proof, and S2 IP data is used only for hashing, never logged or returned. No live database or HTTP behavior was exercised.
**Debug log** — No test failure loop was entered. Formatting and static checks passed; the only verification omissions are listed above as Not run under the resource rule.
**Performance** — n/a; no query plan or runtime measurement was possible or required by this task.
**Security** — SBD-30 T3/T4 triggers. SBD-81: (1) auth/permission chain n/a (no endpoint); (2) tenant is set by `begin_system`, checked before data access, and the `entries` RLS boundary remains active; checkpoint reads are explicitly tenant-filtered; (3) no schema field is added, S1/S2 classes follow ERD, IP is never logged or returned, and canonicalization failures use a static error; (4) no DTO/input boundary; (5) no HTML/CSV output; (6) no secrets or key operations; (7) external calls n/a; (8) hash/link lengths, content, checkpoint anchor and proof are checked while mutations use the security-definer function; (9) no new security event is emitted; (10) named abuse tests are in T-109f and remain Not run until that task. Changed-path Gitleaks scans found no leaks.
**Deviations** — RATIFY TD-084: the parent signature does not define endpoint semantics for `from_seq`/`to_seq`; treat both as included so the requested endpoints are verified instead of silently omitted. RATIFY TD-085: the task Files table omitted the existing repository adapter, so keep SQL in `repo.rs` per CA-11/CQ-24. RATIFY TD-086: the parent does not define handling for an unsealed sequence before an already sealed later row; fail closed rather than extend or rewrite an out-of-order chain. T-109d TD-081–083 and parent T-109 TD-071–075/TD-066–067 were read; they do not change this part's chain interface.
**Dependencies affected** — T-109f and audit job parts.
**Rollback plan** — Revert this part commit; no schema changes.
**Follow-ups** — Before aggregate T-109 review, decide whether concurrent audit writers must coordinate with the same `audit.seal:` advisory lock: record inserts allocate sequence values before commit, while T-109d intentionally does not take the sealer lock, so concurrent commit order can differ from `seq` order. T-109f currently tests concurrent sealers, not concurrent writers. No writer-side behavior was changed here.

**Disk hygiene** — T-109e pre-implementation and post-check snapshots measured the repository at 409M; both the repository `target/` and fixed `$HOME/.cache/ritma/target` were absent. For the fixed target, `du -sh "$HOME/.cache/ritma/target" 2>/dev/null` produced no output (exit 1 because the path is absent) before and after. `timeout 5s docker system df` produced no output and exited 124 before and after. The resumed finalization snapshot matched: fixed and repository targets absent; Docker usage query timed out with exit 124. No Docker container, image, volume, test database or Cargo target was created or removed for this task.

**Workflow steps 0–12** — Step 0: resumed the claimed T-109e on `dev`, confirmed the index and preserved unrelated WIP. Step 1: read the task, parent contract and cited specifications. Step 2: read T-109d's Report/deviations/follow-ups and shipped record code, plus T-109f's delegated test contract. Step 3: n/a, backend-only. Step 4: wrote the Plan, assumptions and AC/error/edge/abuse test plan before implementation; recorded TD-084 and TD-085. Step 5: implemented the public sealer API, JCS/SHA-256 chain logic, verification, repository queries and offline metadata. Step 6: no DB runtime exercise was possible under the no-container/no-target rule; no endpoint or UI is in scope. Step 7: no runtime test failure loop occurred; all runnable static checks passed. Step 8: reviewed scope, canonical fields, SQL ownership, error handling and tenant boundaries. Step 9: n/a; no measured optimization target. Step 10: completed SBD-81 as recorded above and scanned task-owned paths. Step 11: reran available static, dependency, task, metadata, guard and secret checks; build/database checks are explicitly Not run. Step 12: completed this Report, set `review`, regenerated the index, and prepared one signed commit/push.

## Questions
- **Self-resolved (implementer, run 6; TD-084, needs architect ratification):** `verify(tenant, from_seq, to_seq)` names both bounds but does not say whether the range is inclusive → include both endpoints, because this verifies every explicitly requested boundary sequence and changes no business, schema, security or tenant-isolation rule.
- **Self-resolved (implementer, run 6; TD-085, needs architect ratification):** T-109e's Files table names `sealer.rs` but omits the repository adapter → keep all audit SQL in the existing `repo.rs` and add only its task-owned query helpers, because CA-11/CQ-24 require persistence SQL to stay in the module's repository layer.
- **Self-resolved (implementer, run 6; TD-086, needs architect ratification):** the parent does not define what sealing should do if an unsealed sequence precedes an already sealed row → fail closed without changing any row, because extending or rewriting the chain would violate the specified sequence order and append-only hash integrity.

## Review
n/a until review.
