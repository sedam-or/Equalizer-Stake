---
id: T-105
title: Add the Module trait, registry, dependency validation, ModuleGate and module toggling service
milestone: M1
status: blocked
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
| crates/eq-core/src/module/ | add module identity, manifest, registry, toggle domain, gate and service |
| crates/eq-core/src/ports/ | add the typed PortRegistry and the PolicyInit, AuditSink and EventSink ports |
| crates/eq-core/src/outbox.rs, crates/eq-core/src/jobs.rs | define the handler and cron interfaces consumed by Module |
| crates/eq-core/src/error.rs, crates/eq-core/Cargo.toml | register module error codes and task-required mapped dependencies |
| crates/eq-{identity,audit,scheduling,enrollment,billing,attendance,rental,notify,reports,feedback}/src/lib.rs | add each module's stub type and manifest |
| crates/eq-api/src/modules.rs, crates/eq-api/tests/registry.rs | build and verify the feature-gated registry in enum order |
| crates/eq-core/tests/module_gate.rs | verify disabled routes, notification invalidation and transactional sink calls |
| crates/eq-testkit/src/module_sinks.rs, crates/eq-testkit/src/lib.rs, crates/eq-testkit/Cargo.toml | provide reusable in-memory policy, audit and event sink doubles |
| Cargo.lock, this task file, tasks/INDEX.md | record dependencies, report the work and regenerate task status index |

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
| `crates/eq-core/tests/module_gate.rs` | `disabled_route_404`, `gate_invalidated_by_notify`, `toggle_audits_and_emits` |
| `crates/eq-api/tests/registry.rs` | `registry_graph_valid`, `registry_order_matches_enum` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `scripts/check-deps.sh` passes (stub crates must respect the layering)
- [ ] Commit `feat(core): module system and runtime gating [T-105]`, signed off

## Report

**Plan** — (1) Align the normative module trait with the existing API state without creating a crate cycle, and add the required core ports and handler interfaces → verify workspace layering and compilation. (2) Implement ModuleId/manifests, registry graph validation, toggle planning and the exact MOD-03 dependencies → verify all named domain tests and property cases. (3) Add tenant-scoped enabled-module cache, PostgreSQL notification invalidation, ModuleGate and transactional ModuleService → verify route, cache, rollback and sink behavior against PostgreSQL 16. (4) Add feature-gated module stubs and the eq-api registry → verify enum order in default and reduced-feature builds. (5) Run runtime checks, security review, Rust tests, guards, and E2E where this phase requires it.

**Assumptions** — The enable request represents optional modules only; the always-on set is derived from ModuleManifest and remains implicitly enabled. Empty module stubs use an empty JSON object as their policy and schema and have no routes, handlers, jobs, cron entries, permissions or navigation until their owning tasks implement them. The PortRegistry is a type-indexed registry; no module business behavior is added.

**Test plan** —
- AC1: Given billing requested with enrollment and rental disabled, when plan_toggle runs, then it returns module_dependency with both alternatives → billing_needs_enrollment_or_rental.
- AC2: Given feedback currently enabled, when enrollment is disabled, then plan_toggle returns module_required_by naming feedback → disable_required_module_rejected.
- AC3: For every generated optional request set accepted by plan_toggle, when the resulting set is inspected, then all enabled modules' requirements hold → plan_toggle_closure_holds.
- AC4: Given a resolved tenant with billing disabled, when a billing-gated request runs, then it returns 404 module_disabled with module=billing → disabled_route_404.
- AC5: Given a warm disabled cache, when ModuleService commits an enable and emits the tenant notification, then the gate observes it within one second → gate_invalidated_by_notify.
- AC6: Given a successful change, when ModuleService commits, then exactly one audit and event record are produced per changed module → toggle_audits_and_emits.
- Registry: the dependency graph is valid and compiled modules follow enum order → registry_graph_valid, registry_order_matches_enum.
- Edges: always-on IDs, empty/no-op changes, multiple changes, absent optional modules, dependency chains, cycles, duplicate manifests, malformed notification payload.
- Errors: unmet requirements → module_dependency; reverse dependency → module_required_by; disabled route → module_disabled; database/sink/callback failure aborts the caller's transaction.
- Abuse: request cannot select another tenant's module set; tenant-scoped RLS denies cross-tenant reads/writes; always-on modules remain enabled; cache invalidation only evicts the tenant named by a valid NOTIFY payload.

**Summary** — A complete prototype of the module registry, toggle domain, tenant-scoped gate/cache, module service, ports, handler contracts and stubs compiled and passed the task tests. The implementation is not shipped: `cargo deny check` fails on the unmaintained `paste` transitive dependency required by the spec-mandated `utoipa-axum` 0.2 / Utoipa 5 route type, and available upstream `utoipa-axum` 0.3 requires Utoipa 6, contrary to OSS-02. The prototype code and generated SQLx metadata were reverted; only this blocked task record and the generated task index remain changed.

**Spec read** — `spec/03-module-system.md` §§2–5; `spec/02-architecture.md` ARC-03/07/08/09; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§3–4/7; `spec/18-security-by-design.md` §§2/4/7–8; `spec/19-error-handling.md` §§2/4; `spec/24-open-source-stack.md` §§2/6; `spec/29-erd.md` §§4–5/7. Read T-104's Report and shipped TenantCtx, resolver and request gate before implementation.

**Files** — Prototype paths within the task's Files list were implemented and verified, then reverted after the dependency/security conflict was confirmed. No implementation, test or SQLx metadata file is included in the blocked commit. This task file and `tasks/INDEX.md` record the block.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `billing_needs_enrollment_or_rental` | Passed on the verified prototype; not shipped. |
| AC2 | `disable_required_module_rejected` | Passed on the verified prototype; not shipped. |
| AC3 | `plan_toggle_closure_holds` | Passed on the verified prototype; not shipped. |
| AC4 | `disabled_route_404` | Passed on the verified prototype; not shipped. |
| AC5 | `gate_invalidated_by_notify` | Passed on the verified prototype; not shipped. |
| AC6 | `toggle_audits_and_emits` | Passed on the verified prototype; not shipped. |
| Registry | `registry_graph_valid`, `registry_order_matches_enum` | Both passed on the verified prototype; not shipped. |

**Checks run** —
- ✅ `scripts/tasks-index.sh --check`, `git diff --check`, `cargo fmt --all --check`.
- ✅ `DATABASE_URL=postgres://eq_app:eq_app@localhost:5433/ritma SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --all-targets`; generated metadata matched the prototype queries and was reverted with the blocked implementation.
- ✅ `cargo clippy --workspace --all-targets -- -D warnings` after removing an unfulfilled `#[expect(clippy::panic)]` on `assert!`-based registry checks.
- ✅ `DATABASE_URL=postgres://eq_app:eq_app@localhost:5433/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:eq@localhost:5433/ritma cargo test --workspace`; all unit, API, database integration, CLI and doc tests passed, including the three T-105 integration tests.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 scripts/erd.py --migrations && python3 scripts/erd.py --check`; no migration changed, and both checks passed.
- ❌ `cargo deny check`; blocked by RUSTSEC-2024-0436 (`paste 1.0.15`, unmaintained) pulled by `utoipa-axum 0.2.0`. No advisory exception or security-gate change was made.
- ⚠ `cargo audit`; exit 0 with the same unmaintained `paste` advisory reported.
- ✅ `make e2e`; after inserting one disposable fake tenant into the empty local dev database, all 6 Chromium/mobile WebKit smoke checks passed. The fixture was deleted and the database returned to zero tenant rows.
- Not run: hosted GitHub Actions, which requires the hosted CI environment. Web lint/typecheck/test and i18n checks are n/a because this task changes no web source; `i18n:check` is owned by T-306.
- ✅ `gitleaks protect --staged --redact`; staged task/index documentation contains no secrets.

**Manual checks** — The browser E2E suite exercised real HTTP requests for `/`, `/healthz` and SPA fallback. On an empty database, the first run received 503 `setup_required` for the SPA routes, matching T-104's unprovisioned-tenant rule; after seeding a temporary fake tenant, all routes returned the expected responses and the suite passed. The module gate and toggle paths were exercised against PostgreSQL 16 by the three task integration tests using the `eq_app` role. No HTTP toggle endpoint exists in this task; T-113 owns it.

**Debug log** — First clippy run found the stale `#[expect(clippy::panic)]` was unfulfilled because the registry uses `assert!`, not `panic!`; removing that unnecessary expectation made clippy pass. First E2E run had 4 failures because the local database had no tenant and T-104 correctly returned 503; seeding a disposable tenant made all 6 checks pass, after which the row was removed. `cargo deny check` consistently rejects the transitive unmaintained dependency; the available 0.3.0 upstream release depends on Utoipa 6, which conflicts with the project's Utoipa 5 stack, so no compliant implementation change was available.

**Performance** — n/a; no benchmark was taken. Cache TTL follows MOD-05 at 30 seconds and is a named technical constant.

**Security** — Triggers: tenant isolation, state mutation and module enablement (SBD-30). The prototype derived tenant identity from `TenantCtx`, scoped enabled-set SQL by `tenant_id` inside the caller's tenant transaction, used RLS with an `eq_app` test connection, kept notification invalidation tenant-specific, and tested disabled-route behavior plus transactional audit/event sinks. No secrets, PII fields, new permissions or external calls were introduced. `cargo deny check` failed on the transitive unmaintained dependency; no allow-list, lint, test or security control was weakened. The security conflict prevents review status.

**Deviations** — RATIFY: (1) moved `AppState` into eq-core and derived ordering for `ModuleId`; (2) added omitted Files/Security sections from the stated scope and security specs; (3) used shipped T-107/T-108/T-314 interfaces and T-113's exact `required_by` extension. The prototype was reverted because the spec-mapped OpenAPI dependency combination cannot pass the current advisory gate without an architectural/spec decision.

**Dependencies affected** — None shipped. T-106, T-107 and later dependants remain ineligible until T-105 is resolved and reviewed.

**Rollback plan** — No implementation or migration is committed. Reverting this task-record commit restores the task's prior in-progress state; an architect resolution must provide a supported dependency combination before implementation resumes.

**Follow-ups** — Architect decision required on the T-105 dependency question below: keep Utoipa 5 and provide a compatible maintained `utoipa-axum` dependency, or revise the mapped OpenAPI stack. Do not add an advisory exception. Naming validation maps the former product/display identity to **Ritma**. The active UI, CLI, binary, config namespace and repository description use Ritma; the remaining 18 old-name literals in 11 files are six current GitHub slug locators, six explicit historical/provenance references, and six earlier task audit notes. `gh repo view` confirms the live slug is still `sedam-or/Equalizer-Stake`; updating those URLs before an actual repository rename would break them. No repository setting or unrelated task/spec/brand file was changed.

**Step 0 — Start** — Resumed the existing T-105 `in_progress` work on `milestone/m1-core` at `01fb5fc`; the previous session's prototype was present. Re-read `RUN_LOG.md`, task index, branch tip and status; `scripts/tasks-index.sh --check` and `git diff --check` passed before changes.

**Step 1 — Task and specs** — Read T-105 end to end and the cited MOD/ARC contracts plus CQ, CA, SBD, ERR, OSS and ERD requirements. Confirmed no migration is in scope and `scripts/erd.py --migrations` is not needed for a schema change (both ERD commands were still run).

**Step 2 — Dependencies** — Read T-104's Report and its shipped tenant resolver, request gate and integration tests. The `TenantCtx` interface matched the current implementation and exposed the unprovisioned behavior that the E2E fixture needed.

**Step 3 — Design reference** — n/a; backend task.

**Step 4 — Plan and test plan** — Written above before this resumed implementation.

**Step 5 — Implement** — Prototyped all listed module contracts, registry, dependency planner, gate/cache, service, stubs, API registry and tests. Removed the stale lint expectation found during verification. Reverted all prototype files and lock/SQLx changes when the security dependency conflict prevented an honest `review` result.

**Step 6 — Run locally** — Ran workspace tests against PostgreSQL 16 and `make e2e` against the release binary with a temporary tenant fixture. The task has no module toggle HTTP endpoint; T-113 adds that endpoint.

**Step 7 — Debug** — See Debug log; each failure had a distinct diagnosis and an alternative test setup where applicable.

**Step 8 — Self-review** — Reviewed the prototype against CQ-16, CA-30/31, ERR-21 and SBD-81. Tenant queries included explicit tenant scope, SQL was parameterized, audit/events shared the caller's transaction, and no forbidden `#[allow]`, `todo!`, `dbg!` or plaintext secrets were introduced. The implementation was reverted after the dependency gate failure.

**Step 9 — Optimize** — n/a; no measured performance change.

**Step 10 — Security audit** — Completed the task's tenant/module controls review and ran `cargo audit`/`cargo deny check`. The advisory finding is the blocking issue recorded below.

**Step 11 — Final verification** — Ran the commands listed under Checks run. All code, test, format and guard checks passed on the prototype; the mandatory dependency advisory gate did not.

**Step 12 — Report** — Blocked per WORKFLOW §4. The prototype was reverted; status and index were set to `blocked`. No DoD box is checked.

## Questions
- **Self-resolved (implementer, run 8, needs architect ratification):** The normative Module trait is in eq-core but its route signature needs AppState, which T-004 placed in eq-api, and T-105 requires BTreeSet<ModuleId> while the normative derives omit Ord → move the shared AppState definition to eq-core and re-export it from eq-api, and derive Ord/PartialOrd on ModuleId, preserving the normative route signature and required set API while keeping the dependency direction.
- **Self-resolved (implementer, run 8, needs architect ratification):** T-105 omits Files and Security sections despite requiring module state changes, tenant-scoped caching and source/test additions → add the exact paths from Scope/Tests and the controls from MOD-01/06, CA-30/31 and SBD-42/73/81.
- **Self-resolved (implementer, run 8, needs architect ratification):** The Module signature refers to ChecklistItem, EventHandler, JobHandler, CronSpec, PortRegistry and sink interfaces without declaring all their shapes; T-107/T-108 define event and job handlers, T-314 defines checklist fields, and ARC-08 defines typed port registration → use those contracts for the scaffolding, keep preset resolution behind PolicyInit, and use T-113's exact `required_by` response extension because its API shape is more precise than T-105's `(+ missing)` note.
- **Q1 (blocking):** How can T-105 keep the normative `utoipa-axum` route type and Utoipa 5 stack while `cargo deny check` passes, when `utoipa-axum 0.2.0` pulls in unmaintained `paste 1.0.15` (RUSTSEC-2024-0436) and `utoipa-axum 0.3.0` requires Utoipa 6?
  - Context: `spec/03-module-system.md` §3 specifies `utoipa_axum::router::OpenApiRouter<AppState>`; `spec/24-open-source-stack.md` §2 maps Utoipa 5 and `utoipa-axum`; SBD-75 and the current `cargo-deny` gate reject the transitive unmaintained crate. `cargo info utoipa-axum@0.3.0` confirms that release requires Utoipa 6 and `pastey`, conflicting with the mapped Utoipa 5 version.
  - Options: (a) keep Utoipa 5 and wait for or approve a maintained compatible `utoipa-axum` release; (b) architect revises OSS-02 and the module route contract to a compatible Utoipa 6 stack; (c) add an advisory exception, which weakens the required security gate and is not permitted.
  - Recommendation: (a), or (b) only through an explicit spec update; keep the advisory gate intact and do not implement option (c).
