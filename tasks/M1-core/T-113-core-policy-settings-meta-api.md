---
id: T-113
title: Add CorePolicy, the settings API (policies and modules), meta endpoints and the audit read API
milestone: M1
status: todo
depends_on: [T-104, T-105, T-106, T-109, T-112b]
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
- Track the split task contract and dependency order only.
### Out
- All implementation and tests, which are assigned to the child parts below.

## Split
| Part | Owns | Depends on | Status |
|---|---|---|---|
| T-113a | Core policy and versioned settings API; AC 1, 2, 3, 4, 9 | T-102, T-104, T-105, T-106, T-109, T-112b | draft |
| T-113b | Module metadata, RBAC registry and audit read API; AC 5, 6, 7, 8, 10 | T-113a | draft |

## Files
| Path | Action | Purpose |
|---|---|---|
| `tasks/M1-core/T-113a-*.md` | create or modify | Path named in this task's contract. |
| `tasks/M1-core/T-113b-*.md` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Every original acceptance criterion and test is assigned exactly once to a child part in the Split table.

## Tests to write
| File | Test names |
|---|---|
| n/a | Child parts own all implementation tests. |

## Definition of done
- [ ] Every original criterion and test name is assigned once to a dependent child.
- [ ] The task index reflects the child dependency chain.

## Report
**Deviations** — RATIFY: TD-063, split this CQ-101-sized scope into dependent implementation parts while retaining this file as the contract parent.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
