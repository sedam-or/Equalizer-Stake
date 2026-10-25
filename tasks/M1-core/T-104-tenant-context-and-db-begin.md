---
id: T-104
title: Add TenantCtx resolution (single/multi mode), the request ActorCtx extractor and the setup-required gate
milestone: M1
status: blocked
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
| `crates/eq-core/src/tenant.rs`, `crates/eq-core/src/lib.rs` | create and export tenant context/resolver types |
| `crates/eq-core/src/actor.rs`, `crates/eq-core/src/config.rs` | add anonymous actor/client-IP config support |
| `crates/eq-api/src/lib.rs`, `crates/eq-api/src/tenant.rs` | wire the resolver and request gates into the router |
| `crates/eq-server/src/main.rs` | attach peer address information for trusted-proxy handling |
| `crates/eq-core/tests/tenant.rs` | create the five named acceptance tests |

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
| `crates/eq-core/tests/tenant.rs` | the AC names above |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(core): tenant resolution and request actor [T-104]`, signed off

## Report

**Plan** — (1) Add the task's missing Files/Security details and model the named multi-mode base-domain setting → verify it uses the shipped `Config`/`AppError` contracts. (2) Implement tenant lookup and a 60-second slug cache from T-101's resolver API → verify single and multi resolution and exact tenant metadata. (3) Add trusted-proxy client IP, anonymous request actor, setup and suspended-tenant gates → verify the five named router/database tests and spoofing cases. (4) Run the full Rust, database, guard and runtime checks, including real `make dev` requests. This plan cannot proceed because T-101's resolvers do not expose the tenant metadata T-104 requires, and there is no authorized forward-only migration for the missing resolver contract.

**Assumptions** — No new business values, permission keys, error codes, table/column definitions or migration numbers are inferred. The task's named `RITMA_BASE_DOMAIN` is an optional Config input in single mode and is required only in multi mode; T-002 already defines a safe missing-config error.

**Test plan** —
- AC1: Given single mode and exactly one tenant, when tenant context resolves, then it returns `Resolved`: `tenant::tests::single_mode_resolves`.
- AC2: Given `Host: ayu.app.test` and a tenant slug `ayu`, when multi-mode resolution runs, then that tenant resolves; a host outside the configured base domain returns 404: `multi_mode_by_host`.
- AC3: Given a routed handler, when `RequireTenant` and `RequestActor` call `begin`, then the transaction has the resolved tenant setting: `tenant::tests::handler_tx_scoped`.
- AC4: Given a suspended tenant and POST request, when the tenant layer runs, then it returns 423 `locked`: `suspended_is_read_only`.
- AC5: Given single mode with no tenant, when `/api/v1/people` and `/readyz` are requested, then the first returns 503 `setup_required` and the second remains 200: `unprovisioned_gate`.
- Edges: no/one/multiple tenants; Host with case changes or port; missing/wrong base domain; absent `ConnectInfo`; GET/HEAD vs write methods; health/meta/setup allowlist paths.
- Errors: unknown/out-of-domain host → 404 `not_found`; missing required tenant → existing 503 `setup_required`; suspended write → existing 423 `locked`; DB failures use existing `AppError` mapping.
- Abuse: forged forwarded headers from untrusted peers, tenant-id injection in body/query, cross-tenant lookup, direct `eq_app` reads of tenants, suspended writes and setup-gate bypass attempts.

**Summary** — Blocked before implementation. The required `TenantInfo` includes timezone, locale and currency, but the only T-101 resolver functions return id, slug and status. `eq_app` has no direct SELECT on `tenants`, which has no RLS. Returning the extra fields requires a security-definer resolver contract and a forward-only migration assignment that the current task/spec map does not provide.

**Spec read** — `spec/02-architecture.md` §4 (ARC-04) and §5 (ARC-06); `spec/10-security-privacy.md` SEC-05; `spec/16-code-quality.md` §§9/12; `spec/17-clean-architecture.md` §§1/4/7 (CA-01/30); `spec/18-security-by-design.md` §§2/4/5.3/8 (SBD-03/20/42/48/81); `spec/19-error-handling.md` §§2/4/5 (ERR-10/30/42/43); `spec/29-erd.md` §§4–5/7; `WORKFLOW.md` §§3–4.

**Files** — Updated only this task file and regenerated `tasks/INDEX.md`; no application source or migration was changed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `tenant::tests::single_mode_resolves` | Not run: blocked before implementation. |
| AC2 | `multi_mode_by_host` | Not run: blocked before implementation. |
| AC3 | `tenant::tests::handler_tx_scoped` | Not run: blocked before implementation. |
| AC4 | `suspended_is_read_only` | Not run: blocked before implementation. |
| AC5 | `unprovisioned_gate` | Not run: blocked before implementation. |

**Checks run** —
- ✅ `scripts/tasks-index.sh --check` after setting the task status to `blocked`.
- ✅ `git diff --check`.
- Not run: Rust, database, runtime, guard, Playwright and E2E checks; implementation stopped before code.
- Not run: `cargo sqlx prepare --workspace`; no SQL query or migration was changed.
- Not run: hosted GitHub Actions; requires the hosted CI environment.

**Manual checks** — Read the shipped T-101 migration and its tests: `resolve_single_tenant()` and `resolve_tenant_by_slug()` each return only `(id, slug, status)`, and the migration revokes direct `tenants` access from `eq_app`; T-101's test confirms direct app-role reads are denied. Read the T-004/T-002 reports and `Config`: `RITMA_BASE_DOMAIN` is named by T-104 but is not a field in the shipped Config. No database, server or browser test ran.

**Debug log** — No code failure occurred. Contract review identified that the requested `TenantInfo` cannot be assembled through the shipped least-privilege resolver surface. Direct table access would violate SEC-05; editing the already committed `0002_core_tenancy.sql` would mutate a migration, and a new migration number is not assigned by ERD-06.

**Performance** — n/a; no implementation or query was added.

**Security** — Triggers T1/T2/T4: request trust boundary, tenant isolation, proxy headers and database roles. No source change was made. The task's future Security section now records the derived controls and abuse tests: tenant only from Host/config, forwarded headers only from trusted peers, no direct `tenants` access via `eq_app`, unprovisioned allowlist and suspended-write rejection. SBD-81 implementation checks are not run because the required resolver contract is unresolved.

**Deviations** — RATIFY: (1) added the missing Files and Security sections from the existing Scope and SBD-20/42/81; (2) modeled the task-named `RITMA_BASE_DOMAIN` as an optional Config setting, required only in multi mode, using the existing Config missing-value error. The task is blocked independently on the missing tenant-info resolver/migration contract.

**Dependencies affected** — T-105 directly depends on T-104; its blocked status makes the remaining M1 tasks with transitive T-104 dependencies ineligible. M2/M3 integration chains remain dependent on T-117/T-213.

**Rollback plan** — Revert this signed task-status/question/report commit and regenerate the index. No source, migration or persistent data changed.

**Follow-ups** — Architect must authorize a forward-only, least-privilege resolver path for `(id, slug, status, timezone, locale, currency)` and map its migration/function ownership before T-104 can continue. Do not grant direct SELECT on the un-RLS `tenants` table.

**Step 0 — Start** — Started on clean `milestone/m1-core` at `eadfbb3`; `scripts/tasks-index.sh --check` passed. T-101 and T-004 were `review`; after blocked T-103, T-104 was the lowest-ID eligible task. Set it to `in_progress` and regenerated the index.

**Step 1 — Task and spec** — Read the entire task and cited ARC-04/06/SEC-05 contract plus the mandatory CQ-16, CA-17, SBD-81 and ERR-19 guidance. The task names the base-domain input and six tenant-info fields but does not list its Files or Security sections.

**Step 2 — Dependencies** — Read the full T-101 and T-004 Reports, including Deviations, Follow-ups and RATIFY items, and T-002's Config/error Report because T-004 inherits that interface. Inspected the T-101 resolver migration, `Tenant` model, app-role test, T-004 router/request-id layer/server and T-002 `Config`. T-101's resolver output and revoked table permissions are the controlling shipped contract; its existing RATIFY items do not authorize broader tenant access.

**Step 3 — Design reference** — n/a; no UI.

**Step 4 — Plan and test plan** — Recorded above before implementation, mapping every AC, edge, error and abuse case.

**Step 5 — Implement** — n/a; blocked before implementation by the resolver/migration contract.

**Step 6 — Run locally** — n/a; no request behavior was implemented.

**Step 7 — Debug loop** — n/a; no implementation failure. Re-read the blocker: no direct tenant SELECT and no authorized forward-only migration give one safe path to the missing metadata.

**Step 8 — Self code review** — n/a; no source diff.

**Step 9 — Optimize** — n/a; no code or query.

**Step 10 — Security audit** — Reviewed the SBD-81 tenant/proxy controls. No source change or abuse test exists to audit.

**Step 11 — Revise and verify** — n/a; implementation is blocked. Task index and diff hygiene checks pass.

**Step 12 — Report, commit** — Recorded Q1 with options and recommendation, status set to `blocked`, index regenerated. The signed-off blocker commit is pushed on `milestone/m1-core`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** T-104 names `RITMA_BASE_DOMAIN` for multi-mode Host routing, but the shipped T-002 `Config` has no field for it → add an optional validated Config field, required only in multi mode, using the existing missing-config error; single mode keeps no base domain because the tenant is resolved from the database function.
- **Self-resolved (implementer, run 6, needs architect ratification):** The task has no Files or Security section despite naming source/test work and triggering SBD-30 → add the paths derived from Scope/Tests and the controls directly required by SBD-20/42/81, without adding permissions or new behavior.
- **Q1 (blocking):** How can T-104 obtain `timezone`, `locale` and `currency` for `TenantInfo` while preserving tenant isolation and forward-only migrations?
  - Context: T-104 requires these values from T-101's denormalized tenant columns. The shipped `resolve_single_tenant()` and `resolve_tenant_by_slug()` return only `(id, slug, status)`. T-101 revokes direct table access from `eq_app`; `tenants` has no RLS, so a direct SELECT grant would expose every tenant. ERD-06 maps the existing tenant schema to T-101 migrations `0002_core_tenancy.sql`/`0003_core_policies_files.sql`; no T-104 resolver migration or number is assigned. Editing the committed `0002` migration violates forward-only migration handling.
  - Options: (a) expand `TenantInfo` to omit the three required fields, which contradicts T-104; (b) grant `eq_app` SELECT on `tenants`, which violates SEC-05 tenant isolation; (c) authorize a forward-only security-definer resolver returning exactly the six fields and assign its migration/function ownership in the task/ERD map.
  - Recommendation: (c), because it preserves the required `TenantInfo` contract without exposing the un-RLS tenant table or mutating an applied migration.
