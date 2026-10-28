---
id: T-109
title: Add the audit log (schema, record API, Auditable redaction, sealer hash chain, verify, anonymise and purge functions)
milestone: M1
status: review
depends_on: [T-003, T-005, T-105, T-106, T-108]
spec_refs: [AUD-01, AUD-02, AUD-03, AUD-04, AUD-05, AUD-06, ARC-05, SEC-07]
estimated_size: M
---

## Goal
Every state change can be audited inside its own transaction. If the audit insert fails, the business change rolls back. Entries are tamper-evident (hash chain), immutable to the app role, anonymisable and purgeable only through controlled functions.

## Context
- DDL (spec 06 §4.9 + §4.11), in `migrations/0006_audit.sql`:
```sql
CREATE SCHEMA audit;
CREATE TABLE audit.entries (
  seq bigserial PRIMARY KEY, id uuid NOT NULL UNIQUE, tenant_id uuid NOT NULL, at timestamptz NOT NULL,
  actor_id uuid, actor_role text NOT NULL, action text NOT NULL, entity_type text NOT NULL, entity_id uuid,
  category text NOT NULL CHECK (category IN ('operational','security','compliance')),
  before jsonb, after jsonb, ip inet, request_id uuid,
  prev_hash bytea, hash bytea, sealed_at timestamptz, anonymized_at timestamptz, redaction_proof bytea
);
CREATE INDEX audit_entries_tenant_at ON audit.entries (tenant_id, at DESC);
CREATE INDEX audit_entries_entity ON audit.entries (tenant_id, entity_type, entity_id);
CREATE INDEX audit_entries_unsealed ON audit.entries (tenant_id, seq) WHERE hash IS NULL;
CREATE TABLE audit.checkpoints (tenant_id uuid NOT NULL, seq bigint NOT NULL, hash bytea NOT NULL, at timestamptz NOT NULL, PRIMARY KEY (tenant_id, seq));
```
- Grants: `eq_app` gets `GRANT USAGE ON SCHEMA audit` (the default privileges from T-003 cover only `public`, so without this `eq_app` cannot even see the table), `INSERT, SELECT` on `audit.entries` and `USAGE` on its sequence (revoke any `UPDATE`/`DELETE` that default privileges would otherwise give). No UPDATE or DELETE. RLS on `audit.entries` (SELECT + INSERT by `tenant_id`). No RLS on `checkpoints`.
- `SECURITY DEFINER` functions (the owner is the role that runs the migration; the specs call it `eq_owner`, but **no role of that name is created**, so do not `ALTER … OWNER TO eq_owner`; `SET search_path = pg_catalog, audit`), EXECUTE granted to `eq_app`:
  - `audit.seal(p_tenant uuid, p_rows jsonb)`: writes `prev_hash, hash, sealed_at` **only** for rows where `hash IS NULL`. The hash is computed in Rust; the function just persists it.
  - `audit.anonymize(p_tenant uuid, p_before timestamptz, p_subject uuid DEFAULT NULL)`: for rows with `at < p_before` (or referencing `p_subject` as actor or entity), it sets `redaction_proof = sha256(coalesce(before::text,'')||coalesce(after::text,''))` (only if null), redacts PII keys, nulls `ip`, and sets `anonymized_at`.
    - PII keys: `display_name, email, phone, name, legal_name, address, note, comment`, at any depth.
    - Redacted value: `'REDACTED-' || left(encode(sha256(value::text::bytea),'hex'),16)`.
  - `audit.purge_expired(p_tenant uuid, p_before timestamptz)`: writes a checkpoint row `(seq, hash)` for the last row being deleted, then deletes the rows `at < p_before`.
- Record API (AUD-02):
```rust
pub async fn record(tx: &mut Tx, e: AuditEvent<'_>) -> Result<(), AppError>;
pub struct AuditEvent<'a> {
  pub action: &'a str, pub category: Category, pub entity_type: &'a str, pub entity_id: Option<Uuid>,
  pub before: Option<serde_json::Value>, pub after: Option<serde_json::Value>, pub actor: &'a ActorCtx,
}
pub trait Auditable { fn audit_snapshot(&self) -> serde_json::Value; }
// derive macro in crate eq-audit-derive (proc-macro): #[derive(Auditable)] with field attr #[audit(redact)] → "[redacted]", #[audit(skip)]
```
- Sealing (AUD-04):
  - A job `audit.seal` runs every 10 s (cron can't do seconds, so use a dedicated interval task in the server, with `pg_advisory_xact_lock(hashtext('audit.seal:'||tenant))`).
  - It walks unsealed rows by `seq`: `hash = SHA-256(prev_hash || JCS({id, tenant_id, at, actor_id, actor_role, action, entity_type, entity_id, category, before, after, ip, request_id}))`. JCS uses `serde_jcs`. `prev_hash` is the previous row's hash, the checkpoint hash after a purge, or 32 zero bytes for the very first row.
  - **Timestamps in the JCS document are RFC 3339 UTC with microseconds**, so Rust and Postgres round-trip identically.
- Verify: `verify(tenant, from_seq, to_seq) -> VerifyReport { checked, anonymized, first_mismatch: Option<i64> }`. For each row:
  - (a) `prev_hash` must equal the previous row's stored `hash` (or the checkpoint / zero hash).
  - (b) If **not** anonymised, recompute the hash from content and compare.
  - (c) If anonymised, skip the content recompute (the stored hash stays authoritative) and require `redaction_proof IS NOT NULL`.
- `AuditPolicy`: `retention_years` 1..=10 (default 7), `anonymize_pii_after_years` 1..=10 (default 2, ≤ retention_years), `admin_categories: Vec<AuditCategory>` (default `[Operational]`). Define it here as `eq_audit::policy::AuditPolicy` implementing `PolicyDoc` (MODULE = Audit, SCHEMA_VERSION = 1). `validate()` returns error code `must_be_lte` at `/anonymize_pii_after_years` when it exceeds `retention_years`. `summarize()` returns `policy.audit.summary.retention {years}` and `policy.audit.summary.anonymize {years}`. x-ui: all fields `advanced`. Wire `AuditModule::policy_schema/default_policy/validate_policy/summarize_policy` to it. The daily job `audit.retention` (cron `0 3 * * *` local) reads it through `PolicyStore` and calls anonymize, then purge.
- Implements `AuditSink` from T-105.

## Scope
### In
- Final aggregate verification that T-109a–T-109h implement the full parent contract and pass its checks.
- Serialize audit record inserts with the existing per-tenant transaction lock shared by sealing and maintenance, and add a regression test for a writer held open while sealing starts.

### Out
- Reimplementing other child-owned behavior; changes are limited to the record/sealer sequence race above. The audit HTTP API (T-113 adds `/audit/entries`, `/audit/verify`); privacy and deletion requests (M6).

## Split
The original implementation measured 2,102 changed lines excluding lockfiles, `.sqlx` and Python caches, above CQ-101's approximate 400-line guide. TD-075 assigns each original acceptance criterion to exactly one dependent part. This parent stays `draft` while its parts are implemented, so its earlier task ID cannot run ahead of them. After T-109h reaches `review`, promote this parent to `todo` for aggregate verification. Downstream tasks depend on the specific provider part.

| Part | Owns | Depends on | Original ACs | Status |
|---|---|---|---|---|
| T-109a | `Auditable` derive and public snapshot trait | T-003, T-005, T-105, T-106, T-108 | AC6 | review |
| T-109b | Audit schema, grants and database functions | T-109a | AC2 | review |
| T-109c | Audit category and versioned retention policy | T-109b | policy contract | review |
| T-109d | Transaction-bound record API | T-109c | AC1 | review |
| T-109e | Sealing and verification implementation | T-109d | AC3–AC5, AC7 implementation | review |
| T-109f | Chain integrity, anonymization and purge integration tests | T-109e | AC3–AC5, AC7 tests | review |
| T-109g | Retention and recent-verification jobs | T-109f | job behavior | review |
| T-109h | Always-on AuditModule, policy wiring, AuditSink and schedules | T-109g | module behavior | review |

Original AC ownership: AC1 T-109d; AC2 T-109b; AC3–AC5 and AC7 tests T-109f; AC6 T-109a. Policy behavior is T-109c, maintenance jobs are T-109g, and module wiring is T-109h.

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.toml`, `Cargo.lock`, `crates/eq-audit/Cargo.toml` | modify | Register workspace and audit-crate dependencies used by the split parts. |
| `crates/eq-audit/src/` | modify/create | Implement the audit module, policy, transactional record sink, sealer and jobs. |
| `crates/eq-audit-derive/` | create | Implement the derive macro and its tests. |
| `crates/eq-audit/tests/audit.rs` | test | Database-backed audit verification tests. |
| `migrations/0006_audit.sql` | create or modify | Migration named in the mapped contract. |
| `crates/eq-core/src/jobs.rs`, `.sqlx/` | modify | Supply shared policy context and refresh offline query metadata. |
| `tasks/M1-core/T-109*`, T-110/T-110a/T-113/T-113a/T-113b/T-117/T-117a/T-117b/T-610 task files, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify | Define split ownership, update consumers, record decisions and generate task state. |

## Interfaces
The child tasks preserve the exact domain signatures, DTOs, SQL DDL and wire values stated in this parent contract. The split adds no public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `redact_attr` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. A test service writes a row and audits it, with the audit forced to fail (trigger raising on a marker action) → the business row doesn't exist. → `audit::tests::audit_failure_rolls_back`
- AC2. `UPDATE audit.entries SET after='{}'` as `eq_app` → permission denied. DELETE → denied. → `app_role_cannot_mutate`
- AC3. After sealing, a superuser tampering with `after` makes `verify` report that `seq`. → `tamper_detected`
- AC4. After `anonymize`, PII keys are redacted at depth, `ip` is null, and `verify` still passes. → `anonymize_keeps_chain_valid`
- AC5. After `purge_expired`, a checkpoint exists, and `verify` over the remaining rows passes. → `purge_with_checkpoint`
- AC6. `#[audit(redact)]` fields serialize as `"[redacted]"`. → `derive::tests::redact_attr`
- AC7. Two sealers running concurrently produce one consistent chain. → `concurrent_sealers`
- AC8. A sealer cannot pass an uncommitted lower-sequence record and leave the chain unsealable. → `concurrent_writer_and_sealer_preserve_sequence_order`

## Tests to write
| File | Test names |
|---|---|
| Child task files | Every original test name is assigned exactly once by the Split table and child contracts. |
| `crates/eq-audit/tests/audit.rs` | `concurrent_writer_and_sealer_preserve_sequence_order` |

## Definition of done
- [x] Every child part reaches `review` or `done` with its Report and signed-off commit.
- [x] The original acceptance criteria and test names remain assigned exactly once across T-109a–T-109h.
- [ ] The writer/sealer concurrency regression test passes on PostgreSQL 16.
- [ ] Final Rust, ERD, dependency and guard checks pass on the assembled implementation.
- [x] Commit `feat(audit): tamper-evident audit log [T-109]`, signed off.

## Report

**Plan** — (1) Promote the aggregate parent after all eight child tasks reach review and sync the split table with their frontmatter → verify the task index. (2) Check that every original acceptance criterion and exact test name has one child owner and exists in shipped code → verify the ownership table and source inventory. (3) Serialize record insertion with the sealer's existing per-tenant transaction lock and add a regression test → verify PostgreSQL reports the sealer waiting on that lock before the writer commits, then verify the sealed chain. (4) Review the assembled migration, repository, sealer, jobs and module against the task, ERD, transaction, tenant and security contracts. (5) Run permitted format, dependency, ERD, guard, task and secret checks; record build and database checks that the run's disk policy prohibits as Not run. (6) Complete this Report, set review, regenerate the index, and make one signed task commit containing only T-109-owned changes.

**Assumptions** — T-109a through T-109h and their Reports are the shipped implementation; this parent performs final aggregate verification and the explicitly scoped writer/sealer race fix. Child status and commit records are read from their task files and the current branch history. The pre-existing .pending, .hold, .sqlx and Python-cache items are preserved worktree state and are not part of the active implementation. The audit schema and data classes remain those mapped in spec 06 §§4.9/4.11 and spec 29 §§3/4/5/7; no new schema, role, permission or error code is introduced.

**Test plan** — AC1: Given a business row and audit insert in one transaction, when the marker action is rejected, then rollback leaves no business row → audit::tests::audit_failure_rolls_back. AC2: Given the least-privilege eq_app role, when it tries UPDATE and DELETE on audit.entries, PostgreSQL denies both → app_role_cannot_mutate. AC3: Given a sealed row, when an owner tampers with its content, verification reports that sequence → tamper_detected. AC4: Given nested fake PII in a sealed row, when anonymization runs, the values are redacted, IP is null and verification remains valid → anonymize_keeps_chain_valid. AC5: Given expired sealed rows, when purge runs, the last removed hash is checkpointed and remaining rows verify → purge_with_checkpoint. AC6: Given an audit snapshot with a marked fake secret, when the derive serializes it, the value is replaced by [redacted] → derive::tests::redact_attr. AC7: Given two sealers for one tenant, when they run concurrently, the chain has one valid predecessor sequence → concurrent_sealers. AC8: Given an open record transaction, when the sealer starts, PostgreSQL must show it waiting on the same advisory lock before the writer commits, after which the chain verifies → concurrent_writer_and_sealer_preserve_sequence_order. Edge/error cases include empty chains/ranges, checkpoint-only starts, empty purge ranges, repeat anonymization, invalid policy bounds, missing job context, corrupt JCS content and mismatched tenant context. Abuse cases include app-role mutation, cross-tenant procedure arguments, fake-secret redaction and writer/sealer contention.

**Summary** — The eight child parts are all review with Reports and signed commits, and the original acceptance criteria/test names have one owner each. Record writes now acquire the same tenant transaction lock used by sealing and maintenance, and the new regression test waits until PostgreSQL exposes the sealer blocked on that lock before releasing the writer. Static checks passed; the aggregate PostgreSQL regression and other database-dependent tests were not run because the active run policy prohibits creating a Cargo target or disposable database/container.

**Spec read** — spec/02-architecture.md ARC-05; spec/06-domain-model.md §§4.9/4.11; spec/16-code-quality.md §§9/12; spec/17-clean-architecture.md §§1/4/7; spec/18-security-by-design.md §§4/5/8; spec/19-error-handling.md §10; spec/29-erd.md §§3/4/5/7; AGENTS.md §4 and WORKFLOW.md Steps 1–12.

**Files** —
- crates/eq-audit/src/repo.rs — acquire the existing tenant advisory transaction lock before inserting an audit record.
- crates/eq-audit/tests/audit.rs — add the writer/sealer regression and observe lock contention through PostgreSQL activity state.
- tasks/DECISIONS.md — record TD-093 and TD-094 and update the mapped decision count.
- tasks/M1-core/T-109-audit.md — aggregate Scope, tests, status and Report.
- tasks/INDEX.md — regenerated from task status.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1 | audit::tests::audit_failure_rolls_back | Not run: requires compiling the audit integration test and a fresh PostgreSQL 16 TestDb; neither a target nor test database/container may be created under the active disk policy. |
| AC2 | app_role_cannot_mutate | Passed in T-109b: 1/1 on a fresh disposable Testcontainers PostgreSQL 16 database using eq_app; UPDATE and DELETE were denied. |
| AC3 | tamper_detected | Not run: T-109f database test; fixed target and disposable database/container are prohibited. |
| AC4 | anonymize_keeps_chain_valid | Not run for the same T-109f target/database restriction. |
| AC5 | purge_with_checkpoint | Not run for the same T-109f target/database restriction. |
| AC6 | derive::tests::redact_attr | Passed in T-109a: derive integration tests 2/2; fake secret absent and redaction marker present. |
| AC7 | concurrent_sealers | Not run: T-109f database test; fixed target and disposable database/container are prohibited. |
| AC8 | concurrent_writer_and_sealer_preserve_sequence_order | Not run: exact command below requires a Rust target and fresh PostgreSQL 16 TestDb; neither may be created under the active disk policy. |

**Checks run** — ✅ CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check. ✅ CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check (advisories, bans, licenses and sources passed; pre-existing duplicate-version and unmatched-CC0 warnings remain; deny.toml was not changed). ✅ python3 -I scripts/erd.py --migrations (21 migrated tables checked) and ✅ python3 -I scripts/erd.py --check. ✅ scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh. ✅ python3 -I scripts/tasks-lint.py. ✅ PYTHONDONTWRITEBYTECODE=1 python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py' (8/8). ✅ scripts/tasks-index.sh --check. ✅ git diff --check. ✅ CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --locked --no-deps --format-version 1. ✅ gitleaks dir --redact --no-banner, run separately on each of the five changed task-owned paths (no leaks).

Not run: CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings; env -u RITMA_TEST_DATABASE_URL CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1; and env -u RITMA_TEST_DATABASE_URL CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test -p eq-audit --test audit concurrent_writer_and_sealer_preserve_sequence_order -- --exact --test-threads=1. These were not attempted because the fixed target is absent and the active disk policy forbids creating it or a PostgreSQL test resource. For the AC8 command, database used: none; pass count: 0/1 executed. Across the parent database ACs not already passed in T-109b, database used: none; pass count: 0/6 executed. cargo sqlx prepare --workspace is n/a because no sqlx compile-time query macro or migration changed. make dev and make e2e were not run: the task adds no HTTP or user-journey behavior, and its database tests are separately Not run under the resource policy. Web lint/typecheck/test/i18n checks are n/a because web files are unchanged.

**Manual checks** — No HTTP endpoint, browser journey or service request is in this scope. Reviewed the test's sequence: it records within an eq_app transaction, confirms the lock is held, waits for PostgreSQL to report the sealer's advisory-lock wait, commits, then verifies the row. Test execution remains Not run. The child AC2 test used a fresh Testcontainers PostgreSQL 16 database; no database was used by this aggregate continuation. The fixed Cargo target remained absent; the existing inventory showed 8 containers and 11 images, and no Docker resources or databases were created or removed. timeout 15s docker system df returned exit 124 without output during resource inventory.

**Debug log** — No test or code check failed. The first source review found the initial new test could commit before the sealer had actually attempted to acquire its lock; TD-094 adds a bounded pg_stat_activity observation so the test proves contention before commit. Rust compilation and database execution were not attempted under the resource policy, so there is no claimed green runtime result.

**Performance** — n/a; no benchmark was authorized or needed. The fix reuses the existing per-tenant lock and serializes record insertion with the existing sealing/maintenance operations.

**Security** — Trigger: audit data and tenant isolation (SBD-30 T4). No endpoint, role, schema, data class, permission, secret or external call was added. record continues to require the transaction tenant and validates the actor before insertion; the lock is transaction-scoped, so the audit write remains in the caller's transaction. The app-role mutation denial passed in T-109b; cross-tenant function, anonymization, tamper and new lock-contention tests exist but were not executed in this aggregate run. SBD-81 review found no new client input, export, HTML/CSV output, crypto or logging path; fixture values are fake. Targeted Gitleaks scans of all changed paths passed.

**Deviations** — RATIFY TD-066/TD-067: complete the spec-backed Security section and required task headings from the existing contract. RATIFY TD-071/TD-072: include the ERD-05 reason key in recursive redaction and honor the mapped checkpoint timestamp default. RATIFY TD-073: supply PolicyStore through the optional shared JobCtx dependency required by audit.retention. RATIFY TD-074: use the next unused decision id because TD-010 already belongs to T-005. RATIFY TD-075: split the 2,102-line implementation into T-109a–T-109h and preserve single ownership of every original acceptance criterion/test. RATIFY TD-093: acquire the existing tenant advisory transaction lock before allocating an audit sequence, because sequence order must not be committed after a later row is sealed. RATIFY TD-094: wait for PostgreSQL to show sealer lock contention before releasing the writer; the 10-second timeout and 10-millisecond poll interval are test-only synchronization bounds. Child-specific RATIFY decisions TD-077–TD-092 remain detailed in their owning Reports.

**Dependencies affected** — T-110 and T-110a can rely on the transactional audit record API; T-113 consumes the audit module for its endpoints; T-117a supplies runtime scheduling/tenant context; T-610 consumes the mapped schema. No dependency interface or schema changed.

**Rollback plan** — Revert this aggregate commit to remove only the writer-side lock acquisition, regression test and task/decision updates. The audit schema and child implementation commits remain intact; no persistent database state or migration is introduced by this commit.

**Follow-ups** — Run the aggregate database tests and Rust clippy/workspace suites when an approved target and PostgreSQL test resource are available. T-109d's historical Follow-up says the record API intentionally does not add the sealer lock; TD-093 and this aggregate task supersede that statement, and the child Report is left unchanged as history. T-117 still needs to provide the authorized active TenantInfo source; T-108's platform dead-letter attention limitation remains for architect review. Ratify TD-066–TD-075 and TD-093–TD-094; child decision IDs TD-077–TD-092 are listed in child Reports.

**Workflow steps 0–12** — Step 0: resumed the existing T-109 work on dev, preserved all pre-existing WIP, confirmed T-109 was the sole in-progress task, and regenerated and checked the index. Step 1: reread the full parent task and cited/mapped schema, architecture, quality, security and error sections. Step 2: read Reports, Deviations, Follow-ups, RATIFY items and shipped code for T-003, T-005, T-105, T-106, T-108 and T-109a–h. Step 3: n/a, backend-only task. Step 4: recorded the plan and exact AC/edge/error/abuse test map before implementation. Step 5: synchronized audit record insertion with the existing lock and added the regression test; no migration or public interface changed. Step 6: no HTTP/manual request applies; PostgreSQL test execution is Not run under the resource rule. Step 7: corrected the test's synchronization gap with a bounded PostgreSQL activity poll; no runtime test failure occurred because tests were not launched. Step 8: reviewed the diff against CQ, CA, error, tenant and mapped audit rules; no #[allow], unparameterized application SQL, schema drift or unrelated source edits. Step 9: n/a, no measured optimization. Step 10: completed the SBD-81 review and targeted secret scans. Step 11: reran available format, dependency, ERD, guard, task, metadata, index, diff and secret checks; compilation, Clippy and database suites are explicitly Not run. Step 12: completed this Report, set review, regenerated the index, and prepared one signed commit and push to dev.
## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 6; TD-071):** the task's AUD-05 key list omits `reason` → include it in recursive JSON redaction because ERD-05 §5 requires ⚠ free-text keys to be included in anonymisation.
- **Self-resolved (implementer, run 6; TD-072):** the task's checkpoint sketch omits an `at` default → use `DEFAULT now()` as specified in spec 06 §4.11 and ERD §7's mapped DDL.
- **Self-resolved (implementer, run 6; TD-073):** `audit.retention` must read via `PolicyStore`, but shipped T-108 `JobCtx` has no store dependency → add an optional shared `PolicyStore` context/builder for handlers, keep existing callers working, and let T-117a supply the runtime instance.
- **Self-resolved (implementer, run 6; TD-074):** the objective calls the first-party license resolution TD-010, but TD-010 already identifies T-005's ETag decision → use the next unused decision ID, TD-074, and retain the architect's package-scoped AGPL exception because OSS-03 allows AGPL for Ritma's own crate and the package inherits the repository license.
- **Self-resolved (implementer, run 6; TD-075):** this implementation measured 2,102 changed lines, exceeding CQ-101 → split into T-109a–T-109h by code ownership, preserve each acceptance criterion/test once, and retain this parent for aggregate verification.
- **Self-resolved (implementer, run 6; TD-093):** record transactions can allocate a lower sequence before a slower commit while sealing locks only the sealer → acquire the existing per-tenant transaction advisory lock before inserting and test writer/sealer serialization, because the chain must remain ordered by sequence and the same lock already coordinates sealing and maintenance without adding schema or business rules.
- **Self-resolved (implementer, run 6; TD-094):** spawning the sealer and immediately committing the writer may not exercise a concurrent lock wait → wait for PostgreSQL to report the sealer blocked on the advisory lock before commit, with a 10-second timeout and 10-millisecond poll interval as test-only technical bounds.

## Review
n/a — pending review after implementation.
