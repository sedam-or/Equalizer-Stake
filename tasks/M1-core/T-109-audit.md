---
id: T-109
title: Add the audit log (schema, record API, Auditable redaction, sealer hash chain, verify, anonymise and purge functions)
milestone: M1
status: draft
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

### Out
- Reimplementing child-owned code. The audit HTTP API (T-113 adds `/audit/entries`, `/audit/verify`); privacy and deletion requests (M6).

## Split
The original implementation measured 2,102 changed lines excluding lockfiles, `.sqlx` and Python caches, above CQ-101's approximate 400-line guide. TD-075 assigns each original acceptance criterion to exactly one dependent part. This parent stays `draft` while its parts are implemented, so its earlier task ID cannot run ahead of them. After T-109h reaches `review`, promote this parent to `todo` for aggregate verification. Downstream tasks depend on the specific provider part.

| Part | Owns | Depends on | Original ACs | Status |
|---|---|---|---|---|
| T-109a | `Auditable` derive and public snapshot trait | T-003, T-005, T-105, T-106, T-108 | AC6 | review |
| T-109b | Audit schema, grants and database functions | T-109a | AC2 | todo |
| T-109c | Audit category and versioned retention policy | T-109b | policy contract | todo |
| T-109d | Transaction-bound record API | T-109c | AC1 | todo |
| T-109e | Sealing and verification implementation | T-109d | AC3–AC5, AC7 implementation | todo |
| T-109f | Chain integrity, anonymization and purge integration tests | T-109e | AC3–AC5, AC7 tests | todo |
| T-109g | Retention and recent-verification jobs | T-109f | job behavior | todo |
| T-109h | Always-on AuditModule, policy wiring, AuditSink and schedules | T-109g | module behavior | todo |

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

## Tests to write
| File | Test names |
|---|---|
| Child task files | Every original test name is assigned exactly once by the Split table and child contracts. |

## Definition of done
- [ ] Every child part reaches `review` or `done` with its Report and signed-off commit.
- [ ] The original acceptance criteria and test names remain assigned exactly once across T-109a–T-109h.
- [ ] Final Rust, ERD, dependency and guard checks pass on the assembled implementation.
- [ ] Commit `feat(audit): tamper-evident audit log [T-109]`, signed off.

## Report

**Plan** — Split the measured 2,102-line implementation into the eight dependent parts in `## Split`, keeping this parent as the final aggregate check. Implement T-109a through T-109h in order, preserve every original AC/test name, then run the full Rust, SQLx, cargo-deny, ERD and guard checks on the assembled implementation.

**Assumptions** — Migration DDL and ownership follow `spec/06-domain-model.md` §4.9/§4.11 and the T-109 row in `spec/29-erd.md` §7; no new table, column, role, error code or permission is added. Audit records use the tenant and transaction supplied to `record`; job handlers use a shared `PolicyStore` supplied through the existing job context and do not query `policies` directly. The 10-second sealing interval is started by the M1 runtime integration task T-117a, which consumes the per-tenant `Sealer` operation provided here.

**Test plan** —
- AC1: Given a business row inserted in the same transaction, when the audit trigger rejects the marker action, then the transaction is rolled back and the row is absent → `audit::tests::audit_failure_rolls_back`.
- AC2: Given an app-role connection, when it attempts UPDATE or DELETE on `audit.entries`, then PostgreSQL denies both → `app_role_cannot_mutate`.
- AC3: Given a sealed row, when an owner changes its content, then verify reports that row's sequence as the first mismatch → `tamper_detected`.
- AC4: Given nested PII keys in a sealed row, when anonymization runs, then those values are replaced, IP is NULL, proof exists and verification passes → `anonymize_keeps_chain_valid`.
- AC5: Given expired sealed rows, when purge runs, then the last deleted hash is checkpointed and the remaining chain verifies → `purge_with_checkpoint`.
- AC6: Given a derived audit snapshot containing a marked fake secret, when it is serialized, then the result contains `[redacted]` and not the value → `derive::tests::redact_attr`; skipped fields are absent → `derive::tests::skip_attr`.
- AC7: Given two sealers targeting the same tenant, when they run concurrently, then the stored chain has one consistent predecessor/hash sequence → `concurrent_sealers`.
- Edge/error cases: empty chains, checkpoint-only chain starts, empty purge ranges, repeated anonymization, invalid and cross-field policy values, missing job tenant/policy context, corrupted JCS content, and a supplied tenant that differs from the transaction context.
- Abuse cases: `eq_app` cannot directly rewrite/delete rows; direct cross-tenant function arguments are rejected; derived redaction fails its assertion if the marked field leaks; concurrent sealing preserves one chain.

**Workflow trace** — Steps 0–3: resumed the existing T-109 working tree on `milestone/m1-core`; dependencies and shipped interfaces were reviewed; UI reference n/a. Step 4: measured the complete uncommitted implementation and split it under CQ-101, assigning each AC to one child. Steps 5–11: each child owns its implementation and listed tests; this parent runs final aggregate checks after T-109h. Step 12: pending aggregate Report, signed commit and push.

**Deviations** — RATIFY TD-071–TD-075: add the ERD-05 `reason` redaction key; preserve the mapped checkpoint timestamp default; pass the shared policy store through the job context required by `audit.retention` while leaving server interval scheduling to T-117a; record the package-scoped first-party AGPL exception under the next available decision id because TD-010 is already assigned; and split the 2,102-line implementation into dependent parts under CQ-101. RATIFY TD-066–TD-067: preserve the completed spec-backed Security section and task-template headings from the task audit.
**Security deviation** — RATIFY TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 6; TD-071):** the task's AUD-05 key list omits `reason` → include it in recursive JSON redaction because ERD-05 §5 requires ⚠ free-text keys to be included in anonymisation.
- **Self-resolved (implementer, run 6; TD-072):** the task's checkpoint sketch omits an `at` default → use `DEFAULT now()` as specified in spec 06 §4.11 and ERD §7's mapped DDL.
- **Self-resolved (implementer, run 6; TD-073):** `audit.retention` must read via `PolicyStore`, but shipped T-108 `JobCtx` has no store dependency → add an optional shared `PolicyStore` context/builder for handlers, keep existing callers working, and let T-117a supply the runtime instance.
- **Self-resolved (implementer, run 6; TD-074):** the objective calls the first-party license resolution TD-010, but TD-010 already identifies T-005's ETag decision → use the next unused decision ID, TD-074, and retain the architect's package-scoped AGPL exception because OSS-03 allows AGPL for Ritma's own crate and the package inherits the repository license.
- **Self-resolved (implementer, run 6; TD-075):** this implementation measured 2,102 changed lines, exceeding CQ-101 → split into T-109a–T-109h by code ownership, preserve each acceptance criterion/test once, and retain this parent for aggregate verification.

## Review
n/a — pending review after implementation.
