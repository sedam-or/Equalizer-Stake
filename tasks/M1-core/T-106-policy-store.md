---
id: T-106
title: Add PolicyDoc, PolicyStore (versioned, effective-dated, cached), validation pipeline and preset merge
milestone: M1
status: todo
depends_on: [T-102, T-105]
spec_refs: [POL-01, POL-03, POL-04, POL-05, POL-06, POL-07, POL-08, POL-09, POL-10, VAL-22]
estimated_size: M
---

## Goal
Any module can read the policy in force at an instant and append a new version. Validation runs in three layers and errors carry JSON Pointers. Initial policy rows are created from the default merged with the preset patch.

## Context
- Table `policies` (from T-101): `(tenant_id, module_id, version, schema_version, effective_from, document jsonb, note, created_by, created_at)`, PK `(tenant_id, module_id, version)`, append-only.
- Normative API (04 §3):
```rust
pub trait PolicyDoc: Serialize + DeserializeOwned + JsonSchema + Default + Clone + Send + Sync + 'static {
    const MODULE: ModuleId;
    const SCHEMA_VERSION: u32;
    fn validate(&self) -> Result<(), Vec<FieldError>>;
    fn summarize(&self) -> Vec<SummaryLine>;
}
impl PolicyStore {
    pub async fn get<P: PolicyDoc>(&self, tx: &mut Tx, tenant: TenantId, at: Timestamp) -> Result<Versioned<P>>;
    pub async fn current<P: PolicyDoc>(&self, tx: &mut Tx, tenant: TenantId) -> Result<Versioned<P>>;
    pub async fn put<P: PolicyDoc>(&self, tx: &mut Tx, tenant: TenantId, doc: P, effective_from: Timestamp, actor: &ActorCtx, note: Option<String>) -> Result<Versioned<P>>;
}
pub struct Versioned<P> { pub version: i32, pub effective_from: Timestamp, pub doc: P }
```
- POL-04: cache `(tenant, module) → Vec<Versioned<Value>>` sorted by effective_from, invalidated by `NOTIFY eq_policy_changed` (payload `tenant_id:module`). Missing row → `P::default()` merged with the tenant preset patch, plus `warn!(tenant, module, "policy row missing")`.
- POL-05: `get(at)` = the highest version with `effective_from <= at`. If several versions share the same `effective_from`, the highest version wins. A future-dated version doesn't affect `current`.
- POL-08: if the stored `schema_version < P::SCHEMA_VERSION`, call `Module::migrate_policy` in memory on read.
- POL-09: `merge_patch(base: Value, patch: &Value) -> Value` per RFC 7386 (null deletes, objects merge recursively, arrays replace).
- POL-10, in order: (1) JSON Schema validation with `jsonschema` (draft 2020-12, compiled schema cached per module); (2) serde into `P`; (3) `P::validate()`. Errors are `FieldError { pointer, code, params }`. Schema errors map to the VAL-03 field codes per VAL-22: `required`→`required`, `type`→`invalid_type`, `minLength`/`maxLength`→`too_short`/`too_long`, `minItems`/`maxItems`→`too_few_items`/`too_many_items`, `minimum`/`exclusiveMinimum`→`must_be_gte`, `maximum`/`exclusiveMaximum`→`must_be_lte`, `pattern`/`format`→`invalid_format`, `enum`/`const`/unmatched tagged `oneOf`→`not_allowed`, `uniqueItems`→`not_unique`, `additionalProperties`→`not_allowed`.
- `put` steps: validate; `version = max+1` under `SELECT … FOR UPDATE` of the tenant's `tenant_modules` row (or `pg_advisory_xact_lock(hashtext(tenant||module))`); insert; audit `core.policy.changed` (before = previous doc, after = new doc); emit `core.policy_changed.v1 {module, version, effective_from}`; `NOTIFY eq_policy_changed`.
- `ensure_initial(tx, tenant, module, preset_patch: Option<&Value>)` inserts v1 if absent. It implements the `PolicyInit` port from T-105.
- The x-ui helper: `eq_core::policy::ui::{XUi, Group, Widget, Unit}` plus a `ui!(group = Essentials, order = 1, widget = Money)` macro, or a builder producing the `serde_json::Value` used in `#[schemars(extend("x-ui" = …))]`. Key names exactly as in 04 §4: `group, order, widget, unit, visible_if, requires_module`.

## Scope
### In
- `eq_core::policy::{PolicyDoc, PolicyStore, Versioned, merge_patch, validate_document, ui}`.
- `eq_core::policy::DynPolicy`, an object-safe helper so the settings API (T-113) can work on `Value` per `ModuleId` through the registry (`Module::validate_policy` / `summarize_policy`).

### Out
- The concrete policy structs (T-113 for core/identity/audit; module tasks for the rest). The HTTP API (T-113).

## Implementation notes
- Audit and outbox writes go through the `AuditSink` / `EventSink` traits from T-105, so this task doesn't depend on T-107/T-109. The real implementations are wired in T-117.

## Acceptance criteria
- AC1. Given v1 effective 2026-01-01 and v2 effective 2026-03-01, `get(at=2026-02-15)` → v1 and `get(at=2026-03-01T00:00)` → v2. → `policy::tests::effective_dating`
- AC2. A future-dated v3 does not change `current` before its date. With `TestClock` advanced past it, it does. → `future_version_applies_later`
- AC3. `put` with `due_day < issue_day` on a test policy returns `Validation` with pointer `/cycle/due_day`. → `cross_field_error_pointer`
- AC4. `merge_patch` matches the RFC 7386 appendix A test vectors (all 15). → `merge_patch_rfc7386_vectors`
- AC5. Two concurrent `put`s produce versions 2 and 3 (no duplicate key error surfaced). → `concurrent_put_serialized`
- AC6. `UPDATE policies` as `eq_app` fails. → `append_only` (already covered in T-101; re-assert through the store's connection)
- AC7. Cache: after `put` on instance A, a second `PolicyStore` instance sees the new version within 1 s. → `cache_invalidated_by_notify`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/policy_store.rs` | the AC names above, with a test-only `TestPolicy { cycle: {issue_day: DayOfMonth, due_day: DayOfMonth} }` |
| `crates/eq-core/src/policy/merge.rs` | `merge_patch_rfc7386_vectors` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(core): versioned policy store [T-106]`, signed off

## Report

## Questions
