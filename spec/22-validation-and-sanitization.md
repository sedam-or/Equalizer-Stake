# 22 — Validation & Sanitisation

This document is the **single standard for checking and cleaning input**, on the server and in the browser. It says which rule runs at which stage, which library implements it, which error code it produces, and how the client and the server are kept identical.

- Canonical value forms are `23-contracts-and-canonical-forms.md` (CAN-\*). This file says how input *becomes* canonical, or is rejected.
- Error classes, shapes and HTTP mapping are `19-error-handling.md` (ERR-\*). This file extends ERR-31's field-code set (VAL-03).
- Security rules SBD-20 and SBD-44..47/55/56 are the "why". This file is the "how", and it supersedes the input parts of SBD-44 where they differ.

Tags: **[tool]** CI enforces it, **[review]** reviewers check it, **[D-nn]** a decision listed in §10.

## 1. Principles

- **VAL-01. The server is the authority. The client is a convenience.** Every rule is enforced on the server. The client repeats the *structural* rules (types, lengths, formats, required) for instant feedback and may run advisory server checks (dry runs). A request that skips the SPA (curl, a modified client) meets exactly the same server rules (SBD-08).
- **VAL-02. One rule, one definition.** A field rule (limits, pattern, sanitiser) is defined **once**, as a *field kind* in `eq_core::fields` (VAL-20). The OpenAPI component, the server parser and the client zod schema are all derived from it. Nobody re-types a limit in a second place. [tool: VAL-31 parity tests, CAN-96 fuzzing]
- **VAL-03. Closed set of field error codes.** Field errors (ERR-31) use only:

  | Code | Meaning | `params` |
  |---|---|---|
  | `required` | Missing, `null` on a required field, or empty after sanitising | — |
  | `invalid_type` | Wrong JSON type (string for number, object for array…) | `{expected}` |
  | `too_short` / `too_long` | Text length (code points after sanitising) out of bounds | `{min}` / `{max}` |
  | `too_few_items` / `too_many_items` | Array length out of bounds | `{min}` / `{max}` |
  | `must_be_gte` / `must_be_lte` | Number, date or time out of bounds, or relative to another field | `{min}` / `{max}` or `{other}` (a JSON Pointer) |
  | `invalid_format` | Pattern, email, phone, date, time, UUID, timezone, currency… didn't parse | `{format}` (the kind name) |
  | `invalid_characters` | Text contains a forbidden character (VAL-12) | — |
  | `not_allowed` | Value not in a closed set, or a field that mustn't be sent | `{allowed}` when the set is small (≤ 20) |
  | `not_unique` | Duplicate inside the request, or already taken (when it's a field problem, e.g. email) | — |
  | `unknown_reference` | An id that doesn't exist or isn't visible to the actor | — |

  ERR-31's list is extended by `invalid_type`, `too_few_items`, `too_many_items` and `invalid_characters`. Each has `errors.field.<code>` in `en` and `id` (TST-09).
- **VAL-04. Report all field errors at once.** Stages V2–V4 (§2) collect every field error before failing. The user fixes the form once, not field by field (ERR-31).
- **VAL-05. Sanitise, then validate, then canonicalise, then use.** Lengths are measured *after* sanitising. Comparisons, uniqueness checks, hashes and lookups use the canonical form (CAN-02). Code past the boundary only sees typed, canonical values (SBD-20).
- **VAL-06. Reject rather than repair, except for the listed normalisations.** The sanitisation profiles (VAL-11) list the only silent changes: whitespace, Unicode normalisation, case where the kind says so, and phone formatting. Anything else that's wrong is rejected with a code; it's never "fixed" silently (ERR-03).

## 2. The server pipeline

**VAL-07.** Every request passes these stages in order. A stage that fails stops the request with its error; later stages don't run.

| Stage | What runs | Implemented by | Failure → |
|---|---|---|---|
| **V0 Transport** | TLS (proxy), body limit (ARC-04), request timeout, rate limit, `Content-Type` check | `tower-http` `RequestBodyLimitLayer`, `TimeoutLayer`, `tower_governor` | 413 `payload_too_large`, 429 `rate_limited`, 400 `bad_request` (wrong/missing content type [D-16]) |
| **V1 Syntax** | Valid UTF-8 JSON, depth ≤ 32, no lone surrogates, no duplicate keys [D-17] | `serde_json` via `ValidatedJson<T>` | 400 `bad_request` (no pointer) |
| **V2 Shape** | Required fields present, JSON types right, enums known, **unknown fields rejected** (CAN-21), query/path params typed | `serde` + `serde_path_to_error` (pointer), `axum_extra::extract::Query` | 422 `validation`. Serde stops at the first shape error, so V2 returns one error; DTOs therefore use *raw* field types (`Raw<K>`, VAL-21) so that value-level problems are caught in V3 together |
| **V3 Fields** | Per field: sanitise (VAL-11) → check bounds/format → build the canonical newtype. All fields, all errors collected | `eq_core::fields` (`FieldKind`, `Raw<K>`, `Collector`), built on `nutype` newtypes and `unicode-normalization`, `email_address`, `phonenumber`, `chrono-tz`, `iso_currency` | 422 `validation` with every error |
| **V4 Request rules** | Cross-field rules that need no data: `from ≤ until`, `ends_at > starts_at`, no duplicate ids in a list, mutually exclusive fields | `TryFrom<XxxDto> for XxxCommand` in `http.rs` (pure) | 422 `validation` (all collected) |
| **V5 Context** | Auth, module gate, permission (ARC-04 steps 5–7); references exist **and are in the actor's scope**; module-dependent fields allowed | Extractors, then the service, inside the transaction | 401/403/404 per ERR-10; 422 `validation` with `unknown_reference` |
| **V6 Domain** | State machine guards (STM), invariants, capacity, windows, eligibility | Pure `domain/` functions (CA) | 409 `invalid_transition` / `version_conflict` / module codes, 403 rule codes (ERR-10) |
| **V7 Policy** | Tenant business rules in force (POL), with snapshotted terms where they exist (POL-05) | `domain/` given the `Versioned<P>` | 422 `policy_violation` with `rule` |
| **V8 Database** | Constraints: exclusion (holds), unique, FK, CHECK, RLS | PostgreSQL | Mapped per ERR-43. A CHECK violation is a bug (V3–V7 should have caught it) |

**VAL-08. Time-of-check vs time-of-use.** Checks that depend on stored state (V5–V7) run **inside the same transaction** as the write, after taking the locks the use-case needs (`SELECT … FOR UPDATE` on the aggregate, or the optimistic `version` check). Uniqueness and overlap are never decided by a pre-check alone: the constraint in V8 is the authority, and the pre-check exists only to produce a nicer message (DOM-11, SBD-57).

**VAL-09. Dry runs reuse the pipeline.** `POST /sessions/check`, `/rental/quote`, `/settings/policies/{m}/preview` and `/import/*/preview` run V0–V7 with the same code and **no writes** (a rolled-back transaction or a read-only one). They are the only "server-side validation APIs"; there's no generic `/validate` endpoint.

## 3. Sanitisation

**VAL-10. Encoding.** All text is UTF-8. `serde_json` rejects invalid UTF-8 and lone-surrogate escapes (`"\ud800"`) at V1. Multipart text parts are decoded as UTF-8 strictly; invalid bytes → 400 `bad_request`.

**VAL-11. Profiles.** Each field kind uses exactly one profile. Steps run in the order listed.

| Profile | Used by | Steps |
|---|---|---|
| `line` | Names, titles, reasons, purposes, search `q`, `I18nText` values | 1. Unicode NFC. 2. Reject forbidden characters (VAL-12) → `invalid_characters`. 3. Remove zero-width/format characters that carry no meaning (U+200B, U+FEFF, U+2060, U+00AD). 4. Replace every whitespace run (any `char::is_whitespace`, including NBSP and tabs) with one U+0020. 5. Trim both ends. 6. Empty → `null` if optional, `required` if required (CAN-22). 7. Length in code points |
| `multiline` | Notes, comments, descriptions, review notes, consent text | As `line`, except: `\r\n` and `\r` become `\n`; tabs become a space; whitespace runs **within** a line collapse; trailing spaces per line are removed; more than 2 consecutive blank lines become 2; trim the whole text |
| `identifier` | `Key`, `Slug`, codes, `reference_code`, `booking_type_key` | NFC, trim. **No** internal whitespace allowed. Case is **not** changed (keys are case-sensitive and already lowercase by pattern). Then the kind's pattern |
| `email` | `Email` | NFC, trim, remove one trailing `.` on the domain, lowercase the whole address (IDN-01), then `email_address` parsing (non-quoted local part, DNS-style domain with at least one dot, no IP literal) and length checks (≤ 254, local ≤ 64) |
| `phone` | `Phone` | Trim, then `PHONE_NUMBER_UTIL.parse(input, Some(region))` (`region` = the tenant country as `rlibphonenumber::enums::Region`), then `is_valid()`. Output E.164. Letters (vanity numbers) are rejected |
| `secret` | Passwords | NFKC only (CAN-10). No trimming, no collapsing. Reject only C0 controls except none → `invalid_characters`. Never logged, never echoed, never included in field-error `params` |
| `none` | Tokens, ids, enums, numbers, dates | No text normalisation. The kind's parser is strict (CAN-10) |

**VAL-12. Forbidden characters** (all profiles except `secret`):
- C0 controls U+0000–U+001F except `\n` in `multiline`, and DEL U+007F.
- C1 controls U+0080–U+009F.
- Bidirectional **override and isolate** controls U+202A–U+202E and U+2066–U+2069 ("Trojan Source"). LRM/RLM (U+200E/U+200F) are allowed in `line`/`multiline` but stripped from `identifier` and `email`.
- Unicode noncharacters (U+FDD0–U+FDEF, U+xFFFE/U+xFFFF) and private-use code points (U+E000–U+F8FF, planes 15–16) [D-18].
- More than 8 consecutive combining marks (category `M`) on one base character ("Zalgo" text) [D-19].

Emoji (including ZWJ sequences), all scripts and accents are allowed in `line`/`multiline` names and text. Lengths are code points after NFC, which is what PostgreSQL `length()` counts, so DB CHECKs (e.g. `display_name` 1–80) agree with the app.

**VAL-13. No markup on input.** No field accepts HTML or Markdown that is rendered as markup. Input is stored as plain text, never HTML-escaped on input (CAN-11). The one exception path, sanitised SVG logos, is a file (VAL-15), not a text field.

**VAL-14. Encode on output, per sink.** Sanitising input never replaces output encoding (SBD-45):

| Sink | Encoding | Implemented by |
|---|---|---|
| React DOM | JSX text escaping. `dangerouslySetInnerHTML` banned | React, Biome `noDangerouslySetInnerHtml` |
| `href` / `src` from data | Allow only `https:`, `mailto:`, `tel:`, same-origin relative | `lib/safeUrl.ts` (T-306a) |
| Server HTML (printables, invoice document, email HTML) | Autoescape on, no `|safe` on data | `minijinja` with `AutoEscape::Html` |
| Email headers (subject, names) | `line` profile already bans CR/LF; `lettre` encodes headers (RFC 2047) | `lettre` |
| WhatsApp / plain-text messages | Plain text; no escaping needed; length capped by the provider adapter | `eq-notify` adapters |
| CSV export | RFC 4180 quoting + formula guard (SBD-46) | `csv` crate + `eq_core::export::guard_cell` |
| iCal | RFC 5545 text escaping (CAN-83) | `icalendar` |
| `Content-Disposition` | VAL-16 + RFC 6266/8187 | `eq_core::files` |
| Logs | Structured JSON fields (control characters escaped by the serializer); ids only (SEC-21) | `tracing-subscriber` json |
| SQL | Bind parameters only (CQ-42) | `sqlx::query!` |
| URLs | Path segments and query values through the `url` crate | `url` |

**VAL-15. Files** (SEC-13/14, SBD-55). Every upload goes through `eq_core::files::accept(purpose, part)`:

| Purpose | Allowed (by magic bytes via `infer`) | Max size | Decode / transform | Stored | Served |
|---|---|---|---|---|---|
| `avatar` | JPEG, PNG, WebP | 2 MB | Decode with `image` limits (≤ 4096×4096 px, ≤ 64 MiB alloc), resize to ≤ 512 px, re-encode WebP/JPEG, EXIF stripped | plain | inline, public within tenant (SEC-15) |
| `logo` | PNG, SVG | 512 KB | PNG as avatar; SVG parsed and re-serialised by `usvg` (no scripts, `foreignObject`, event attributes or external refs), else rasterised with `resvg` | plain | inline, public |
| `checkin_photo` | JPEG, PNG, WebP | `attendance` policy `photo.max_bytes` | Decode with limits, check ≥ `min_width`×`min_height`, re-encode JPEG q=85, EXIF stripped (ATT-02) | **encrypted** | signed URL, attachment for download, inline in review |
| `payment_proof` | JPEG, PNG, WebP, PDF | 5 MB | Images re-encoded; PDF: `%PDF-` magic, stored as-is | **encrypted** | signed URL, PDF always `attachment` |
| `registration_doc` | JPEG, PNG, WebP, PDF | 5 MB | as `payment_proof` | **encrypted** | signed URL |
| `import` | CSV (text sniff), XLSX (zip magic + `[Content_Types].xml`) | 10 MB | VAL-17 | plain, deleted after 24 h | never served |
| `export` | produced by the server | — | — | **encrypted** | signed URL, 24 h (AUD-10) |

- The declared `Content-Type` and the filename extension are **ignored** for the decision; only the sniffed type counts. A mismatch is not an error by itself.
- Decoding runs in `tokio::task::spawn_blocking` with a timeout (5 s, technical).
- Failure codes: wrong type → `not_allowed` (pointer `/$files/<part>`), too big → 413 `payload_too_large`, undecodable or too small → `invalid_format`.

**VAL-16. Client filenames** are display metadata only (SBD-55): sanitised with `sanitize-filename` (no path separators, no reserved names, no controls), NFC, `line` profile, ≤ 120 code points, extension replaced by the one matching the sniffed type. Never used to build a storage path.

**VAL-17. Imports** (ENR-11, closures CSV):
- CSV: `csv` crate, streaming, max 5 000 data rows, max 64 columns, max 4 KiB per field (technical). BOM tolerated, separator auto-detected (CAN-82).
- XLSX: `calamine`, first sheet only, after checking the zip: ≤ 50 MiB total uncompressed, ≤ 100 entries, compression ratio ≤ 100:1 per entry (zip-bomb guard, read with the `zip` crate before handing to calamine). Formulas are read as their cached values, never evaluated.
- Every cell runs the same field kind as the API field it maps to (VAL-02), so `" Ayu  S. "` imports as `"Ayu S."`.
- Preview returns per-row `{index, ok, errors: [FieldError]}` with pointers `/rows/<index>/<column>`. Commit is all-or-nothing (ENR-11).

**VAL-18. Structural limits** (technical constants in `eq_core::limits`, each with `// technical:`):

| Limit | Value |
|---|---|
| JSON body | 1 MiB (uploads 10 MiB, ARC-04) |
| JSON depth | 32 |
| Default array length | 100 items unless the kind says otherwise |
| `q` search text | 1–100 code points |
| Page size | 1–200 (API-05) |
| Cursor | ≤ 512 bytes |
| Multipart parts | ≤ 4 files + 1 `json` part |
| Headers | `hyper` defaults; `Idempotency-Key` ≤ 64 bytes |

**VAL-19. Numbers.** Integers are range-checked by the kind (e.g. `Bps` 0..=10000, `Minutes` per field). Floats are allowed only for `Geo` (rounded to 6 decimals, CAN-10) and `gps_accuracy_m` (0..=10 000, rounded to 0.1). `NaN`/`Infinity` can't be expressed in JSON and are rejected at V1.

## 4. Field kinds (the shared catalog)

**VAL-20. Implementation.** `eq_core::fields` (T-102a) defines each field kind once:

```rust
/// One reusable field rule: sanitiser + bounds + canonical type + OpenAPI schema.
pub trait FieldKind: Sized {
    /// The raw JSON type accepted at V2 (usually String, i64 or f64).
    type Input: DeserializeOwned + ToSchema;
    /// OpenAPI component name, e.g. "DisplayName".
    const NAME: &'static str;
    /// Sanitising profile, exported as `x-sanitize` (VAL-11).
    const PROFILE: Profile;
    /// Sanitise + validate + canonicalise. Returns every problem for this value.
    fn parse(raw: Self::Input, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>>;
    /// JSON Schema for OpenAPI, built from the same constants used by `parse`.
    fn schema() -> utoipa::openapi::Schema;
}

/// DTO field wrapper: deserialises the raw input without judging it (V2),
/// and documents itself in OpenAPI as the kind's schema (`$ref: DisplayName`).
pub struct Raw<K: FieldKind>(K::Input, PhantomData<K>);

/// Collects issues for all fields, prefixing each with its JSON Pointer (VAL-04).
pub struct Collector { /* … */ }
impl Collector {
    pub fn field<K: FieldKind>(&mut self, pointer: &str, raw: Raw<K>) -> Option<K>;
    pub fn optional<K: FieldKind>(&mut self, pointer: &str, raw: Option<Raw<K>>) -> Option<Option<K>>;
    pub fn finish(self) -> Result<(), AppError>; // AppError::Validation { errors } if any
}
/// `FieldCtx` carries what a kind may need: the tenant country (phone), locale set, currency.
```

- Newtypes are declared with `nutype` (sanitise + validate + private constructor + `serde`) or by hand when the kind needs context (phone needs the tenant country). Either way the type can't hold an invalid value (CQ: "constructors validate").
- `FieldKind::schema()` emits `minLength`/`maxLength`/`pattern`/`minimum`/`maximum`/`format` from the **same constants** `parse` uses, plus `x-sanitize: "<profile>"` and `x-field-kind: "<NAME>"`. DTOs reference kinds as components, so the OpenAPI document carries every rule once.
- Policy documents keep their own pipeline (POL-10: `jsonschema`, then serde, then `validate()`) but use the same newtypes for their value types, and map schema keyword errors to VAL-03 codes (VAL-22).

**VAL-21. Catalog (v1).** Limits come from DDL where the DDL has a CHECK; the rest are set here [D-20].

| Kind | Profile | Rule | Codes |
|---|---|---|---|
| `DisplayName` | line | 1–80 | `required`, `too_long`, `invalid_characters` |
| `LegalName` | line | 1–120 | same |
| `TenantName` | line | 2–80 | + `too_short` |
| `PlaceName` (location, resource, booking type label) | line | 1–80 | same |
| `ShortText` (relationship, purpose, cancel/reject reason, leave reason) | line | 1–200 | same |
| `Address` | multiline | 1–300 | same |
| `Note` (session notes, review notes, admin notes) | multiline | 1–1000 | same |
| `Comment` (progress note, rating comment) | multiline | 1–2000 (FBK-01) | same |
| `I18nLine<MAX>` | line per value | keys ⊆ `Locale`; ≥ tenant default locale; each value 1–MAX | `required` (at `/<field>/<locale>`), `not_allowed` (unknown locale) |
| `Email` | email | VAL-11 | `invalid_format`, `too_long` |
| `Phone` | phone | VAL-11 | `invalid_format` |
| `Password` | secret | `password_min_length`..128 (policy-driven minimum, VAL-23) | `too_short`, `too_long`, `invalid_characters`, and module code `password_weak` (common-password list, IDN-02) |
| `Key` | identifier | `^[a-z0-9_]{2,40}$` | `invalid_format` |
| `Slug` | identifier | DDL pattern, 3–40 | `invalid_format` |
| `ReferenceCode` | identifier | digits only, length = policy `digits` | `invalid_format` |
| `SearchQuery` | line | 1–100 | `too_long` |
| `Id` | none | CAN-10 | `invalid_format` (body), 404 (path, CAN-33) |
| `LocalDate` | none | `YYYY-MM-DD`, 1900-01-01..2100-12-31 | `invalid_format`, `must_be_gte`, `must_be_lte` |
| `BirthDate` | none | `LocalDate` and ≤ today (tenant TZ) | + `must_be_lte {max: today}` |
| `LocalTime` | none | `HH:MM` (`24:00` only as an interval end) | `invalid_format` |
| `Instant` | none | RFC 3339 with offset, 2000-01-01..2100-01-01 | `invalid_format`, `must_be_gte`, `must_be_lte` |
| `Minutes` | none | per field (`duration_minutes` 10..=480 per DDL; others per policy) | `must_be_gte`, `must_be_lte`, `invalid_type` |
| `MoneyAmount` | none | integer, `0..=MONEY_MAX_ABS_MINOR` (negative only where the field says so: credit lines) | `must_be_gte`, `must_be_lte`, `invalid_type` |
| `Currency` | none | ISO 4217 in the v1 supported list (CAN-10), and = tenant currency for requests other than setup | `invalid_format`, `not_allowed` |
| `Bps` | none | 0..=10000 | `must_be_gte`, `must_be_lte` |
| `Country` | none | ISO 3166-1 alpha-2 known | `invalid_format` |
| `TimeZone` | none | IANA via `chrono-tz` | `invalid_format` |
| `Locale` | none | `en` \| `id` | `not_allowed` |
| `Geo` | none | lat −90..90, lng −180..180 | `must_be_gte`, `must_be_lte` |
| `Hue` | none | 0..=359 | same |
| `Capacity` | none | 1..=500 (resource DDL), offering `capacity_min ≥ 1`, `capacity_max ≥ capacity_min` | same + `must_be_gte {other}` |
| `Stars` | none | 1..=5 | same |
| `Rating` (progress category) | none | 1..=`scale_max` | same |
| `Token` | none | 43 chars base64url | 400 `token_invalid` (never a field error, so validity isn't revealed per field) |
| `IdList<MAX>` | none | each `Id`, no duplicates, 0..=MAX | `invalid_format`, `not_unique`, `too_many_items` |

**VAL-22. JSON Schema keyword → field code** (policy documents, T-106, and the client's ajv):

| Keyword | Code | `params` |
|---|---|---|
| `required` | `required` | — |
| `type` | `invalid_type` | `{expected}` |
| `minLength` / `maxLength` | `too_short` / `too_long` | `{min}` / `{max}` |
| `minItems` / `maxItems` | `too_few_items` / `too_many_items` | `{min}` / `{max}` |
| `minimum`, `exclusiveMinimum` | `must_be_gte` | `{min}` |
| `maximum`, `exclusiveMaximum` | `must_be_lte` | `{max}` |
| `pattern`, `format` | `invalid_format` | `{format}` |
| `enum`, `const`, `oneOf` with no match on a tagged enum | `not_allowed` | `{allowed}` |
| `uniqueItems` | `not_unique` | — |
| `additionalProperties: false` | `not_allowed` | — |

**VAL-23. Policy-driven rules on the client.** Rules whose bounds depend on tenant policy (password minimum, leave notice, booking windows) can't live in the static OpenAPI document. The server stays authoritative; for instant feedback the SPA reads them from the endpoint that serves the form:
- `GET /meta/app` gains `rules: { password_min_length }` (public, S0) [D-21].
- Policy forms get bounds from the policy JSON Schema (FE-20).
- Booking and scheduling windows come from dry runs (VAL-09), not from re-implemented rules.

## 5. Validation matrix per request

**VAL-25.** For each request: the field kinds (V3), the request rules (V4), and the stateful checks (V5–V7) with their codes. "Kind" names refer to VAL-21. Module specs remain the source of the business rules; this table is the index that a task copies into its own Interfaces section.

### 5.1 Identity and setup

| Request | V3 fields | V4 | V5–V7 (codes) |
|---|---|---|---|
| `POST /auth/login` | `email: Email`, `password: Password` (length checks **skipped**: any length 1..=128 is accepted so that the response doesn't reveal policy), `totp?: ^\d{6}$` or a 10-character recovery code (T-111) | — | 401 `invalid_credentials` for every failure incl. malformed email (SBD-43), 401 `totp_required`, 423 `locked` (only after a correct password, IDN-06), 429 |
| `POST /auth/forgot` | `email: Email` | — | Always 202, even for a malformed email [D-22]; per-email limit `reset_max_per_hour` silently applied |
| `POST /auth/reset` | `token: Token`, `new_password: Password` | — | 400 `token_invalid` (unknown, used, expired), 422 `password_weak` |
| `POST /auth/invites/{token}/accept` | `password: Password` | — | 404 / 410 `invite_expired`, 422 `password_weak` |
| `POST /auth/switch-role` | `role: Enum<Role>` | — | 403 `forbidden` if the person has no active membership with that role |
| `POST /me/password` | `current_password: Password` (no length check), `new_password: Password` | new ≠ current is **not** checked (allowed) | 401 `invalid_credentials` for a wrong current password, 422 `password_weak` |
| `PATCH /me` | `display_name?: DisplayName`, `phone?: Phone \| null`, `locale?: Locale \| null` | — | email changes aren't allowed here (`not_allowed`) [D-23] |
| `POST /me/totp/confirm` | `code: ^\d{6}$` | — | 422 `validation` `{pointer:"/code", code:"invalid_format"}` for a wrong code |
| `POST /setup/school` | `name: TenantName`, `country: Country`, `locale: Locale`, `currency: Currency` (any supported, not "= tenant"), `timezone: TimeZone`, `preset_id: Enum<PresetId>`, `room_count: 1..=50`, `owner?: {name: DisplayName, email: Email, password: Password}`, `setup_token?: Token` | `owner` required iff self-host first run | 403 `forbidden` (bad setup token, compared in constant time), 409 `invalid_transition` if setup step ≠ 1 |
| `PUT /setup/modules`, `PUT /settings/modules` | `enabled: IdList`-like list of `ModuleId` (closed enum), no duplicates | — | 422 `module_dependency` / `module_required_by` (MOD-06) |
| `PUT /settings/policies/{module}`, `POST …/preview` | `document` per the module's JSON Schema (VAL-22), `effective_from?: Instant` ≥ now − 5 min (clock skew) [D-24], `note?: ShortText` | POL-10 step 3 `validate()` | 422 `policy_violation` (e.g. `core.currency.locked`), `validate_policy_in_context` → `unknown_reference` |

### 5.2 People and catalog

| Request | V3 fields | V4 | V5–V7 |
|---|---|---|---|
| `POST /people` | `display_name: DisplayName`, `legal_name?: LegalName`, `email?: Email`, `phone?: Phone`, `birth_date?: BirthDate`, `address?: Address`, `roles: [Role] 1..=5` unique, `invite: bool`, `guardian_of?: IdList<10>`, `guardians?: [{guardian_id: Id, relationship?: ShortText, is_primary: bool}] ≤ 5` | `invite` requires `email` or `phone`; at most one `is_primary` in `guardians`; `guardian_of` and `student` role are exclusive | 409 `email_taken` (blind-index unique), `unknown_reference` for linked ids, 409 `primary_exists`, 403 for granting `admin`/`owner` without `identity.admins.manage` |
| `PATCH /people/{id}` | as above, all optional, `If-Match` | — | 409 `version_conflict`, `email_taken` |
| `POST /people/{id}/links` | `student_id: Id`, `relationship?: ShortText`, `is_primary: bool` | `student_id ≠ id` | 409 `primary_exists`, `unknown_reference` |
| `POST/PATCH /locations` | `name: PlaceName`, `address?: Address`, `timezone?: TimeZone \| null`, `geo?: Geo \| null` | — | — |
| `POST/PATCH /resources` | `location_id: Id`, `name: PlaceName`, `kind: Enum`, `capacity: Capacity`, `rentable: bool`, `color?: ColorToken`, `sort: 0..=10000` | — | 409 `not_unique` on `/name` (`UNIQUE (tenant_id, name)`) |
| `POST /closures` | `location_id?: Id`, `dates: DateRange` (≤ 366 days), `name: I18nLine<80>`, `kind: Enum` | `from ≤ until` | — |
| `POST /closures/import` | CSV `date_from,date_to,name` (VAL-17) | per row as above | all-or-nothing |
| `POST/PATCH /catalog/offerings` | `key: Key` (create only), `name: I18nLine<80>`, `description?: I18nLine<500>`, `level_track_id?: Id`, `durations_minutes: [Minutes 10..=480] 1..=8` unique ascending, `capacity_min/max: Capacity`, `min_age_months?/max_age_months?: 0..=1200`, `internal: bool`, `color?: ColorToken` | `capacity_max ≥ capacity_min`; `max_age ≥ min_age` | 409 `not_unique` (`/key`) |
| `POST …/price-plans` | `level_id?: Id`, `key: Key`, `kind: Enum`, `amount: Money`, `sessions_included?: 1..=100` (required iff `package`), `effective_from: LocalDate` ≥ today | `sessions_included` iff `kind=package` | currency = tenant; past `effective_from` → `must_be_gte` |

### 5.3 Scheduling

| Request | V3 fields | V4 | V5–V7 |
|---|---|---|---|
| `POST /sessions`, `/sessions/check` | `offering_id: Id`, `resource_id: Id`, `instructor_ids: IdList<5> ≥ 1`, `participant_ids: IdList<500>`, `starts_at: Instant`, `duration_minutes: Minutes`, `kind: Enum`, `override_hours?: bool`, `notes?: Note`; `Idempotency-Key` | — | V6/V7: SCH-01 duration ∈ offering durations (or `custom_duration` perm) → `policy_violation rule=scheduling.slot_step_minutes` / `not_allowed` on `/duration_minutes`; step alignment; SCH-02 opening hours (`override_hours` needs perm, else 403); SCH-04 closure; SCH-06 capacity → 422 `validation` `{pointer:"/participant_ids", code:"too_many_items", params:{max}}`; frozen participants per BIL-08; V8 → 409 `slot_taken` (+ conflicts, alternatives) |
| `POST /sessions/{id}/reschedule` | `starts_at: Instant`, `resource_id?: Id`, `instructor_ids?: IdList<5>`, `reason: ShortText` (required if `reason_required`); `If-Match` | — | session `scheduled` (else 409 `invalid_transition`), same V6–V8 as create |
| `POST /sessions/{id}/cancel` | `reason?: ShortText`, `scope: Enum<session\|participant>`, `participant_id?: Id`, `cause?: Enum<instructor_absence\|other>` | `participant_id` required iff `scope=participant` | reason required per policy → `required` on `/reason`; payer may cancel only own participant; 409 `invalid_transition` |
| `POST /series` | `offering_id, location_id, resource_id, instructor_id: Id`, `weekday: Weekday`, `start_local: LocalTime`, `duration_minutes: Minutes`, `capacity: Capacity`, `valid_from: LocalDate`, `valid_to?: LocalDate` | `valid_to ≥ valid_from` | SCH-07 first `materialize_weeks_ahead` occurrences all free → 409 `slot_taken` + `first_conflict_date` |
| `GET /sessions`, `/availability` | `from/to: Instant` (span ≤ 62 days), `date: LocalDate`, ids, `status: [Enum]`, `limit`, `cursor` | `to > from` | scope applied in SQL (SBD-41) |
| `PUT /instructors/{id}/availability` | `[{weekday, start_local, end_local}] ≤ 21` | `end_local > start_local`; no overlap per weekday | self or staff |

### 5.4 Enrollment

| Request | V3 fields | V4 | V5–V7 |
|---|---|---|---|
| `POST /public/registrations` (multipart) | `contact: {name: DisplayName, email?: Email, phone?: Phone, address?: Address}`, `students: [{name: DisplayName, birth_date?: BirthDate, offering_id: Id, level_id?: Id, preferred_slots?: [{weekday, from: LocalTime, to: LocalTime}] ≤ 7, note?: Note}] 1..=10`, `consent: true` (literal), `website: ""` honeypot, `captcha_token?: ShortText`; file part `payment_proof` (VAL-15) | email or phone required; `to > from` per slot | Honeypot filled → **202 accepted and discarded** (no signal to bots) [D-25]; captcha failure → 422 `{pointer:"/captcha_token", code:"invalid_format"}`; policy `collect_birth_date=Required` → `required`; `collect_address=Required` → `required`; public offerings only (`unknown_reference`); 429 per ENR-03 |
| `POST /registrations/{id}/approve` | `students: [{index: 0..=9, offering_id, level_id?, series_id? \| new_series?: SeriesCreate, starts_on: LocalDate}]`, `link_guardian_id?: Id`, `send_invite: bool` | exactly one of `series_id`/`new_series`; every `index` exists once | ENR-01 guardian rule, ENR-02 age → `policy_violation`; series capacity → 409 (whole approval rolls back) |
| `POST /enrollments/{id}/change` | `effective_on: LocalDate ≥ today`, `level_id?, series_id?, price_plan_id?: Id` | at least one change | `If-Match`; enrollment `active` |
| `POST /enrollments/{id}/end` | `reason: Enum<ended_reason>`, `on: LocalDate` | — | 409 `invalid_transition` |
| `POST /enrollments/{id}/leaves` | `dates: DateRange` (≤ 365 days), `reason?: ShortText`, `confirm_end?: bool` | — | ENR-07 with **snapshotted** `leave_terms`: notice → `policy_violation rule=enrollment.leave.min_notice_days`; limit → `rule=enrollment.leave.max_days_per_year` + `{used, remaining}`; overlap → 409 `leave_overlap` (exclusion constraint) [D-30] |
| `POST /import/students/preview` | file part `file` (VAL-15/17); columns per ENR-11, each with its kind | per row | per row: `unknown_reference` for offering/level/instructor/resource keys |

### 5.5 Billing

| Request | V3 fields | V4 | V5–V7 |
|---|---|---|---|
| `POST /invoices` (manual draft) | `payer_id: Id`, `lines: [{kind: Enum<adjustment\|tuition\|…>, description: I18nLine<200>, student_id?: Id, quantity: 1..=1000, unit: Money}] 1..=50`; `Idempotency-Key` | `credit` lines may be negative, others ≥ 0 | totals computed server-side only (SBD-57); currency = tenant |
| `POST /invoices/{id}/lines` | one line as above, `reason: ShortText`; `If-Match` | — | invoice not `paid`/`cancelled` → else 409 `invalid_transition`; total ≥ 0 after the line → else 422 `validation` `{pointer:"/unit/amount_minor", code:"must_be_gte", params:{min}}` [D-26] |
| `POST /invoices/{id}/cancel` | `reason: ShortText`; `If-Match` | — | `paid_minor = 0` → else 409 `has_payments` (owner override exempt) [D-30] |
| `PATCH /invoices/{id}/reference` | `reference_code: ReferenceCode` | — | status `needs_manual_reference`; 409 `not_unique` among open invoices |
| `POST /invoices/{id}/payments` | `amount: Money` (> 0), `method: Enum`, `received_on: LocalDate ≤ today`, `reference_entered?: ShortText`, `proof_file_id?: Id`; `Idempotency-Key` | — | payer: own invoice only (404 otherwise), BIL-07 partial / minimum first payment → `policy_violation`; staff: warnings only; invoice open |
| `POST /payments/{id}/verify` / `reject` | `note?: Note` / `reason: ShortText` | — | payment `submitted` → else 409 `invalid_transition`; duplicate warning (BIL-07) |
| `POST /payers/{id}/unfreeze` | `note: ShortText`, `override?: bool` | — | 409 `unpaid_overdue` unless override with `billing.override.unfreeze` |
| `POST /billing/run-cycle` | `period_key: ^\d{4}-(0[1-9]\|1[0-2])$`, `dry_run: bool` | period ≤ next month | — |

### 5.6 Attendance, rental, notify, feedback, privacy

| Request | V3 fields | V4 | V5–V7 |
|---|---|---|---|
| `POST /sessions/{id}/checkin` (multipart) | `json: {client_at: Instant, gps?: {lat, lng: Geo, accuracy_m: 0..=10000}, face_ratio_bps?: Bps}`, file `photo` (VAL-15 `checkin_photo`), header `X-EQ-Queued-At?` | — | ATT-01 (instructor of the session, `scheduled`, 409 `already_checked_in`); ATT-02 checks never *reject the request* — they decide the outcome |
| `POST /checkins/{id}/review` | `decision: Enum<approve\|reject>`, `note?: Note` | `note` required for reject [D-27] | status `pending_review` → else 409 `invalid_transition` |
| `PUT /sessions/{id}/marks` | `[{person_id: Id, status: Enum, note?: ShortText}] 1..=500`, unique `person_id` | — | each `person_id` is a participant (`unknown_reference`); window → 403 `marks_locked` |
| `POST /reservations`, `/rental/quote` | `resource_id: Id`, `booking_type: Key`, `starts_at: Instant`, `duration_minutes: Minutes`, `purpose?: ShortText`; `Idempotency-Key` (create) | — | RNT-02 (each → `policy_violation rule=rental.<key>`), rate card missing → `rule=rental.rate_card_missing`, frozen → `rule=billing.overdue.action`, V8 → 409 `slot_taken` |
| `POST /public/reservations` | as above + `guest: {name: DisplayName, email: Email, phone: Phone}`, honeypot | — | 202 always (verification link); rate limits |
| `PUT /settings/channels/{channel}` | email: `{host: hostname ≤ 253, port: 1..=65535, username: ShortText, password: Password-like secret 1..=256 (no policy min), from_name: DisplayName, from_email: Email, security: Enum}`; whatsapp per NTF-08 (`url: HttpsUrl` for `whatsapp_http`, headers ≤ 10, `body_template` ≤ 4 KiB, MiniJinja-parsed at save) | — | secrets omitted in a PATCH keep the stored value (`"••••"` placeholder is rejected as a value) [D-28]; step-up password for non-owner (SEC-23); outbound host is operator-allow-listed (SBD-47) |
| `POST /students/{id}/notes` | `ratings: {<category key>: Rating}`, `comment?: Comment`, `session_id?: Id` | keys ⊆ policy categories | FBK-01 taught in last 90 days → 403 `not_eligible` |
| `POST /instructors/{id}/ratings` | `student_id: Id`, `stars: Stars`, `comment?: Comment` | — | FBK-03 → 403 `not_eligible`, 409 `not_unique` (monthly) |
| `POST /privacy/deletion-requests` | `subject_id: Id`, `reason?: ShortText` | — | scope: self/linked/staff; open invoices → 409 `open_invoices` (AUD-11) [D-30] |
| `PUT /me/drafts/{form_key}` | `form_key` path: `Key`-like dotted `^[a-z0-9_.]{3,80}$`; body `payload`: any JSON ≤ 256 KiB | — | drafts may contain unvalidated user input by design; they are never executed, only returned to the same person (FE-30) |

## 6. Client-side validation

**VAL-30. Generated schemas.** `pnpm gen:api` runs:
1. `openapi-typescript` → `src/api/schema.gen.ts` (types for `openapi-fetch`, unchanged from API-02);
2. `@hey-api/openapi-ts` with only the **`zod` plugin** → `src/api/zod.gen.ts` (one zod schema per component, including every field kind with its `minLength`/`maxLength`/`pattern`/bounds);
3. `scripts/gen-field-meta.ts` → `src/api/fields.gen.ts`: a map `component name → { sanitize: Profile }` read from `x-sanitize`.

Forms import `zDisplayName`, `zSessionCreate`, … from the generated file. Hand-written zod is allowed only for client-only fields (password confirmation) and for composing generated schemas. [tool: CI regenerates and fails on diff (TST-10); Biome rule forbids `z.string().max(` literals in `src/modules/**` — a `scripts/check-web-boundaries.sh` grep]

**VAL-31. Parity.** `contracts/validation/cases/<kind>.json` holds cases `{input, ok, canonical?, codes?}` (boundary lengths, every forbidden-character class, whitespace variants, emails, phones per supported country, dates, times, money edges). The same files run in:
- Rust: `eq_core::fields::tests::cases_match` (every kind's `parse`);
- Web: `src/lib/validation/cases.test.ts` (generated zod + client sanitiser).

For every case both sides agree on `ok`, the canonical value and the code. Phone cases use numbers whose validity is stable across libphonenumber metadata versions. [tool: TST-17]

**VAL-32. Client sanitisers** (`src/lib/validation/sanitize.ts`) implement the VAL-11 profiles with `String.prototype.normalize('NFC'/'NFKC')` and the VAL-12 character classes as Unicode property regexes (`/\p{Cc}/u`, explicit bidi ranges, `/\p{M}{9,}/u`). Phones use `libphonenumber-js` (`parsePhoneNumberWithError(input, tenantCountry)`, `.isValid()`, `.number` = E.164). The sanitiser runs as `z.preprocess` **on blur and on submit**, never on every keystroke (it would move the caret and eat spaces while typing).

**VAL-33. Form behaviour** (react-hook-form + `@hookform/resolvers/zod`):
- `mode: "onTouched"` then `reValidateMode: "onChange"`: a field shows its first error after the user leaves it, and clears it as soon as it's fixed.
- On submit, all fields validate; focus moves to the first invalid field, and an error summary at the top lists every error with links to the fields (WCAG 3.3.1, 3.3.3).
- Each input has `aria-invalid` and `aria-describedby` pointing at its error text (FE-36). Errors are text, not only colour.
- Messages: `t('errors.field.<code>', params)` with `params` from zod 4 issues mapped to VAL-03 codes by `src/lib/validation/zodToFieldCode.ts`: `too_small`/`too_big` → `too_short`/`too_long` on strings, `too_few_items`/`too_many_items` on arrays, `must_be_gte`/`must_be_lte` on numbers; `invalid_format` → `invalid_format`; `invalid_value` → `not_allowed`; `invalid_type` → `invalid_type` (or `required` when the input is `undefined`/`null`); `unrecognized_keys` → `not_allowed`; `custom` issues carry the VAL-03 code in `params.code`.
- Server field errors (422) map back onto the form: `pointer` `/students/0/name` → RHF path `students.0.name` via `setError`; pointers with no matching field go to the error summary. `policy_violation` with a `rule` shows next to the field that owns that policy key if the form has one, otherwise in the summary (ERR-61).

**VAL-34. Submit discipline.**
- The submit button is disabled while a mutation is pending; a second click does nothing.
- Money and booking forms create one `Idempotency-Key` (`crypto.randomUUID()`) when the form opens, reuse it for retries of the same submission, and rotate it only after success or an edit of the form (API-08).
- Mutations are never retried automatically (ERR-50). A network failure shows "Not sent. Try again" with the same key.
- The client never sends computed money totals, prices or statuses (CAN-21). Price displays during editing come from `/rental/quote`, `/sessions/check` or `/invoices/preview`-style endpoints (VAL-09), debounced 300 ms, labelled as estimates until the server confirms.

**VAL-35. Async advisory checks** (`/sessions/check`, quote, policy preview, email uniqueness on `/people` via a debounced `GET /people?email=` lookup that returns only `{exists: bool}` to staff with `identity.people.write` [D-29]) are UX only. Their result never blocks submit by itself; the submit response is what counts.

**VAL-36. Offline.** Offline-queued check-ins (ATT-07) are validated with the same zod schema before queueing. The server re-validates on replay; a 422 on replay moves the item to a visible "Couldn't send" list with the error, never silently drops it (STM-46).

## 7. Server-side implementation rules

- **VAL-40.** Handlers take `ValidatedJson<XxxDto>` / `ValidatedQuery<XxxQuery>` / `ValidatedMultipart<…>` (in `eq_core::http`), never axum's plain `Json` [review; a `scripts/check-layers.sh` grep forbids `axum::Json` in request position]. These extractors map rejections to problem+json per V0–V2.
- **VAL-41.** The adapter converts the DTO into a command with `TryFrom` and a `Collector` (V3–V4). The service receives only the command type. Services and domain functions never see `Raw<_>`, `String` for a typed field, or `serde_json::Value` for a typed body.
- **VAL-42.** Context checks (V5) are in the service, using repo lookups **scoped by the actor** (SBD-41). An id that exists but is out of scope gives the same `unknown_reference` / 404 as one that doesn't exist.
- **VAL-43.** Domain and policy checks (V6–V7) are pure functions that return typed errors (ERR-40). They take values that are already canonical; they never re-sanitise.
- **VAL-44.** Jobs and event handlers validate their payload against the payload struct (CAN-62); a payload that doesn't parse is `JobError::Permanent` (ERR-51). Data read back from the DB is trusted (it passed the pipeline when written), except legacy imports, which run the pipeline.
- **VAL-45.** Ports and events never pass raw input across modules; they pass canonical newtypes or ids (CA-12).

## 8. Testing

- **VAL-50.** Every field kind: boundary tests at min−1, min, max, max+1, the empty string, whitespace-only, each forbidden-character class, and an idempotence property test `parse(canonical(x)) == canonical(x)` (`proptest`). [TST-02]
- **VAL-51.** Every endpoint with a body: an API test sending an unknown field (`not_allowed`), a mass-assignment attempt (`status`, `tenant_id`, `version`, `total_minor`), a wrong type (`invalid_type`), and a request with several invalid fields asserting all errors are returned together (VAL-04). [TST-04]
- **VAL-52.** Abuse cases (SBD-30): bidi override in a display name, a 10 000-character name, a zip-bomb XLSX, a PNG with a 30 000×30 000 header, an SVG with `<script>` and `xlink:href="javascript:"`, a CSV cell `=HYPERLINK(...)` round-tripped through import and export, a PDF payment proof served with `attachment`, a polyglot JPEG/HTML upload served with `nosniff`.
- **VAL-53.** Schemathesis (CAN-96) fuzzes every endpoint; any 5xx is a bug.
- **VAL-54.** Web: `cases.test.ts` (VAL-31), RHF mapping tests for server errors (ERR-83), and a Playwright test per form type checking focus-on-first-error and the error summary.

## 9. Checklist: adding a field

1. Pick an existing kind from VAL-21, or add one to `eq_core::fields` with its constants, profile, `parse`, `schema` and cases in `contracts/validation/cases/`.
2. Classify the field (SBD-10). S2 → encrypted column and blind index if looked up.
3. Reference the kind in the DTO as `Raw<Kind>`; add it to the command with the `Collector`.
4. Add V4–V7 rules with their codes, and list them in the module spec's endpoint table.
5. `make api`; use the generated zod schema in the form; add `errors.field.*`/module codes to `en` and `id` if new.
6. Tests per §8.

## 10. Decisions taken in this document

| ID | Decision | Why |
|---|---|---|
| D-16 | A missing/wrong `Content-Type` is 400 `bad_request`, not 415 | Keeps the error registry small; axum's 415 is remapped |
| D-17 | Duplicate JSON keys are rejected | `{"amount":1,"amount":1000}` must not be ambiguous between proxies and the app |
| D-18 | Noncharacters and private-use code points are forbidden in text | They're invisible or font-dependent and only useful for spoofing |
| D-19 | More than 8 combining marks in a row are forbidden | Stops "Zalgo" text breaking layouts; real scripts need far fewer |
| D-20 | Text limits not fixed by DDL are set in VAL-21 (place names 80, short text 200, notes 1000, address 300) | Every string needs a bound (SBD-56) |
| D-21 | `GET /meta/app` exposes `rules.password_min_length` | The SPA can show the real minimum before submit |
| D-22 | `/auth/forgot` answers 202 even for a malformed email | No difference between "bad format" and "unknown" (SBD-43) |
| D-23 | Email can't be changed through `PATCH /me` | A login identifier change needs verification; out of scope for v1 (staff can change it on the person) |
| D-24 | Policy `effective_from` may be up to 5 minutes in the past | Tolerates clock skew between browser and server without allowing back-dating |
| D-25 | A filled honeypot returns 202 and discards the registration | Gives bots no signal |
| D-26 | Adjustments can't make an invoice total negative | Credit beyond the total is a refund, which v1 doesn't do (BIL-10) |
| D-27 | Rejecting a check-in requires a note | The instructor sees why (`attendance.checkin_reviewed`) |
| D-28 | Secret fields are write-only; omitting them keeps the stored value | Redacted reads (SEC-23) must round-trip through the settings form |
| D-29 | A staff-only `{exists}` email lookup supports the "already a person" hint | Same information staff already get from 409 `email_taken`, earlier |
| D-30 | New conflict codes `leave_overlap` (enrollment), `has_payments` (billing) and `open_invoices` (audit/privacy), registered per ERR-21 | Each names a distinct situation the UI explains differently. For `open_invoices` the request is refused (409) instead of being stored as `rejected` |
