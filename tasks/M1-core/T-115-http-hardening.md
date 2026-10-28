---
id: T-115
title: Add Idempotency-Key replay, ETag/If-Match helpers, rate limits and security headers
milestone: M1
status: todo
depends_on: [T-101, T-104, T-111, T-113b]
spec_refs: [API-07, API-08, API-15, SEC-06, SEC-11, SEC-20, CAN-31, CAN-70, STM-16]
estimated_size: M
---

## Goal
The HTTP layer enforces the cross-cutting API rules once, so module handlers just opt in: safe retries of money and booking creation, optimistic concurrency, rate limits and browser security headers.

## Context
- API-08 idempotency:
  - Table `idempotency_keys(tenant_id, key, route, request_hash, status, body jsonb, created_at, PRIMARY KEY (tenant_id, key))` (created in T-101). TTL 24 h; a daily cleanup job deletes older rows.
  - `Idempotent<T>` wrapper/extractor:
    1. Reads the `Idempotency-Key` header (UUID; required on opted-in routes → 400 `bad_request` if missing or malformed).
    2. `request_hash = sha256(method || route_template || canonical body)`.
    3. If a row exists with the same hash → replay the stored status and body with header `Idempotent-Replayed: true`. A different hash → 422 `idempotency_mismatch`.
    4. Otherwise run the handler and store the result **in the same tx as the business change**: the handler receives an `IdempotencyGuard` and calls `guard.store(tx, status, &body)` before commit.
    5. A concurrent duplicate waits on `pg_advisory_xact_lock(hashtext(tenant||key))`.
- API-07 ETag:
  - `IfMatch` extractor: missing → 428 `precondition_required`. Malformed → 400. Parse `"v<int>"` → `expected_version: i32`.
  - The helper `etag(version) -> HeaderValue`.
  - Repos use `UPDATE … WHERE id=$1 AND version=$2 RETURNING version` and map 0 rows to `AppError::Conflict{code:&codes::VERSION_CONFLICT}`, with `current` attached by the handler (helper `version_conflict_with(current_dto)`).
- API-15 / SEC-11 rate limits (technical consts), via `tower_governor` keyed as follows:
  - General: 100/min per user (JWT `sub`), 300/min per IP.
  - `POST /auth/login`: 10/min per IP.
  - `/api/v1/public/*`: 20/min per IP.
  - Upload routes: 20/min per user. Expose `RateLimitLayer::uploads()` for later use.
  - 429 `rate_limited` with `Retry-After`. All responses carry `RateLimit-Limit`, `RateLimit-Remaining` and `RateLimit-Reset`.
  - The IP comes from `client_ip` (T-104), which honours trusted proxies only.
- SEC-20 headers on **every** response (API and SPA):
  - `Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: <tile-host>; connect-src 'self'; font-src 'self'; worker-src 'self' blob:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'`. `<tile-host>` comes from config `RITMA_MAP_TILE_HOST` (default `tile.openstreetmap.org`); add it to config if missing.
  - `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy: camera=(self), geolocation=(self), microphone=()`, `Cross-Origin-Opener-Policy: same-origin`.
  - HSTS `max-age=31536000; includeSubDomains`, only when `RITMA_PUBLIC_URL` is https (SEC-06).
- SEC-06: at startup, if `RITMA_PUBLIC_URL` is http and the host is not localhost or a private/link-local IP, log `warn!` with "Insecure public URL…" and set `meta/app.insecure_public_url = true` (add the field).

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `request_hash` = SHA-256 of `method \n route_template \n JCS(canonical body)` (`serde_jcs`, CAN-70), so harmless differences (whitespace, key order, `"Ayu "` vs `"Ayu"`) replay instead of returning `idempotency_mismatch` (D-14).
  - Every `/api` response gets `Cache-Control: no-store`, except signed file downloads and `/meta/app` (`max-age=60`) (CAN-31, D-10).

## Scope
### In
- `eq_core::http::{Idempotent, IdempotencyGuard, IfMatch, etag, security_headers_layer, rate_limit}` and their wiring in `eq-server`'s router.
- Apply `IfMatch` to `PATCH /me`, `PATCH /people/{id}` and `PUT /settings/policies/{module}`, replacing any ad-hoc parsing from T-111–T-113.
- The `core.idempotency_cleanup` cron (`15 4 * * *`).

### Out
- Captcha and honeypot (ENR-03, M4).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-server/tests/http_hardening.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T2, T4 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `security_headers_present` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Same key + same body twice → the second response equals the first and has `Idempotent-Replayed: true`. Only one business row. → `http::tests::idempotent_replay` (use a test route that inserts a row)
- AC2. Same key + different body → 422 `idempotency_mismatch`. → `idempotency_mismatch`
- AC3. 10 concurrent requests with the same key → 1 business row, 10 identical responses. → `idempotency_concurrent`
- AC4. The 11th login within a minute from one IP → 429 with `Retry-After`. → `login_rate_limited`
- AC5. All SEC-20 headers present on `/api/v1/meta/app` and on `/` (SPA). HSTS absent for http and present for https. → `security_headers_present`
- AC6. `X-Forwarded-For` from an untrusted peer is ignored for rate limiting. → `xff_untrusted_ignored`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/http_hardening.rs` | AC names above |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated (headers documented)
- [ ] Commit `feat(core): idempotency, concurrency, rate limits and headers [T-115]`, signed off

## Report

**Deviations** — RATIFY: TD-063, dependency references now target the final split part that provides the referenced contract.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** this task depended on the original parent contract after its CQ-101 split → depend on the final implementation part that provides the contract, because the parent now owns only the split map.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
