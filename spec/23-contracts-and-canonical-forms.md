# 23 — Contracts & Canonical Forms

This document defines **one canonical representation for every kind of value** that crosses a boundary: the HTTP API, events, jobs, policies, notifications, audit, files, CSV and iCal. It also defines the shape of every contract (requests, responses, events, jobs) and how contracts are versioned and tested.

- Validation and sanitisation of these values (which rule runs where, server and client) is `22-validation-and-sanitization.md` (VAL-\*).
- The behaviour behind each contract (states, transitions) is `20-state-machines.md` (STM-\*), and how data moves is `21-data-flows.md` (FLW-\*).
- Where this file and an older section disagree on a **representation**, this file wins. API-01..15 (`07-api-conventions.md`) stay normative for HTTP semantics and point here for formats.

Tags: **[tool]** = enforced by a test or CI, **[review]** = checked in review. **[D-nn]** marks a decision taken in this document that wasn't explicit before; all of them are listed in §12 for review.

## 1. Principles

- **CAN-01. One canonical form per value.** Every value kind has exactly one canonical form. The server **outputs only** the canonical form. It **accepts** the canonical form, plus the lenient input forms listed in this file, which it converts to canonical before any comparison, hashing, storage or lookup.
- **CAN-02. Canonicalise before you compare, hash, index or sign.** Blind indexes (SEC-04), idempotency hashes (API-08), audit hashes (AUD-04), dedupe keys and signed URLs all take canonical input. Two inputs that mean the same thing produce the same bytes.
- **CAN-03. Contracts are generated, never hand-copied.** The OpenAPI document (utoipa) is the source for the TypeScript client, the client-side zod schemas and the published error registry. Event payload schemas are generated with `schemars`. Nobody writes a type twice by hand (TST-10, VAL-30). [tool]
- **CAN-04. Strict in, tolerant out for our own clients, tolerant reader for events.**
  - HTTP request bodies reject unknown fields (CAN-21).
  - Event and job consumers ignore unknown fields (CAN-61), so producers can add fields without a coordinated deploy.

## 2. Value catalog

**CAN-10.** The table is normative. "Kind" is the name of the reusable field kind in `eq_core::fields` (VAL-20) and of the OpenAPI component schema that DTOs reference.

| Kind | Canonical form (wire) | Accepted input (converted to canonical) | Rust type | TS type | Notes |
|---|---|---|---|---|---|
| `Id` | UUID, lowercase, hyphenated, 36 chars: `0192f3a4-…` | Uppercase hex is lowercased. Braced, URN and simple (32-char) forms are **rejected** | `uuid::Uuid` inside a newtype per entity (`SessionId`) | `string` (branded `Id<'session'>` in `lib/ids.ts`) | Generated as UUIDv7 by the server only (DOM-01). Clients never mint entity ids |
| `Key` | `^[a-z0-9_]{2,40}$` | none (case-sensitive; no trimming) | `Key` | `string` | Offering, level, track, price-plan, booking-type keys |
| `Slug` | `^[a-z0-9](-?[a-z0-9])*$`, 3–40 | Derived from a name: NFKD, strip marks, lowercase ASCII, non-alnum runs → `-`, trim `-` | `Slug` | `string` | Tenant slug (DDL). Collision → append `-2`, `-3` … (server) |
| `Line` | NFC, trimmed, single spaces, no controls | Any Unicode text; sanitised per VAL-11 `line` | `Line<MIN, MAX>` | `string` | Names, titles, reasons |
| `Multiline` | NFC, `\n` line breaks, no other controls | `\r\n`/`\r` → `\n`; sanitised per VAL-11 `multiline` | `Multiline<MAX>` | `string` | Notes, comments, descriptions |
| `DisplayName` | `Line<1, 80>` | as `Line` | `DisplayName` | `string` | S1, plaintext by design (SEC-04) |
| `LegalName` | `Line<1, 120>` | as `Line` | `LegalName` | `string` | S2, encrypted |
| `Email` | NFC, trimmed, **all lowercase**, `local@domain`, ≤ 254 chars, local part ≤ 64 | Surrounding whitespace and a trailing dot on the domain are removed | `Email` | `string` | S2. Quoted local parts and IP-literal domains are rejected [D-01]. Blind index input = canonical form |
| `Phone` | E.164: `+` then 8–15 digits, `+6281234567890` | National format with the tenant country as default region (`0812-3456-7890` → `+6281234567890`), spaces, dashes, dots, parentheses | `PhoneE164` | `string` | S2. Parsed with Google libphonenumber metadata (`phonenumber` / `libphonenumber-js`). Must be a *valid* number for its region. Blind index input = E.164 |
| `Password` | Unicode **NFKC**, not trimmed | any | `secrecy::SecretString` | `string` (never stored client-side) | S3. Length counted in code points after NFKC: `password_min_length`..128 (IDN-02) [D-02] |
| `Token` | base64url without padding, 43 chars (32 bytes) | none | `SecretString` | `string` | S3. Invite, reset, refresh, unsubscribe, ICS tokens (SBD-49) |
| `Instant` | RFC 3339 with offset, in the **tenant timezone**, second precision unless the fraction is non-zero: `2026-11-05T15:00:00+07:00` | Any offset including `Z`. Up to 6 fractional digits. **A missing offset is rejected** | `chrono::DateTime<Utc>` | `string` | Stored as `timestamptz` (µs). Output uses `SecondsFormat::AutoSi` |
| `LocalDate` | `YYYY-MM-DD` | none | `chrono::NaiveDate` | `string` | Calendar dates (birth date, leave, closures) |
| `LocalTime` | `HH:MM`, 00:00–23:59; `24:00` only as the `end` of a `TimeInterval` (midnight, T-102) | `HH:MM:SS` with `:00` seconds | `policy::types::LocalTime` | `string` | Minute precision (POL types) |
| `Weekday` | `mon` … `sun` | none | `Weekday` | union | |
| `DateRange` | `{ "from": LocalDate, "until": LocalDate }`, **both inclusive**, `from ≤ until` | none | `DateRangeInclusive` | object | Stored as PG `daterange '[from, until+1)'` [D-03] |
| `InstantRange` | `{ "starts_at": Instant, "ends_at": Instant }`, half-open `[starts_at, ends_at)` | none | `TimeRange` | object | Stored as `tstzrange '[)'` (ARC-10). Never serialised as a PG range string |
| `Minutes` / `Hours` / `Days` | Integer, field name ends `_minutes`, `_hours`, `_days` | none | `u32` newtypes | `number` | No ISO 8601 durations anywhere |
| `Money` | `{ "amount_minor": int, "currency": "IDR" }` | none | `eq_core::money::Money` | `Money` | `|amount_minor| ≤ 10^15` (`MONEY_MAX_ABS_MINOR`, technical: below JS `Number.MAX_SAFE_INTEGER` ≈ 9.007·10^15, so the SPA never loses precision) [D-04] |
| `MinorAmount` | Bare integer ≥ 0 in the tenant currency | none | `MinorAmount` | `number` | Policies only (POL-12) |
| `Currency` | ISO 4217 alpha-3, uppercase, from the v1 supported list (T-102: IDR, USD, SGD, MYR, PHP, THB, VND, EUR, GBP, AUD, JPY) | lowercase is uppercased | `Currency` (exponent from `iso_currency`) | `string` | Requests whose currency ≠ tenant currency → `not_allowed` |
| `Bps` | Integer 0..=10000 | none | `Bps` | `number` | Percentages (ARC-11). The UI shows `%` |
| `Locale` | BCP 47 language tag from the **closed** set `en`, `id` | `en-US` → `en`, `id-ID` → `id`; case-insensitive | `Locale` (enum) | union | `Accept-Language` negotiation per CAN-35 |
| `Country` | ISO 3166-1 alpha-2, uppercase | lowercase is uppercased | `Country` | `string` | |
| `TimeZone` | IANA name, canonical spelling (`Asia/Jakarta`) | Aliases that `chrono-tz` resolves are mapped to their canonical name | `chrono_tz::Tz` | `string` | Abbreviations (`WIB`, `PST`) and fixed offsets are rejected |
| `Geo` | `{ "lat": number, "lng": number }`, at most 6 decimals | More decimals are rounded half-away-from-zero to 6 | `GeoPoint` | object | ~0.1 m precision. lat −90..90, lng −180..180 |
| `I18nText` | `{ "en": Line, "id": Line }` (keys = Locale, ≥ the tenant default locale) | Empty-string values are dropped as if absent | `I18nText` | `Record<Locale,string>` | Resolved to a string on portal/public endpoints (API-11) |
| `HttpsUrl` | Absolute `https://` URL, normalised by the `url` crate (WHATWG), no credentials, no fragment | `http://` only for `localhost` in `RITMA_ENV=dev` | `url::Url` newtype | `string` | Operator config only (SBD-47). Users never submit URLs the server fetches |
| `ColorToken` | One of the design-system category tokens (`cat-1` … `cat-12`) | none | enum | union | Resource/offering colours. No raw hex (DS) [D-05] |
| `Hue` | Integer 0..=359 | none | `u16` | `number` | `brand_hue` (DS-10) |
| `Boolean` | JSON `true` / `false`; query string `true` / `false` | none (`1`, `0`, `yes` rejected) | `bool` | `boolean` | |
| `Enum` | `snake_case` string from a closed set | none | Rust enum, `rename_all = "snake_case"` | union | Unknown value → `not_allowed` with `params.allowed` (SBD-44) |
| `Cursor` | Opaque base64url (no padding) of `{"k": sort_key, "id": Id}`, ≤ 512 chars | none | `Cursor` | `string` | A tampered or foreign cursor → 400 `bad_request` (API-05) |

**CAN-11. Text never carries markup.** Every text field is plain text. Input is **not** HTML-escaped on the way in (that would double-escape on output). `<b>` in a name is stored as those 3 characters and rendered escaped (SBD-45). There is no rich-text field in v1; if one is added, its spec must name `ammonia` with an allow-list.

**CAN-12. Numbers.** JSON numbers are integers except `Geo` and `gps_accuracy_m`. Integers outside ±2^53 never appear on the wire. Floats are never used for money, bps or durations (ARC-11).

## 3. HTTP contract shapes

**CAN-20. DTO naming (OpenAPI component names).**

| Shape | Name | Used for |
|---|---|---|
| Full resource | `Invoice` | `GET /invoices/{id}`, responses of create/update |
| List item | `InvoiceSummary` | Items of `GET /invoices` |
| Create body | `InvoiceCreate` | `POST /invoices` |
| Partial update body | `InvoicePatch` | `PATCH /invoices/{id}` |
| Action body | `InvoiceCancel`, `PaymentVerify` (verb-noun of the action route) | `POST /invoices/{id}/cancel` |
| Page | `Page_InvoiceSummary` (generic `Page<T>`) | `{ items, next_cursor }` |
| Dry run / preview | `SessionCheckResult`, `PolicyPreview` | `/check`, `/preview`, `/quote` |
| Error | `Problem`, `FieldError` | All errors (ERR-30/31) |

Each DTO lives in the module's `http.rs` (CA-01). Domain types never derive `ToSchema` or `Deserialize` from requests; the adapter maps DTO ⇄ domain.

**CAN-21. Request bodies.**
- `Content-Type: application/json; charset=utf-8` (a missing charset means UTF-8). UTF-8 only, no BOM.
- **Unknown fields are rejected** (`#[serde(deny_unknown_fields)]`) with a 422 `validation` entry `{pointer: "/the_field", code: "not_allowed"}`. This blocks mass-assignment (`"status": "paid"`, `"tenant_id": …`) by construction (SBD-42).
- `tenant_id`, `created_by`, `version`, `status`, totals and prices never appear in a request body. The tenant comes from context, the version from `If-Match`, the status from the action route, money totals from the server (SBD-57).
- Maximum nesting depth 32 and the body limits of ARC-04 (technical).

**CAN-22. Absent vs `null` [D-06].**
- **Create (`POST`)**: an absent optional field and `null` mean the same thing: "not set".
- **Patch (`PATCH`)**: JSON Merge Patch semantics (RFC 7396) over the `XxxPatch` shape, sent as `application/json`:
  - absent → unchanged;
  - `null` → clear (only for nullable fields; `null` on a required field → `required`);
  - value → set;
  - arrays and `I18nText` maps are replaced whole, never merged.
- **Responses**: optional fields are **always present**, with `null` when unset. The SPA never has to distinguish "missing" from "null".
- Empty strings are not "unset". After sanitisation an empty required `Line` is `required`; an empty optional `Line` is converted to `null` [D-07].

**CAN-23. Responses.**
- `200` with the full resource for reads and updates, `201` with the full resource and a `Location` header for creates, `202` with `{ "job_id" }` or `{}` for accepted async work, `204` for deletes and fire-and-forget actions.
- Every mutable aggregate response carries `ETag: "v<version>"` and a `version` field (API-07).
- Lists: `Page<T>` (API-05). `next_cursor` is `null` on the last page. Items are ordered by the documented sort, ties broken by `id` (UUIDv7 = creation order).
- Warnings that don't fail the request are `warnings: [ "below_min_capacity", … ]` (stable `snake_case` codes, translated as `warnings.<code>`) on the resource or result DTO.
- Instants are serialised in the tenant timezone. Where the UI needs local parts, the DTO also carries `local_date` / `local_start` (API-09).

**CAN-24. Problem bodies** are ERR-30/31 exactly. Field errors use JSON Pointer (RFC 6901) into the request body, with `~0`/`~1` escaping. For multipart requests, the pointer is into the `json` part, and a file part is addressed as `/$files/<part_name>` [D-08].

## 4. Headers and query parameters

**CAN-30. Request headers (closed list).**

| Header | Canonical form | Where | Invalid → |
|---|---|---|---|
| `Authorization` | `Bearer <jwt>` | Bearer endpoints | 401 `unauthenticated` |
| `If-Match` | `"v<positive int>"` (strong ETag). `*` is not accepted | PATCH/actions on versioned aggregates | missing → 428 `precondition_required`; malformed → 400 `bad_request`; stale → 409 `version_conflict` |
| `Idempotency-Key` | UUID (any version), canonical lowercase | Money and booking creates (API-08) | malformed → 400 `bad_request` |
| `X-EQ-CSRF` | literal `1` | Cookie endpoints (IDN-05) | 403 `forbidden` |
| `X-EQ-Queued-At` | `Instant` | Offline check-in replay (ATT-07) | ignored if malformed, logged at `debug` |
| `Accept-Language` | RFC 9110 list | Everywhere | ignored if unparsable |
| `X-Request-Id` | **Ignored on input** [D-09] | — | The server always generates its own UUIDv7 (ARC-04) so a client can't forge correlation ids |

**CAN-31. Response headers** set by the server: `ETag`, `Location`, `Retry-After`, `RateLimit-*`, `X-Request-Id`, `X-EQ-Payer-State` (BIL-08), the security headers (SEC-20), and `Cache-Control: no-store` on every `/api` response except signed file downloads and `/meta/app` (`max-age=60`) [D-10].

**CAN-32. Query parameters.**
- Names are `snake_case`. Values use the canonical forms of §2.
- Multi-value filters repeat the parameter: `?status=issued&status=overdue` (parsed with `axum_extra::extract::Query`). Comma-separated lists are not accepted [D-11].
- Ranges are `from` / `to`, with `from` inclusive and `to` **exclusive** for instants, and `from` / `until` both inclusive for dates (matching CAN-10 `DateRange`).
- `limit` 1..=200 (default 50), `cursor`, `sort` from an allow-list (`sort=-created_at`), `q` free text (`Line<1, 100>`).
- Unknown query parameters are rejected with 400 `bad_request` [D-12]. This keeps typos (`?stauts=`) from silently returning unfiltered data.

**CAN-33. Path parameters.** A path id that isn't a canonical `Id` returns **404 `not_found`**, like an id that doesn't exist or is out of scope (SBD-41) [D-13]. Path keys (`/settings/policies/{module}`) are closed enums: unknown → 404.

**CAN-34. Cookies.** Only `eq_rt` (IDN-04): `HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=<absolute_session_days·86400>`. `Secure` is dropped only for `http://localhost` in dev. No other cookie is set.

**CAN-35. Locale negotiation** (API-11), first match wins: `Accept-Language` (portal/public endpoints only) → `persons.locale` → `tenants.locale` → `en`. Matching is by primary language subtag against the closed `Locale` set.

## 5. Identifiers and names of things (string contracts)

**CAN-40.** These strings are part of the public contract. They're never renamed (renaming is a breaking change, COMMIT_CONVENTION §5). Each has a format test [tool].

| Thing | Format | Example | Registry |
|---|---|---|---|
| Module id | `ModuleId` snake_case | `billing` | MOD-02 |
| Permission | `<module>.<entity>.<action>` | `billing.payment.verify` | `perms.rs` + IDN-10 |
| Error code | `^[a-z][a-z0-9_]{2,47}$` | `slot_taken` | ERR-20/21 |
| Field error code | closed set VAL-03 | `too_long` | VAL-03 |
| Warning code | same format as error codes | `below_min_capacity` | module `errors.rs` (`WARNINGS`) |
| Policy rule | JSON-Pointer-like dotted path of the policy key | `billing.partial_payment.allowed` | POL |
| Event type | `<module>.<event_name>.v<N>` | `billing.invoice_overdue.v1` | 03 §6 |
| Audit action | `<module>.<entity>.<verb_past>` or `security.<event>` | `billing.invoice.issued` | AUD-02 |
| Job kind | `<module>.<job_name>` | `billing.issue_cycle` | module `jobs.rs` |
| Dedupe key | `<purpose>:<tenant?>:<ids/period…>` with `:` separators, ids canonical | `billing.issue:{tenant}:2026-11` | ARC-09, NTF-01 |
| Attention kind | snake_case | `materialize_conflict` | 06 §4.11 |
| Template key | `<module>.<template_name>` (+ `.digest`) | `scheduling.session_reminder.digest` | NTF-07 |
| Nav id | `<module>.<screen>` | `billing.invoices` | MOD-09 |
| i18n key | dotted, `<namespace>.<path>` | `errors.slot_taken`, `term.instructor` | FE, TST-09 |
| Form key (drafts) | `<module>.<entity>.<form>` | `scheduling.session.new` | FE-30 |
| Problem type URI | `PROBLEM_BASE` + code with `_` → `-` | `https://ritma.dev/problems/slot-taken` | ERR-20 |
| Invoice number | from `numbering.reset`: `INV-YYYY-NNNNN` or `INV-YYYYMM-NNNN` | `INV-2026-00042` | BIL-04 |
| Period key | `YYYY-MM` (local calendar month) | `2026-11` | BIL-02 |

## 6. Async contracts: events, jobs, notifications

**CAN-60. Event envelope.** Every outbox row is delivered to handlers as:

```json
{
  "id": "0192f…",                      // outbox.id, UUIDv7, also the idempotency key for receipts
  "type": "billing.invoice_overdue.v1",
  "tenant_id": "…",
  "aggregate_id": "…",                 // the entity whose order must be preserved, or null
  "occurred_at": "2026-11-11T00:10:00+07:00",
  "actor": { "id": "…" | null, "role": "admin" | "system" },
  "request_id": "…" | null,            // request or job that caused it
  "payload": { … }                      // the versioned payload of 03 §6
}
```

- Payload structs live in the emitting module's `events.rs`, derive `Serialize + Deserialize + JsonSchema`, and are snapshot-tested as JSON Schema files under `contracts/events/<type>.schema.json` [tool, TST-17].
- **Payloads carry ids and minimal facts only. No S2/S3 values ever** (SBD B4). Consumers re-read through ports (03 §6).
- Money in payloads is `amount_minor` plus `currency` when the currency isn't implied by the tenant; instants are `Instant`; date ranges are `DateRange`.

**CAN-61. Event evolution.**
- Adding an optional field is non-breaking: same version. Consumers MUST ignore unknown fields (no `deny_unknown_fields` on payload structs).
- Removing, renaming or changing the meaning of a field means a **new version** (`.v2`). The producer emits both versions for one minor release, then drops the old one (ERR-23 style).

**CAN-62. Job payloads** are JSON objects with a `v` integer (start at 1) and ids only. Example `billing.issue_cycle`: `{"v":1,"period_key":"2026-11"}`. `jobs.last_error` is a single PII-free line ≤ 500 chars (ERR-51).

**CAN-63. Notification params** (NTF-01): only S0/S1 values plus ids. Money is `{"$money": Money}`, durations `{"$minutes": n}`, instants `Instant`, dates `LocalDate`. Addresses (email/phone, S2) are **never** in params; notify decrypts them at send time (NTF-09). URLs in params are built by the server from `RITMA_PUBLIC_URL` (CAN-80).

**CAN-64. Policy documents** follow POL-13 (externally tagged `snake_case` enums), with money as `MinorAmount`, percentages as `Bps`, times as `LocalTime`. The JSON Schema of each policy (schemars, draft 2020-12) is the contract for `PolicyForm` and presets (POL-06, TST-08).

## 7. Canonical bytes for hashing and signing

**CAN-70.** Every hash, MAC or signature over structured data uses **RFC 8785 JSON Canonicalization Scheme** (`serde_jcs`) of the canonical values:

| Use | Input | Output |
|---|---|---|
| Audit chain (AUD-04) | JCS of the entry fields listed in AUD-04 | SHA-256 |
| Idempotency `request_hash` (API-08, T-115) | `method` + `\n` + `route_template` + `\n` + JCS of the parsed request body **after** canonicalisation (§2) [D-14] | SHA-256 |
| Blind index (SEC-04) | The canonical `Email` / `Phone` string as UTF-8 | HMAC-SHA256, first 16 bytes |
| Signed file URL (SEC-15) | `tenant_id|file_id|exp|actor_id`, ids canonical, `exp` decimal Unix seconds | HMAC-SHA256, base64url |
| Consent text hash (AUD-12) | The consent text in NFC with `\n` line breaks | SHA-256 |
| Event receipt key | `(handler_id, event.id)` | — |

Because the idempotency hash is over the canonical body, a retry that differs only in whitespace, key order or `"Ayu "` vs `"Ayu"` replays the original response instead of returning `idempotency_mismatch`.

## 8. URLs, files and exports

**CAN-80. URLs built by the server** always start with `RITMA_PUBLIC_URL` (multi mode: the tenant host). Paths are built with the `url` crate's path-segment API, never by string concatenation. Token-bearing links put the token in the **path** (`/invite/{token}`) or a single query parameter and are single-use where the spec says so (SBD-49).

**CAN-81. File references.** APIs return `{ "file_id": Id }` and, where the caller may read the file, a short-lived `url` (SEC-15, 5 minutes). Downloads set `Content-Type` from the stored (sniffed) type, `X-Content-Type-Options: nosniff`, and `Content-Disposition: attachment; filename="<ascii fallback>"; filename*=UTF-8''<percent-encoded>` (RFC 6266/8187), with the name sanitised per VAL-16.

**CAN-82. CSV.**
- Export: RFC 4180, UTF-8 **with BOM** (so Excel opens it correctly), `,` separator, `\r\n` line endings, a header row of `snake_case` column names, values in canonical form (money as decimal major units with the currency exponent, e.g. `600000.00`, plus a `currency` column), formula-injection guard SBD-46.
- Import (ENR-11, closures): UTF-8 with or without BOM; separator `,` or `;` auto-detected from the header row [D-15]; dates **only** `YYYY-MM-DD` (XLSX native date cells are converted); times `HH:MM`; headers matched case-insensitively after trimming. Rows are validated with the same field kinds as the API (VAL-20).

**CAN-83. iCal feed** (SCH-16): RFC 5545. `UID:<session_id>@<public host>`, `DTSTART`/`DTEND` in UTC (`…Z`), `SEQUENCE` = session `version`, `STATUS:CANCELLED` for cancelled sessions (kept 30 days), `SUMMARY` = offering name in the person's locale, text escaped per RFC 5545 §3.3.11. Generated with the `icalendar` crate.

## 9. Versioning and compatibility

**CAN-90.** Contract surfaces and their compatibility rules:

| Surface | Version marker | Additive change | Breaking change |
|---|---|---|---|
| HTTP API | `/api/v1`, OpenAPI `info.version` = app version | New endpoint, new optional request field, new response field, new enum value **in a response** (the SPA treats unknown values as `unknown` with a neutral badge) | Anything else → `/api/v2` (API-13) |
| Error / warning codes | registry in `x-error-codes` | New code | Rename/remove (ERR-23) |
| Events | `.vN` suffix | New optional payload field | New version (CAN-61) |
| Job payloads | `v` field | New optional field | New `v`; the handler accepts both for one release |
| Policy documents | `schema_version` | New field with a default | `migrate_policy` (POL-08) |
| Presets | `preset_version` | Any (existing tenants are never re-applied, SET-02) | — |
| CSV import columns | documented header names | New optional column | Rename/remove |
| DB schema | migration number | Expand step | Contract step only after the code stopped using it (OPS-04) |

**CAN-91. New enum values in requests** (e.g. a new payment `method`) are additive for the server but the SPA must be regenerated before it can send them; the old SPA keeps working.

## 10. Contract testing

- **CAN-95.** OpenAPI drift (TST-10) proves the committed contract equals the code.
- **CAN-96.** Schemathesis (property-based fuzzing from the OpenAPI document) runs against a seeded server in CI and asserts: no 5xx, every error is `application/problem+json` with a registered `code`, every 2xx body validates against its response schema, and `deny_unknown_fields` is honoured (TST-18).
- **CAN-97.** Event payload schemas are snapshot-tested (CAN-60). A diff in `contracts/events/` needs a matching CAN-61 decision in the PR.
- **CAN-98.** The canonicalisation cases in `contracts/validation/cases/*.json` run in Rust **and** in the web test suite, so server and client produce the same canonical value or the same field error code for the same input (VAL-31, TST-17).

## 11. Where each canonical form is implemented

| Concern | Rust | Web |
|---|---|---|
| Field kinds, sanitisers, limits | `eq_core::fields`, `eq_core::text` (T-102a) | generated `web/src/api/zod.gen.ts` + `web/src/lib/validation/` (T-306a) |
| Money, Bps, Currency | `eq_core::money` (T-102) | `lib/format.ts`, `components/domain/MoneyInput` |
| Time, ranges | `eq_core::time`, `policy::types` (T-102) | `lib/format.ts` with `@date-fns/tz` |
| JCS, hashing | `eq_core::crypto`, `eq-audit` (T-103, T-109) | — |
| Problem / FieldError | `eq_core::error` (T-002) | `lib/api.ts` (T-306) |
| Event envelope | `eq_core::outbox` (T-107) | — |
| OpenAPI components | `eq-api` merge (T-114) | `web/openapi.json` → `pnpm gen:api` |

## 12. Decisions taken in this document

These were implicit or missing before. They are normative once this document is merged; reviewers should challenge them in the PR.

| ID | Decision | Why |
|---|---|---|
| D-01 | Emails: reject quoted local parts and IP-literal domains | Not used by real school contacts; they complicate normalisation and blind indexing |
| D-02 | Passwords are NFKC-normalised, not trimmed, length in code points | NIST 800-63B §5.1.1.2; a user who types a visually identical password on another keyboard still logs in |
| D-03 | API date ranges are inclusive `from`/`until`; storage is `[from, until+1)` | Matches how people say "leave from the 3rd until the 7th" and ENR-07's `until + 1` |
| D-04 | Money magnitude cap 10^15 minor units | Safe in JS numbers; far above any school invoice |
| D-05 | Colours are design-system category tokens, not hex | DS forbids raw hex; tokens also adapt to dark mode |
| D-06 | PATCH uses JSON Merge Patch semantics over `XxxPatch` | Standard, simple, works with openapi-fetch |
| D-07 | Empty optional strings become `null` | One representation for "not set" |
| D-08 | Multipart file parts are addressed as `/$files/<part>` in field errors | Lets the UI attach an error to the file input |
| D-09 | Client `X-Request-Id` is ignored | Prevents log/correlation forgery |
| D-10 | `/api` responses are `Cache-Control: no-store` (except listed) | No PII in shared caches; signed files and `meta/app` are the exceptions |
| D-11 | Multi-value query params repeat the name | Unambiguous with values containing commas |
| D-12 | Unknown query params → 400 | A typo never silently widens a filter |
| D-13 | Malformed path ids → 404 | Same answer as "not found / out of scope" (SBD-41) |
| D-14 | Idempotency hash over the canonical body | Harmless client differences don't cause `idempotency_mismatch` |
| D-15 | CSV import auto-detects `,` or `;` | Indonesian/European Excel exports use `;` |
