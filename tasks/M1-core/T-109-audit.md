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
- [x] The writer/sealer concurrency regression test passes on PostgreSQL 16.
- [x] Final format, Clippy, ERD, dependency and guard checks pass on the assembled implementation; the full workspace test suite is explicitly Not run because its shared TestDb fixture creates additional databases/containers outside the fixed-resource rule.
- [x] Commit `feat(audit): tamper-evident audit log [T-109]`, signed off.

## Report

**Plan** — (1) Confirm all eight child tasks are in review and verify the parent split table/index. (2) Check original AC/test ownership against the child contracts and shipped code. (3) Verify record insertion uses the sealer's per-tenant transaction lock and run the writer/sealer regression on PostgreSQL 16. (4) Review the assembled implementation against the task, schema, transaction, tenant and security contracts. (5) Run Rust, dependency, ERD, guard, task and secret checks using the existing Cargo target and `ritma-db-1`; run only `ritma_test_T-109` for database tests. (6) Record the workspace-suite resource limitation, set review, regenerate the index, and make one signed T-109-only commit on dev.

**Assumptions** — T-109a through T-109h and their Reports are the shipped implementation; this parent performs final aggregate verification and the explicitly scoped writer/sealer race fix. Child status and commit records are read from their task files and the current branch history. The pre-existing .pending, .hold, .sqlx and Python-cache items are preserved worktree state and are not part of the active implementation. The audit schema and data classes remain those mapped in spec 06 §§4.9/4.11 and spec 29 §§3/4/5/7; no new schema, role, permission or error code is introduced.

**Test plan** — AC1: Given a business row and audit insert in one transaction, when the marker action is rejected, then rollback leaves no business row → audit::tests::audit_failure_rolls_back. AC2: Given the least-privilege eq_app role, when it tries UPDATE and DELETE on audit.entries, PostgreSQL denies both → app_role_cannot_mutate. AC3: Given a sealed row, when an owner tampers with its content, verification reports that sequence → tamper_detected. AC4: Given nested fake PII in a sealed row, when anonymization runs, the values are redacted, IP is null and verification remains valid → anonymize_keeps_chain_valid. AC5: Given expired sealed rows, when purge runs, the last removed hash is checkpointed and remaining rows verify → purge_with_checkpoint. AC6: Given an audit snapshot with a marked fake secret, when the derive serializes it, the value is replaced by [redacted] → derive::tests::redact_attr. AC7: Given two sealers for one tenant, when they run concurrently, the chain has one valid predecessor sequence → concurrent_sealers. AC8: Given an open record transaction, when the sealer starts, PostgreSQL must show it waiting on the same advisory lock before the writer commits, after which the chain verifies → concurrent_writer_and_sealer_preserve_sequence_order. Edge/error cases include empty chains/ranges, checkpoint-only starts, empty purge ranges, repeat anonymization, invalid policy bounds, missing job context, corrupt JCS content and mismatched tenant context. Abuse cases include app-role mutation, cross-tenant procedure arguments, fake-secret redaction and writer/sealer contention.

**Summary** — All eight child parts are in review with Reports and signed commits, and the original acceptance criteria/test names remain assigned exactly once. Record writes use the same tenant transaction lock as sealing and maintenance; all 12 audit integration tests, including the writer/sealer regression, passed against the single task database. The derive tests and static gates also passed. The full workspace test suite is Not run because its shared fixture would create `eq_template` and per-test databases or start additional containers, contrary to the fixed-resource rule.

**Spec read** — spec/02-architecture.md ARC-05; spec/06-domain-model.md §§4.9/4.11; spec/16-code-quality.md §§9/12; spec/17-clean-architecture.md §§1/4/7; spec/18-security-by-design.md §§4/5/8; spec/19-error-handling.md §10; spec/29-erd.md §§3/4/5/7; AGENTS.md §4 and WORKFLOW.md Steps 1–12.

**Files** —
- `crates/eq-audit/src/repo.rs` — acquire the existing tenant advisory transaction lock before inserting an audit record.
- `crates/eq-audit/tests/audit.rs` — regression test plus task-local owner/app pools that connect only to the named task database.
- `tasks/DECISIONS.md` — TD-093–TD-094, fixed-database TD-185 and license-ID TD-186.
- `tasks/M1-core/T-109-audit.md` — aggregate contract, status and current Report.
- `tasks/INDEX.md` — regenerated from task status.

**Acceptance criteria → tests**

| AC | Test | Result |
|---|---|---|
| AC1 | `audit_failure_rolls_back` | Passed in the task database: failed audit insertion left no business row. |
| AC2 | `app_role_cannot_mutate` | Passed in the task database: `eq_app` could not UPDATE or DELETE audit entries. |
| AC3 | `tamper_detected` | Passed in the task database: tampered sealed content reported the mismatched sequence. |
| AC4 | `anonymize_keeps_chain_valid` | Passed in the task database: nested PII was redacted, IP nulled and the chain remained valid. |
| AC5 | `purge_with_checkpoint` | Passed in the task database: purge checkpointed the last removed hash and remaining rows verified. |
| AC6 | `redact_attr` | Passed: derive integration tests 2/2; fake secret absent and redaction marker present. |
| AC7 | `concurrent_sealers` | Passed in the task database: simultaneous sealers produced one consistent chain. |
| AC8 | `concurrent_writer_and_sealer_preserve_sequence_order` | Passed in the task database: PostgreSQL showed the sealer waiting on the tenant advisory lock before writer commit, then the sealed chain verified. |

**Checks run** — ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check` (passed after fixing one rustfmt line break). ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`. ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true RITMA_TEST_DATABASE_URL=postgres://eq:eq@127.0.0.1:5433/ritma_test_T-109 RITMA_TEST_DATABASE_APP_URL=postgres://eq_app:test-secret-app@127.0.0.1:5433/ritma_test_T-109 cargo test --locked -p eq-audit --test audit -- --test-threads=1` — 12 passed, 0 failed. ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked -p eq-audit-derive` — `redact_attr` and `skip_attr` passed (2/2). ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" cargo deny check` — advisories, bans, licenses and sources passed; pre-existing duplicate-version warnings remain; `deny.toml` was unchanged. ✅ `python3 -I scripts/erd.py --migrations` — 26 migrated tables checked; ✅ `python3 -I scripts/erd.py --check`. ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`. ✅ `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py`. ✅ `git diff --check` and `gitleaks protect --staged --redact` — no leaks.

**Not run** — `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace`: the shared `eq_testkit::TestDb` fixture creates `eq_template` and per-test databases or starts Testcontainers, while this run permits only existing `ritma-db-1` and `ritma_test_T-109`. The audit integration tests were instead run through a task-local fixture against that fixed database. `cargo sqlx prepare --workspace` is n/a because this task did not change a compile-time query or migration. `make dev`, `make e2e`, and Web checks are n/a because the aggregate task adds no HTTP route, SPA behavior or user journey.

**Manual checks** — Used existing `ritma-db-1` (PostgreSQL 16.15), created only `ritma_test_T-109`, applied the checked-in migration set through version 13, and confirmed the test app pool used `eq_app` with `NOSUPERUSER NOBYPASSRLS`. The 12 audit tests exercised transaction rollback, role restrictions, tamper detection, anonymization, purge checkpoints, policy jobs and concurrent sealing. Verified zero active connections, then dropped only `ritma_test_T-109`. No Docker container, image or volume was created or removed. The pre-existing `ritma_test_T-110a4b` database was left untouched. No branch refs changed; local/remote branch lists were `dev`, `main`, `staging` (plus `origin/HEAD -> origin/main`). No new bundle was created because no branch cleanup or ref update was needed.

**Disk accounting** — Before: `du -sh "$HOME/.cache/ritma/target"` = 9.5G; `docker system df`: images 8 total / 7 active / 14.55GB, containers 10 / 10 / 10.49MB, volumes 7 / 7 / 5.145GB, build cache 0. After: target = 11G; images and containers unchanged, volumes 7 / 7 / 5.146GB, build cache 0. The target grew because the required focused Rust checks compiled artifacts in the mandated shared cache. I did not run `cargo clean` or remove cache files because cleanup is prohibited; this disk-size invariant could not be met for build artifacts. No resource outside the named task database was removed.

**Debug log** — The first `cargo fmt --all --check` found a required line break in the task-local pool setup; ran `cargo fmt --all` and the check then passed. `cargo deny check` exited 0 with existing duplicate-version warnings and no policy changes. The SQLx migration command applied all 12 migration files through version 13. The focused audit suite passed on its first run; no Testcontainers container was started.

**Performance** — n/a; no benchmark was authorized or needed. The fix reuses the existing per-tenant lock and serializes record insertion with the existing sealing/maintenance operations.

**Security** — Trigger: audit data and tenant isolation (SBD-30 T4). No endpoint, role, schema, data class, permission, secret or external call was added. Record still requires the transaction tenant and validates the actor; its transaction-scoped lock keeps audit insertion atomic. `app_role_cannot_mutate`, cross-tenant function checks, anonymization and tamper tests all passed using the `eq_app` app pool, and fixture values are fake. SBD-81 review found no new client input, export, HTML/CSV output, crypto or logging path. Staged Gitleaks passed.

**Deviations** — RATIFY TD-066/TD-067: complete required task headings from the existing contract. RATIFY TD-071/TD-072: include the ERD-05 reason key in recursive redaction and honor the mapped checkpoint timestamp. RATIFY TD-073: supply PolicyStore through the shared JobCtx dependency. RATIFY TD-074: keep the package-scoped first-party AGPL exception. RATIFY TD-075: split the oversized implementation into T-109a–T-109h. RATIFY TD-093: acquire the existing tenant lock before allocating an audit sequence. RATIFY TD-094: observe PostgreSQL lock contention before releasing the writer. RATIFY TD-185: use a task-local fixture so tests use only the permitted named database. RATIFY TD-186: preserve TD-010's existing T-005 ownership and rely on TD-074 for the audit derive license. Child-specific RATIFY TD-077–TD-092 remain in their owning Reports.

**Dependencies affected** — T-110 and T-110a can rely on the transactional audit record API; T-113 consumes the audit module for its endpoints; T-117a supplies runtime scheduling/tenant context; T-610 consumes the mapped schema. No dependency interface or schema changed.

**Rollback plan** — Revert the signed T-109 aggregate verification commit to remove only its task-local test fixture and report/decision updates. The previously shipped audit implementation remains intact; no persistent database state or migration remains after the task database was dropped.

**Follow-ups** — The full workspace test suite remains unverified under the fixed-resource rule; it can run when the test harness can reuse one named database without creating a template, random databases or containers. T-109d's historical Follow-up about omitting the sealer lock is superseded by TD-093; its Report remains unchanged as history. T-117 still needs the authorized active TenantInfo source; T-108's platform dead-letter attention limitation remains for architect review. Human review should ratify TD-066–TD-075 and TD-093–TD-094, TD-185 and TD-186; TD-077–TD-092 remain in child Reports.

**Workflow steps 0–12** — Step 0: resumed T-109 on `dev`, preserved existing unrelated WIP, confirmed only `main`, `staging` and `dev` refs, and checked the task index. Step 1: reviewed the parent task and cited architecture, quality, security, error and ERD sections. Step 2: reviewed dependency and child Reports/code; the eight child parts are `review`. Step 3: n/a, backend-only. Step 4: retained the existing AC/edge/error/abuse test map and updated it for the fixed task database. Step 5: retained the shared-lock implementation and made the integration fixture use explicit owner/app URLs for `ritma_test_T-109`; no migration or production interface changed. Step 6: the audit API/database behavior ran through its integration suite; no HTTP server request applies. Step 7: corrected one rustfmt finding and reran the check; tests passed on the first run. Step 8: reviewed transactional locking, actor/tenant checks, app-role privileges and task-local secret handling; no `#[allow]` or schema changes. Step 9: n/a, no measured optimization. Step 10: completed SBD-81 review and staged Gitleaks. Step 11: format, workspace Clippy, focused audit/derive tests, cargo-deny, ERD, guards and task gates passed; full workspace tests are Not run due to extra container/database creation. Step 12: updated this Report, set `review`, regenerated the index, staged only T-109 files, ran required pre-commit checks, and created one signed commit pushed to `dev`.
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
- **Self-resolved (implementer, run 25; TD-185):** the shared `eq_testkit::TestDb` may create a template, random test databases and a container, but the disk rule permits only `ritma-db-1` and `ritma_test_T-109` → use test-local owner and app pools with explicit task URLs because that keeps the database isolation while respecting the fixed resource boundary.
- **Self-resolved (implementer, run 25; TD-186, needs architect ratification):** the resumed instructions request the audit derive license resolution as TD-010, but TD-010 already belongs to T-005 and TD-074 already records this exact AGPL package exception → preserve the unique TD-010 entry and rely on TD-074, because the registry must not duplicate decision IDs.

## Review
Ready for human review. T-109 is `review`; the focused PostgreSQL tests and final static checks are recorded above. The full workspace suite is explicitly Not run under the fixed-resource rule.
