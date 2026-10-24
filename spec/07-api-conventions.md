# 07 — API Conventions

**API-01. Base.** JSON over HTTPS. All endpoints are under `/api/v1`. Public, unauthenticated endpoints are under `/api/v1/public/*` or explicitly marked `auth: none`. Names: plural nouns, kebab-case path segments, `snake_case` JSON fields. IDs are UUID strings.

**API-02. Contract-first via code.** Every handler is annotated with `#[utoipa::path]`, and every DTO derives `ToSchema`. `eq-server openapi > web/openapi.json` dumps the spec (OpenAPI 3.1). CI fails if the committed `web/openapi.json` differs from the generated one (TST-10). The SPA uses `openapi-typescript` and `openapi-fetch`, so it can't call an endpoint or field that doesn't exist.

**API-03. Auth.** `Authorization: Bearer <access JWT>` (IDN-03). Cookie-based endpoints are listed in IDN-04/05. Unauthenticated → 401 problem.

**API-04. Errors.** RFC 9457 `application/problem+json`. The complete standard (taxonomy, code registry, field errors, retries, UI handling) is `19-error-handling.md`. This section is the summary.

```json
{
  "type": "https://ritma.dev/problems/slot-taken",
  "title": "Slot already taken",
  "status": 409,
  "code": "slot_taken",
  "detail": "Room 2 is booked 15:00–15:45.",
  "request_id": "0192f…",
  "conflicts": [ … ],          // extension members per code
  "alternatives": [ … ]
}
```

| HTTP | `code` examples | When |
|---|---|---|
| 400 | `bad_request`, `token_invalid` | Malformed input |
| 401 | `unauthenticated`, `invalid_credentials`, `session_expired`, `totp_required` | |
| 403 | `forbidden` (+ `perm`), `marks_locked`, `cancel_window_passed`, `not_eligible` | |
| 404 | `not_found`, `module_disabled` (+ `module`) | |
| 409 | `slot_taken`, `version_conflict`, `invalid_transition`, `already_checked_in`, `last_owner`, `unpaid_overdue`, `idempotency_mismatch` | |
| 410 | `invite_expired` | |
| 428 | `precondition_required` (missing If-Match) | |
| 413 | `payload_too_large` | |
| 422 | `validation` (+ `errors: [{pointer, code, params}]`), `policy_violation` (+ `rule`, the policy key path), `module_dependency` / `module_required_by` (+ `missing`) | |
| 423 | `locked` (+ `until`) | |
| 429 | `rate_limited` (+ `Retry-After`) | |
| 500 | `internal` (detail hidden; `request_id` shown) | |
| 503 | `unavailable` (+ `Retry-After`) | Database or required provider temporarily unavailable |

`detail` is English for logs and developers. The SPA displays `t("errors.<code>", params)` and falls back to `detail`.

**API-05. Pagination.** Cursor-based: `?limit=50&cursor=<opaque>`, with a max limit of 200. Response: `{ "items": [...], "next_cursor": "..." | null }`. The cursor encodes `(sort_key, id)` as base64url JSON. List endpoints that need totals (dashboards) use a dedicated `/count` endpoint.

**API-06. Filtering and sorting.** Explicit query params per endpoint (documented in OpenAPI), plus `sort=field` / `sort=-field` from an allow-list. Free text: `q` (trigram on `display_name` / number).

**API-07. Concurrency.** Mutable aggregates return `ETag: "v<version>"`. Mutations require `If-Match: "v<version>"`. A mismatch returns 409 `version_conflict` with the current resource in `current`. A missing header returns 428 `precondition_required`.

**API-08. Idempotency.** `POST` endpoints that create money or bookings accept `Idempotency-Key: <uuid>` (payments, sessions, reservations, registrations, invoices). The server stores `(tenant, key, route, response_status, response_body_hash, response_body)` for 24 h in `idempotency_keys` and replays the same response for a repeated key with the same body, or returns 422 `idempotency_mismatch` for a different body. *(Table `idempotency_keys(tenant_id, key, route, request_hash, status, body jsonb, created_at, PRIMARY KEY (tenant_id, key))`.)*

**API-09. Time.** Instants are RFC 3339 with offset (`2026-11-05T00:05:00+07:00` in responses, in tenant TZ). Requests accept any offset. Local dates are `YYYY-MM-DD`, local times `HH:MM`. Every response that contains instants also provides `local_*` convenience fields where useful (Session DTO).

**API-10. Money.** `{ "amount_minor": 60000000, "currency": "IDR" }`. Never floats. The SPA formats with `Intl.NumberFormat`.

**API-11. I18n fields.** I18nText is returned as the full map `{ "en": "...", "id": "..." }` on admin endpoints, and **resolved** to a string on portal and public endpoints (by `Accept-Language`, then the person locale, then the tenant locale).

**API-12. Uploads.** `multipart/form-data` with the part `file` and an optional `json` part. Size limits are per purpose. Content type is sniffed server-side (magic bytes) against an allow-list. Responses return `{file_id}`. Downloads use signed URLs (SEC-15).

**API-13. Versioning.** `/api/v1` is stable within major 1. Additive changes are allowed. Breaking changes go to `/api/v2` with a deprecation period of ≥ 2 minor releases, and are announced in the changelog.

**API-14. Meta endpoints.**
- `GET /api/v1/meta/modules` (MOD-09)
- `GET /api/v1/meta/app` → `{version, commit, mode: single|multi, brand: {name, logo_url, brand_hue, corner_style}, tenant: {name, locale, currency, timezone, terminology}, features: {captcha, guests}}`. It is public for the tenant branding on the login page.
- `GET /healthz` (liveness), `GET /readyz` (DB reachable, migrations applied), outside `/api/v1`.

**API-15. Rate limits.** Defaults (technical constants, SEC-11):
- 100 req/min per user and 300/min per IP generally.
- Login 10/min per IP plus lockout.
- `/public/*` 20/min per IP.
- Upload routes 20/min per user.

Responses carry `RateLimit-*` headers (draft-ietf-httpapi-ratelimit-headers).
