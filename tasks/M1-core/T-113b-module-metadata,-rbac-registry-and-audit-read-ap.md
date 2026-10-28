---
id: T-113b
title: Module metadata, RBAC registry and audit read API
milestone: M1
status: draft
depends_on: [T-113a]
spec_refs: [POL-06, POL-07, POL-10, POL-12, MOD-06, MOD-09, API-14, AUD-07, PRD-06, STM-12, STM-13, VAL-23, AUTH-01, AUTH-04]
estimated_size: M
---

## Goal
The owner can read, preview and save any enabled module's rules, toggle modules, and see the live summary. The SPA can discover modules, navigation and tenant branding from meta endpoints. Owners and admins can read the audit log within their category scope.

## Context
- `CorePolicy` (04 §6.1), in `eq_core::policy::core_policy`, MODULE Core:
```rust
pub struct CorePolicy {
  pub locale: Locale,                 // E, default en
  pub currency: Currency,             // E, default USD
  pub timezone: TzName,               // E, IANA, default "UTC" (newtype over chrono_tz::Tz, serde as string, schema enum of all tz names)
  pub week_starts_on: Weekday,        // A, Mon|Sun only (validate), default Mon
  pub terminology: Terminology,       // A, { overrides: BTreeMap<TermKey, I18nText> }, default {}
  pub privacy: PrivacyPolicy,         // A
  pub branding: Branding,             // A, x-ui hidden in setup (group advanced, widget "hidden" — the SPA edits it on the Branding page)
}
pub struct Branding { pub brand_hue: u16 /*0..=359, 255*/, pub corner_style: CornerStyle /*sharp|default|round, default*/, pub logo_file_id: Option<Uuid> /*None*/ }
pub enum TermKey { Instructor, Guardian, Student, Offering, Session, Resource, Location, Enrollment }
pub struct PrivacyPolicy { pub deletion_sla_business_days: u16 /*1..=30, 14*/, pub block_deletion_with_open_invoices: bool /*true*/, pub photo_retention_days: u16 /*7..=3650, 365*/ }
```
  - POL-12: changing `currency` when any invoice exists → `PolicyViolation{rule:"core.currency.locked"}`. Implement this through a port `CurrencyLockCheck` in `eq_core::ports`, with a default impl returning "unlocked". Billing registers the real one in M4.
  - Saving CorePolicy also updates the `tenants.locale/currency/timezone` columns (denormalised for tenant resolution, T-104) in the same tx.
  - Summary lines: `policy.core.summary.locale_currency {locale, currency}`, `policy.core.summary.timezone {timezone}`.
- Settings API (04 §7):

| Method | Path | Perm | Body / Result |
|---|---|---|---|
| GET | `/settings/policies` | core.policy.read | `[{module, version, effective_from, schema, document, summary, pending_future?: {version, effective_from}}]` for enabled modules (core, identity, audit always) |
| GET | `/settings/policies/{module}` | core.policy.read | the above + `history: [{version, effective_from, created_by: {id, display_name}, note}]` |
| GET | `/settings/policies/{module}/versions/{v}` | core.policy.read | the full stored document |
| POST | `/settings/policies/{module}/preview` | core.policy.write | `{document}` → `{valid, errors: [FieldError], summary: [SummaryLine]}` (no write) |
| PUT | `/settings/policies/{module}` | core.policy.write | `{document, effective_from?, note?}` + `If-Match: "v<current>"` → Versioned (runs `validate_policy` then `validate_policy_in_context`); 409 `version_conflict`, 428, 422 `validation`/`policy_violation` |
| GET | `/settings/modules` | core.modules.read | same shape as `/meta/modules` `modules[]` |
| PUT | `/settings/modules` | core.modules.write | `{enabled: [ModuleId]}` (the full set of optional modules wanted on) → `/meta/modules` body; 422 `module_dependency {missing: {module: [..]}}`, 422 `module_required_by {required_by: [..]}` |

  - `effective_from` must be ≥ now − 1 min, else 422 `validation` at `/effective_from`. Absent means now.
  - A disabled module's policy endpoints → 404 `module_disabled`.
- Meta (API-14, MOD-09):
  - `GET /meta/modules` (any authenticated) → `{modules: [{id, compiled, enabled, always_on, requires_all, requires_any, has_data, policy_version}], nav: [NavItem filtered by perms + enabled]}`.
  - `has_data` comes from `Module::has_data(tx, tenant)` (trait method from T-105).
  - `compiled:false` entries are listed for all `ModuleId`s not in the registry.
  - `GET /meta/app` (public, tenant-resolved, or unprovisioned) → `{version, commit, mode, brand: {name, logo_url?}, tenant?: {name, locale, currency, timezone, terminology}, features: {captcha: false, guests: false}, setup_required: bool}`. `brand.name` comes from `eq_core::brand::NAME`; hue, corner style and logo come from `CorePolicy.branding` (logo URL `/api/v1/files/{id}`, public within the tenant per SEC-15; until T-303 exists, return `logo_url: null`).
- Permissions declared by core:

| key | roles |
|---|---|
| core.policy.read | owner, admin |
| core.policy.write | owner |
| core.modules.read | owner, admin |
| core.modules.write | owner |

- Audit read API (AUD-07):
  - `GET /audit/entries?from&to&actor_id&action(prefix)&entity_type&entity_id&category&limit&cursor` (perm `audit.read`: owner, admin).
  - **Admins are filtered to `AuditPolicy.admin_categories` in the query layer.** The owner sees all.
  - `GET /audit/entries/{id}`. `GET /audit/export?...` → `text/csv` streamed; the export itself is audited (`audit.export`, compliance). `GET /audit/verify?from_seq&to_seq` (owner only, via perm `audit.verify`: owner).
  - Entry DTO: `{id, seq, at, actor: {id?, display_name?, role}, action, entity_type, entity_id, category, before, after, request_id, sealed: bool}`. `ip` is shown to owners only.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `GET /meta/app` adds `rules: { password_min_length }` (public, S0) for client-side hints (VAL-23, D-21).
  - Module and policy transitions follow STM-12/13: scheduled policy versions are not editable (D-34); `effective_from` may be at most 5 minutes in the past (D-24).
- RBAC matrix test (spec 25 §6): `crates/eq-api/tests/fixtures/rbac_matrix.csv` is a transcription of the spec 25 §6 table, one row per permission: `key,owner,admin,instructor,guardian,student`, each cell `y` when the matrix shows ✅, `own`, `linked`, `self` or `read`, else `n`. The test builds the registry with every compiled module and asserts that, for each module present, the set of `(key, roles)` from its `PermissionDef`s equals the fixture rows whose key starts with `<module>.`. Rows of modules that aren't compiled in are skipped, so later module tasks only need the fixture to stay true.

## Scope
### In
- Implement only AC 5, 6, 7, 8, 10 and the tests assigned to T-113b in this split.
- Use the task's inline context and canonical interface definitions for those criteria.
### Out
- AC and behavior owned by the sibling split part(s) listed in T-113.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-api/tests/api_settings.rs` | test | Task-listed verification file. |
| `crates/eq-api/tests/rbac_matrix.rs` | test | Task-listed verification file. |
| `crates/eq-audit/tests/api_audit.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T4, T5 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- Abuse-case acceptance criterion: `admin_category_scope` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC5. `PUT /settings/modules` with `[billing]` only → 422 `module_dependency`. With `[enrollment, billing]` → 200 and both enabled. → `toggle_modules_api`
- AC6. `/meta/modules` as a guardian lists nav items they may access only. → `meta_nav_filtered`
- AC7. `/meta/app` with no tenant → `setup_required: true` and no `tenant`. → `meta_app_unprovisioned`
- AC8. An admin can't see `security` entries with the default AuditPolicy, and the owner can. → `api_audit::admin_category_scope`
- AC10. The `PermissionDef`s registered by every compiled module equal the spec 25 §6 matrix fixture. → `rbac_matrix_matches_spec`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/api_settings.rs` | `toggle_modules_api`, `meta_nav_filtered`, `meta_app_unprovisioned` |
| `crates/eq-api/tests/rbac_matrix.rs` | `rbac_matrix_matches_spec` |
| `crates/eq-audit/tests/api_audit.rs` | `admin_category_scope`, `export_is_audited`, `verify_owner_only` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated; TST-09 i18n keys for core/identity/audit policy paths are listed in the Report (the SPA catalog is added in M3)
- [ ] Commit `feat(core): settings, meta and audit read APIs [T-113b]`, signed off

## Report
**Deviations** — RATIFY: TD-063, CQ-101 split of the original task contract; the assigned AC and test names are preserved in this part.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
