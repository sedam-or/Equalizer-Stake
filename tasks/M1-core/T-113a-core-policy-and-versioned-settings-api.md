---
id: T-113a
title: Core policy and versioned settings API
milestone: M1
status: review
depends_on: [T-102, T-104, T-105, T-106, T-109h, T-112b]
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
pub struct Branding { pub brand_hue: BrandHue /*0..=359, 255*/, pub corner_style: CornerStyle /*sharp|default|round, default*/, pub logo_file_id: Option<Uuid> /*None*/ }
pub enum TermKey { Instructor, Guardian, Student, Offering, Session, Resource, Location, Enrollment }
pub struct PrivacyPolicy { pub deletion_sla_business_days: DeletionSlaBusinessDays /*1..=30, 14*/, pub block_deletion_with_open_invoices: bool /*true*/, pub photo_retention_days: PhotoRetentionDays /*7..=3650, 365*/ }
```
  - POL-12: changing `currency` when any invoice exists → `PolicyViolation{rule:"core.currency.locked"}`. Implement this through a port `CurrencyLockCheck` in `eq_core::ports`, with a default impl returning "unlocked". Billing registers the real one in M4.
  - Saving CorePolicy also updates the `tenants.locale/currency/timezone` columns (denormalised for tenant resolution, T-104) in the same tx through the tenant-context-checked function `public.update_tenant_policy_defaults` so `eq_app` does not receive direct table privileges.
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
- Implement only AC 1, 2, 3, 4, 9 and the tests assigned to T-113a in this split.
- Use the task's inline context and canonical interface definitions for those criteria.
### Out
- AC and behavior owned by the sibling split part(s) listed in T-113.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-api/tests/api_settings.rs` | test | Task-listed verification file. |
| `crates/eq-core/src/policy/core_policy.rs` | create | Validated CorePolicy value types, defaults, schema and summary. |
| `crates/eq-core/src/policy/mod.rs` | modify | Export CorePolicy and its validated range types. |
| `crates/eq-core/src/module.rs` | modify | Register Core policy behavior and policy permissions. |
| `crates/eq-core/src/ports.rs` | modify | Add the default-unlocked CurrencyLockCheck port. |
| `crates/eq-core/src/app_state.rs` | modify | Permit the API runtime to receive a CurrencyLockCheck implementation. |
| `crates/eq-core/src/policy/store.rs` | modify | Add current-version conditional writes and typed policy history reads. |
| `crates/eq-core/src/policy/repo.rs` | modify | Update existing tenant locale/currency/timezone columns transactionally. |
| `migrations/0013_core_tenant_policy_defaults.sql` | create | Tenant-context-checked update function for the existing Core defaults. |
| `crates/eq-identity/src/api_auth.rs` | modify | Expose the shared lazy PolicyStore to the Core settings handlers. |
| `crates/eq-api/src/settings.rs` | create | Core policy preview, read and versioned write routes. |
| `crates/eq-api/src/lib.rs` | modify | Register the settings route module. |
| `crates/eq-api/src/modules.rs` | modify | Mount Core settings routes under Core's authenticated module gate. |
| `crates/eq-api/Cargo.toml` | modify | Add integration-test dependencies used by `api_settings.rs`. |
| `.sqlx/` | modify | SQLx offline metadata for changed compile-time queries. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T4 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `admin_cannot_write_policy` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Preview with an invalid document returns `valid:false` and pointer errors, and nothing is written. → `api_settings::preview_no_write`
- AC2. PUT with a correct If-Match creates v2 and an audit `core.policy.changed` with before/after. A stale If-Match → 409. → `put_policy_versioned`
- AC3. A future `effective_from` shows up as `pending_future` while `version` stays the current one. → `pending_future_shown`
- AC4. An admin calling PUT → 403 `forbidden {perm: core.policy.write}`. → `admin_cannot_write_policy`
- AC9. Changing currency when the `CurrencyLockCheck` test double says locked → 422 `policy_violation` rule `core.currency.locked`. → `currency_lock`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/api_settings.rs` | `preview_no_write`, `put_policy_versioned`, `pending_future_shown`, `admin_cannot_write_policy`, `currency_lock` |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated; TST-09 i18n keys for core/identity/audit policy paths are listed in the Report (the SPA catalog is added in M3)
- [x] Commit `feat(core): settings, meta and audit read APIs [T-113a]`, signed off

## Report
**Plan** —
1. Add validated CorePolicy value types and Core module policy behavior → verify defaults, numeric bounds, serde validation and summaries with focused unit tests.
2. Extend Core-owned policy persistence with history, conditional versions and transaction-scoped tenant-default updates → verify it against the fresh PostgreSQL task database and SQLx metadata.
3. Add authenticated settings preview/read/versioned-write routes and integration tests → verify every assigned AC with its exact test name, including forbidden and tenant-context cases.
4. Review layering, SQL plans and security; run Rust, guards, ERD, dependency, SQLx and browser checks → record unavailable checks and environmental E2E failures accurately.
5. Fill this report, set `review`, regenerate/check the task index and linter, commit once with sign-off, then push only `dev`.

**Assumptions** — The current effective version is the required `If-Match` value; a scheduled future version is displayed separately and remains immutable. D-24's five-minute clock-skew bound is canonical over the older one-minute sentence. Validated private CorePolicy wrappers preserve the task's numeric JSON wire forms. Dependency Reports and shipped code for T-102, T-104, T-105, T-106, T-109h and T-112b were reviewed; their interfaces used here match.

**Test plan** — Given an invalid Core document, when previewed, then `valid:false`, pointer errors and an unchanged policy count are asserted by `preview_no_write`. Given v1 and matching `If-Match`, when a valid policy is saved, then v2 and its before/after audit are asserted by `put_policy_versioned`; the same test also asserts stale-version 409. Given a future effective time, `pending_future_shown` asserts current v1 plus `pending_future`. Given an admin actor, `admin_cannot_write_policy` asserts 403 `forbidden` for `core.policy.write`. Given a locked currency port, `currency_lock` asserts 422 `policy_violation` with `core.currency.locked` and no write. Edge/error cases cover missing or malformed preconditions, invalid numeric/serde bounds, and cross-tenant actor context. Abuse cases cover forbidden admin mutation and tenant-context mismatch. Relevant problem codes are `forbidden`, `precondition_required`, `version_conflict`, `validation` and `policy_violation`.

**Summary** — Added validated Core policy values, registered Core policy permissions and behavior, and extended PolicyStore for typed history and optimistic version writes. The settings handlers now preview, read and version Core policy through the tenant-scoped transaction; successful writes audit and update the existing tenant locale/currency/timezone fields atomically through a narrowly granted tenant-context-checked function. All five assigned API tests and both Core value tests pass.

**Spec read** — `spec/04-policy-engine.md` §§6.1, 7; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§7–8; `spec/18-security-by-design.md` §§4, 5, 8; `spec/19-error-handling.md` §10; `spec/22-validation-and-sanitization.md` §5.1 and D-24; `spec/29-erd.md` Core `tenants` dictionary, §5 data classes and §7 migration map. UI design reference: n/a (backend-only task).

**Files** —
- Modified `Cargo.lock` and `crates/eq-api/Cargo.toml` for the test/runtime dependencies.
- Modified `crates/eq-api/src/lib.rs` and `crates/eq-api/src/modules.rs` to register and mount Core settings routes.
- Created `crates/eq-api/src/settings.rs` with policy preview/read/versioned-write handlers.
- Created `crates/eq-api/tests/api_settings.rs` with the five named HTTP integration tests and forbidden/context cases.
- Modified `crates/eq-core/src/app_state.rs`, `crates/eq-core/src/module.rs` and `crates/eq-core/src/ports.rs` for settings runtime wiring, Core policy behavior and the currency-lock port.
- Created `crates/eq-core/src/policy/core_policy.rs`; modified `crates/eq-core/src/policy/mod.rs`, `crates/eq-core/src/policy/repo.rs` and `crates/eq-core/src/policy/store.rs` for validated policy types and persistence.
- Modified `crates/eq-identity/src/api_auth.rs` to expose the shared PolicyStore to settings handlers.
- Created `migrations/0013_core_tenant_policy_defaults.sql`; it adds only the scoped function over existing tenant columns, no table or column.
- Added `.sqlx/query-b3793f7a232d56ff30dfd6746f81809e9967cdb2d99e00109e1c0947d7c67b0b.json` for SQLx offline metadata.
- Updated `tasks/DECISIONS.md`, this task file and generated `tasks/INDEX.md`.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `preview_no_write` | Pass |
| AC2 | `put_policy_versioned` | Pass, including stale `If-Match` → 409 |
| AC3 | `pending_future_shown` | Pass |
| AC4 | `admin_cannot_write_policy` | Pass, 403 before policy data access |
| AC9 | `currency_lock` | Pass, 422 and no policy/default write |

**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked -p eq-core --lib policy::core_policy::tests` — 2 passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true DATABASE_URL='postgres://eq_app:test-secret-app@localhost:5433/ritma_test_T-113a' RITMA_DATABASE_OWNER_URL='postgres://eq:eq@localhost:5433/ritma_test_T-113a' RITMA_SECRET_KEY='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=' RITMA_PUBLIC_URL='https://example.invalid' TEST_PASSWORD_VERSION=1700000000 cargo test --locked -p eq-api --test api_settings -- --test-threads=1` — 5 passed, 0 failed, using `ritma_test_T-113a` on the existing `ritma-db-1` PostgreSQL 16 container.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" DATABASE_URL='postgres://eq:eq@localhost:5433/ritma_test_T-113a' SQLX_OFFLINE=true cargo sqlx prepare --check --workspace -- --all-targets` — passed with the owner connection; metadata is current.
- ✅ `cargo deny check` — advisories, bans, licences and sources pass; the existing duplicate `base64` versions warning remains.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --migrations && python3 -I scripts/erd.py --check` — 26 migrated tables checked; ERD matches DDL. Migration 0013 adds no table/column.
- ✅ `docker exec ritma-db-1 psql -U eq -d ritma_test_T-113a ... EXPLAIN (ANALYZE, BUFFERS)` for policy history and max-version lookups — both use `policies_lookup`.
- ✅ `pnpm -C web build` (run by `make e2e`). Web lint/typecheck/unit/i18n checks not run because this task changes no web source.
- ⚠️ `make e2e` with `CARGO_TARGET_DIR=$HOME/.cache/ritma/target`, `E2E_SKIP_DB_START=1`, `E2E_PLAYWRIGHT_CONTAINER=0` and both database URLs targeting `ritma_test_T-113a` — 4/6 passed against the disposable task DB in `ritma-db-1`: Chromium 3/3 and WebKit health request 1/1. `RITMA_SECRET_KEY` used an obvious fake test key. WebKit `loads home` and `spa fallback` could not launch because host libraries `libicu74` and `libflite1` are missing. No Playwright container was created; a temporary `target` symlink to the fixed cache directory was removed after the run.
- ❌ `make api` — exits 2 with `OpenAPI generation is added by T-114`; generation is owned by T-114, consistent with existing TD-052. No generated OpenAPI artifact is claimed.
- Not run: `cargo test --workspace`; its Testcontainers fixtures would create another PostgreSQL container/database, outside the single-container/task-database limit. Focused unit and task integration suites were run instead.
- Not run: `make dev`; it sources the shared `.env` and default dev database. `make e2e` ran the local server against the disposable task database, and the API integration suite exercised the task endpoints.
- Not run: hosted CI and `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check`; hosted CI is not available here and no web source was changed.
- ✅ `scripts/tasks-index.sh --check && python3 -I scripts/tasks-lint.py` — both pass; task index regenerated after setting status to `review`.
- ✅ `git diff --cached --check` and `gitleaks protect --staged --redact` — staged task-owned paths have no whitespace errors or detected secrets.

**Manual checks** — The HTTP integration suite made local requests through the mounted settings router for invalid preview, successful and stale versioned writes, future versions, admin denial and locked currency. `make e2e` built the SPA and release server, then used local Chromium/WebKit against the fresh task database; all Chromium journeys and the WebKit health request passed. Browser-page WebKit checks were unavailable for the missing host libraries. The disposable `ritma_test_T-113a` database was dropped after the checks; the existing `ritma-db-1` container remains healthy.

**Debug log** — The first migration invocation lacked required `RITMA_SECRET_KEY`; rerunning with the fake test key and public URL applied migrations successfully. SQLx metadata validation first used the restricted `eq_app` role and failed at an existing compile-time query reading `tenants`; using the owner connection for compile-time introspection passed without changing grants or query code. Earlier API fixture attempts exposed a wrong test-role password, an overlong generated tenant slug and a token timestamp rounding race; the fixture now uses the configured fake app password, a bounded slug and stable `TEST_PASSWORD_VERSION`, and the final API suite passes 5/5. The E2E run's two failures are environmental WebKit launch failures, not application assertions.

**Performance** — `EXPLAIN (ANALYZE, BUFFERS)` on the fresh, seeded task database showed `policies_lookup` index scans for history and max-version queries (0.158 ms and 0.035 ms respectively on an empty policy set). This verifies index selection only; it is not a production-scale latency measurement. No additional index was justified.

**Security** — Triggers T1, T2 and T4. Authenticated handlers require `core.policy.read` or `core.policy.write` before policy access; tenant identity comes from the actor/transaction context, and the tests use the non-superuser `eq_app` pool so RLS applies. CorePolicy numeric bounds validate at construction and serde boundaries. The migration function pins its `SECURITY DEFINER` search path, requires its tenant argument to match `app.tenant_id`, updates only the existing locale/currency/timezone fields, revokes `PUBLIC` and grants `EXECUTE` only to `eq_app`. Policy, audit and denormalized defaults share the caller transaction. Abuse tests cover forbidden admin writes and cross-tenant context mismatch; no new data field or sensitive output was added. The shared API-15 limiter is not claimed here: existing TD-152 tracks it and T-115 owns the cross-cutting limiter and If-Match helpers.

**Deviations** — RATIFY: TD-063 (original contract split), TD-066 (spec-backed security section), TD-067 (required task headings), TD-180 (implementation paths and tenant-context-checked function), TD-181 (D-24 five-minute bound), TD-182 (validated Core value wrappers), TD-183 (shared rate limiting is owned by T-115), TD-184 (CQ-101 size review). The six TST-09 keys are recorded here for later SPA catalog work: `policy.core.summary.locale_currency {locale, currency}`, `policy.core.summary.timezone {timezone}`, `policy.identity.summary.sessions {idle_admin, idle_guardian, absolute_days}`, `policy.identity.summary.lockout {max_failed, window, lock}`, `policy.audit.summary.retention {years}` and `policy.audit.summary.anonymize {years}`. Translation catalog work remains with T-306.

**Dependencies affected** — T-113b can use the Core policy types, shared PolicyStore and settings runtime introduced here. T-115 must apply its shared If-Match and rate-limit layers to these endpoints before API-15 is considered satisfied. No other dependency contract changes.

**Rollback plan** — Revert the signed task commit before merge. Migration 0013 adds only an additive function and grants; it creates no schema data to reverse, and the function is inert if the application code is reverted. Do not delete or rewrite a deployed migration.

**Follow-ups** — T-115 owns shared API-15 rate limiting and replacing this handler's conditional-write parsing with the standard If-Match helper (TD-152/183). T-114 owns OpenAPI generation (TD-052). Install WebKit host dependencies to run the two unavailable browser checks. Full workspace tests remain unverified under the one-container restriction. Review TD-184 and decide whether to require a follow-up split/refactor for CQ-101 size; no new policy/rate-limit value was invented here.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-180):** the Files table lists only the verification test although AC1–4/9 require Core policy types, transactional policy storage, permission wiring and HTTP routes, and `eq_app` has no direct tenant update privilege → add the specified implementation paths and use the tenant-context-checked function in migration 0013; expose `/api/v1` route prefixes from spec/04 §7.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-181):** the task repeats a one-minute `effective_from` lower bound while its canonical alignment cites D-24's five-minute bound → use five minutes because spec/22 D-24 is the canonical clock-skew rule.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-182):** the interface sketch uses plain `u16` for bounded hue and retention values → represent those ranges with private validated value types that preserve the specified numeric JSON form, because CQ-23/CAN-10 require invalid values to be unconstructable and deserialization to validate.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-183):** API-15 calls for rate limits on the new settings routes, but the contract assigns the shared limiter to T-115 → keep the missing control explicit as a required T-115 follow-up and do not claim it is satisfied here.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-184):** the assigned implementation exceeds CQ-101's approximate size guidance → retain required ACs/tests without editing sibling task ownership, and surface a follow-up split/refactor for maintainer judgment.

## Review
n/a — pending human review after implementation.
