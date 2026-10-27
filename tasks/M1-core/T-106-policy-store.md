---
id: T-106
title: Add PolicyDoc, PolicyStore (versioned, effective-dated, cached), validation pipeline and preset merge
milestone: M1
status: blocked
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

**Plan** — (1) Add `PolicyDoc`, `Versioned`, RFC 7386 merge and the `x-ui` builder → verify with the listed policy and merge-vector tests. (2) Add the JSON Schema → serde → `validate()` pipeline and schema-keyword-to-`FieldError` mapping → verify JSON Pointers and exact validation codes. (3) Add core-owned `policies` reads/inserts inside the supplied tenant `Tx`, with transaction-scoped version serialization → verify concurrent writes and append-only behavior as `eq_app`. (4) Add effective-date reads, schema migration through `ModuleRegistry`, cache and `NOTIFY` invalidation → verify with a fresh PostgreSQL 16 database. (5) Wire audit/event and initial-policy ports, then run Rust, SQLx, guard, security and E2E checks. **Stopped at Step 4:** the task and shipped interfaces do not provide an authorized source for the tenant preset patch required by `get`/`current` when the row is missing (Q1 below); implementation did not start.

**Assumptions** — The existing `policies` table and RLS/append-only grants from migration `0003_core_policies_files.sql` are authoritative; this task adds no migration or schema object. The table owner is core and its columns are S1 per ERD-01/05. The task explicitly allows either a row lock or advisory lock for version allocation; the intended implementation choice is a transaction-scoped advisory lock because always-on modules have no `tenant_modules` row (MOD-06). No business constant was selected. Preset lookup was not assumed or implemented.

**Test plan** —
- AC1: Given policy versions effective 2026-01-01 and 2026-03-01, when reading at 2026-02-15 and 2026-03-01T00:00, then the earlier and later versions are returned → `policy::tests::effective_dating`.
- AC2: Given a future-dated v3, when `current` runs before and after `TestClock` advances, then it returns v2 and v3 respectively → `future_version_applies_later`.
- AC3: Given `due_day < issue_day`, when a test policy is stored, then validation returns `Validation` at `/cycle/due_day` → `cross_field_error_pointer`.
- AC4: Given the RFC 7386 Appendix A vectors, when each patch is merged, then the expected JSON is returned → `merge_patch_rfc7386_vectors`.
- AC5: Given two concurrent writes for one tenant/module, when both commit, then versions 2 and 3 are assigned → `concurrent_put_serialized`.
- AC6: Given `eq_app`, when it attempts `UPDATE policies`, then PostgreSQL rejects it → `append_only`.
- AC7: Given two stores and a committed write, when the second store reads, then `NOTIFY eq_policy_changed` invalidates its cache within 1 second → `cache_invalidated_by_notify`.
- Additional exact validation cases: JSON Schema `required`, `type`, range, length, item-count, pattern/format, enum/const/oneOf, `uniqueItems`, and `additionalProperties` map to the VAL-22 codes with escaped JSON Pointer segments; serde and `PolicyDoc::validate` failures remain distinguishable.
- Edge cases: same `effective_from` selects the greatest version; future versions do not alter current; empty and null merge patches; array replacement; nested object deletion; missing-row fallback; schema-version migration; malformed or unknown NOTIFY payloads; duplicate `ensure_initial` calls.
- Error cases: invalid schema/documents return field errors; migration, SQL, audit or event failure propagates to the caller so the transaction can roll back. No new error code is specified or planned.
- Abuse cases: an `eq_app` tenant transaction cannot read or write another tenant's policy; a warm cache cannot return another tenant's document; policy changes are audited and emitted transactionally.

**Summary** — No policy-store code or tests were changed. Analysis found a blocking mismatch between POL-04's requirement to merge a tenant's preset on a missing row and the available least-privilege interfaces: `PolicyStore::get/current` and the shipped T-105 `PolicyInit` port receive no preset data, while `eq_app` cannot select `tenants.preset_id/preset_version`. The task is blocked for an architect decision on the approved lookup/API path.

**Spec read** — `spec/03-module-system.md` §§2–3 (MOD-06); `spec/04-policy-engine.md` §§2–5 (POL-01/03–10); `spec/05-presets-and-setup.md` SET-01/02 and §2.2 step 7; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1–2/4 (CA-01/10/11/30/31); `spec/18-security-by-design.md` §§4–5/7–8 (SBD-30/42/53/71/73/81); `spec/19-error-handling.md` §§5/8/10; `spec/22-validation-and-sanitization.md` VAL-22; `spec/24-open-source-stack.md` §§2/6; `spec/29-erd.md` §§3.1/5/7. Read the `policies` DDL in `spec/04` §2, its ERD dictionary entry, and the table-to-migration row.

**Files** — No implementation files changed. Updated this task's Report/Questions and regenerated `tasks/INDEX.md` for the blocked status.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `policy::tests::effective_dating` | Not run; blocked before implementation by Q1. |
| AC2 | `future_version_applies_later` | Not run; blocked before implementation by Q1. |
| AC3 | `cross_field_error_pointer` | Not run; blocked before implementation by Q1. |
| AC4 | `merge_patch_rfc7386_vectors` | Not run; blocked before implementation by Q1. |
| AC5 | `concurrent_put_serialized` | Not run; blocked before implementation by Q1. |
| AC6 | `append_only` | Not run; blocked before implementation by Q1. |
| AC7 | `cache_invalidated_by_notify` | Not run; blocked before implementation by Q1. |

**Checks run** —
- ✅ `scripts/tasks-index.sh --check`.
- ✅ `python3 scripts/erd.py --check` — ERD matches the DDL.
- ✅ `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh`.
- ✅ `gitleaks dir --redact --no-banner --no-color tasks/M1-core/T-106-policy-store.md` — no leaks found.
- ✅ `git diff --check`.
- n/a `python3 scripts/erd.py --migrations`; no migration changed.
- Not run: `cargo fmt --all --check`, strict Clippy, workspace/unit/integration tests, `cargo deny check`, SQLx prepare, `make dev`, and `make e2e`; Step 4 blocked before code or dependency changes.
- Not run: hosted GitHub Actions; it requires the hosted CI environment.
- n/a web checks; no web path is in scope.

**Manual checks** — n/a; no runtime code was changed and the missing preset source prevented a truthful end-to-end policy initialization check.

**Debug log** — None; no implementation attempt was made. The blocker was found during interface/security analysis before coding.

**Performance** — n/a; no query or runtime path was implemented or measured.

**Security** — Trigger T4 applies because policy reads/writes use the RLS-protected core table and policy changes must be audited. SBD-81 review before coding: (1) no HTTP handler or permission is added here; T-113 owns the settings endpoint; (2) tenant scope is provided by `Tx` and must stay protected by RLS; (3) `policies` fields are S1 and no new field is introduced; (4–8) no DTO, external call, crypto operation, or money/booking behavior is added; (9) POL-03/SBD-53 require the policy change audit and outbox event in the caller transaction; (10) planned abuse tests include app-role cross-tenant isolation and append-only enforcement. The implementation is blocked because the only approved tenant metadata resolver omits preset identity, and direct `eq_app` reads from `tenants` are revoked; no access-control bypass or new resolver was introduced.

**Deviations** — RATIFY: (1) T-106 sketches a patch argument on `ensure_initial`, while the shipped T-105 `PolicyInit::ensure_initial(tx, tenant, module)` has none. Preserve the shipped trait signature and keep the patch-taking store API for explicit setup callers; spec 05 §2.2 step 7 says setup writes initial policies for every module. This resolves the mechanical call-shape mismatch, but does not resolve the missing-row lookup in Q1. (2) T-106 says to use T-105 `AuditSink`/`EventSink` for policy effects, but the shipped traits expose module-toggle-only methods. On resumption, add policy-specific methods to those same typed sink traits using the task's specified `core.policy.changed` audit action and `core.policy_changed.v1` event, preserving the named port boundary. Both choices need architect ratification; no source or port code was changed while blocked.

**Dependencies affected** — T-109 and T-110 depend directly or transitively on T-106 and are ineligible while it is blocked. T-107/T-108 are not blocked by this task.

**Rollback plan** — No source or schema change to roll back. Revert the task-status/report commit and regenerate `tasks/INDEX.md` if the architect reopens or replaces this task.

**Follow-ups** — Architect must answer Q1 with the authorized way to resolve a tenant's immutable `preset_id/preset_version` for a missing policy row, including the least-privilege `eq_app` path and the task/migration mapping if a resolver function is needed. The exact audit/event adapter implementation remains for T-106 after that answer.

**Step 0 — Start** — Started the lowest eligible task T-106 on `milestone/m1-core` at `a519f7c`; `T-102` and `T-105` were `review`, the tree was clean, and `scripts/tasks-index.sh --check` passed. Set T-106 to `in_progress` and regenerated the index.

**Step 1 — Task and specs** — Read the full T-106 contract and cited/mandatory policy, validation, architecture, security, error, stack and ERD sections listed above. Confirmed that the task adds no migration and that the existing `policies` migration row is T-101/`0003_core_policies_files.sql`.

**Step 2 — Dependencies** — Read T-102 and T-105 Reports, including Deviations and Follow-ups, and their shipped policy value types, module registry, `PolicyInit`, `AuditSink`, and `EventSink` interfaces. T-102's private validated types and two RATIFY items do not change this task's required policy-store API. T-105's `PolicyInit` accepts only tenant/module; its audit/event sinks only cover module toggles. T-105's corrected Report records 151 serial workspace tests and 6/6 E2E checks; its three RATIFY items and CQ-101 follow-up remain for human review.

**Step 3 — Design reference** — n/a; no UI implementation is in scope. The `x-ui` builder is a schema helper only.

**Step 4 — Plan and test plan** — Recorded above before implementation. Interface/security review found that the missing-row preset path has no authorized source, so the task was blocked before Step 5.

**Step 5 — Implement** — n/a; no code or tests changed because Q1 is unresolved.

**Step 6 — Run locally** — n/a; no runtime behavior was implemented.

**Step 7 — Debug** — n/a; no implementation failure occurred.

**Step 8 — Self-review** — n/a; no implementation diff exists. The interface review found the tenant-metadata path issue documented in Q1.

**Step 9 — Optimize** — n/a; no measured implementation exists.

**Step 10 — Security audit** — Reviewed the applicable SBD-81 items before coding. No control was changed; the missing approved preset lookup was escalated instead of reading the un-RLS `tenants` table directly.

**Step 11 — Revise and verify** — No implementation checks apply before Q1 is resolved. Task/index, ERD, guard, diff and targeted Gitleaks results are listed above; implementation checks remain not run because the task is blocked.

**Step 12 — Report** — Wrote the analysis, self-resolved interface choices, and Q1 with options/recommendation; set status to `blocked` and regenerated the index. No source change was committed.

## Questions
- **Self-resolved (implementer, run 5, needs architect ratification):** T-106's patch-taking `ensure_initial` sketch differs from the shipped T-105 `PolicyInit` signature → preserve the no-patch port signature and keep a patch-taking `PolicyStore` method for setup callers, because T-105 is the shipped code contract and spec 05 §2.2 step 7 initializes every module's policy during setup.
- **Self-resolved (implementer, run 5, needs architect ratification):** T-106 requires policy-change audit/event effects through T-105 sink traits, whose shipped methods only cover module toggles → add policy-specific methods to those same typed traits using the task's exact audit action and event name, because the task explicitly selects these ports and POL-03/SBD-53 require the effects.
- **Q1 (blocking):** Which authorized path supplies a tenant's preset patch to `get`/`current` and `PolicyInit` when no policy row exists, without widening `eq_app` table access or changing the normative API silently?
  - Context: POL-04 and T-106 require `P::default()` merged with the tenant's preset patch on a missing row. `PolicyStore::get/current` accept only `Tx`, `TenantId`, and time; the shipped T-105 `PolicyInit` accepts only `Tx`, `TenantId`, and `ModuleId`. SET-02 stores `preset_id/preset_version` on `tenants`, but SEC-05 revokes direct `eq_app` SELECT. The only shipped SECURITY DEFINER resolver `resolve_tenant_info_by_slug` returns `id, slug, status, timezone, locale, currency`, not preset identity. `presets::get` is assigned to T-301 and is not shipped yet; it cannot identify a tenant's preset by itself.
  - Options: (a) define a least-privilege resolver contract that returns the stored preset identity to `eq_app`, and assign its exact migration row in `spec/29-erd.md` §7 and task ownership; (b) revise the normative `PolicyStore`/`PolicyInit` APIs so callers pass a resolved preset or patch, and update the owning specs/tasks to define how setup and runtime callers obtain it.
  - Recommendation: (a), because it preserves POL-04's normative `get/current` signatures and the existing deny-by-default tenant-table boundary; the architect must assign the function contract and migration ownership before implementation.
