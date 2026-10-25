---
id: T-104
title: Add TenantCtx resolution (single/multi mode), the request ActorCtx extractor and the setup-required gate
milestone: M1
status: review
depends_on: [T-101, T-004]
spec_refs: [ARC-04, ARC-06, SEC-05]
estimated_size: M
---

## Goal
Every request resolves exactly one tenant, or returns 404. Handlers get a `TenantInfo` and a request `ActorCtx` to pass to `core::db::begin` (which T-003 already provides), so RLS is always in force.

## Context
- ARC-04 step 3:
  - In `RITMA_MODE=single`, the tenant is the single row (`resolve_single_tenant()`).
  - In `RITMA_MODE=multi`, it is the left-most label of the `Host` header under `RITMA_BASE_DOMAIN` (e.g. `ayu.app.example.com` → `ayu`), looked up by `resolve_tenant_by_slug`.
  - Unknown tenant → 404 `not_found` problem. A `suspended` tenant → read-only: methods other than GET/HEAD return 423 `locked` (CLD-35).
  - In single mode with **no** tenant row yet, `TenantCtx` resolves to `TenantCtx::Unprovisioned`. Only `/healthz`, `/readyz`, `/api/v1/meta/app` and `/api/v1/setup/*` accept it; everything else returns 503 `setup_required`.
- **Already done in T-003 (reuse, do not redefine):** `core::db::{Tx, begin(pool, TenantId, &ActorCtx), begin_system}`, the pool `after_release` `RESET ALL` hook, and `eq_core::actor::{ActorCtx, ActorRole}`. This task adds the HTTP-side pieces that produce those inputs.
- `X-Forwarded-*` is honoured only from `RITMA_TRUSTED_PROXIES` (from T-002 config).
- Tenant lookups are cached in `moka` (TTL 60 s), keyed by slug.

## Scope
### In
- `eq_core::tenant::{TenantCtx, TenantResolver, TenantInfo {id, slug, status, timezone, locale, currency}}` and an axum extractor for `TenantCtx`, plus `RequireTenant`.
- `ActorCtx` additions: `ActorCtx::anonymous(ip, request_id)`, `ActorCtx::system(job_kind: &'static str)` (if T-003 didn't add it), and an axum extractor `RequestActor` that fills `ip` (via `client_ip`) and `request_id` (from the request-id layer) with role `Anonymous`. **Add `ActorRole::Anonymous`** if missing. T-110 upgrades it to an authenticated actor.
- A `client_ip` helper that respects trusted proxies.
- The suspended-tenant read-only layer and the unprovisioned gate.

### Out
- Auth/JWT (`Actor` with permissions, T-110). Rate limiting (T-115).

## Files
| Path | Action |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `crates/eq-core/Cargo.toml` | declare the task-required tenant cache dependency in the workspace |
| `migrations/0009_core_tenant_resolver.sql` | add a function-only SECURITY DEFINER resolver for the existing tenant columns |
| `crates/eq-core/src/tenant.rs`, `crates/eq-core/src/lib.rs` | create and export tenant context/resolver types |
| `crates/eq-core/src/actor.rs`, `crates/eq-core/src/config.rs` | add anonymous actor/client-IP config support |
| `crates/eq-api/src/lib.rs`, `crates/eq-api/src/tenant.rs` | wire the resolver and request gates into the router |
| `crates/eq-server/src/main.rs` | attach peer address information for trusted-proxy handling |
| `crates/eq-core/tests/tenant.rs`, `crates/eq-api/tests/tenant.rs` | create resolver and end-to-end router acceptance tests |
| `crates/eq-api/tests/health.rs`, `crates/eq-api/tests/spa.rs`, `crates/eq-testkit/src/api.rs` | keep API tests and the shared router harness compatible with tenant resolution |
| `.sqlx/` | refresh offline query metadata for the added compile-time queries |

## Interfaces
```rust
pub enum TenantCtx { Resolved(Arc<TenantInfo>), Unprovisioned }
impl<S> FromRequestParts<S> for TenantCtx where AppState: FromRef<S> { … }
pub struct RequireTenant(pub Arc<TenantInfo>); // extractor; Unprovisioned → 503 setup_required
pub struct RequestActor(pub ActorCtx);         // extractor; anonymous actor with ip + request_id
// usage in services: core::db::begin(&pool, tenant.id, &actor)   (T-003)
```

## Implementation notes
- `TenantInfo.timezone/locale/currency` come from the denormalised `tenants` columns added in T-101. T-113 keeps them in sync with `CorePolicy`. Location-level timezone overrides are handled by callers (`locations.timezone`).
- In multi mode, a host without the base domain → 404. Ports are stripped. Host matching is case-insensitive.
- Use `AppError::SetupRequired` (503 `setup_required`) and `AppError::Locked { until: None }` (423 `locked`); both variants and the `setup_required` code are defined in T-002 (do not redefine them).

## Security
- Tenant identity comes only from the validated Host/config resolution path; never from request body or query data (SBD-42).
- Use `resolve_single_tenant()` / `resolve_tenant_by_slug()` for tenant lookup. Do not read `tenants` directly through the `eq_app` pool; it has no RLS and direct access is revoked by T-101 (SEC-05).
- Honor forwarded host/client-IP data only when the socket peer is in `RITMA_TRUSTED_PROXIES` (SBD-20). Without trusted peer information, use the socket peer and ordinary Host header.
- A missing tenant is unprovisioned only in single mode; only the explicitly listed health/meta/setup paths pass the setup gate. A suspended tenant allows only GET/HEAD.
- Abuse tests cover forged forwarded headers from untrusted peers, tenant-id injection attempts, unprovisioned-route restrictions, suspended writes, and transaction tenant context (SBD-42/73/81).

## Acceptance criteria
- AC1. Single mode with one tenant → `TenantCtx::Resolved`. → `tenant::tests::single_mode_resolves`
- AC2. Multi mode `Host: ayu.app.test` with slug `ayu` → resolved. `Host: x.other.test` → 404. → `multi_mode_by_host`
- AC3. A handler using `RequireTenant` + `RequestActor` + `begin` sees `current_setting('app.tenant_id')` equal to the resolved tenant (end-to-end through the router). → `tenant::tests::handler_tx_scoped`
- AC4. A suspended tenant + POST → 423. → `suspended_is_read_only`
- AC5. No tenant + GET `/api/v1/people` → 503 `setup_required`. GET `/readyz` → 200. → `unprovisioned_gate`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/tenant.rs` | `single_mode_resolves`, `multi_mode_by_host`, `handler_tx_scoped`, `app_role_can_resolve_metadata_without_table_select` |
| `crates/eq-api/tests/tenant.rs` | `suspended_is_read_only`, `unprovisioned_gate` |

## Definition of done
- [x] Code quality and architecture: spec/16-code-quality.md §12 and spec/17-clean-architecture.md followed (decisions in domain/, own tables only, no new #[allow]); check-layers.sh and check-web-boundaries.sh pass once T-006 has created them
- [x] Rust checks pass
- [x] scripts/erd.py --migrations passes for the resolver migration
- [x] Commit feat(core): tenant resolution and request actor [T-104], signed off

## Report

**Plan** — (1) Add the task-authorized function-only resolver migration and verify it against the ERD map and app-role privileges. (2) Add validated multi-mode base-domain configuration, tenant metadata resolution and the required slug cache. (3) Add tenant/request actor extractors, trusted-proxy address handling, suspended-write middleware and the unprovisioned setup gate. (4) Exercise database, API and server paths, complete the security review, and run the Rust, web, guard, SQLx, schema and E2E checks.

**Assumptions** — No table, column, permission key, error code, or business value is added. T-104's own Files section assigns the function-only resolver to migration 0009. The named RITMA_BASE_DOMAIN input is optional in single mode and required in multi mode; the existing Config missing-value error is reused.

**Test plan** —
- AC1: Given single mode with one tenant, when context resolves, then it is Resolved → single_mode_resolves.
- AC2: Given Host ayu.app.test and slug ayu, when multi-mode resolution runs, then it resolves; a host outside the base domain returns 404 → multi_mode_by_host.
- AC3: Given a routed handler, when RequireTenant and RequestActor call begin, then current_setting('app.tenant_id') equals the resolved tenant → handler_tx_scoped.
- AC4: Given a suspended tenant and POST, when the tenant layer runs, then it returns 423 locked → suspended_is_read_only.
- AC5: Given single mode with no tenant, GET /api/v1/people returns 503 setup_required while GET /readyz remains 200 → unprovisioned_gate.
- Additional abuse and edge coverage: direct app-role table access is denied; host matching is case-insensitive and strips ports; out-of-domain and nested labels are rejected; forwarded host/IP values from untrusted peers are ignored; trusted proxy chains are walked from the socket inward; only GET/HEAD are allowed for suspended tenants; only the declared setup routes bypass the unprovisioned gate; absent socket peer data does not trust forwarding headers.
- Error cases: unknown/out-of-domain host → 404 not_found; unprovisioned required tenant → 503 setup_required; suspended write → 423 locked; SQL failures use existing AppError mapping.

**Summary** — Implemented single- and multi-mode tenant resolution, the anonymous request actor, trusted-proxy-aware client IP and host handling, and request gates for suspended or unprovisioned tenants. Tenant metadata is fetched only through a pinned SECURITY DEFINER function and the app role still has no direct SELECT on the un-RLS tenants table.

**Spec read** — spec/02-architecture.md §§4–5 (ARC-04/06); spec/06-domain-model.md §4.1; spec/10-security-privacy.md SEC-05; spec/16-code-quality.md §§9/12; spec/17-clean-architecture.md §§1/4/7; spec/18-security-by-design.md §§2/4/5/8; spec/19-error-handling.md §§2/4/5; spec/29-erd.md §§4–5/7; WORKFLOW.md §§3–4.

**Files** —
- Added migrations/0009_core_tenant_resolver.sql and two SQLx offline query metadata files.
- Added crates/eq-core/src/tenant.rs and crates/eq-core/tests/tenant.rs; updated eq-core actor, config, exports and manifest.
- Added crates/eq-api/src/tenant.rs and crates/eq-api/tests/tenant.rs; updated API state, health router, API tests and the shared test router.
- Updated eq-server to attach socket peer information; updated Cargo.lock, the task file and tasks/INDEX.md.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | single_mode_resolves | Passed. |
| AC2 | multi_mode_by_host | Passed. |
| AC3 | handler_tx_scoped | Passed; the transaction saw the resolved tenant setting. |
| AC4 | suspended_is_read_only | Passed; POST returned 423 locked. |
| AC5 | unprovisioned_gate | Passed; API path returned 503 setup_required and /readyz returned 200. |
| Resolver privilege | app_role_can_resolve_metadata_without_table_select | Passed against the eq_app role. |

**Checks run** —
- ✅ scripts/tasks-index.sh --check and git diff --check.
- ✅ python3 scripts/erd.py --migrations and python3 scripts/erd.py --check.
- ✅ cargo sqlx prepare --workspace against the local PostgreSQL 16 database; refreshed and committed .sqlx metadata.
- ✅ cargo fmt --all --check.
- ✅ SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings.
- ✅ SQLX_OFFLINE=true cargo test --workspace; all unit, API, CLI, database integration and doc tests passed.
- ✅ scripts/check-deps.sh, scripts/check-literals.sh, scripts/check-web-boundaries.sh and scripts/check-layers.sh.
- ✅ cargo deny check advisories, cargo audit, pnpm -C web audit --prod and typos.
- ✅ pnpm -C web lint, pnpm -C web typecheck and pnpm -C web test.
- ✅ make e2e; 6/6 browser cases passed on Chromium and mobile WebKit.
- ❌ gitleaks detect --redact; it reports the already documented false positive at Makefile:80 for Docker's non-secret --user flag. No scanner rule or allow-list was changed.
- ✅ gitleaks protect --staged --redact; no leaks found in the staged task changes.
- n/a make api; this task adds middleware and extractors but no OpenAPI operation. T-114 owns the contract generation target.
- n/a pnpm -C web i18n:check; no web source changed and the script is owned by T-306.
- Not run: hosted GitHub Actions; it requires the hosted CI environment.

**Manual checks** — With the local PostgreSQL 16 dev database and server running, provisioned-tenant requests returned 200 for /healthz, /readyz and /, and 404 not_found for the not-yet-implemented /api/v1/people endpoint. After removing the only tenant, /api/v1/people and / returned 503 setup_required while /readyz remained 200. Restored the disposable fixture and stopped the server. The integration tests also applied the migration to fresh test databases and verified metadata access through eq_app without direct table access.

Project-name audit: the canonical product mapping is the former project/display name → Ritma. README titles and copy in both languages, the SPA title/brand constant and the ritma CLI use the canonical name. A case-insensitive repository scan (excluding the legacy archive, git metadata and RUN_LOG) found 18 old-name matches across 11 files: six live repository URL/slug references, six historical brand/spec records and six earlier task reports. The configured GitHub repository still uses the existing slug; gh repo view confirmed that the planned ritmahq/ritma target does not resolve. Those URL and provenance references are retained, not shown as product labels.

**Debug log** — SQLx prepare first encountered a connection refusal when the local dev database was stopped during testcontainer cleanup. Restarted it with make dev-db, reran SQLx preparation and applied the migration; the retry passed. No code failure remained.

**Performance** — The task-required 60-second slug cache is a named technical constant. No new list query was added and no p95 benchmark was recorded; performance profiling is a follow-up if the resolver becomes a measured hot spot.

**Security** — Triggers include request trust boundaries and tenant isolation. SBD-81 review: (1) tenant resolution precedes tenant handlers; auth remains the responsibility of T-110, and this task adds no protected data handler; (2) tenant identity comes from Host/config, no client tenant ID is used, there is no new table, and app-role lookup uses only the resolver function; (3) no stored data class or PII field is added and the function returns only existing tenant metadata; (4) base-domain config and Host values are validated, with untrusted forwarded values ignored; (5) existing problem responses expose stable codes without tenant metadata; (6) no secret is added and Config/Resolver Debug output does not expose credentials; (7) no external call is added; (8) no state mutation is added; (9) no auditable state change is added; (10) tests cover spoofed forwarding, tenant transaction context, resolver-only access, suspended writes and setup-gate bypass attempts. SQL is parameterized, the definer search_path is pinned to pg_catalog, and direct eq_app table reads remain denied.

**Deviations** — RATIFY: (1) added the missing Files and Security sections using the task Scope and SBD-20/42/81; (2) added validated RITMA_BASE_DOMAIN to Config, optional in single mode and required in multi mode, using T-002's existing missing-config error; (3) the task required six metadata fields while T-101's resolvers exposed three, so added the function-only migration explicitly named by T-104's Files section, returning only existing columns through a pinned SECURITY DEFINER function. The original Q1 and its options remain below as history with the run-7 resolution.

**Dependencies affected** — T-105 directly depends on the new TenantCtx and request middleware. Later API tasks can use RequireTenant and RequestActor. No other task interface was changed.

**Rollback plan** — Revert the T-104 source and configuration in a task revert. Migrations are forward-only; after migration 0009 has been applied, remove the function only with a compensating migration that revokes its grant and drops this task-owned function. No tenant data is changed.

**Follow-ups** — Human review should assess the broad implementation size: the new resolver/API/test/migration files are 774 lines before the other touched manifests and adapters, though the task itself specifies the full feature and named abuse tests. Keep the currently valid GitHub slug URLs until the repository transfer is completed; then update Cargo metadata, branch helper and README badges. Review the known Makefile:80 gitleaks false positive without weakening secret detection.

**Step 0 — Start** — Resumed the task on milestone/m1-core at 20a8662, the prior signed blocker commit. The worktree contained this task's in-progress implementation; scripts/tasks-index.sh --check passed and the task was set to in_progress through its frontmatter followed by scripts/tasks-index.sh.

**Step 1 — Task and spec** — Read the whole T-104 contract, including the resolved migration path, Files, Security, interfaces, acceptance tests and DoD. Read the cited ARC-04/06 and SEC-05 contracts plus the mandatory CQ-16, CA-17, SBD-81 and ERR guidance.

**Step 2 — Dependencies** — Read T-101 and T-004 Reports, including Deviations, Follow-ups and RATIFY items, and T-002's Config/error Report. Inspected the shipped resolver migrations and app-role tests, Tenant model, API router/request-ID layer/server and Config. The dependencies' least-privilege resolver contract controls; no direct tenant-table grant was added.

**Step 3 — Design reference** — n/a; no UI.

**Step 4 — Plan and test plan** — Recorded above before implementation, mapping each AC to its named test and listing edge, error and abuse cases.

**Step 5 — Implement** — Added the task-authorized function-only resolver migration, SQLx metadata, validated base-domain setting, tenant metadata cache and extractors, anonymous actor support, trusted-proxy handling, request middleware and required tests. No table, column, permission key, error code or business value was added.

**Step 6 — Run locally** — Ran make dev with the local PostgreSQL database and issued real requests for health, readiness, the embedded SPA, an unimplemented API route and the unprovisioned path. The results are recorded under Manual checks.

**Step 7 — Debug loop** — One transient database connection failure interrupted SQLx preparation. Restarted the dev database and reran the migration/prepare workflow successfully; no repeated code failure or unresolved runtime issue.

**Step 8 — Self code review** — Reviewed the task diff against spec/16 §9, spec/17 §7, spec/19 and SBD-81. Confirmed module ownership, parameterized queries, bounded cache, no new #[allow], no request-path unwrap/expect, no raw tenant table access through eq_app, no new error code, and no new cross-module query. The full-history secret scan's inherited Makefile false positive is recorded above.

**Step 9 — Optimize** — Kept the task's required 60-second cache as a named technical constant. No list query or measured performance target was introduced; no speculative optimization was added.

**Step 10 — Security audit** — Completed SBD-81 review item by item and verified the abuse tests and eq_app privilege boundary. No secret or personal data was added.

**Step 11 — Revise and verify** — Reran the full Rust, web, guard, schema/ERD, SQLx and local E2E checks listed above; each implementation check passed except the inherited full-history gitleaks finding. Rechecked the task index and diff whitespace.

**Step 12 — Report, commit** — Filled the Report, retained Q1 and its resolution as history, set status to review and regenerated the task index. This report and task status are included in the one signed-off Conventional Commit required by the task.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** T-104 names `RITMA_BASE_DOMAIN` for multi-mode Host routing, but the shipped T-002 `Config` has no field for it → add an optional validated Config field, required only in multi mode, using the existing missing-config error; single mode keeps no base domain because the tenant is resolved from the database function.
- **Self-resolved (implementer, run 6, needs architect ratification):** The task has no Files or Security section despite naming source/test work and triggering SBD-30 → add the paths derived from Scope/Tests and the controls directly required by SBD-20/42/81, without adding permissions or new behavior.
- **Self-resolved (implementer, run 7, needs architect ratification):** The task requires six fields in `TenantInfo`, but T-101's resolvers expose only three and `tenants` has no RLS → use a function-only `0009_core_tenant_resolver.sql` in ERD-06's reserved migration gap, returning exactly the existing `id`, `slug`, `status`, `timezone`, `locale`, and `currency` columns through a pinned `SECURITY DEFINER` function granted to the already-authorized `eq_app` resolver role; this follows spec/06 §4.1 and spec/29 ERD-02/06, adds no table or column, and applies the user's authorization to handle this blocker in the relevant task files.
- **Q1 (historical blocker; resolved for implementation):** How can T-104 obtain `timezone`, `locale` and `currency` for `TenantInfo` while preserving tenant isolation and forward-only migrations?
  - Context: T-104 requires these values from T-101's denormalized tenant columns. The shipped `resolve_single_tenant()` and `resolve_tenant_by_slug()` return only `(id, slug, status)`. T-101 revokes direct table access from `eq_app`; `tenants` has no RLS, so a direct SELECT grant would expose every tenant. ERD-06 maps the existing tenant schema to T-101 migrations `0002_core_tenancy.sql`/`0003_core_policies_files.sql`; no T-104 resolver migration or number is assigned. Editing the committed `0002` migration violates forward-only migration handling.
  - Options: (a) expand `TenantInfo` to omit the three required fields, which contradicts T-104; (b) grant `eq_app` SELECT on `tenants`, which violates SEC-05 tenant isolation; (c) authorize a forward-only security-definer resolver returning exactly the six fields and assign its migration/function ownership in the task/ERD map.
  - Recommendation: (c), because it preserves the required `TenantInfo` contract without exposing the un-RLS tenant table or mutating an applied migration.
  - **Resolved for implementation (run 7):** The function-only resolver is assigned to T-104 in the open ERD-06 migration gap `0009`; `eq_app` receives EXECUTE on the pinned function and retains no direct table privilege. The question remains in history for architect ratification.
