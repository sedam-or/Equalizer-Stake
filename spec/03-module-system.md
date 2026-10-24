# 03 — Module System

## 1. Two gates

**MOD-01.** A module is available to a tenant only if it passes **both** gates:

| Gate | Where | Who decides | Effect when off |
|---|---|---|---|
| **Compile-time** | Cargo feature on `eq-api` / `eq-server` (`enrollment`, `billing`, `attendance`, `rental`, `notify`, `reports`, `feedback`) | Whoever builds the binary. The default build includes **all** features | The code isn't in the binary. `/meta/modules` reports it as `compiled: false` |
| **Runtime** | `tenant_modules` row | Owner, via setup step 2 or Settings → Features | Routes return 404 `module-disabled`, jobs and event handlers skip the tenant, and the SPA hides nav and widgets |

The official release ships with all features. Compile-time removal is for embedded or minimal builds (e.g. `--no-default-features --features rental` produces a studio-rental-only binary).

**MOD-02.** The database schema is always complete, whatever features are compiled (ADR-0012). Turning a module off never drops data. Turning it back on restores everything.

## 2. Module identity and catalog

```rust
// eq-core::module
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ModuleId {
    Core, Identity, Audit, Scheduling,           // always on
    Enrollment, Billing, Attendance, Rental, Notify, Reports, Feedback,
}
```

**MOD-03.** Dependency rules (closed set, v1):

| Module | `requires_all` | `requires_any` | Always on |
|---|---|---|---|
| core, identity, audit, scheduling | — | — | ✅ |
| enrollment | scheduling | — | |
| billing | — | enrollment, rental | |
| attendance | scheduling | — | |
| rental | scheduling | — | |
| notify | — | — | |
| reports | — | — | |
| feedback | enrollment | — | |

## 3. The `Module` trait (normative signature)

```rust
// eq-core::module
pub struct ModuleManifest {
    pub id: ModuleId,
    pub name_key: &'static str,          // i18n key, e.g. "module.billing.name"
    pub description_key: &'static str,   // i18n key
    pub icon: &'static str,              // lucide icon name for the SPA card
    pub always_on: bool,
    pub requires_all: &'static [ModuleId],
    pub requires_any: &'static [ModuleId],
    pub policy_schema_version: u32,      // bump on breaking policy struct change (POL-08)
}

#[async_trait]
pub trait Module: Send + Sync + 'static {
    fn manifest(&self) -> &'static ModuleManifest;

    /// JSON Schema (draft 2020-12) of this module's policy document, with UI hints (POL-06).
    fn policy_schema(&self) -> serde_json::Value;

    /// The module's built-in default policy document (= serde_json::to_value(XPolicy::default())).
    /// Preset overrides are deep-merged on top of this by core::setup (POL-09), never here.
    fn default_policy(&self) -> serde_json::Value;

    /// Validate a full policy document (schema + cross-field rules). Returns field errors.
    fn validate_policy(&self, doc: &serde_json::Value) -> Result<(), Vec<FieldError>>;

    /// Checks that need data (e.g. an offering key must exist in the catalog). Default: Ok.
    async fn validate_policy_in_context(&self, tx: &mut Tx, tenant: TenantId, doc: &serde_json::Value) -> Result<(), Vec<FieldError>> { Ok(()) }

    /// True if the tenant has data owned by this module (drives the "Has data" chip, MOD-07). Default: false.
    async fn has_data(&self, tx: &mut Tx, tenant: TenantId) -> anyhow::Result<bool> { Ok(false) }

    /// Seed this module's part of a preset `catalog` section (e.g. rental rate cards) during setup step 1. Default: Ok.
    async fn seed_from_preset(&self, tx: &mut Tx, tenant: TenantId, catalog: &serde_json::Value) -> anyhow::Result<()> { Ok(()) }

    /// Items for the post-setup checklist (05 §2.5), computed from current data. Default: none.
    async fn checklist(&self, tx: &mut Tx, tenant: TenantId) -> anyhow::Result<Vec<ChecklistItem>> { Ok(vec![]) }

    /// Plain-language summary of the policy (PRD-06) as i18n keys + args; the SPA renders
    /// them with ICU MessageFormat (POL-07). Server never builds human sentences itself.
    fn summarize_policy(&self, doc: &serde_json::Value) -> Vec<SummaryLine>;

    /// Migrate a stored policy document from an older schema version (POL-08).
    fn migrate_policy(&self, from_version: u32, doc: serde_json::Value) -> anyhow::Result<serde_json::Value>;

    /// HTTP routes (already prefixed by /api/v1 by eq-api). Wrapped in ModuleGate by the registry.
    fn routes(&self) -> utoipa_axum::router::OpenApiRouter<AppState>;

    /// Event handlers this module subscribes to.
    fn event_handlers(&self) -> Vec<Box<dyn EventHandler>>;

    /// Job handlers + cron specs.
    fn jobs(&self) -> Vec<Box<dyn JobHandler>>;
    fn cron(&self) -> Vec<CronSpec>;

    /// Ports this module implements (ARC-08).
    fn register_ports(&self, ports: &mut PortRegistry);

    /// Permissions this module defines, with default role grants (IDN-10).
    fn permissions(&self) -> &'static [PermissionDef];

    /// Navigation entries for the SPA, filtered by permission at runtime.
    fn nav(&self) -> &'static [NavItem];

    /// Called when a tenant enables / disables this module (MOD-06). Must be idempotent.
    async fn on_enable(&self, tx: &mut Tx, tenant: TenantId) -> anyhow::Result<()> { Ok(()) }
    async fn on_disable(&self, tx: &mut Tx, tenant: TenantId) -> anyhow::Result<()> { Ok(()) }
}
```

`NavItem { id, label_key, icon, path, perm, section: NavSection::{Main, Admin, Settings}, order }`.

## 4. Registry and wiring

**MOD-04.** `eq-api::modules::registry()` builds `Vec<Arc<dyn Module>>` from the compiled features, in a fixed order (the `ModuleId` enum order). At startup the server does the following:
1. Validates that the dependency graph has no unknown ids or cycles. A failure panics with a clear message.
2. Builds the axum router. Each module's routes are wrapped with `ModuleGate(id)`.
3. Registers event handlers with the dispatcher and job handlers and cron specs with the job runner.
4. Calls `register_ports`.
5. Builds the merged OpenAPI document. **All compiled modules appear in OpenAPI**, whatever their runtime state.

**MOD-05.** `ModuleGate` reads the tenant's enabled set from an in-memory cache (`moka` or `DashMap` keyed by `tenant_id`, TTL 30 s, invalidated on change via `NOTIFY eq_modules_changed`). It never queries the DB on the hot path.

## 5. Enabling and disabling (runtime)

**MOD-06.** Table `tenant_modules(tenant_id, module_id, enabled bool, changed_at, changed_by)`. Always-on modules have no row and are implicitly enabled.

`PUT /api/v1/settings/modules` takes `{ "enabled": ["enrollment","billing",...] }` and does the following:
1. Computes the closure: if a module is requested and its requirements aren't met, the request fails with 422 `module-dependency`. The error lists the missing requirements. The server does **not** auto-enable dependencies; the SPA does that interactively (MOD-07).
2. Computes the reverse check: disabling X while an enabled Y requires X fails with 422 `module-required-by` and the list of Y.
3. For each changed module, it calls `on_enable` / `on_disable` in one tx, writes audit `module.enabled` / `module.disabled`, and emits `core.module_toggled.v1`.
4. On enable, if the tenant has **no** policy row for that module, it inserts version 1 from `default_policy()` merged with the tenant's preset overrides (`tenants.preset_id`).

**MOD-07.** SPA toggle behaviour (setup step 2 and Settings → Features):
- Turning on a module whose `requires_all` are off asks: "Billing needs Enrollment or Rental. Turn on Enrollment too?" with buttons [Turn on both] [Cancel]. For `requires_any` it offers the first option in the list.
- Turning off a module that others need asks: "Turning off Enrollment also turns off Billing and Feedback." with buttons [Turn off all] [Cancel].
- Always-on modules render as locked cards with a lock icon and the tooltip "Always included".
- A module with data shows a "Has data" chip when off. The confirm dialog when turning it off says: "Your data is kept. Turn it back on any time."

**MOD-08.** `on_disable` effects (normative):

| Module | On disable |
|---|---|
| enrollment | Recurring session materialisation stops. Existing future sessions are kept, and a banner tells the admin to review them |
| billing | Cron jobs skip the tenant. Frozen payers are **unfrozen** automatically (audit reason `module-disabled`). Open invoices remain readable via a read-only route |
| attendance | Check-in is no longer required. Pending reviews remain visible |
| rental | New reservations are blocked. Confirmed future reservations remain (and keep their holds) |
| notify | Outgoing messages are suppressed and logged as `suppressed` |
| reports | — |
| feedback | — |

**MOD-09.** `GET /api/v1/meta/modules` (any authenticated user) returns the following, which drives every module-aware part of the SPA:

```json
{
  "modules": [
    { "id": "billing", "compiled": true, "enabled": true, "always_on": false,
      "requires_all": [], "requires_any": ["enrollment","rental"],
      "has_data": true, "policy_version": 3 }
  ],
  "nav": [ { "id": "billing.invoices", "label_key": "nav.invoices", "icon": "receipt", "path": "/billing/invoices", "section": "main", "order": 40 } ]
}
```

`nav` is already filtered by the caller's permissions and the enabled modules.

## 6. Event catalog (v1)

Payloads are the minimum needed to act. Consumers re-read state through ports if they need more. All payloads include `tenant_id` and `occurred_at` in the envelope.

| Event type | Emitted by | Payload | Consumed by |
|---|---|---|---|
| `core.module_toggled.v1` | core | `{module, enabled}` | notify (admin notice), reports (cache bust) |
| `core.policy_changed.v1` | core | `{module, version, effective_from}` | owning module (recompute caches), notify |
| `identity.user_invited.v1` | identity | `{person_id, role, invite_id}` | notify (send invite) |
| `identity.account_locked.v1` | identity | `{person_id, until}` | notify (admin alert) |
| `identity.password_reset_requested.v1` | identity | `{person_id, token_id}` | notify |
| `scheduling.session_scheduled.v1` | scheduling | `{session_id, offering_id, starts_at, instructor_ids, participant_ids}` | notify (instructor), attendance |
| `scheduling.session_rescheduled.v1` | scheduling | `{session_id, old_range, new_range, reason}` | notify |
| `scheduling.session_cancelled.v1` | scheduling | `{session_id, cancelled_by_role, reason, hours_before, participants: [{person_id, credit_bps}]}` | enrollment (make-up credit per policy), notify |
| `scheduling.session_reverted.v1` | scheduling | `{session_id, participant_ids}` | enrollment (void credits granted by the cancellation), notify |
| `scheduling.occurrence_skipped.v1` | scheduling | `{series_id, occurrence_date, reason: closure\|no_slot\|extra_occurrence, person_ids, credit_bps}` | enrollment (credit if > 0), notify |
| `scheduling.session_completed.v1` | scheduling | `{session_id, participant_ids}` | attendance (finalise), reports |
| `enrollment.registration_submitted.v1` | enrollment | `{registration_id}` | notify (admin) |
| `enrollment.enrollment_activated.v1` | enrollment | `{enrollment_id, student_id, payer_id, offering_id, starts_on}` | billing (registration fee line), notify |
| `enrollment.enrollment_on_leave.v1` | enrollment | `{enrollment_id, from, until}` | billing (suspend lines), scheduling (cancel future sessions in range) |
| `enrollment.enrollment_ended.v1` | enrollment | `{enrollment_id, reason}` | billing, scheduling (cancel future) |
| `billing.invoice_issued.v1` | billing | `{invoice_id, payer_id, total_minor, currency, due_at}` | notify |
| `billing.payment_recorded.v1` | billing | `{payment_id, invoice_id, amount_minor}` | notify |
| `billing.invoice_paid.v1` | billing | `{invoice_id, payer_id}` | notify, identity (unfreeze if auto) |
| `billing.invoice_overdue.v1` | billing | `{invoice_id, payer_id, late_fee_minor}` | notify |
| `billing.payer_frozen.v1` | billing | `{payer_id, reason, invoice_ids}` | notify, scheduling (block), rental (block) |
| `billing.payer_unfrozen.v1` | billing | `{payer_id, by}` | notify, scheduling, rental |
| `attendance.checkin_recorded.v1` | attendance | `{checkin_id, session_id, status}` | notify (admin if pending) |
| `attendance.checkin_reviewed.v1` | attendance | `{checkin_id, decision}` | notify (instructor) |
| `attendance.review_overdue.v1` | attendance | `{checkin_id, age_minutes}` | notify (escalate to owner) |
| `attendance.no_show_recorded.v1` | attendance | `{session_id, person_id}` | enrollment (apply `student_no_show`) |
| `attendance.no_show_reverted.v1` | attendance | `{session_id, person_id}` | enrollment (void credit) |
| `enrollment.leave_fee_charged.v1` | enrollment | `{leave_id, payer_id, student_id, amount_minor}` | billing (pending charge) |
| `enrollment.makeup_credit_granted.v1` | enrollment | `{credit_id, enrollment_id, credit_bps}` | notify |
| `rental.reservation_requested.v1` | rental | `{reservation_id, needs_approval}` | notify (admin) |
| `rental.reservation_confirmed.v1` | rental | `{reservation_id, booker_id, range, price_minor}` | billing (one-off invoice), notify |
| `rental.reservation_cancelled.v1` | rental | `{reservation_id, by, refund_bps}` | billing (credit note), notify |
| `rental.reservation_expired.v1` | rental | `{reservation_id}` | notify |
| `feedback.progress_note_published.v1` | feedback | `{note_id, student_id}` | notify (guardian) |
| `notify.delivery_failed.v1` | notify | `{notification_id, category}` | audit (critical failure), reports |

## 7. Checklist: adding a new module (also in `CONTRIBUTING.md`)

1. Write an RFC in `rfcs/` (GOVERNANCE.md). A new `ModuleId` is a public-API change.
2. Add a variant to `ModuleId` and a row to the dependency table in this doc.
3. Create `crates/eq-<name>` with the layout in 02 §3, and implement `Module`.
4. Add the migration(s) to `migrations/` with the `<name>` tag in the filename.
5. Add a Cargo feature to `eq-api` and `eq-server` (default-on), and the registry entry.
6. Add the policy struct with defaults, validate, summarize and i18n keys (`en`, `id`).
7. Add preset overrides for each preset that should enable it.
8. Add the SPA route folder `web/src/modules/<name>/` and register its nav and routes in `web/src/modules/registry.ts`.
9. Add tests: unit (domain), integration (repo/service with testcontainers), API (axum-test), and a Playwright smoke test.
10. Update `docs/` (user guide page + developer page).
