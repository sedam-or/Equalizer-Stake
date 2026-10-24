---
id: T-105
title: Add the Module trait, registry, dependency validation, ModuleGate and module toggling service
milestone: M1
status: todo
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
- Trait methods: `manifest, policy_schema, default_policy, validate_policy, validate_policy_in_context (default Ok), has_data (default false), seed_from_preset (default Ok), summarize_policy, migrate_policy, routes, event_handlers, jobs, cron, register_ports, permissions, nav, on_enable, on_disable`.

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
  1. Requirements are unmet → `AppError::Conflict{code:"module_dependency"}` (HTTP 422 per spec; map with `status_override`) listing missing modules per requested module.
  2. Disabling X while an enabled Y requires X → 422 `module_required_by`.
  3. For each change: `on_enable`/`on_disable`, audit `core.module.enabled|disabled`, emit `core.module_toggled.v1 {module, enabled}`.
  4. On enable with no policy row: insert v1 = default ⊕ preset patch. **That insert is done by T-106's `PolicyStore::ensure_initial`.** In this task, call a `PolicyInit` trait object and stub it in tests.
  5. `NOTIFY eq_modules_changed`.
- `tenant_modules(tenant_id, module_id, enabled, changed_at, changed_by)`. Always-on modules have no row.
- `requires_any` satisfied means at least one listed module is enabled.

## Scope
### In
- `eq_core::module::{ModuleId, ModuleManifest, Module, NavItem, NavSection, PermissionDef, SummaryLine, FieldError}`.
- `eq_core::module::ModuleRegistry { modules: Vec<Arc<dyn Module>> }` with `validate_graph()` (panics on unknown ids or cycles) and `get(id)`.
- `eq_core::module::gate::{ModuleGate, EnabledModulesCache}` as a tower layer that returns problem `module_disabled` (404, extension `module`).
- `eq_core::module::service::ModuleService` as above, plus a pure domain function `plan_toggle(current, requested, manifests) -> Result<TogglePlan, ToggleError>`.
- **Stub crates** for every module (`eq-identity` … `eq-feedback`). Each gets a `pub struct XModule;` implementing `Module` with empty routes/handlers, `default_policy() = json!({})`, and a correct manifest. Later tasks fill them in.
- `eq-api::modules::registry()` builds from the compiled features (cfg-gated) in enum order.

### Out
- The HTTP endpoint `PUT /settings/modules` and `/meta/modules` (T-113). Policy storage (T-106).

## Business rules
1. The dependency rules come from manifests only. No `match` on module ids in services.
2. Disabling never deletes data (MOD-02).

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

## Questions
