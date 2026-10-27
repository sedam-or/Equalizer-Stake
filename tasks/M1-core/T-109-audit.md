---
id: T-109
title: Add the audit log (schema, record API, Auditable redaction, sealer hash chain, verify, anonymise and purge functions)
milestone: M1
status: todo
depends_on: [T-106, T-108]
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
- The migration, the functions and grants above.
- Add `crates/eq-audit-derive` (proc-macro, deps `syn`, `quote`, `proc-macro2`) to the workspace members. `eq-audit` re-exports the derive.
- The crate `eq-audit`: `record`, `Auditable` + the derive crate `eq-audit-derive`, `Sealer`, `verify`, the jobs `audit.retention`, and `AuditModule` (manifest always_on) replacing the stub.
- A nightly verify job `audit.verify_recent` (cron `30 3 * * *`) that verifies the last 24 h. A mismatch inserts an `attention_items` row (severity `high`, kind `audit_chain_mismatch`, module_id `audit`).

### Out
- The audit HTTP API (T-113 adds `/audit/entries`, `/audit/verify`). Privacy and deletion requests (M6).

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
| `crates/eq-audit/tests/audit.rs` | AC1–AC5, AC7 |
| `crates/eq-audit-derive/tests/derive.rs` | `redact_attr`, `skip_attr` |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(audit): tamper-evident audit log [T-109]`, signed off

## Report

## Questions
