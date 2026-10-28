---
id: T-306
title: Add the typed API client, problem mapping, in-memory auth with single-flight refresh, i18n (ICU, terminology) and formatters
milestone: M3
status: todo
depends_on: [T-305]
spec_refs: [FE-10, FE-11, FE-12, FE-37, API-04, API-09, API-10, POL-07, PRD-15, VAL-30, AUTH-03, CACHE-30]
estimated_size: M
---

## Goal
Every screen talks to the server through one typed client, stays logged in safely (no tokens in storage), and renders text, money, dates and terminology correctly for the tenant.

## Context
- `src/lib/api.ts`: `openapi-fetch` `createClient<paths>({ baseUrl: '/api/v1' })` + middleware:
  1. Attach `Authorization: Bearer` from `auth.getToken()`.
  2. On 401 (except `/auth/*`), run a single-flight `auth.refresh()` and retry once. If that fails, `auth.logout({reason: 'expired'})`.
  3. Parse `application/problem+json` into `class ApiProblem extends Error { status, code, detail, requestId, errors?, conflicts?, alternatives?, perm?, module?, rule?, until?, current? }`.
  4. Add `X-EQ-CSRF: 1` on `/auth/refresh|logout|switch-role`.
  5. Generate `Idempotency-Key` (crypto.randomUUID) when the caller passes `{idempotent: true}`. Reuse the same key on retry of the same mutation instance.
- TanStack Query defaults: `retry` only for network errors and 5xx (max 2); never retry 4xx. `staleTime` 30 s. Mutations never auto-retry.
- `src/lib/auth.ts` (FE-11/12):
  - The token lives in a module-level variable.
  - Schedule a refresh 60 s before `exp` (decode without verifying).
  - `BroadcastChannel('eq-auth')` messages: `token`, `logout`. A tab receiving `token` adopts it; on `logout`, it clears.
  - Single-flight: concurrent `refresh()` calls share one promise.
  - `logout()`: POST `/auth/logout`, `queryClient.clear()`, **remove every `localStorage` key starting with `eq:` and run the registered `onLogout` cleanup hooks** (`registerLogoutCleanup(fn)` exported from `lib/auth.ts`; the offline queue registers one in M5), broadcast, navigate `/login`. A tab receiving the `logout` broadcast runs the same cleanup (SBD-58).
  - `switchRole(role)`.
  - **Never** touch localStorage or sessionStorage for tokens (a lint rule: grep in `check-raw-colors.mjs`'s sibling `check-storage.mjs` for `localStorage.*token`).
- Bootstrap (FE-10): `GET /meta/app` → if `setup_required` or `/setup/status.completed == false` → `/setup`; else `POST /auth/refresh` → on success `GET /me` + `GET /meta/modules`; else `/login`. Exposed as the `useSession()` hook: `{status: 'loading'|'anonymous'|'authenticated', me, modules, nav, app}`.
- i18n (`src/lib/i18n.ts`):
  - i18next + react-i18next + i18next-icu. Namespaces: `common`, `errors`, `policy`, `status`, `term`, `modules.<id>`. JSON in `src/locales/{en,id}/`, lazy-loaded per namespace.
  - Language: person locale → tenant locale → browser → `en`.
  - **Terminology (FE-37):** `term.instructor` and friends resolve from `meta/app.tenant.terminology` overrides (I18nText for the current language), else the catalog default. Expose `useTerm()` and allow `{{term.instructor}}` interpolation inside ICU messages via a custom formatter.
  - `errors.<code>`: every API problem code from spec 07 API-04 plus those in M1/M2 tasks (`slot_taken`, `version_conflict`, `invalid_transition`, `last_owner`, `module_dependency`, `module_required_by`, `module_disabled`, `setup_required`, `already_set_up`, `setup_reset_blocked`, `password_weak`, `invite_expired`, `token_invalid`, `totp_required`, `locked`, `rate_limited`, `idempotency_mismatch`, `email_taken`, `primary_exists`, `resource_in_use`, `cancel_window_passed`, `needs_decision`, `policy_violation`, `validation`, `forbidden`, `not_found`, `internal`, `precondition_required`), in en and id.
  - `pnpm i18n:check`: a script asserting the en and id key sets are equal per namespace, that no key in code is missing (scan `t('...')` literals), and that **every code in `web/openapi.json` `x-error-codes` has `errors.<code>`** in both languages (ERR-22).
- `src/lib/format.ts` (Intl, tenant tz via `@date-fns/tz`):
  - `money({amount_minor, currency})` (exponent from `Intl.NumberFormat().resolvedOptions().maximumFractionDigits`).
  - `date(d, 'short'|'long')`, `time(d)`, `dateTime`, `duration(minutes)` (e.g. "45 min", "1 h 30 min"), `ordinal(n)` (ICU selectordinal), `relative(d)`.
  - **SummaryLine rendering** (POL-07): `renderSummary(line)` resolves `$money` and `$minutes` args, then `t(line.key, args)`.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `gen:api` is extended by T-306a with generated zod schemas (`@hey-api/openapi-ts`, zod plugin only) and field metadata; keep `openapi-fetch` as the client (D-57).
- The access token lives in memory only, never in storage (AUTH-03, spec 25 §2). Query cache rules (staleTime 30 s, cleared on logout, role switch and tenant change) are CACHE-30 (spec 26).

## Scope
### In
- `lib/api.ts`, `lib/auth.ts`, `lib/i18n.ts`, `lib/format.ts`, `useSession`, the locale catalogs for common, errors, status and term, the scripts `i18n:check` and `check-storage`, and removing the T-305 i18n shim.

### Out
- Pages.

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/openapi.json` | create or modify | Path named in this task's contract. |
| `web/src/lib/*.test.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T4 (SBD-30).
- Data classes touched: S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `ApiProblem` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Two concurrent 401s → exactly one `/auth/refresh` call, and both requests retried (msw). → `api.test.ts › single flight refresh`
- AC2. A problem response → `ApiProblem` with `code` and `requestId`. → `api.test.ts › problem mapping`
- AC3. A token received over BroadcastChannel is adopted. → `auth.test.ts › broadcast adoption`
- AC4. `money` hides a zero minor part (`minimumFractionDigits: 0` when `amount_minor % 10^exp == 0`): `{10000000, IDR}` in `id` → `Rp 100.000` (Intl output, normalise NBSP in the assertion); `{1250, USD}` in `en` → `$12.50`. → `format.test.ts`
- AC5. A terminology override `instructor: {en: "Coach"}` → `t('schedule.assign_instructor')` = "Assign Coach". → `i18n.test.ts › terminology`
- AC6. `pnpm i18n:check` passes, and fails on a fixture with a missing id key.

## Tests to write
| File | Test names |
|---|---|
| `web/src/lib/*.test.ts` | the AC names above |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks pass; `make api` (generated types) up to date
- [ ] Commit `feat(web): API client, auth, i18n and formatting [T-306]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
