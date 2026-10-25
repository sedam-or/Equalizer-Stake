---
id: T-104
title: Add TenantCtx resolution (single/multi mode), the request ActorCtx extractor and the setup-required gate
milestone: M1
status: todo
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

## Questions
