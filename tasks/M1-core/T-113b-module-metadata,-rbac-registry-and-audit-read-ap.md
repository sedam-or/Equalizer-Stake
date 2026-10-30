---
id: T-113b
title: Module metadata, RBAC registry and audit read API
milestone: M1
status: review
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
| `Cargo.toml` | modify | Add the spec-named CSV encoding dependency. |
| `Cargo.lock` | modify | Lock the CSV dependency. |
| `crates/eq-api/Cargo.toml` | modify | Add API test/runtime dependencies if required by the contract. |
| `crates/eq-api/src/lib.rs` | modify | Register the public meta routes. |
| `crates/eq-api/src/meta.rs` | create | Serve tenant module metadata, permission-filtered navigation and public app metadata. |
| `crates/eq-api/src/modules.rs` | modify | Mount authenticated metadata and module settings routes with the shared runtime. |
| `crates/eq-api/src/settings.rs` | modify | Add module read and toggle handlers. |
| `crates/eq-api/tests/api_settings.rs` | test | Task-listed verification file. |
| `crates/eq-api/tests/rbac_matrix.rs` | test | Task-listed verification file. |
| `crates/eq-api/tests/fixtures/rbac_matrix.csv` | test | Spec 25 §6 permission fixture. |
| `crates/eq-core/src/export.rs` | create | Provide the spec-named CSV formula guard. |
| `crates/eq-core/src/lib.rs` | modify | Export the CSV guard helper. |
| `crates/eq-core/src/module.rs` | modify | Register the Core module permissions. |
| `crates/eq-core/src/policy/store.rs` | modify | Expose the current persisted policy version by module. |
| `crates/eq-core/src/tenant.rs` | modify | Read the tenant display name through a tenant-scoped transaction. |
| `crates/eq-core/src/tenant/repo.rs` | create | Read the Core-owned tenant name within its RLS transaction. |
| `crates/eq-identity/src/api_auth.rs` | modify | Share the lazy PolicyStore and resolve the spec-defined blank preset's empty policy patch. |
| `crates/eq-scheduling/src/lib.rs` | modify | Register the spec-defined Scheduling permission grants. |
| `crates/eq-enrollment/src/lib.rs` | modify | Register the spec-defined Enrollment permission grants. |
| `crates/eq-billing/src/lib.rs` | modify | Register the spec-defined Billing permission grants. |
| `crates/eq-attendance/src/lib.rs` | modify | Register the spec-defined Attendance permission grants. |
| `crates/eq-rental/src/lib.rs` | modify | Register the spec-defined Rental permission grants. |
| `crates/eq-notify/src/lib.rs` | modify | Register the spec-defined Notify permission grants. |
| `crates/eq-reports/src/lib.rs` | modify | Register the spec-defined Reports permission grants. |
| `crates/eq-feedback/src/lib.rs` | modify | Register the spec-defined Feedback permission grants. |
| `crates/eq-audit/Cargo.toml` | modify | Add the spec-named CSV and stream dependencies. |
| `crates/eq-audit/src/lib.rs` | modify | Register the audit HTTP route module. |
| `crates/eq-audit/src/http.rs` | create | Implement tenant-scoped audit reads, export and verification routes. |
| `crates/eq-audit/src/module.rs` | modify | Register audit permissions and mount its routes. |
| `crates/eq-audit/src/repo.rs` | modify | Add parameterized, category-scoped audit read queries. |
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
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass.
- [x] Targeted Rust checks pass; full workspace test execution is not run because its other integration tests start extra Testcontainers databases, outside this run's one-container limit. `make api` is deferred to dependent task T-114 under TD-192. TST-09 keys are listed in the Report; the SPA catalog is added in M3.
- [x] Commit `feat(core): settings, meta and audit read APIs [T-113b]`, signed off.

## Report
**Plan** —
1. Register Core, Audit and compiled-module permissions and compare the registry to the spec 25 §6 fixture.
2. Add module settings plus authenticated `/meta/modules` and public `/meta/app`, and exercise the named endpoint tests.
3. Add Audit repository and HTTP list/detail/export/verify routes; verify category filtering, export audit, owner-only verification and tenant scoping on the task database.
4. Make no schema changes; prepare SQLx metadata for checked queries, run focused suites and static checks, and record dependency-ordered checks accurately.
5. Review scope, architecture and security; finish this Report, set `review`, regenerate/check the task index and linter, commit once with sign-off and push only `dev`.

**Assumptions** — The task Scope and inline endpoint contracts own AC5–8/10. TD-187 records the missing implementation paths without changing those contracts. Registered permissions follow spec 25 §6; handlers and repositories retain scope checks. Uncompiled manifest fields use empty dependency lists and false booleans. TD-188 records technical pagination and parser bounds. T-114 owns OpenAPI generation, so T-113b does not add its generator or generated contract files (TD-192).

**Test plan** — Given billing without enrollment, the owner receives 422 `module_dependency`; enrollment plus billing succeeds (`toggle_modules_api`). Given a guardian, only enabled navigation items allowed by its permission set appear (`meta_nav_filtered`). With no resolved tenant, public metadata has `setup_required: true` and omits `tenant` (`meta_app_unprovisioned`). Given operational and security rows, admin lists/details do not disclose security while owner can read it (`admin_category_scope`). An owner CSV export is audited and a formula-leading action is neutralized (`export_is_audited`). Admin verification is forbidden and owner verification succeeds for the actual tenant sequence range (`verify_owner_only`). The compiled permission registry equals the spec fixture (`rbac_matrix_matches_spec`). Edge/error/abuse paths include invalid filters/cursors, empty results, missing auth, cross-tenant IDs, forbidden verification, restricted categories and CSV formula injection.

**Workflow steps** — Step 0: resumed the existing `in_progress` T-113b work on `dev`; preserved unrelated WIP and used the fixed Cargo target and existing DB container. Step 1: read this task and its cited contract; consulted the spec sections below. Step 2: read T-113a's Report and shipped Core settings, ModuleRegistry, PolicyStore, tenant resolver and audit policy code. Step 3: n/a, backend/API task. Step 4: recorded Plan, Assumptions and Test plan before continuing implementation. Step 5: implemented only AC5–8/10 and their named tests, with the minimum paths added under TD-187. Step 6: integration tests made real HTTP requests over loopback against the task database; `make dev` was not used because its recipe sources the existing operator `.env`, while the integration fixture pins its own database. Step 7: fixed audit test ID/sequence assumptions, Clippy findings and the HTTP-layer SQLx type boundary; reran affected tests and checks. Step 8: reviewed the diff against CQ-09, CA-01/04/11, ERR-22 and SBD-81. Step 9: n/a, no optimization measurement was justified. Step 10: completed the implementer security audit and abuse-case tests; independent maintainer security review remains pending. Step 11: ran the checks below and recorded the single-container exclusions. Step 12: completed this Report, set `review`, regenerated and checked the index, then made one signed-off commit and pushed `dev`.

**Summary** — Core module settings and metadata routes now expose the task-specified enablement and discovery behavior. Audit read/export/verify endpoints enforce role/category scope, tenant context and PII visibility; exports are streamed, audited and formula-safe. Every compiled module's permission definitions now match the spec 25 fixture. No migration was added.

**Spec read** — `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 3, 7; `spec/18-security-by-design.md` §§5.1–5.4, 7–8 (SBD-40–46, SBD-70, SBD-73, SBD-81); `spec/19-error-handling.md` ERR-22; `spec/25-auth-and-access-control.md` §6; `spec/29-erd.md` §5. Also read the T-113a dependency Report and the shipped Core/Audit code named above.

**Files** —
- `.sqlx/query-0a212e02879a6aa56d8b03b356bab442b6896012d5bf2c7edbb1f725a5a07c1b.json` — generated SQLx metadata for a checked query.
- `.sqlx/query-550a8b14b37eee4ca9b7e281702edb91ca66247c5bcccacfa877f02a44028c10.json` — generated SQLx metadata for a checked query.
- `Cargo.toml` — add CSV and bounded-stream dependencies.
- `Cargo.lock` — lock the new dependencies.
- `crates/eq-api/Cargo.toml` — add API test/runtime dependency.
- `crates/eq-api/src/lib.rs` — register metadata routes.
- `crates/eq-api/src/meta.rs` — implement app/module metadata.
- `crates/eq-api/src/modules.rs` — compose module routes and shared runtime state.
- `crates/eq-api/src/settings.rs` — implement module settings reads and toggles.
- `crates/eq-api/tests/api_settings.rs` — verify module toggle and metadata behavior.
- `crates/eq-api/tests/fixtures/rbac_matrix.csv` — transcribe spec 25 §6 grants.
- `crates/eq-api/tests/rbac_matrix.rs` — compare compiled permissions to the fixture.
- `crates/eq-attendance/src/lib.rs` — register spec-defined permissions.
- `crates/eq-audit/Cargo.toml` — add CSV/stream dependencies and OpenAPI integration.
- `crates/eq-audit/src/lib.rs` — register the Audit HTTP module.
- `crates/eq-audit/src/http.rs` — implement scoped list/detail/export/verify endpoints.
- `crates/eq-audit/src/module.rs` — register permissions/routes and update the permission assertion.
- `crates/eq-audit/src/repo.rs` — add checked, tenant-scoped reads.
- `crates/eq-audit/tests/api_audit.rs` — verify category scope, export audit/formula guard and owner-only verification.
- `crates/eq-billing/src/lib.rs` — register spec-defined permissions.
- `crates/eq-core/src/export.rs` — provide the CSV formula guard.
- `crates/eq-core/src/lib.rs` — export the formula guard.
- `crates/eq-core/src/module.rs` — register Core permissions and optional manifest permissions.
- `crates/eq-core/src/policy/store.rs` — expose current persisted policy version.
- `crates/eq-core/src/tenant.rs` — resolve tenant display name through Core.
- `crates/eq-core/src/tenant/repo.rs` — query the Core-owned resolver.
- `crates/eq-enrollment/src/lib.rs` — register spec-defined permissions.
- `crates/eq-feedback/src/lib.rs` — register spec-defined permissions.
- `crates/eq-identity/src/api_auth.rs` — share the lazy PolicyStore and resolve only the blank v1 preset patch.
- `crates/eq-notify/src/lib.rs` — register spec-defined permissions.
- `crates/eq-rental/src/lib.rs` — register spec-defined permissions.
- `crates/eq-reports/src/lib.rs` — register spec-defined permissions.
- `crates/eq-scheduling/src/lib.rs` — register spec-defined permissions.
- `tasks/DECISIONS.md` — record TD-192/193 and clarify TD-188 bounds.
- `tasks/INDEX.md` — regenerate task status index.
- `tasks/M1-core/T-113b-module-metadata,-rbac-registry-and-audit-read-ap.md` — complete report, checks, DoD and TD references.

**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC5 | `toggle_modules_api` | ✅ pass |
| AC6 | `meta_nav_filtered` | ✅ pass |
| AC7 | `meta_app_unprovisioned` | ✅ pass |
| AC8 | `admin_category_scope` | ✅ pass; admin cannot list/fetch security rows, owner can |
| Audit export | `export_is_audited` | ✅ pass; audit event and formula-guarded cell verified |
| Owner verification | `verify_owner_only` | ✅ pass; admin 403, owner verification succeeds |
| AC10 | `rbac_matrix_matches_spec` | ✅ pass |

**Checks run** —
- ✅ `cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace --no-run` — every workspace test target compiled; this command executes no tests.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace --lib --exclude eq-core --exclude eq-identity` — 7 tests passed. Those two crates' in-source tests start Testcontainers.
- ✅ Isolated `ritma_test_T-113b` environment, `cargo test -p eq-api --test api_settings -- --test-threads=1` — 8 passed, including all three T-113b settings/meta tests.
- ✅ Same isolated environment, `cargo test -p eq-api --test rbac_matrix` — 1 passed.
- ✅ Same isolated environment, `cargo test -p eq-audit --test api_audit -- --test-threads=1` — 3 passed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=false DATABASE_URL=postgres://eq:eq@localhost:5433/ritma_test_T-113b cargo sqlx prepare --workspace -- --all-targets` — refreshed and added checked-query metadata.
- ✅ `cargo deny check` — exit 0; advisories, bans, licenses and sources passed. Existing duplicate-version warnings remain; no deny policy changed.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check` — ERD matches DDL.
- n/a `scripts/erd.py --migrations` — no migration changed.
- ✅ `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py` — run before commit.
- ✅ `git diff --cached --check`.
- ✅ `gitleaks protect --staged --redact` — no leaks found.
- ❌ `make api` — exit 2: `OpenAPI generation is added by T-114.` T-114 depends on T-113b and owns this generator; recorded as TD-192, not reported as passed.
- Not run: `cargo test --workspace` execution. Other integration tests instantiate extra Testcontainers PostgreSQL instances, contrary to this run's one-container rule. All targets compiled, safe unit tests and each T-113b DB test ran on the isolated database.
- Not run: `make dev`; its recipe sources the existing `.env`. The API suites made real loopback HTTP requests with explicit task-database configuration instead.
- n/a web lint/typecheck/test/i18n commands — no `web/` file changed; TST-09 key handoff is below.
- Not run: hosted CI; unavailable in this local task run.

**Manual checks** — `ritma-db-1` was the only PostgreSQL container used. The fresh task database `ritma_test_T-113b` was migrated; app queries used `eq_app` (`NOSUPERUSER NOBYPASSRLS`) and fixture setup used the owner connection. API suites issued real HTTP requests to loopback Axum listeners. The task database is dropped after checks; the existing container and unrelated resources remain untouched.

**Debug log** — Initial audit assertions compared an audit row ID to its entity ID and assumed sequence values `1..2`; the fixture now reads the actual audit ID and tenant sequence bounds, and the rerun passed 3/3. Initial Clippy found generated SQLx macro lint attributes, an oversized CSV helper and a narrowing cast; scoped `#[expect]` attributes with reasons plus two small helpers/conversion fixed them. The layer guard rejected a direct SQLx pool type in the HTTP adapter; stream state now carries `AppState`, and the rerun passed. The existing Audit module test asserted an empty permission list; it now asserts the two spec grants and passes. No failure remains.

**Performance** — No benchmark was indicated; list and export reads remain bounded by the technical limits in TD-188.

**Security** — Triggers T1/T2/T4/T5 from the task. `audit.read`/`audit.verify` are checked before data access; tenant comes from resolved context and is included in SQL/RLS transactions; admins receive only policy-allowed categories in the query; inaccessible entries return 404; verify is owner-only. Before/after JSON is returned/exported only with `identity.people.read_pii`; IP is owner-only and export requires that permission. Export is streamed, audited as `audit.export`, and applies SBD-46 to every cell. `admin_category_scope`, `verify_owner_only` and `export_is_audited` cover the forbidden-category, forbidden-role and formula-injection cases. No new data field, table, migration, permission key or error code was added. Implementer SBD-81 review complete; independent maintainer security review remains required.

**Deviations** — RATIFY: TD-063 contract split; TD-066 security-trigger classification; TD-067 task-template completion from its existing contract; TD-187 omitted implementation paths; TD-188 technical bounds (default page 50, maximum 200, export chunks 200, filter length 128 and decimal tenant-local sequence cursor); TD-189 `futures-util` for bounded CSV streaming; TD-190 shipped Core tenant-name resolver; TD-191 only spec-defined blank v1 gets an empty preset patch while unknown presets fail closed; TD-192 defer `make api` to dependent T-114; TD-193 preserve all assigned behavior despite CQ-101's approximate 400-line review guide. TST-09 key handoff: `policy.core.summary.locale_currency {locale, currency}`, `policy.core.summary.timezone {timezone}`, `policy.identity.summary.sessions {idle_admin, idle_guardian, absolute_days}`, `policy.identity.summary.lockout {max_failed, window, lock}`, `policy.audit.summary.retention {years}`, `policy.audit.summary.anonymize {years}`. Translation catalog work remains with T-306.

**Dependencies affected** — T-114, T-115, T-117, T-117a, T-202, T-302a, T-410 and T-608 depend directly on T-113b. Shared rate limiting remains T-115's responsibility; OpenAPI generation remains T-114's; translation catalog work remains T-306's.

**Rollback plan** — Revert the single T-113b commit. No migration or persistent schema change was added. Drop only task-created `ritma_test_T-113b` after tests; retain pre-existing `ritma-db-1`.

**Follow-ups** — Maintainer to ratify TD-187–193; decide whether CQ-101 warrants a follow-up split/refactor before merge; T-114 to generate OpenAPI and its contract; T-115 to apply the shared limiter; T-306 to add the six TST-09 policy strings to both catalogs; preset-loader task to support nonblank preset versions.

## Questions
- **Self-resolved (implementer, run 25, needs architect ratification; TD-187):** the task's implementation ACs and endpoint contracts have no corresponding source paths in `## Files` → add the minimum route, module permission, audit repository/API, CSV guard and RBAC fixture paths needed to complete the existing Scope.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-188):** audit pagination names `limit` and `cursor` without bounds or encoding → default to 50 rows, cap a list at 200, stream 200-row chunks, bound filter strings at 128 characters and use a decimal tenant-local sequence cursor; each numeric bound is technical.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-189):** the export must be streamed, but no direct response-stream utility is listed in the workspace → add `futures-util` for bounded response chunks, as the existing spec-named CSV crate handles encoding.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-190):** direct `public.tenants` access is denied to `eq_app` and bypasses the shipped Core boundary → use the existing `public.resolve_tenant_name` function inside the tenant-scoped transaction, which validates the tenant context.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-191):** module initialization cannot resolve policy patches because the current runtime has no preset loader → recognize only spec 05's `blank` preset version 1 as no patch, and keep unknown preset ids fail-closed; use blank for the module-toggle integration fixture.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-192):** the task DoD requests `make api`, but the Makefile intentionally defers OpenAPI generation to dependent T-114 → run and report the placeholder, keep generator work in T-114, and retain the task dependency order.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-193):** the task's assigned endpoints, permissions and tests exceed CQ-101's approximate 400 changed-line review guide → preserve all assigned acceptance criteria and tests, and flag the size for maintainer review without editing sibling task files.

## Review
n/a — pending review after implementation.
