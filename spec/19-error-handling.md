# 19 — Error Handling

This document is the **single standard for errors**, from a domain rule being broken to the message a parent reads on their phone. It consolidates and supersedes the scattered rules: ARC-12 (the `AppError` model), API-04 (problem+json), CQ-22/66 (code-level rules), CA-03 (errors across layers) and SBD-52 (what clients may see). Where those disagree, this file wins, and they point here.

Every task follows it. Tags: **[tool]** = CI enforces it, **[review]** = the reviewer checks it.

## 1. Principles

- **ERR-01. Errors are part of the contract.** Every failure a user or client can hit has a **stable `code`**, an HTTP status and a translated message. Codes are API: never renamed, never reused for something else.
- **ERR-02. Expected vs unexpected.**
  - *Expected* failures are typed and handled: validation, a taken slot, a missing permission, an expired invite. They're never logged as errors.
  - *Unexpected* failures are bugs or outages. They become `internal` (500), are logged once with full context, and show the user a `request_id`.
- **ERR-03. Fail fast, fail closed.** Validate at the boundary (SBD-20/44). In security checks, an error means **deny** (SBD-01). Never continue with a half-valid value or a default that hides a failure.
- **ERR-04. One translation per layer edge.** The domain says *what rule broke*, the service adds *context*, the HTTP adapter decides *status and shape*. No layer reaches past its neighbour (CA-03).
- **ERR-05. Errors never leak.** No PII, secrets, SQL, stack traces or file paths reach the client, and no PII or secrets reach the logs (SBD-52/54, CQ-26).

## 2. Taxonomy and HTTP mapping

**ERR-10.** Every error belongs to exactly one class. `AppError` (in `eq_core::error`) is the closed set the HTTP edge understands.

| Class | `AppError` variant | HTTP | `code` | Retry? | Log level |
|---|---|---|---|---|---|
| Malformed request | `BadRequest { code, detail }` | 400 | `bad_request`, `token_invalid`, … | No | `debug` |
| Not authenticated | `Unauthenticated { code }` | 401 | `unauthenticated`, `invalid_credentials`, `session_expired`, `totp_required` | After re-auth | `info` (security event, SBD-53) |
| Not allowed (permission) | `Forbidden { perm }` | 403 | `forbidden` (+ `perm`) | No | `info` |
| Not allowed (rule) | `ForbiddenRule { code, detail }` | 403 | `marks_locked`, `cancel_window_passed`, `not_eligible`, … | No | `debug` |
| Not found / out of scope | `NotFound { entity }` | 404 | `not_found` | No | `debug` |
| Module off | `ModuleDisabled { module }` | 404 | `module_disabled` (+ `module`) | No | `debug` |
| State conflict | `Conflict { code, detail, extra }` | 409 | `slot_taken`, `version_conflict`, `invalid_transition`, `already_checked_in`, `last_owner`, `unpaid_overdue`, `idempotency_mismatch`, … | After the user decides | `debug` |
| Gone | `Gone { code }` | 410 | `invite_expired` | No | `debug` |
| Payload too large | `PayloadTooLarge` | 413 | `payload_too_large` | No | `debug` |
| Semantic rule on the request | `Unprocessable { code, detail, extra }` | 422 | `module_dependency`, `module_required_by`, … | No | `debug` |
| Field validation | `Validation { errors }` | 422 | `validation` (+ `errors[]`) | No | `debug` |
| Policy rule | `PolicyViolation { rule, detail, extra }` | 422 | `policy_violation` (+ `rule`) | No | `debug` |
| Account locked | `Locked { until }` | 423 | `locked` (+ `until`) | After `until` | `info` |
| Missing precondition | `PreconditionRequired` | 428 | `precondition_required` | With `If-Match` | `debug` |
| Rate limited | `RateLimited { retry_after_s }` | 429 | `rate_limited` + `Retry-After` | After the delay | `info` |
| Temporarily unavailable | `Unavailable { retry_after_s }` | 503 | `unavailable` + `Retry-After` | Yes | `warn` |
| Bug / unexpected | `Internal(anyhow::Error)` | 500 | `internal` (detail hidden) | Maybe, idempotent ops only | `error` (once, at the edge) |

**Notes:**
- Use 404 (not 403) for objects outside the actor's scope, so their existence isn't revealed (SBD-41).
- Module toggling uses **422 `module_dependency` / `module_required_by`**. The request asks for a set of modules that breaks MOD-03; that's a semantic error, not a state conflict.
- `Unavailable` is for DB pool exhaustion or connection loss, and for a required provider being down when the request can't be queued. It's never used for bugs.

## 3. The error code registry

**ERR-20. Code format.** Codes are `snake_case` ASCII, `^[a-z][a-z0-9_]{2,47}$`, and describe the *situation*, not the HTTP status: `slot_taken`, not `conflict_1`. The problem `type` URI is `PROBLEM_BASE` + the code with `_` → `-` (T-002).

**ERR-21. One registry, declared in code.**
- Each crate declares its codes in `src/errors.rs` as `pub static` items of type `ErrorCodeDef { code, status, title }`, plus `pub const CODES: &[&ErrorCodeDef]` listing them. `eq-core` holds the shared ones (`not_found`, `validation`, `internal`, …).
- Handlers and services refer to the constant, never to a string literal: `AppError::Conflict { code: &codes::SLOT_TAKEN, … }` (the variant carries the `&'static ErrorCodeDef`, so `eq-core` reads the title from it). [review]
- `eq-api` merges every compiled crate's `CODES` into the OpenAPI document as the extension `x-error-codes: [{code, status, title}]` (T-114). That makes `web/openapi.json` the published registry, covered by the contract-drift check (TST-10).
- A code is defined **once**. Two crates needing the same situation reuse the owner's code or a core code. [tool: a test in `eq-api` fails on duplicates or invalid formats]

**ERR-22. Translation.** Every registered code has `errors.<code>` in `en` **and** `id` (TST-09). `pnpm i18n:check` reads `x-error-codes` from `web/openapi.json` and fails on a missing key (T-306). [tool]

**ERR-23. Lifecycle.**
- Adding a code means: register it, translate it, add an API test that triggers it, and mention it in the module spec's endpoint table.
- Removing or renaming a code is a **breaking API change** (API-13). Deprecate first, keep emitting it for one minor version, and note it in the changelog.

## 4. Problem response shape

**ERR-30.** Errors are RFC 9457 `application/problem+json` (API-04):

```json
{
  "type": "https://ritma.dev/problems/slot-taken",
  "title": "Slot already taken",
  "status": 409,
  "code": "slot_taken",
  "detail": "Room 2 is booked 15:00–15:45.",
  "request_id": "0192f…",
  "conflicts": [ … ],
  "alternatives": [ … ]
}
```

- `title` is a fixed English phrase per code, taken from `ErrorCodeDef`. `detail` is English, for developers and logs, and **free of PII**. Use ids or neutral nouns ("Room 2", "the selected slot").
- Extension members (`conflicts`, `alternatives`, `perm`, `module`, `rule`, `until`, `missing`) are documented per code in the module spec and typed in OpenAPI.
- `request_id` is always present when the request passed the request-id layer (ARC-04).

**ERR-31. Field errors** (`validation`, and policy errors that point at a field):
- Each entry is `{ "pointer": "/cycle/due_day", "code": "must_be_gte", "params": { "other": "/cycle/issue_day" } }`.
- `pointer` is a JSON Pointer (RFC 6901) into the **request body**.
- Field codes come from a small, closed shared set (VAL-03): `required`, `invalid_type`, `too_short`, `too_long`, `too_few_items`, `too_many_items`, `must_be_gte`, `must_be_lte`, `invalid_format`, `invalid_characters`, `not_allowed`, `not_unique`, `unknown_reference`. JSON Schema keywords map to them per VAL-22. The SPA translates them as `errors.field.<code>` (POL-10).
- All field errors are returned together in one response, never one at a time.

## 5. Rust: per-layer rules

**ERR-40. Domain (`domain/`).**
- It returns typed enums that name the rule broken: `enum LeaveError { OverAnnualLimit { max_days: u16 }, StartsInPast, … }`.
- There's no `AppError`, HTTP, `anyhow` or logging in the domain.
- It panics only on *programmer* errors that the type system should have prevented, and those are rare (CQ-21 `panic` lint).

**ERR-41. Application (`service.rs`, jobs, handlers).**
- One `thiserror` enum per crate boundary, `<Area>Error`. It wraps domain errors and infrastructure errors with context.
- It implements `From<<Area>Error> for AppError` in **one place** per crate (`errors.rs`), with an exhaustive `match`: no `_ =>` arm (CQ-22).
- Use `?` for propagation. Add context where the cause alone is ambiguous: `.context("load invoice for late fee")` in `anyhow` paths, or a variant field in typed paths.
- **Never swallow an error.** `let _ =`, `.ok()` and `unwrap_or_default()` on a `Result` require a comment saying why losing the error is correct (CQ-22).

**ERR-42. HTTP edge (`http.rs`, `eq-api`).**
- Handlers return `Result<T, AppError>`. Rendering happens only in `impl IntoResponse for AppError`.
- The `Internal` variant is logged **here, once**, at `error`, with the full chain, `request_id`, `tenant_id`, `actor_id` and route. No other layer logs an error it also returns. That avoids logging the same failure several times.
- A `CatchPanicLayer` turns panics into `internal` responses and logs them. A panic in a request is always a bug and gets a follow-up task.

**ERR-43. Database errors** are mapped in `repo.rs` or the service, never passed raw:

| SQLSTATE / sqlx | Meaning | Mapping |
|---|---|---|
| `RowNotFound` | Missing or out of RLS scope | `NotFound` |
| `23505` unique_violation | Duplicate | `Conflict`, with the code chosen **by constraint name** (`persons_email_bidx_key` → `email_taken`). An unknown constraint → `Internal` |
| `23P01` exclusion_violation | Overlapping hold | `Conflict { slot_taken }` with conflicts and alternatives (T-203) |
| `23503` foreign_key_violation | A referenced row is missing or still in use | `Unprocessable { unknown_reference }` on insert, `Conflict { resource_in_use }` on delete |
| `23514` check_violation | A domain bug: the domain should have caught it | `Internal`, and add a domain check |
| `40001` / `40P01` | Serialization failure or deadlock | The service retries the **whole transaction** up to 2 times with jitter, but only if the use-case is idempotent (ERR-50). Otherwise `Unavailable` |
| `P0001` raised by an append-only trigger (SEC-07) | Forbidden mutation | `Internal` (a bug) |
| Pool timeout / connection lost | Database unavailable | `Unavailable { retry_after_s }` |

## 6. Retries, idempotency and async work

**ERR-50. Retry only what's safe.** A retry needs either an idempotent operation (GET, PUT with `If-Match`, POST with `Idempotency-Key`, API-07/08), or a transaction that was fully rolled back. Never retry a non-idempotent request automatically, on the server or the client.

**ERR-51. Jobs (ARC-09).**
- A job handler returns `Result<(), JobError>` with two variants:
  - `JobError::Retryable(anyhow::Error)`: transient. Retried with the configured backoff up to `max_attempts`.
  - `JobError::Permanent(anyhow::Error)`: bad data or a business-rule failure. No retry, so it goes straight to dead-letter.
- `last_error` stores a short, **PII-free** message.
- Dead-lettered jobs and events create an `attention_items` row and an alert (OPS-08).

**ERR-52. Event handlers (ARC-07).** These are idempotent through `event_receipts`. The same retryable/permanent split applies. A handler for a disabled module returns `Ok` and skips (MOD-01). That's not an error.

**ERR-53. External providers** (SMTP, WhatsApp, gateways):
- Calls go through `eq_core::outbound` with timeouts (SBD-47). Provider failures are classified retryable (timeouts, 5xx, 429) or permanent (4xx such as invalid recipient or rejected template).
- Notify applies retries, fallback channels and the circuit breaker (NTF-03/04).
- A failed provider call is **never** surfaced as `internal` to a user request when the work can be queued. The user's action succeeds, and the delivery status shows the problem.

## 7. Frontend error handling

**ERR-60. Typed problems.**
- `lib/api.ts` turns every non-2xx response into `ApiProblem { status, code, detail, requestId, extra }`.
- Network failures become `ApiProblem { code: "network" }`.
- Components never inspect raw `Response` objects or English `detail`.

**ERR-61. Where errors show** (FE-34):

| Situation | UI |
|---|---|
| `validation` / `policy_violation` | Inline under each field (mapped by `pointer`), plus focus moved to the first invalid field |
| `slot_taken` | The conflict panel with alternatives (FE-31) |
| `version_conflict` | "Someone changed this. Reload to see the latest" dialog, keeping the user's draft (FE-30) |
| `unauthenticated` / `session_expired` | Silent refresh once, then the login screen (FE-11) |
| `forbidden`, `not_found`, `module_disabled` on a page | A full-page `ErrorState` with a way back |
| `rate_limited`, `unavailable`, `network` | A toast with automatic retry for queries, and a manual "Try again" for mutations |
| `internal` | `ErrorState` / toast: "Something went wrong" + the `request_id` (copyable) |
| A render crash | A route-level error boundary showing `ErrorState`. Never a white screen |

**ERR-62. Messages.**
- Text comes from `t("errors.<code>", params)`, which falls back to `t("errors.unknown")` (never to the English `detail`, in production builds).
- Error copy follows BRAND §6: say what happened and what to do, no blame, no exclamation marks.
- Queries retry automatically only for `network`, `unavailable` and `rate_limited` (at most 2 times, with backoff). Mutations never auto-retry (ERR-50).
- No empty `catch {}` (CQ-66).

## 8. Logging, observability, audit

- **ERR-70.** Log levels follow the table in ERR-10. Expected failures stay at `debug`/`info`, so `error` in production always means "a human should look" (CQ-26).
- **ERR-71.** Every logged error carries `request_id` (or `job_id`/`event_id`), `tenant_id`, `actor_id` and `code`, and never S2/S3 data (SBD-10, SEC-21). An `internal` error's log line includes the `anyhow` chain.
- **ERR-72.** Security-relevant failures (repeated `invalid_credentials`, `forbidden` bursts, `locked`) are **audited** (SBD-53). Logging them isn't enough.
- **ERR-73.** Error counts per `code` and route are exposed as metrics (`/metrics`, OPS). A rising `internal` rate or dead-letter count raises an alert (OPS-08).

## 9. Testing errors

- **ERR-80.** Every registered code has at least one API test that triggers it and snapshots the problem body (`insta`, with `request_id` redacted). [review]
- **ERR-81.** Every endpoint's forbidden-role test (TST-04) asserts the exact `code`, not just the status.
- **ERR-82.** Domain error variants are unit-tested, including the boundary values (for example `max_days` and `max_days + 1`).
- **ERR-83.** The web tests cover ERR-61: inline field mapping, the conflict panel, the `version_conflict` dialog, and the error boundary rendering `ErrorState` with the request id.

## 10. Checklist: adding a failure case

1. Name the situation and choose its class and status (ERR-10).
2. Reuse an existing code if the situation is the same. Otherwise add an `ErrorCodeDef` to the owning crate's `errors.rs` (ERR-21).
3. Domain: add a typed variant (ERR-40). Service: map it in the crate's `From` impl (ERR-41).
4. Document the code and any extension members in the module spec's endpoint table.
5. Add `errors.<code>` in `en` + `id` (ERR-22).
6. Add the API test with a problem-body snapshot (ERR-80). Add the UI handling if a screen can hit it (ERR-61).
