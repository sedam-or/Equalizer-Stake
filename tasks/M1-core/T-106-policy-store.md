---
id: T-106
title: Add PolicyDoc, PolicyStore (versioned, effective-dated, cached), validation pipeline and preset merge
milestone: M1
status: review
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `migrations/0011_core_tenant_preset_resolver.sql` | create | Least-privilege resolver for the existing tenant preset identity |
| `crates/eq-core/src/policy/` | create | Policy types, validation, merge patch, store, cache and UI metadata |
| `crates/eq-core/src/ports/sinks.rs` | modify | Add policy audit/event operations to the existing typed sinks |
| `crates/eq-core/tests/policy_store.rs` | create | Store integration tests and abuse cases |
| `crates/eq-core/src/policy/merge.rs` | create | RFC 7386 vectors |
| `crates/eq-testkit/Cargo.toml`, `crates/eq-testkit/src/module_sinks.rs` | modify | Add JSON value support and record policy audit/event calls in test doubles |
| `crates/eq-testkit/tests/` | modify | Keep the existing sink-double contract tests compiling |
| `crates/eq-core/Cargo.toml`, `Cargo.toml`, `Cargo.lock`, `.sqlx/` | modify | Use mapped dependencies and refresh offline query metadata |
| `tasks/M1-core/T-106-policy-store.md`, `tasks/INDEX.md` | modify | Record the implementation and regenerate the task index |
| `tasks/DECISIONS.md` | modify | Record T-106 self-resolutions TD-041–TD-053 |

## Interfaces
`PolicyStore::get/current/put` keep the task's normative signatures. A `PresetPatchSource` held by the store maps the `preset_id` and `preset_version` returned by `resolve_tenant_preset(uuid)` to a module's raw merge-patch value. T-301's embedded preset loader can implement this port without giving application code direct access to `tenants`.

The migration adds only `public.resolve_tenant_preset(p_tenant_id uuid) RETURNS TABLE(preset_id text, preset_version integer)`, as a `STABLE SECURITY DEFINER` function with `search_path = pg_catalog, public`; it reads the existing `tenants` columns, revokes PUBLIC execution and grants execution to `eq_app`. The store supplies the tenant id from its caller context and does not grant direct `tenants` table access.

## Business rules
None beyond POL-01..10 and the task context above. The resolver returns the tenant's stored preset identity; it does not choose or update a preset.

## Security
- Triggers: tenant isolation, policy data, audit and event side effects (SBD-30).
- Data classes touched: policy documents and preset identity follow the existing ERD classifications; the store adds no persisted fields.
- Tenant identity comes from the tenant transaction's caller; the narrowly projected resolver returns only the stored preset id/version and does not grant direct `tenants` table access. The cache key includes `(tenant_id, module_id)` (SBD-42/71).
- Policy SQL touches only the core-owned `policies` table; all writes, audit records and events share the caller's transaction (SBD-53, CA-30).
- Abuse-case tests: cross-tenant read/cache isolation, `eq_app` append-only rejection, and rollback when audit/event persistence fails.

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
| `crates/eq-core/tests/policy_store.rs` | the AC names above, with a test-only `TestPolicy { cycle: {issue_day: DayOfMonth, due_day: DayOfMonth} }`; `missing_policy_fallback_uses_requested_instant`, `resolver_returns_stored_preset_for_eq_app`, `resolver_unknown_tenant_returns_no_rows`, `app_cannot_select_preset_identity`, `resolver_works_inside_tenant_transaction`, `put_records_policy_audit_and_event`, `audit_sink_failure_rolls_back_policy_write`, `event_sink_failure_rolls_back_policy_write`, `policy_init_merges_preset_patch_once`, `cache_is_tenant_scoped`, `transaction_context_rejects_another_tenant`, `sink_double_records_policy_initialization`, `test_database_debug_redacts_credentials`, `policy_change_debug_redacts_documents` |
| `crates/eq-core/src/policy/merge.rs` | `merge_patch_rfc7386_vectors` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass
- [x] Commit `feat(core): versioned policy store [T-106]`, signed off

## Report

**Plan** — (1) Keep the architect-assigned next free migration `0011_core_tenant_preset_resolver.sql` and implement objective directive TD-001 exactly → verify the four resolver probes on PostgreSQL 16 and run `python3 -I scripts/erd.py --migrations`. (2) Add `PolicyDoc`, `Versioned`, RFC 7386 merge, the `PresetPatchSource` port and `x-ui` builder → verify merge vectors and preset identity/version handoff. (3) Add JSON Schema → serde → `validate()` and schema-keyword-to-`FieldError` mapping → verify JSON Pointers and exact validation codes. (4) Put policy SQL behind `policy/repo.rs`, keep reads/writes inside the supplied tenant `Tx`, and serialize versions with a transaction advisory lock → verify concurrent writes and append-only behavior as `eq_app`. (5) Add effective-date reads, schema migration through `ModuleRegistry`, tenant/module cache and `NOTIFY` invalidation → verify the two-store test on PostgreSQL. (6) Add policy audit/event operations to the existing typed sinks and implement `PolicyInit` → verify side effects and caller-transaction rollback. (7) Run Rust, SQLx, deny, guard, ERD, secret-scan and disposable PostgreSQL checks, then replace this placeholder with the completed Step 12 Report.

**Assumptions** — The existing `policies` table and RLS/append-only grants from migration `0003_core_policies_files.sql` are authoritative; the resolver migration adds a function only, not a table or column. The table owner is core and its columns are S1 per ERD-01/05. The task allows either a row lock or advisory lock for version allocation; use a transaction-scoped advisory lock because always-on modules have no `tenant_modules` row (MOD-06). `PresetPatchSource` is an injected adapter over the stored identity and version; the later embedded loader supplies its implementation. No business constant is selected.

**Test plan** —
- AC1: Given policy versions effective 2026-01-01 and 2026-03-01, when reading at 2026-02-15 and 2026-03-01T00:00, then the earlier and later versions are returned → `policy::tests::effective_dating`.
- AC2: Given a future-dated v3, when `current` runs before and after `TestClock` advances, then it returns v2 and v3 respectively → `future_version_applies_later`.
- AC3: Given `due_day < issue_day`, when a test policy is stored, then validation returns `Validation` at `/cycle/due_day` → `cross_field_error_pointer`.
- AC4: Given the RFC 7386 Appendix A vectors, when each patch is merged, then the expected JSON is returned → `merge_patch_rfc7386_vectors`.
- AC5: Given two concurrent writes for one tenant/module, when both commit, then versions 2 and 3 are assigned → `concurrent_put_serialized`.
- AC6: Given `eq_app`, when it attempts `UPDATE policies`, then PostgreSQL rejects it → `append_only`.
- AC7: Given two stores and a committed write, when the second store reads, then `NOTIFY eq_policy_changed` invalidates its cache within 1 second → `cache_invalidated_by_notify`.
- Additional exact validation cases: JSON Schema `required`, `type`, range, length, item-count, pattern/format, enum/const/oneOf, `uniqueItems`, and `additionalProperties` map to the VAL-22 codes with escaped JSON Pointer segments; serde and `PolicyDoc::validate` failures remain distinguishable.
- Edge cases: same `effective_from` selects the greatest version; future versions do not alter current; a missing-row default uses the requested instant as its synthetic effective time; empty and null merge patches; array replacement; nested object deletion; schema-version migration; malformed or unknown NOTIFY payloads; duplicate `ensure_initial` calls.
- Error cases: invalid schema/documents return field errors; migration, SQL, audit or event failure propagates to the caller so the transaction can roll back. No new error code is specified or planned.
- Abuse cases: an `eq_app` tenant transaction cannot read or write another tenant's policy; a warm cache cannot return another tenant's document; policy changes are audited and emitted transactionally.

**Summary** — Implemented the versioned policy store and preset-merge path. The task's missing-row lookup uses a function-only SECURITY DEFINER resolver for the tenant's existing preset identity, while an injected `PresetPatchSource` keeps embedded preset data out of the database boundary. The store retains the shipped `PolicyInit` method shape and policy writes use the task's named audit/event operations.

**Spec read** — `spec/03-module-system.md` §§2–3 (MOD-06); `spec/04-policy-engine.md` §§2–5 (POL-01/03–10); `spec/05-presets-and-setup.md` SET-01/02 and §2.2 step 7; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1–2/4 (CA-01/10/11/30/31); `spec/18-security-by-design.md` §§4–5/7–8 (SBD-30/42/53/71/73/81); `spec/19-error-handling.md` §§5/8/10; `spec/22-validation-and-sanitization.md` VAL-22; `spec/24-open-source-stack.md` §§2/6; `spec/29-erd.md` §§3.1/5/7. Read the `policies` DDL in `spec/04` §2, its ERD dictionary entry, and the table-to-migration row.

**Files** —
- `Cargo.toml`, `Cargo.lock`, `crates/eq-core/Cargo.toml`, `crates/eq-testkit/Cargo.toml` — added the deny-compatible JSON Schema and JSON value dependencies.
- `crates/eq-core/src/policy/mod.rs` — exported the new policy modules and types.
- `crates/eq-core/src/policy/doc.rs` — added `PolicyDoc`, `Versioned`, dynamic validation and preset source contracts.
- `crates/eq-core/src/policy/merge.rs` — added RFC 7386 merge behavior and vectors.
- `crates/eq-core/src/policy/repo.rs` — added core-owned policy SQL and the preset resolver call.
- `crates/eq-core/src/policy/store.rs` — added versioned storage, validation, transaction checks, cache and invalidation.
- `crates/eq-core/src/policy/ui.rs` — added policy schema UI metadata.
- `crates/eq-core/src/policy/validate.rs` — added JSON Schema, serde and domain validation mapping.
- `crates/eq-core/src/ports/sinks.rs`, `crates/eq-testkit/src/module_sinks.rs` — added typed policy audit/event operations and recording doubles.
- `crates/eq-core/tests/policy_store.rs` — added PostgreSQL store, resolver, isolation and rollback tests.
- `.sqlx/query-*.json` — added 12 offline query metadata files.
- `migrations/0011_core_tenant_preset_resolver.sql` — added the least-privilege preset identity resolver.
- `tasks/M1-core/T-106-policy-store.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` — completed the task report, regenerated the index and recorded TD-041–TD-053.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `policy::tests::effective_dating` (`eq-core --test policy_store`) | ✅ Passed in the workspace suite. |
| AC2 | `future_version_applies_later` (`eq-core --test policy_store`) | ✅ Passed in the workspace suite. |
| AC3 | `cross_field_error_pointer` (`eq-core --test policy_store`) | ✅ Passed in the workspace suite. |
| AC4 | `merge_patch_rfc7386_vectors` (`eq-core` unit test) | ✅ Passed in the workspace suite. |
| AC5 | `concurrent_put_serialized` (`eq-core --test policy_store`) | ✅ Passed in the workspace suite. |
| AC6 | `append_only` (`eq-core --test policy_store`, app role) | ✅ Passed in the workspace suite. |
| AC7 | `cache_invalidated_by_notify` (`eq-core --test policy_store`) | ✅ Passed in the workspace suite. |

**Checks run** — Final results after the requested-instant fallback test was added:
- ✅ `cargo fmt --all --check`.
- ✅ `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` (full workspace; 20 `policy_store` integration tests, including the four resolver probes; all other workspace tests and doc tests green).
- ✅ `DATABASE_URL=postgres://eq:test-secret-owner@127.0.0.1:55435/ritma SQLX_OFFLINE=false cargo sqlx prepare --workspace --check -- --all-targets` against the fresh disposable `ritma-t106-sqlx-final-20261028` PostgreSQL 16.15 database.
- ✅ `cargo deny check`; advisories, bans, licenses and sources passed with existing duplicate-`base64` and unmatched `CC0-1.0` allowance warnings; `deny.toml` was not changed.
- ✅ `python3 -I scripts/erd.py --migrations` (15 migrated tables checked) and `python3 -I scripts/erd.py --check`.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh && git diff --check`.
- ✅ `scripts/tasks-index.sh` after setting `review`, followed by `scripts/tasks-index.sh --check` (INDEX.md up to date).
- ✅ Targeted `gitleaks dir --redact --no-banner` scans on all changed T-106 source, tests, manifests, migration, `.sqlx` metadata, task and decision register paths.
- ✅ `make e2e` on a fresh disposable PostgreSQL 16.15 database with one fake setup tenant: 6/6 Playwright checks; exact command and database are recorded under Manual checks.
- ✅ `pnpm -C web lint`, `pnpm -C web typecheck`, and `pnpm -C web test` passed in the implementation run; `make e2e` also built the production SPA.
- Not run: `pnpm -C web i18n:check`; `web/package.json` has no such script. The available `scripts/check-literals.sh` guard passed.
- Not run: hosted GitHub Actions, which requires a hosted runner.

**Manual checks** — `TestDb` ran each database test against an isolated database cloned from its disposable PostgreSQL 16.15 Testcontainers template. The resolver integration tests confirmed that `eq_app` receives only `(preset_id, preset_version)`, unknown ids return no rows, direct `SELECT` from `tenants` is denied with SQLSTATE `42501`, and the function works inside a tenant transaction. The same suite verified append-only writes, cross-tenant isolation, concurrent version allocation, cache notification, and rollback when either side-effect sink fails. `cargo run -p eq-server -- migrate` ran against a separate fresh PostgreSQL 16.15 container `ritma-t106-e2e-final-20261028` on port `55434`, after which one fake `e2e-test` tenant was seeded. Exact browser command: `E2E_SKIP_DB_START=1 DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:55434/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@127.0.0.1:55434/ritma RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://localhost:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off E2E_BASE_URL=http://127.0.0.1:8080 make e2e`; Chromium and WebKit mobile smoke checks passed 6/6. The task adds no HTTP route, so there is no policy endpoint for direct `make dev` requests.

**Debug log** — (1) The first cache test used a future `effective_from` but read at an earlier clock time; POL-05 correctly returned the earlier version, so the test now uses the same effective instant and verifies the greatest-version tie-break. (2) The required-field schema test initially produced an empty pointer because the code parsed library Debug text; it now matches the structured `ValidationErrorKind::Required` variant and extracts the property name. (3) `jsonschema` 0.30 failed the unchanged license gate through `referencing → fluent-uri → borrow-or-share` (MIT-0); disabling defaults did not remove it, so the same crate's 0.17.1 release with `draft202012` enabled was selected and `cargo deny check` passed. (4) The first browser run against an empty database returned `setup_required` (2/6); after seeding one fake tenant in the disposable E2E database, a fresh run passed 6/6. (5) A missing-row fallback had no defined effective timestamp although `Versioned` requires one; it now reports the caller's requested instant with synthetic version 0, covered by `missing_policy_fallback_uses_requested_instant` (TD-051).

**Performance** — n/a; no performance target or benchmark was specified, so no speculative query or cache optimization was added.

**Security** — SBD-81 review: (1) no HTTP handler or permission is added; T-113 owns the settings endpoint. (2) policy SQL and the resolver use the caller-supplied tenant id; the resolver exposes only the existing preset identity/version and grants `eq_app` function execution, while direct `eq_app` reads from `tenants` remain revoked. (3) policy data and preset identity retain their existing ERD classifications; no table or column is added. (4–8) no DTO, external call, crypto operation, or money/booking behavior is added. (9) POL-03/SBD-53 policy audit and event writes use the caller transaction. (10) abuse tests cover app-role tenant policy isolation, cache isolation, append-only enforcement, direct tenant-table denial, and rollback when a sink fails.

**Deviations** — RATIFY: preserve T-105's shipped no-patch `PolicyInit::ensure_initial(tx, tenant, module)` while keeping a patch-taking store helper (TD-041); add policy-specific methods to the existing typed sinks with the task's action/event names (TD-042); use the existing preset id/version through the injected `PresetPatchSource` (TD-046); implement the exact resolver contract in delegated directive TD-001 without claiming a transaction-local tenant guard (TD-047); use migration slot `0011`, the next free slot not assigned in `spec/29` §7 (TD-048); use `jsonschema` 0.17.1 with only `draft202012` enabled because `0.30`'s `referencing` dependency brings MIT-0 into the strict cargo-deny graph (TD-049); use a typed `PolicyChange` with a redacting `Debug` rather than a long positional audit signature (TD-050); represent an unpersisted fallback with version 0 and the caller's requested instant rather than an invented date (TD-051); bound the process-local cache to 1,024 tenant/module keys as a technical memory guard (TD-052); nest the effective-dating test so its full name matches the task's required `policy::tests::effective_dating` path (TD-053). The complete decision records are in `tasks/DECISIONS.md`.

**Dependencies affected** — T-109 and T-110 can use the policy store after this task is review. The injected preset-source port is available for the M3 preset loader to implement. T-107/T-108 are not blocked by this task.

**Rollback plan** — Revert the signed task commit. The migration is forward-only; if deployed, remove the resolver with a compensating migration that revokes `eq_app` execution and drops only `public.resolve_tenant_preset(uuid)`. No tenant or policy rows are deleted.

**Follow-ups** — The architect may ratify or reverse TD-041, TD-042 and TD-046–TD-053. T-301 must implement `PresetPatchSource` using its embedded preset loader and exact stored version before setup can apply preset policy patches. This task's implementation exceeds CQ-101's approximate 400-line M-size guide; it is already `review`, so it was not split under this run's size rule and needs human review of scope and maintainability.

**Step 0 — Start** — Resumed T-106 in `in_progress` on `milestone/m1-core` from the explicitly preserved uncommitted task work. Dependencies T-102 and T-105 were `review`; `scripts/tasks-index.sh --check` passed. Preserved the pre-existing task-lint and T-107 worktree edits outside this task's commit.

**Step 1 — Task and specs** — Read the full T-106 contract and cited/mandatory policy, validation, architecture, security, error, stack and ERD sections listed above. `policies` is owned by T-101 in `0003_core_policies_files.sql`; T-106 has no table row, and `0011` is the next free migration slot because `0010` is assigned to T-201 and `0012` to T-211. The delegated resolver contract adds no table or column.

**Step 2 — Dependencies** — Read T-102 and T-105 Reports, including Deviations and Follow-ups, and their shipped policy value types, module registry, `PolicyInit`, `AuditSink`, and `EventSink` interfaces. T-102's private validated types and two RATIFY items do not change this task's required policy-store API. T-105's `PolicyInit` accepts only tenant/module; its audit/event sinks only cover module toggles. T-105's corrected Report records 151 serial workspace tests and 6/6 E2E checks; its three RATIFY items and CQ-101 follow-up remain for human review.

**Step 3 — Design reference** — n/a; no UI implementation is in scope. The `x-ui` builder is a schema helper only.

**Step 4 — Plan and test plan** — Replaced the prior blocked plan above after the delegated resolver decision. Tests cover the resolver's four mandated probes, exact policy fallback/merge behavior, validation, effective dating, concurrent writes, cache invalidation, append-only access, and sink rollback.

**Step 5 — Implement** — Added the policy document trait, dynamic adapter, RFC 7386 merge, schema/serde/domain validation, x-ui metadata helper, tenant-scoped versioned store, resolver, cache invalidation listener, and typed audit/event operations. Policy SQL is confined to `policy/repo.rs`; the resolver migration adds no table or column. Added SQLx metadata and database abuse-case coverage.

**Step 6 — Run locally** — `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` passed all 20 policy-store integration tests on per-test databases cloned from disposable PostgreSQL 16.15 Testcontainers templates. The browser E2E command and fresh disposable database details are recorded under Manual checks; it passed 6/6. The backend-only store has no direct HTTP route, so the M1 browser smoke suite was run as the available runtime check.

**Step 7 — Debug** — The cache-notification and schema-pointer failures, dependency-gate issue, and first empty-database E2E failure are detailed in Debug log above. Also resolved the undefined synthetic fallback timestamp using the lookup instant and verified it with a new integration test (TD-051). No unresolved test failure remains.

**Step 8 — Self-review** — Confirmed own-table-only SQL; resolver returns only stored preset identity; writes, audit/event operations and notification use the caller transaction; cache keys include tenant/module; field errors use VAL-22 codes and pointers; no unlisted table or column was introduced. `PolicyChange` formats redact both document values.

**Step 9 — Optimize** — n/a; no performance target was specified or measured.

**Step 10 — Security audit** — Verified the resolver's exact `search_path`, `eq_app`-only execute grant and four required probes; direct app-role tenant SELECT fails with `42501`; transaction context rejects another tenant for reads and writes; cross-tenant cache test passes; audit/event sink failures leave the caller transaction rollbackable; redaction tests prevent policy documents and DB credentials from appearing in `Debug`.

**Step 11 — Revise and verify** — Reran Rust format/Clippy/workspace tests, SQLx metadata verification, `cargo deny`, ERD checks, all four guards, diff check and targeted secret scans. PostgreSQL store tests and a fresh-database Playwright run are green. Web lint/typecheck/unit tests passed in the implementation run; its `i18n:check` script is absent, while the available literal guard passed. Hosted CI was not run.

**Step 12 — Report** — Recorded the final results, resolver probes, disposable database details, and TD-041–TD-053 decisions. Set status to `review` and regenerated `tasks/INDEX.md`; this report and implementation are included in the single signed-off T-106 commit.

## Questions
- **Self-resolved (implementer, run 5, needs architect ratification):** T-106's patch-taking `ensure_initial` sketch differs from the shipped T-105 `PolicyInit` signature → preserve the no-patch port signature and keep a patch-taking `PolicyStore` method for setup callers, because T-105 is the shipped code contract and spec 05 §2.2 step 7 initializes every module's policy during setup. [TD-041]
- **Self-resolved (implementer, run 5, needs architect ratification):** T-106 requires policy-change audit/event effects through T-105 sink traits, whose shipped methods only cover module toggles → add policy-specific methods to those same typed traits using the task's exact audit action and event name, because the task explicitly selects these ports and POL-03/SBD-53 require the effects. [TD-042]
- **Q1 (blocking):** Which authorized path supplies a tenant's preset patch to `get`/`current` and `PolicyInit` when no policy row exists, without widening `eq_app` table access or changing the normative API silently?
  - Context: POL-04 and T-106 require `P::default()` merged with the tenant's preset patch on a missing row. `PolicyStore::get/current` accept only `Tx`, `TenantId`, and time; the shipped T-105 `PolicyInit` accepts only `Tx`, `TenantId`, and `ModuleId`. SET-02 stores `preset_id/preset_version` on `tenants`, but SEC-05 revokes direct `eq_app` SELECT. The only shipped SECURITY DEFINER resolver `resolve_tenant_info_by_slug` returns `id, slug, status, timezone, locale, currency`, not preset identity. `presets::get` is assigned to T-301 and is not shipped yet; it cannot identify a tenant's preset by itself.
  - Options: (a) define a least-privilege resolver contract that returns the stored preset identity to `eq_app`, and assign its exact migration row in `spec/29-erd.md` §7 and task ownership; (b) revise the normative `PolicyStore`/`PolicyInit` APIs so callers pass a resolved preset or patch, and update the owning specs/tasks to define how setup and runtime callers obtain it.
  - Recommendation: (a), because it preserves POL-04's normative `get/current` signatures and the existing deny-by-default tenant-table boundary; the architect must assign the function contract and migration ownership before implementation.
- **Resolved (maintainer, 2026-10-28):** Q1 — use `resolve_tenant_preset(uuid)` in `0011_core_tenant_preset_resolver.sql` as a SECURITY DEFINER function to return the tenant's existing preset identity to `eq_app`; do not change `PolicyStore`/`PolicyInit` semantics or add a table/column.
- **Self-resolved (implementer, run 6, needs architect ratification):** POL-04 needs only the stored `(preset_id, preset_version)` to select the preset patch, so return those existing fields and keep patch data behind an injected `PresetPatchSource` that checks the stored version; T-301 can implement that boundary without a cross-module SQL join. The delegated resolver authorization detail is recorded separately under TD-047. [TD-046]
- **Self-resolved (implementer, 2026-10-28, needs architect ratification):** the resolver contract in the delegated objective is more specific than the prior tenant-context-guard sketch → use the exact function body, `search_path = pg_catalog, public`, and `eq_app` grant from directive TD-001; the store passes the caller's tenant id and the function returns only the existing preset id/version without granting table access. [TD-047]
- **Self-resolved (implementer, 2026-10-28, needs architect ratification):** T-106 has no migration row in `spec/29` §7, and `0011` is the next unused migration slot between the assigned `0010` and `0012` rows → keep `0011_core_tenant_preset_resolver.sql` because it adds only the delegated function and no schema objects. [TD-048]
- **Self-resolved (implementer, run 6, needs architect ratification):** `jsonschema` 0.30 adds `referencing → fluent-uri → borrow-or-share` with MIT-0, which fails the unchanged `cargo deny` license gate even with default features disabled → use `jsonschema` 0.17.1 with default features off and `draft202012` enabled; it supports the task's required draft and `cargo deny check` passes without adding an exception. [TD-049]
- **Self-resolved (implementer, run 6, needs architect ratification):** the policy audit method's unnamed payload would exceed CQ-101's argument guidance when passed positionally → group module/version/before/after in the typed `PolicyChange` value and redact its document fields in `Debug`; the audit content and transaction behavior remain the task's specified ones. [TD-050]
- **Self-resolved (implementer, run 6, needs architect ratification):** the missing-row fallback returns `Versioned<P>` but no persisted `effective_from` exists → use synthetic version 0 and the caller's requested instant as `effective_from`, because it is the exact time being resolved and avoids inventing a date or returning an out-of-range timestamp (POL-04/05). [TD-051]
- **Self-resolved (implementer, run 6, needs architect ratification):** the required tenant/module cache has no capacity value → bound it to 1,024 keys as a technical memory guard, with no change to policy history or effective-date semantics (POL-04, AGENTS §4.1). [TD-052]
- **Self-resolved (implementer, run 6, needs architect ratification):** AC1 names `policy::tests::effective_dating`, while the database fixture lives in an integration test file → nest the test under `policy::tests` in that file so the required fully qualified test name remains exact without duplicating its database setup. [TD-053]

## Review
n/a until reviewer feedback.
