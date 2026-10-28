---
id: T-105
title: Add the Module trait, registry, dependency validation, ModuleGate and module toggling service
milestone: M1
status: review
depends_on: [T-104]
spec_refs: [MOD-01, MOD-02, MOD-03, MOD-04, MOD-05, MOD-06, MOD-08, ARC-03]
estimated_size: M
---

## Goal
Modules plug in through one trait. Disabled modules are invisible at runtime (404 `module_disabled`). The owner can toggle modules, and dependency rules are enforced on the server.

## Context
The `ModuleId` enum, `ModuleManifest` and the `Module` trait are **normative**. Copy them exactly from `spec/03-module-system.md` §2–§3 (the code block in that section). Summary:
- `ModuleId { Core, Identity, Audit, Scheduling, Enrollment, Billing, Attendance, Rental, Notify, Reports, Feedback }`, serde/sqlx snake_case.
- Manifest fields: `id, name_key, description_key, icon, always_on, requires_all, requires_any, policy_schema_version`.
- Trait methods: manifest, policy_schema, default_policy, validate_policy, validate_policy_in_context (default Ok), has_data (default false), seed_from_preset (default Ok), checklist (default empty), summarize_policy, migrate_policy, routes, event_handlers, jobs, cron, register_ports, permissions, nav, on_enable, on_disable.

Dependency table (MOD-03):

| Module | requires_all | requires_any |
|---|---|---|
| enrollment | scheduling | — |
| billing | — | enrollment, rental |
| attendance | scheduling | — |
| rental | scheduling | — |
| feedback | enrollment | — |
| core/identity/audit/scheduling | always on | |

- MOD-05: `ModuleGate` reads the enabled set from a cache (`moka`, TTL 30 s), invalidated by `NOTIFY eq_modules_changed` with the tenant id as payload. No DB hit on the hot path once cached.
- MOD-06 (service `ModuleService::set_enabled(tx, tenant, actor, requested: BTreeSet<ModuleId>)`):
  1. Requirements are unmet → `AppError::Unprocessable { code: &codes::MODULE_DEPENDENCY, extra: {"missing": {<module>: [<required>…]}} }` → 422 `module_dependency` (ERR-10).
  2. Disabling X while an enabled Y requires X → `AppError::Unprocessable` → 422 `module_required_by` (+ `missing`).
  - Register both codes in `eq_core::error::codes` (the module system lives in core; ERR-21).
  3. For each change: `on_enable`/`on_disable`, audit `core.module.enabled|disabled`, emit `core.module_toggled.v1 {module, enabled}`.
  4. On enable with no policy row: insert v1 = default ⊕ preset patch. **That insert is done by T-106's `PolicyStore::ensure_initial`.** In this task, call a `PolicyInit` trait object and stub it in tests.
  5. `NOTIFY eq_modules_changed`.
- `tenant_modules(tenant_id, module_id, enabled, changed_at, changed_by)`. Always-on modules have no row.
- `requires_any` satisfied means at least one listed module is enabled.

## Scope
### In
- eq_core::module::{ModuleId, ModuleManifest, Module, NavItem, NavSection, PermissionDef, SummaryLine, ChecklistItem, FieldError}.
- `eq_core::module::ModuleRegistry { modules: Vec<Arc<dyn Module>> }` with `validate_graph()` (panics on unknown ids or cycles) and `get(id)`.
- `eq_core::module::gate::{ModuleGate, EnabledModulesCache}` as a tower layer that returns problem `module_disabled` (404, extension `module`).
- `eq_core::module::service::ModuleService` as above, plus a pure domain function `plan_toggle(current, requested, manifests) -> Result<TogglePlan, ToggleError>`.
- **Stub crates** for every module (`eq-identity` … `eq-feedback`). Each gets a `pub struct XModule;` implementing `Module` with empty routes/handlers, `default_policy() = json!({})`, and a correct manifest. Later tasks fill them in.
- `eq-api::modules::registry()` builds from the compiled features (cfg-gated) in enum order.

### Out
- The HTTP endpoint `PUT /settings/modules` and `/meta/modules` (T-113). Policy storage (T-106).

## Files
| Path | Action |
|---|---|
| crates/eq-core/src/app_state.rs, crates/eq-core/src/lib.rs, crates/eq-api/src/lib.rs | move shared AppState to eq-core for the normative Module route type and re-export it from eq-api |
| crates/eq-core/src/module.rs, crates/eq-core/src/module/ | add module identity, manifest, registry, toggle domain, gate, service and repository |
| crates/eq-core/src/ports/ | add the typed PortRegistry and the PolicyInit, AuditSink and EventSink ports |
| crates/eq-core/src/outbox.rs, crates/eq-core/src/jobs.rs | define the handler and cron interfaces consumed by Module |
| crates/eq-core/src/error.rs, crates/eq-core/Cargo.toml | register module error codes and task-required mapped dependencies |
| crates/eq-{identity,audit,scheduling,enrollment,billing,attendance,rental,notify,reports,feedback}/src/lib.rs | add each module's stub type and manifest |
| crates/eq-api/src/modules.rs, crates/eq-api/tests/registry.rs | build and verify the feature-gated registry in enum order |
| crates/eq-api/src/health.rs | merge gated module routes into the app router before the tenant middleware |
| crates/eq-core/tests/module_gate.rs | verify disabled routes, notification invalidation and transactional sink calls |
| crates/eq-testkit/src/module_sinks.rs, crates/eq-testkit/src/lib.rs, crates/eq-testkit/Cargo.toml | provide reusable in-memory policy, audit and event sink doubles |
| Cargo.toml, Cargo.lock, .sqlx/ | pin the architect-resolved OpenAPI stack and record query metadata |
| crates/eq-api/Cargo.toml, crates/eq-{identity,audit,scheduling,enrollment,billing,attendance,rental,notify,reports,feedback}/Cargo.toml | add dependencies needed by the module contracts and stubs |
| this task file, tasks/INDEX.md | report the work and regenerate task status index |

## Business rules
1. The dependency rules come from manifests only. No `match` on module ids in services.
2. Disabling never deletes data (MOD-02).

## Security
- Module identity and enablement are derived only from the compiled registry and the current TenantCtx; never accept tenant IDs from request payloads (SBD-42).
- The enabled-set query uses a tenant-scoped Tx and the RLS-protected tenant_modules table. Cache keys and NOTIFY payloads use the resolved tenant id.
- ModuleService writes tenant_modules, invokes callbacks, records audit data and emits events inside the caller's transaction. Any failure propagates so the caller rolls back.
- Always-on modules cannot be disabled. A disabled module route returns the existing module_disabled 404 without exposing other tenants' module state.
- The API permission check is added by T-113; this task adds no HTTP toggle handler. Test doubles contain only fake data and no secrets or personal data.

## Acceptance criteria
- AC1. `plan_toggle` enabling billing alone, with enrollment and rental off → Err `module_dependency {billing: [enrollment|rental]}`. → `module::domain::tests::billing_needs_enrollment_or_rental`
- AC2. Disabling enrollment while feedback is on → Err `module_required_by [feedback]`. → `disable_required_module_rejected`
- AC3. Property: for any requested set accepted by `plan_toggle`, every enabled module's requirements hold. → `proptest plan_toggle_closure_holds`
- AC4. A route of a disabled module returns 404 problem `module_disabled` with `module: "billing"`. → `gate::tests::disabled_route_404`
- AC5. After `set_enabled` commits, the gate reflects the change within 1 s (NOTIFY). → `gate_invalidated_by_notify`
- AC6. Toggling writes one audit row and one outbox row per changed module. → `service::tests::toggle_audits_and_emits` (uses T-107/T-109 once present; until then assert through the `AuditSink`/`EventSink` test doubles, see Notes)

## Implementation notes
- T-107 (outbox) and T-109 (audit) come later in the ID order but are small. To avoid a cycle, `ModuleService` takes `&dyn AuditSink` and `&dyn EventSink` traits defined in `eq_core::ports::sinks`. T-107/T-109 implement them. This task ships in-memory test doubles in `eq-testkit`.
- The migration for `tenant_modules` is already in T-101.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/module/domain.rs` | `billing_needs_enrollment_or_rental`, `disable_required_module_rejected`, `always_on_cannot_be_disabled`, `plan_toggle_closure_holds` (proptest) |
| `crates/eq-core/src/module/gate.rs` | `cache_miss_racing_invalidation_cannot_reinsert_stale_state` |
| `crates/eq-core/tests/module_gate.rs` | `disabled_route_404`, `tenant_modules_are_isolated_under_rls`, `gate_invalidated_by_notify`, `toggle_audits_and_emits` |
| `crates/eq-api/tests/registry.rs` | `registry_graph_valid`, `registry_order_matches_enum` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass; `scripts/check-deps.sh` passes (stub crates must respect the layering)
- [x] Commit `feat(core): module system and runtime gating [T-105]`, signed off

## Report

**Plan** — (1) Apply the architect-resolved OpenAPI versions (`utoipa` 6 and `utoipa-axum` 0.3), move shared `AppState` into `eq-core`, and re-export it from `eq-api` → verify the crate graph and normative route signature compile. (2) Add the normative module identity/trait and the task-authorized ports and handler contracts, following MOD-03, T-107/T-108 and T-314 → verify the core module contract and layering. (3) Add registry graph validation, pure toggle planning, module stubs and the feature-gated API registry in enum order → verify the named domain and registry tests plus reduced-feature builds. (4) Add tenant-scoped enablement service, audit/event sink ports, tenant cache and NOTIFY-backed gate → verify named route/service tests with disposable PostgreSQL 16 and `eq_app`. (5) Run fresh-database E2E, Rust, `cargo deny`, security, guard and task-index checks → record results and finish the Report.

**Assumptions** — The enable request represents optional modules only; MOD-06 means always-on modules remain implicitly enabled and have no `tenant_modules` rows. The manifest is the sole source of dependency rules. Empty module stubs use an empty JSON object as their policy and schema and have no routes, handlers, jobs, cron entries, permissions or navigation until their owning tasks implement them. The PortRegistry is a type-indexed registry; no module business behavior is added. The architect-resolved `utoipa` 6 / `utoipa-axum` 0.3 stack is authoritative and `deny.toml` remains unchanged.

**Test plan** —
- AC1: Given billing requested with enrollment and rental disabled, when plan_toggle runs, then it returns module_dependency with both alternatives → billing_needs_enrollment_or_rental.
- AC2: Given feedback currently enabled, when enrollment is disabled, then plan_toggle returns module_required_by naming feedback → disable_required_module_rejected.
- AC3: For every generated optional request set accepted by plan_toggle, when the resulting set is inspected, then all enabled modules' requirements hold → plan_toggle_closure_holds.
- AC4: Given a resolved tenant with billing disabled, when a billing-gated request runs, then it returns 404 module_disabled with module=billing → disabled_route_404.
- AC5: Given a warm disabled cache, when ModuleService commits an enable and emits the tenant notification, then the gate observes it within one second → gate_invalidated_by_notify.
- Cache race: Given a cache miss reading old state while a NOTIFY arrives, when invalidation follows the insert, then the stale set is evicted before reuse → cache_miss_racing_invalidation_cannot_reinsert_stale_state.
- AC6: Given a successful change, when ModuleService commits, then exactly one audit and event record are produced per changed module → toggle_audits_and_emits.
- Registry: the dependency graph is valid and compiled modules follow enum order → registry_graph_valid, registry_order_matches_enum.
- Edges: always-on IDs, empty/no-op changes, multiple changes, absent optional modules, dependency chains, cycles, duplicate manifests, malformed notification payload.
- Errors: unmet requirements → module_dependency; reverse dependency → module_required_by; disabled route → module_disabled; database/sink/callback failure aborts the caller's transaction.
- Abuse: request cannot select another tenant's module set; tenant-scoped RLS denies cross-tenant reads/writes; always-on modules remain enabled; cache invalidation only evicts the tenant named by a valid NOTIFY payload.

**Summary** — Implemented the module contracts, dependency planner, tenant-scoped ModuleService, cached ModuleGate with PostgreSQL NOTIFY invalidation, feature-gated registry, ten module stubs, and reusable sink test doubles. The architect-resolved Utoipa 6 / utoipa-axum 0.3 stack is in place; `deny.toml` was not changed. All named acceptance tests pass, the workspace suite passed, and the browser E2E suite passed on disposable PostgreSQL 16.

**Spec read** — `spec/03-module-system.md` §§2–5; `spec/02-architecture.md` ARC-03/07/08/09; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§3–4/7; `spec/18-security-by-design.md` §§2/4/7–8; `spec/19-error-handling.md` §§2/4; `spec/24-open-source-stack.md` §§2/6; `spec/29-erd.md` §§4–5/7. Read T-104's Report and shipped TenantCtx, resolver and request gate before implementation.

**Files** — Implemented the paths listed in Files, including the shared AppState move, module contracts and ports, gate/cache/repository/service, module stubs, API registry tests, sink doubles, and SQLx metadata. Added the serialized cache miss/invalidation lock and the cross-tenant RLS integration test. This report is being corrected in a follow-up documentation commit; no migration, web source, spec, or other task file changed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `billing_needs_enrollment_or_rental` | Passed |
| AC2 | `disable_required_module_rejected` | Passed |
| AC3 | `plan_toggle_closure_holds` | Passed |
| AC4 | `disabled_route_404` | Passed; 404 problem includes `module_disabled` and `module: billing` |
| AC5 | `gate_invalidated_by_notify` | Passed against PostgreSQL 16 |
| AC6 | `toggle_audits_and_emits` | Passed; sink doubles record each changed module |
| Registry | `registry_graph_valid`, `registry_order_matches_enum` | Both passed |
| Tenant isolation (abuse) | `tenant_modules_are_isolated_under_rls` | Passed with an `eq_app` connection; a tenant cannot read the other tenant's module state |
| Cache race | `cache_miss_racing_invalidation_cannot_reinsert_stale_state` | Passed; invalidation is serialized with cache fills so stale state is not reinserted |

**Checks run** —
- ✅ `cargo fmt --all --check`.
- ✅ `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `env RITMA_TEST_DATABASE_URL=postgres://postgres:test-secret-suite@127.0.0.1:32769/postgres SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` — 151 passed, 0 failed: 15 `eq-api`, 126 `eq-core`, 5 `eq-server`, and 5 `eq-testkit` tests. Database tests used disposable PostgreSQL 16 containers.
- ✅ `cargo test -p eq-core --lib cache_miss_racing_invalidation_cannot_reinsert_stale_state` — 1 passed.
- ✅ `RITMA_TEST_DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:32769/postgres SQLX_OFFLINE=true cargo test -p eq-core --test module_gate tenant_modules_are_isolated_under_rls -- --exact` — 1 passed with the non-superuser `eq_app` role.
- ✅ `cargo deny check` — exit 0; advisories, bans, licenses, and sources passed. It reports duplicate `base64` lock entries and the existing unmatched `CC0-1.0` license warning; deny configuration was unchanged.
- ✅ `env DATABASE_URL=postgres://postgres:test-secret-suite@127.0.0.1:32769/ritma_t105_e2e_20261027 SQLX_OFFLINE=false cargo sqlx prepare --workspace --check`.
- ✅ `python3 scripts/erd.py --migrations` and `python3 scripts/erd.py --check` — 15 migrated tables checked; ERD matches the DDL. No migration was changed by T-105.
- ✅ `scripts/tasks-index.sh --check`, `git diff --check`, `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh`.
- ✅ `env E2E_SKIP_DB_START=1 DATABASE_URL=postgres://eq_app:test-secret-app@127.0.0.1:32769/ritma_t105_e2e_20261027 RITMA_DATABASE_OWNER_URL=postgres://postgres:test-secret-suite@127.0.0.1:32769/ritma_t105_e2e_20261027 RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://127.0.0.1:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off RITMA_MODE=single SQLX_OFFLINE=true make e2e` — 6/6 browser smoke tests passed (Chromium and mobile WebKit) against fresh database `ritma_t105_e2e_20261027` in disposable PostgreSQL 16.15 container `ritma-t105-suite-20261027`.
- ✅ `pnpm -C web build` (run by `make e2e`).
- ⚠️ `gitleaks detect --redact --no-banner` reported the pre-existing false positive at `Makefile:80-85` (`curl-auth-user`); no secret rule or allow-list was changed.
- Not run: hosted GitHub Actions require the hosted CI environment. `pnpm -C web lint`, `typecheck`, `test`, and `i18n:check` were not run because T-105 changes no web source; browser E2E and the web build ran.

**Manual checks** — `make e2e` launched the server and exercised real browser requests on Chromium and mobile WebKit against a fresh PostgreSQL 16.15 database. After the run, `SELECT count(*) FROM tenants WHERE id = 'f1050000-0000-7000-8000-000000000001'` returned `0`; the disposable `ritma-t105-suite-20261027` container is stopped. The module isolation integration test used the least-privileged `eq_app` role, and the gate test observed a committed toggle after tenant notification. No standalone manual toggle request applies because T-105 adds no toggle HTTP handler.

**Debug log** — The first E2E attempt stopped before building because Corepack tried to load an unavailable pnpm 12.15.0 module. Activating the task-pinned pnpm 9.15.9 with `corepack prepare pnpm@9.15.9 --activate` fixed the local tool cache. Initial migration starts also reported missing `RITMA_SECRET_KEY`, then missing `RITMA_PUBLIC_URL`; adding the required fake local values allowed the migration and E2E run to complete. Default-parallel workspace tests hit PostgreSQL testcontainer startup timeouts; the final serial workspace command above passed. A planner edge case where `module_required_by` named an unrelated module was fixed and is covered by `required_by_only_reports_removed_dependencies`.

**Performance** — No benchmark was required or taken. Cache TTL is 30 seconds as specified by MOD-05.

**Security** — Tenant identity comes from TenantCtx; module-state SQL is scoped by tenant in the caller's transaction and protected by RLS. Cache invalidation accepts only a valid tenant UUID payload. Toggle callbacks, audit and event writes share the caller transaction, and failure propagates for rollback. Disabled routes return the module-specific 404 without exposing another tenant's state. Test data is synthetic. No new permission, external service, secret, or PII field was introduced. `cargo deny check` passed without an advisory exception or gate change.

**Deviations** — Applied D-56's architect resolution: workspace `utoipa` 6 and `utoipa-axum` 0.3; `deny.toml` stayed unchanged. Implementer resolutions requiring architect ratification: (1) move shared AppState to eq-core and derive ordering for ModuleId to preserve the normative route signature and requested BTreeSet API; (2) add the omitted Files/Security paths and controls already required by the Scope and cited architecture/security specs; (3) use the existing T-107/T-108/T-314 handler/checklist contracts, ARC-08 typed ports, and T-113's more precise `required_by` extension. These choices follow the higher-priority decision/spec and preserve the acceptance contract. Because the original task commit was already pushed and must not be rewritten, this factual Report correction is recorded in a separate signed documentation commit; the original commit also contains an extra `Co-authored-by` line, which is retained for human review rather than rewriting pushed history. Decision register references: TD-038, TD-039, TD-040 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).

**Dependencies affected** — T-104 is `review` and its commit is an ancestor of this branch. T-106 and later dependants may proceed after T-105 is recorded as `review`.

**Rollback plan** — Revert the T-105 commit; no migration or persistent data change was added.

**Follow-ups** — Architect ratification remains requested for the three implementer resolutions above. The dependency question is resolved by D-56; no blocking question remains. Human review should assess the implementation size (2,478 insertions and 109 deletions across 50 files, including Cargo.lock, generated SQLx metadata and module stubs) against CQ-101 and decide whether to accept the extra `Co-authored-by` line in the already-pushed implementation commit. No history rewrite is recommended. The repository's existing gitleaks false positive at `Makefile:80-85` remains for maintainer review.

**Step 0 — Start** — Resumed T-105 on `milestone/m1-core`; verified its dependency commit, current branch, task status, and generated index.

**Step 1 — Task and specs** — Re-read T-105 and the cited module, architecture, code-quality, clean-architecture, security, error, stack, and ERD sections.

**Step 2 — Dependencies** — Read T-104's Report and verified the shipped TenantCtx, resolver, request gate, and dependency branch state.

**Step 3 — Design reference** — n/a; backend task.

**Step 4 — Plan and test plan** — Recorded above before final verification.

**Step 5 — Implement** — Added the listed module interfaces, stubs, planner, service, gate/cache, registry and sink doubles. Added named acceptance tests and a cache invalidation race regression test. Applied D-56's OpenAPI dependency versions.

**Step 6 — Run locally** — Ran the complete workspace suite on PostgreSQL 16, Makefile lint gates, SQLx metadata preparation, ERD/index/guard checks, and browser E2E.

**Step 7 — Debug** — Repaired the local Corepack activation, removed the unfulfilled lint expectation, and fixed/covered the planner's `required_by` edge case; details are above.

**Step 8 — Self-review** — Confirmed server-derived tenant identity, tenant-scoped parameterized SQL, transaction propagation, cache invalidation locking, registry feature ordering, and absence of new `#[allow]`, `todo!`, `dbg!`, or real user data.

**Step 9 — Optimize** — n/a; no benchmark was required.

**Step 10 — Security audit** — Reviewed tenant/module controls and ran the dependency/security gate; `cargo deny check` passed with the existing warnings noted above.

**Step 11 — Final verification** — `cargo fmt --all --check`, strict workspace Clippy, serial workspace tests (151/151), targeted cache-race and RLS tests, `cargo deny check`, SQLx metadata check, ERD and task-index checks, guard scripts, and fresh-database browser E2E (6/6) passed as listed above. Hosted GitHub Actions and the web lint/typecheck/unit/i18n suites were not run for the reasons stated above. The gitleaks scan retained its existing false positive.

**Step 12 — Report** — Complete; ready for review.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification):** The normative Module trait is in eq-core but its route signature needs AppState, which T-004 placed in eq-api, and T-105 requires BTreeSet<ModuleId> while the normative derives omit Ord → move the shared AppState definition to eq-core and re-export it from eq-api, and derive Ord/PartialOrd on ModuleId, preserving the normative route signature and required set API while keeping the dependency direction. [TD-038]
- **Self-resolved (implementer, run 8, needs architect ratification):** T-105 omits Files and Security sections despite requiring module state changes, tenant-scoped caching and source/test additions → add the exact paths from Scope/Tests and the controls from MOD-01/06, CA-30/31 and SBD-42/73/81. [TD-039]
- **Self-resolved (implementer, run 8, needs architect ratification):** The Module signature refers to ChecklistItem, EventHandler, JobHandler, CronSpec, PortRegistry and sink interfaces without declaring all their shapes; T-107/T-108 define event and job handlers, T-314 defines checklist fields, and ARC-08 defines typed port registration → use those contracts for the scaffolding, keep preset resolution behind PolicyInit, and use T-113's exact `required_by` response extension because its API shape is more precise than T-105's `(+ missing)` note. [TD-040]
- **Resolved (architect, 2026-10-27):** Q1 — option (b): the OpenAPI stack moves to `utoipa` 6 with `utoipa-axum` 0.3 (`utoipa-scalar` 0.4 when T-114 needs it). `utoipa-axum` 0.3 depends on `pastey`, not the unmaintained `paste`; probed with the repository `deny.toml` (advisories, bans, licenses pass), gate unchanged. The toolchain pin (1.99.1) satisfies their MSRV 1.88. Spec 02 and 24 are updated; the `OpenApiRouter<AppState>` type in spec 03 §3 is unchanged. In the workspace `Cargo.toml` set `utoipa = { version = "6", features = ["axum_extras", "uuid", "chrono"] }` and `utoipa-axum = "0.3"`. If utoipa 6 renamed a feature or macro attribute, adapt mechanically and record it as a RATIFY item; do not change the route signature or any listed test name. T-110 and T-114 only use `#[utoipa::path]` and `OpenApiRouter`, so their text does not change.
- No blocking questions remain; D-56 resolved the former OpenAPI dependency question recorded above.
