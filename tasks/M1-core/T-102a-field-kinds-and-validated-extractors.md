---
id: T-102a
title: Add field kinds, text sanitisers, limits and the ValidatedJson/ValidatedQuery extractors
milestone: M1
status: review
depends_on: [T-002, T-102]
spec_refs: [VAL-03, VAL-04, VAL-05, VAL-06, VAL-07, VAL-10, VAL-11, VAL-12, VAL-18, VAL-19, VAL-20, VAL-21, VAL-23, VAL-25, VAL-31, VAL-40, VAL-41, VAL-50, CAN-10, CAN-21, CAN-22, CAN-32, CAN-33, SBD-20, SBD-44, ERR-31, OSS-02]
estimated_size: M
---

## Goal
Every request field in Ritma is parsed through one shared set of **field kinds**. Each kind sanitises, validates and canonicalises a value, reports every problem with a JSON Pointer and a closed error code, and describes itself in OpenAPI from the same constants. Handlers get typed, canonical values or a 422 with **all** field errors at once.

## Context
- **"Parse, don't validate"** at the HTTP boundary (SBD-20). The request pipeline (VAL-07) is V0 transport → V1 JSON syntax → V2 shape (serde) → **V3 fields** → **V4 request rules** → V5 context → V6 domain → V7 policy → V8 DB. This task builds V1–V4 plumbing and V3 itself.
- **Field error codes** (VAL-03, extends ERR-31), the only ones allowed: `required`, `invalid_type` `{expected}`, `too_short` `{min}`, `too_long` `{max}`, `too_few_items` `{min}`, `too_many_items` `{max}`, `must_be_gte` `{min}|{other}`, `must_be_lte` `{max}|{other}`, `invalid_format` `{format}`, `invalid_characters`, `not_allowed` `{allowed}?`, `not_unique`, `unknown_reference`. `pointer` is RFC 6901 into the request body.
- **Sanitising profiles** (VAL-11), in order:
  - `line`: NFC → reject forbidden chars (`invalid_characters`) → remove U+200B, U+FEFF, U+2060, U+00AD → every whitespace run (`char::is_whitespace`) → one U+0020 → trim → empty ⇒ `null` if optional / `required` → length in code points.
  - `multiline`: like `line`, but `\r\n`/`\r` → `\n`, tab → space, collapse runs within a line, trim line ends, ≥ 3 blank lines → 2, trim the whole text.
  - `identifier`: NFC, trim, no internal whitespace, case unchanged, then the kind's pattern.
  - `email`: NFC, trim, drop one trailing `.` on the domain, lowercase all, `email_address` (no quoted local part, no IP literal, domain has a dot), ≤ 254 total, local ≤ 64.
  - `phone`: trim, `PHONE_NUMBER_UTIL.parse(s, Some(region))` (`region` from `ctx.country`), `is_valid()`, output E.164 via `format_as(PhoneNumberFormat::E164)`; letters rejected.
  - `secret`: NFKC only; no trim; reject C0 controls; never echo the value or put it in `params`.
- **Forbidden characters** (VAL-12, all profiles except `secret`): C0 except `\n` in multiline, DEL, C1, U+202A–U+202E, U+2066–U+2069, noncharacters (U+FDD0–U+FDEF and U+xFFFE/U+xFFFF), private use (U+E000–U+F8FF, planes 15–16), more than 8 consecutive combining marks. LRM/RLM allowed in `line`/`multiline`, stripped in `identifier`/`email`.
- **Canonical forms** (CAN-10): `Id` lowercase hyphenated UUID only; `Instant` RFC 3339 **with offset** (missing offset rejected), ≤ 6 fractional digits; `LocalDate` `YYYY-MM-DD`; `Geo` rounded to 6 decimals; booleans only `true`/`false` (also in query strings); `Money` within ±10^15 (`MONEY_MAX_ABS_MINOR`, technical).
- **Requests** (CAN-21/22): unknown fields rejected → `{pointer:"/field", code:"not_allowed"}`; duplicate JSON keys rejected (D-17); depth ≤ 32; on create, absent = `null`; on PATCH, JSON Merge Patch semantics (absent = unchanged, `null` = clear, arrays replaced).
- **Query and path** (CAN-32/33): repeated params for lists (`axum_extra::extract::Query`); unknown query params → 400 `bad_request`; `limit` 1..=200; a malformed path id → 404 `not_found`.
- **Rejections** (VAL-07 V0–V2): missing/wrong `Content-Type` → 400 `bad_request` (not 415, D-16); JSON syntax → 400 `bad_request`; a shape error → 422 `validation` with the pointer from `serde_path_to_error`.
- Unknown enum variants use the closed field code `not_allowed` with the allowed values in params (VAL-03).

## Scope
### In
- `eq_core::text`: the six profiles as pure functions (`sanitize_line`, `sanitize_multiline`, …) returning `Result<String, TextIssue>`.
- `eq_core::limits`: technical constants of VAL-18 (body sizes are configured in T-115; here only JSON depth, default array length 100, `q` 1–100, page size 1–200, cursor ≤ 512, `MONEY_MAX_ABS_MINOR`), each with `// technical:`.
- `eq_core::fields`: `FieldKind`, `Raw<K>`, `FieldIssue` (code, safe params, optional relative pointer), `FieldCtx { country: Country, tenant_currency: Currency, tenant_locale: Locale, today: NaiveDate, password_min_length: Option<usize> }`, `Collector`, and these kinds from VAL-21: `DisplayName`, `LegalName`, `TenantName`, `PlaceName`, `ShortText`, `Address`, `Note`, `Comment`, `I18nLine<MAX>`, `Email`, `Phone`, `Password`, `Key`, `Slug`, `SearchQuery`, `IdField`, `LocalDateField`, `BirthDate`, `LocalTimeField`, `InstantField`, `Minutes<MIN,MAX>`, `MoneyAmount`, `CurrencyField`, `BpsField`, `CountryField`, `TimeZoneField`, `LocaleField`, `GeoField`, `Hue`, `Stars`, `IdList<MAX>`. `Country` is `celes::Country`. Kinds wrap the T-102 types where they exist (don't duplicate `Money`, `Bps`, `LocalTime`, `GeoPoint`, `Locale`, `Currency`).
- Each kind follows the accepted input in its CAN-10 row. `Key` is checked without trimming per CAN-10, even though VAL-21 classifies it under the `identifier` profile.
- An identifier containing internal whitespace is `invalid_format`; only characters forbidden by VAL-12 use `invalid_characters`.
- Each kind's `schema()` emits OpenAPI constraints from the same constants plus `x-sanitize` and `x-field-kind`; `Raw<K>` implements `ToSchema` as a `$ref` to the kind's component. Const-generic kinds include their bounds in the component name to keep schemas distinct. The password's policy minimum is dynamic and is served separately from static OpenAPI bounds (VAL-23).
- `eq_core::http::{ValidatedJson<T>, ValidatedQuery<T>}` extractors with the rejection mapping above.
- `contracts/validation/cases/<kind>.json` for every kind in Scope, and the Rust runner `eq_core::fields::tests::cases_match` (VAL-31).
- `errors.field.*` code constants registered per ERR-21 (`eq_core::errors::FIELD_ERROR_CODES`).

### Out
- Multipart parsing and file kinds (T-303 adds `ValidatedMultipart` and `files::accept`).
- Client-side zod generation and the web runner of the same cases (T-306a).
- Module-specific kinds (`ReferenceCode`, `Rating`, …) and request rules (each module task).
- Translations of `errors.field.*` (T-306a adds `en` + `id`).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/text.rs` | create | sanitising profiles |
| `crates/eq-core/src/limits.rs` | create | technical limits |
| `crates/eq-core/src/fields/mod.rs`, `fields/kinds.rs`, `fields/collector.rs`, `fields/schema.rs` | create | field kinds |
| `crates/eq-core/src/http/validated.rs` | create | extractors + rejection mapping |
| `crates/eq-core/src/errors.rs` | modify | field code registry |
| `contracts/validation/cases/*.json` | create | shared parity cases |
| `Cargo.toml` | modify | central workspace dependency declarations per CQ-27 |
| `crates/eq-core/Cargo.toml` | modify | use the workspace `nutype`, `unicode-normalization`, `email_address`, `rlibphonenumber`, `serde_path_to_error`, `axum-extra` (query), and `celes` dependencies |

## Interfaces
```rust
// eq_core::fields
pub enum Profile { Line, Multiline, Identifier, Email, Phone, Secret, None }

pub struct FieldIssue { pub code: &'static str, pub params: serde_json::Map<String, serde_json::Value> }

pub trait FieldKind: Sized {
    type Input: serde::de::DeserializeOwned + utoipa::ToSchema;
    const NAME: &'static str;
    fn component_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(Self::NAME)
    }
    const PROFILE: Profile;
    fn parse(raw: Self::Input, ctx: &FieldCtx) -> Result<Self, Vec<FieldIssue>>;
    fn schema() -> utoipa::openapi::schema::Schema;
}

#[derive(serde::Deserialize)]
#[serde(transparent)]
pub struct Raw<K: FieldKind>(K::Input, #[serde(skip)] std::marker::PhantomData<K>);

pub struct Collector<'c> { ctx: &'c FieldCtx, errors: Vec<FieldError> }
impl<'c> Collector<'c> {
    pub fn new(ctx: &'c FieldCtx) -> Self;
    pub fn field<K: FieldKind>(&mut self, pointer: &str, raw: Raw<K>) -> Option<K>;
    pub fn optional<K: FieldKind>(&mut self, pointer: &str, raw: Option<Raw<K>>) -> Option<Option<K>>;
    pub fn list<K: FieldKind>(&mut self, pointer: &str, raw: Vec<Raw<K>>, min: usize, max: usize) -> Option<Vec<K>>;
    pub fn push(&mut self, pointer: &str, issue: FieldIssue); // V4 cross-field rules
    pub fn finish(self) -> Result<(), AppError>;              // Validation { errors } if any
}

// eq_core::http
pub struct ValidatedJson<T>(pub T);   // T: DeserializeOwned; rejects unknown fields via #[serde(deny_unknown_fields)] on T
pub struct ValidatedQuery<T>(pub T); // Query DTOs must use #[serde(deny_unknown_fields)] (CAN-32)
```

Usage pattern every handler follows (VAL-41):
```rust
#[derive(Deserialize, ToSchema)] #[serde(deny_unknown_fields)]
pub struct PersonCreateDto { display_name: Raw<DisplayName>, email: Option<Raw<Email>>, /* … */ }

impl PersonCreateDto {
    fn into_command(self, ctx: &FieldCtx) -> Result<PersonCreate, AppError> {
        let mut c = Collector::new(ctx);
        let display_name = c.field("/display_name", self.display_name);
        let email = c.optional("/email", self.email);
        c.finish()?;
        Ok(PersonCreate { display_name: display_name.expect("checked"), email: email.expect("checked") })
    }
}
```
(`expect` after `finish()` is allowed by an `#[expect(clippy::expect_used, reason = "Collector::finish returned Ok, so every field parsed")]` on the generated helper, or use a small `collect!` macro that avoids it; pick one and document it.)

## Business rules
All limits here are technical (`// technical:`) or come from the DDL. Policy-driven bounds (password minimum) are passed in by the caller; `None` means no policy minimum is available to this field context and does not synthesize a default.

## Security
- Triggers: T1 (all public and cookie endpoints parse through this), T3 (email, phone, legal name are S2), T4 (passwords).
- Data classes touched: S2 (email, phone, names, birth date, address), S3 (password: `secret` profile).
- Threats → controls:
  - **T**ampering / **E**levation: mass assignment (`status`, `tenant_id`, `version`) → `deny_unknown_fields` (CAN-21).
  - **S**poofing: look-alike and bidi-reordered names ("Trojan Source") → VAL-12 forbidden characters.
  - **D**enial of service: giant strings, deep JSON, huge arrays → VAL-18 limits, length checks after NFC.
  - **I**nformation disclosure: passwords or tokens echoed in errors or logs → `secret` profile never appears in `params`, `Debug` redacted (SBD-50).
- Abuse-case acceptance criteria:
  - `display_name` with U+202E → 422 `invalid_characters` → `fields::tests::rejects_bidi_override`.
  - A body with `"tenant_id"` → 422 `{pointer:"/tenant_id", code:"not_allowed"}` → `http::validated::tests::rejects_unknown_field`.
  - `{"a":1,"a":2}` → 400 `bad_request` → `http::validated::tests::rejects_duplicate_keys`.
  - A 33-level nested body → 400 → `http::validated::tests::rejects_deep_json`.

## Implementation notes
1. Start with `text.rs` and its property tests: `sanitize(sanitize(x)) == sanitize(x)` for every profile (VAL-50).
2. Use `nutype` for kinds without context (`#[nutype(sanitize(with = text::line), validate(len_char_min = 1, len_char_max = 80), derive(Debug, Clone, PartialEq, Eq, Serialize, AsRef))]`). `Phone` needs `FieldCtx`, so implement it by hand.
3. Map `nutype` errors to `FieldIssue` codes in one function per kind; never let a `nutype` error message reach the client.
4. Build `schema()` with `utoipa::openapi::ObjectBuilder` from the same `const`s used in `parse` (no literal written twice). Add `x-sanitize`, `x-field-kind` with `.extensions(…)`.
5. Duplicate keys: deserialize through a `serde_json::Value` visitor that errors on a repeated key, or use `serde_json`'s `RawValue` + a small checking visitor; measure and keep it cheap (bodies ≤ 1 MiB).
6. Write cases first (`contracts/validation/cases/display_name.json`, …), then make `cases_match` pass. Include boundary lengths (min−1, min, max, max+1), whitespace variants, each forbidden class, emoji ZWJ sequences (allowed), Indonesian and international phone numbers with stable validity.

## Acceptance criteria
- AC1. `"  Ayu   Sari  "` as `DisplayName` → `"Ayu Sari"`. → `fields::tests::display_name_collapses_whitespace`
- AC2. A request with 3 invalid fields returns one 422 with 3 errors in `Collector` invocation order. The DTO interface discards source-object ordering before V3 collection. → `http::validated::tests::collects_all_field_errors`
- AC3. `"Budi@Example.COM "` → `"budi@example.com"`; `"budi@[127.0.0.1]"` → `invalid_format`. → `fields::tests::email_canonical`
- AC4. `"0812-3456-7890"` with country ID → `"+6281234567890"`. → `fields::tests::phone_national_to_e164`
- AC5. `"2026-11-05T15:00:00"` (no offset) → `invalid_format`. → `fields::tests::instant_requires_offset`
- AC6. Every kind's OpenAPI schema carries the same static bounds as its parser; policy-driven password minima are served separately from static OpenAPI (VAL-23). → `fields::tests::schema_matches_parse_bounds`
- AC7. Every case file passes. → `fields::tests::cases_match`
- AC8. A malformed path UUID on a test route → 404 `not_found`. → `http::validated::tests::bad_path_id_is_404`
- AC9. The four abuse cases in Security pass.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/text.rs` | `line_profile_*`, `multiline_profile_*`, `identifier_profile_*`, `email_profile_*`, `secret_profile_nfkc_no_trim`, `prop_sanitize_idempotent` |
| `crates/eq-core/src/fields/*.rs` | `display_name_collapses_whitespace`, `optional_empty_line_is_null_but_invalid_characters_are_reported`, `identifier_whitespace_is_invalid_format`, `rejects_bidi_override`, `email_canonical`, `phone_national_to_e164`, `instant_requires_offset`, `password_policy_minimum_rejects_short_value`, `schema_matches_parse_bounds`, `raw_field_schema_is_a_reference`, `cases_match`, `validated_newtypes_deserialize_through_their_invariants` |
| `crates/eq-core/src/http/validated.rs` | `rejects_unknown_field`, `unknown_enum_variant_is_not_allowed_with_allowed_values`, `shape_errors_use_nested_object_and_array_pointers`, `rejects_duplicate_keys`, `rejects_deep_json`, `collects_all_field_errors`, `wrong_content_type_is_400`, `bad_path_id_is_404`, `unknown_query_param_is_400` |

## Definition of done
- [x] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` pass
- [x] Guard scripts pass: `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed; no new `#[allow]`
- [x] Docs/comments cite spec IDs (`/// VAL-11: …`)
- [x] Commit `feat(core): add field kinds and validated extractors [T-102a]`, signed off

## Report

**Plan** — 1. Add the task's VAL-21 dependencies through workspace declarations, use architect-approved `rlibphonenumber` 2, and update the lockfile → verify the resolved graph with `cargo deny check`. 2. Implement the six pure sanitising profiles and named technical limits → verify profile boundaries and idempotence. 3. Implement field kinds, collector, canonical wrappers and OpenAPI schemas → verify parser, bounds and schema tests, including ISO alpha-2 to parser-region conversion. 4. Add the shared validation corpus and runner → verify every fixture with `cases_match`. 5. Implement JSON/query extractors and path-id parsing → verify duplicate keys, depth, shape pointers, content type, unknown keys, aggregation and path mapping. 6. Export modules → verify workspace compilation. 7. Exercise request behavior, review SBD-81 and scan for secrets → record non-applicable live paths and any baseline finding. 8. Rerun Rust, policy, guard and task-index checks, complete this Report, set `review`, regenerate the index and make the signed task commit.

**Assumptions** — `celes::Country` is the ISO 3166-1 value from CAN-10/OSS-02. Its alpha-2 code is converted to the `rlibphonenumber::Region` used by the parser; unsupported region mapping is a validation error, and vanity phone numbers remain rejected. Reuse T-102's validated `Currency`, `Money`, `Bps`, `GeoPoint`, `LocalTime` and `Locale`. `FieldCtx` supplies tenant locale, tenant-local `today` and an optional policy-provided password minimum. Secret input uses secrecy-backed storage and redacted formatting per SBD-50. `Key` must already match its CAN-10 canonical pattern; no surrounding whitespace is trimmed. No business bound is inferred.

**Test plan** — Given canonical and invalid field inputs, when each kind parses them, then it returns the canonical typed value or a closed VAL-03 issue (`fields::tests::display_name_collapses_whitespace`, `rejects_bidi_override`, `email_canonical`, `phone_national_to_e164`, `instant_requires_offset`, `schema_matches_parse_bounds`, `cases_match`). Given an optional line with empty sanitized content, when collected, then it becomes absent; forbidden characters remain errors (`fields::collector::tests::optional_empty_line_is_null_but_invalid_characters_are_reported`). Given malformed JSON, duplicate keys, unknown body/query fields, excessive depth, bad content type, nested shape errors or a malformed path ID, when passed to the extractors/helpers, then the specified 400/404/422 problem is returned with the correct pointer and code (`http::validated::tests::*`). Edge cases include min−1/min/max/max+1, whitespace variants, forbidden character classes, allowed emoji ZWJ and LRM/RLM, 8/9 combining marks, identifier no-trim behavior, email bounds, supported country parsing, UUID/date/time/offset forms, money and numeric bounds, duplicate IDs, repeated query values and unknown query keys. Abuse cases cover bidi override, mass assignment, duplicate JSON keys, depth 33 and password-redaction output using a fake secret.

**Summary** — Added shared text sanitizers, named technical limits, 31 validation contract fixtures, typed field kinds, collection and schema support, and JSON/query extractors. Phone parsing follows the architect-resolved `rlibphonenumber` 2 dependency, and request rejection behavior is exercised through in-process Axum requests. The full Rust workspace, dependency policy and guard checks pass; the repository-wide secret scan reports one inspected pre-existing false positive.

**Spec read** — `spec/22-validation-and-sanitization.md` §§1–4 (VAL-03..21, VAL-31, VAL-40/41/50); `spec/23-contracts-and-canonical-forms.md` §§1–3 (CAN-10/21/22/32/33); `spec/16-code-quality.md` §§9, 12 (CQ-27); `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§4, 8 (SBD-20/30/44/50/81); `spec/19-error-handling.md` §§4, 10 (ERR-21/31); `spec/24-open-source-stack.md` OSS-02.

**Files** —
- `Cargo.toml`, `Cargo.lock` — central dependencies and resolved dependency graph.
- `crates/eq-core/Cargo.toml` — workspace dependency references.
- `crates/eq-core/src/lib.rs` — exports for fields, HTTP extractors and limits.
- `crates/eq-core/src/text.rs` — pure sanitising profiles.
- `crates/eq-core/src/limits.rs` — named technical limits.
- `crates/eq-core/src/fields/mod.rs` — field model and public interfaces.
- `crates/eq-core/src/fields/kinds.rs` — typed kinds, parsers and schemas.
- `crates/eq-core/src/fields/collector.rs` — field issue collection.
- `crates/eq-core/src/fields/schema.rs` — OpenAPI schema construction.
- `crates/eq-core/src/fields/tests.rs` — fixture parity and kind/schema tests.
- `crates/eq-core/src/http.rs`, `crates/eq-core/src/http/validated.rs` — validated request extractors and rejection mapping.
- `contracts/validation/cases/*.json` — 31 shared validation fixtures.
- `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md`, `tasks/INDEX.md` — task Report/status and generated index. Existing `eq_core::errors::FIELD_ERROR_CODES` from T-002 is reused; `errors.rs` did not need a change.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `fields::tests::display_name_collapses_whitespace` | Passed |
| AC2 | `http::validated::tests::collects_all_field_errors` | Passed; order follows `Collector` invocation order (RATIFY) |
| AC3 | `fields::tests::email_canonical` | Passed |
| AC4 | `fields::tests::phone_national_to_e164` | Passed |
| AC5 | `fields::tests::instant_requires_offset` | Passed |
| AC6 | `fields::tests::schema_matches_parse_bounds` | Passed |
| AC7 | `fields::tests::cases_match` | Passed for all 31 fixture files |
| AC8 | `http::validated::tests::bad_path_id_is_404` | Passed |
| AC9 | `rejects_bidi_override`, `rejects_unknown_field`, `rejects_duplicate_keys`, `rejects_deep_json` | Passed; password redaction also passed |

**Step 0 — Start** — Resumed on `milestone/m1-core` at `86b5d63`, with the user-provided T-102a implementation already staged and unstaged in this repository. Preserved that work without reset, stash or clean. Confirmed the architect resolution commit `106c137`, the M1 branch, and the task index; continued the existing task rather than creating another branch.

**Step 1 — Task/spec** — Read the full task, the resolved phone-parser instruction and cited validation, canonical-form, error, code-quality, architecture, security and dependency rules. The task adds no business rules, migration, SQL query or UI.

**Step 2 — Dependencies** — Read the Reports, Deviations, Follow-ups and RATIFY items for T-002 and T-102, and inspected their shipped error, AppError and canonical-value code. Confirmed the field issue registry and `AppError::Validation` are available from T-002, and the currency, money, basis-point, local-time, geo and locale value types are available from T-102.

**Step 3 — Design reference** — n/a; this is a backend-only task with no UI.

**Step 4 — Plan and test plan** — Recorded the plan, assumptions and Given/When/Then mappings above before completing the resumed implementation. Boundary, error and abuse cases are included in the plan.

**Step 5 — Implement** — Implemented the sanitizer profiles, technical constants, field kinds, collector, schema generation, shared validation corpus, and JSON/query/path validation support. Replaced the previously blocked parser dependency with the architect-approved `rlibphonenumber` 2 and preserved the repository's unchanged `cargo deny` policy. Added nested JSON pointer mapping, secret-safe password input/debug behavior, invariant-preserving deserialization, query DTO strictness, component references with const-bound-specific names, and unknown-enum rejection details. The password minimum is enforced only when supplied by policy; no default minimum is invented when the context contains `None` (VAL-21/23).

**Step 6 — Run locally** — Exercised extractor success and rejection paths through Axum `Router::oneshot` requests and field parser tests. `make dev` and browser E2E are n/a for this task: no live route or user journey uses these new extractors yet, and no UI was added. Workspace database integration tests ran under `cargo test --workspace` with testcontainers and disposable PostgreSQL 16 instances.

**Step 7 — Debug log** — The first Clippy pass identified a narrowing cast and small lint issues; replaced the cast with checked conversion and simplified the borrow/closure. The first nested-pointer test exposed duplicate path-prefix construction; changed the mapping to use `serde_path_to_error` path segments. A focused test initially failed to compile because `.expect_err()` required `Debug` for a validated type; changed the test to inspect `.err().expect(...)`. A later audit-driven email test caught the non-idempotent repeated-dot case; the sanitizer now rejects the remaining trailing dot after removing the one allowed dot. The final Clippy rerun also caught a collapsible `if`; changed it to a Rust 2024 let-chain and reran Clippy successfully. Focused tests and the full workspace suite passed after these fixes.

**Step 8 — Self code review** — Reviewed the task diff against `spec/16-code-quality.md` §9, `spec/17-clean-architecture.md` §7 and `spec/19-error-handling.md` §10. The free-model OpenCode audit ran read-only in Orca's visible `child-opencode` worktree (`opencode/deepseek-v4.1-flash-free`). It found missing public extractor re-exports, repeated trailing-dot email non-idempotence, the wrong error code for identifier whitespace, missing `$ref` behavior for `Raw<K>`, potential OpenAPI component-name collisions for const-generic kinds, missing `not_allowed`/`allowed` details for unknown enum variants, and an unsupported minimum when no password policy value is present. Those findings were checked against the specs and code; correctness issues were fixed with regression coverage, while the proposed implicit password minimum was not applied because the rule is policy-driven. The audit also questioned `celes::Country::from_alpha2`; the workspace build confirmed the API accepts `&str` through `AsRef<str>`, and the call uses an explicit borrow. The implementation scope is about 4,000 changed non-lockfile lines, above the roughly 400-line guidance in WORKFLOW Step 8/CQ-101; all 31 kinds, fixtures and extractor behavior are explicit in this task, so this size overage is recorded as a follow-up for architect review. No `#[allow]` was added.

**Step 9 — Optimize** — n/a; no database queries, public endpoint or frontend bundle were added or measured. JSON depth and field limits provide the task's structural bounds.

**Step 10 — Security audit** — Triggers: T1 request boundary, T3 S2 personal fields, and T4 S3 password. Applied server-side field validation and structural bounds, closed error codes, strict DTO unknown-field handling, duplicate-key and depth checks, and redacted `SecretString` password formatting. Abuse tests cover bidi override, mass assignment, duplicate JSON keys, depth 33 and fake-secret redaction. No database access, external call or new logging path was added. `gitleaks detect --redact --no-banner` returned one existing `curl-auth-user` finding in `Makefile:80–85`; reviewed the templated readiness URL and Docker user option, which contain no credential. No unrelated file or scanner policy was changed.

**Step 11 — Revise and verify** — Final checks after the implementation and task Report updates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check`, all four guard scripts, `python3 scripts/erd.py --check`, `scripts/tasks-index.sh --check`, and `git diff --check` passed. The workspace suite reported 91 passing `eq-core` unit tests and all other workspace unit, integration, database/RLS and doc tests passed; the database tests used disposable PostgreSQL 16 testcontainers. The secret scan finding is documented above; web, migration, API-generation and live-server checks are n/a for the scope. Not run: hosted GitHub Actions, which requires the hosted CI environment.

**Step 12 — Report and commit** — Set the task to `review`, regenerated and checked `tasks/INDEX.md`, and will create the single required signed-off Conventional Commit on `milestone/m1-core`, then push that branch as required by the run instructions.

**Checks run**
- ✅ `cargo fmt --all --check`
- ✅ `cargo clippy --workspace --all-targets -- -D warnings`
- ✅ `cargo test --workspace` (91 `eq-core` unit tests plus all workspace unit, integration, PostgreSQL 16 testcontainers database/RLS, and doc tests passed)
- ✅ `cargo deny check` (advisories, bans, licenses and sources passed; existing duplicate-entry/license-allowance warnings only)
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`
- ✅ `python3 scripts/erd.py --check`
- ✅ `scripts/tasks-index.sh --check`
- ✅ `git diff --check`
- ⚠️ `gitleaks detect --redact --no-banner` (exit 1 for one inspected pre-existing false positive at `Makefile:80–85`; no credential found)
- n/a `python3 scripts/erd.py --migrations` (no migration changed)
- n/a database migration checks, `cargo sqlx prepare --workspace` and `make api` (no migration, SQL query or published OpenAPI route changed)
- n/a `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check` (no web files changed)
- n/a `make dev` and `make e2e` (no live route, UI or journey was added; request behavior is covered by in-process Axum tests)
- Not run: hosted GitHub Actions, which requires the hosted CI environment.

**Manual checks** — Sent valid and malformed JSON/query requests through the Axum extractor test router; verified status/problem mapping, JSON pointers, unknown query rejection, malformed path IDs, aggregation order, and password redaction using a fake secret. No production route currently uses the extractors, so no live server request can exercise this task in isolation.

**Debug log** — The Clippy and pointer-mapping findings and their fixes are listed in Step 7. Focused regression tests and the final workspace suite passed; no failure needed three attempts.

**Performance** — n/a; no endpoint or query was added, so there was nothing to benchmark.

**Security** — Triggers T1, T3 and T4. The task validates inputs at the boundary, rejects unknown/duplicate fields and excessive depth, and keeps password material in secrecy types with a hand-written redacting `Debug`. The fake-secret test confirms formatting does not reveal the value. The only scanner finding is the reviewed pre-existing Makefile false positive described in Step 10.

**Deviations** — RATIFY: (1) AC2 ordering uses `Collector` invocation order because the DTO conversion has already discarded source object order (VAL-04); (2) password input/output use secrecy-backed wrappers, hand-written redacted `Debug`, and no `Clone`/`Serialize`/plain `Debug` derive (SBD-50); (3) dependencies are declared centrally in the workspace per CQ-27; (4) `Key` rejects surrounding whitespace to honor CAN-10's canonical representation despite VAL-21's identifier profile; (5) generic `ValidatedQuery<T>` requires `#[serde(deny_unknown_fields)]` on each query DTO so CAN-32 unknown parameters map to 400 without inventing a generic allow-list; (6) use T-002's shipped `FIELD_ERROR_CODES` registry name instead of the task's `FIELD_CODES` sketch (T-002, ERR-21). The earlier parser dependency question is resolved by the architect's `rlibphonenumber` 2 decision; `cargo deny` remains unchanged. The OpenCode suggestion to supply a password minimum of 1 when no policy value exists was not adopted because VAL-21/23 make that bound policy-driven; only a caller-provided value is enforced. The roughly 4,000-line non-lockfile diff exceeds CQ-101/WORKFLOW Step 8's approximate size guidance; this size overage is a separate follow-up for architect review, not a self-resolution.

**Dependencies affected** — T-110, T-112, T-116a, T-303 and T-306a can now use the shared field kinds/extractors.

**Rollback plan** — Revert the single T-102a commit. No migration or database state change was introduced.

**Follow-ups** — Human review: decide whether to address or narrowly classify the existing `gitleaks` false positive in `Makefile:80–85`; it predates this task and was not changed. The six RATIFY choices above need architect review. The CQ-101 size overage is also a human follow-up, but it is not a separate implementation self-resolution.

## Questions
- **Self-resolved (implementer, run 4, needs architect ratification):** AC2 requires source body-key ordering, while the specified DTO plus `Collector::field` interface retains only the adapter's later call order → define the returned error order as `Collector` invocation order and test that contract; source key order cannot be recovered after DTO deserialization without changing the interface (VAL-04).
- **Self-resolved (implementer, run 4, needs architect ratification):** The generic `Raw<K>`/`Password` path carries a secret although only the profile behavior is specified → use secrecy-backed input/output wrappers with hand-written redacted `Debug` and no `Clone`/`Serialize`/plain `Debug` derive (SBD-50).
- **Self-resolved (implementer, run 4, needs architect ratification):** The task lists only `crates/eq-core/Cargo.toml` for new dependencies, while CQ-27 requires a single workspace dependency declaration → add those declarations to the workspace `Cargo.toml` and reference them with `.workspace = true` in `eq-core` (CQ-27).
- **Resolved (architect, 2026-10-27):** Q1 — option (a): `phonenumber` is replaced by `rlibphonenumber` 2 (Apache-2.0, maintained libphonenumber port). Probed with the repository `deny.toml`: advisories, bans and licenses pass; the gate is unchanged and no exception is added. Spec 10, 22 and 24 are updated. In the workspace `Cargo.toml` remove `phonenumber` and add `rlibphonenumber = "2"`. API: `rlibphonenumber::PHONE_NUMBER_UTIL.parse(input, Some(region))` returns a `PhoneNumber`; `is_valid()`; `format_as(PhoneNumberFormat::E164)`. `region` is `rlibphonenumber::enums::Region`, built from the tenant's ISO alpha-2 country (a `Region` has no string constructor, so convert through a small private mapping from the alpha-2 code; an unknown code is a validation error, not a panic). Letters (vanity numbers) stay rejected, as before.
- **Self-resolved (implementer, run 5, needs architect ratification):** VAL-11/VAL-21 classify `Key` under the identifier profile, which normally trims input, while normative CAN-10 accepts no alternate `Key` form and says no trimming → require input to already match the canonical `Key` pattern and reject surrounding whitespace as `invalid_format`; CAN-10 §1 says its representation table wins (CAN-10, VAL-21).
- **Self-resolved (implementer, run 5, needs architect ratification):** CAN-32 requires unknown query parameters to return 400, while the specified generic `ValidatedQuery<T>` delegates to `axum_extra::Query<T>` and cannot discover allowed keys from `DeserializeOwned` alone → require each query DTO to use `#[serde(deny_unknown_fields)]` and document that contract; a generic allow-list would add an interface not named by the task or spec (CAN-32).
- **Self-resolved (implementer, run 5, needs architect ratification):** T-102a names `eq_core::errors::FIELD_CODES`, while T-002 shipped the shared registry as `eq_core::errors::FIELD_ERROR_CODES` → use the dependency's exported name and reuse its registered field codes (T-002, ERR-21).
- **Q1 (historical; resolved by architect 2026-10-27):** How should T-102a use the task-mandated `phonenumber` parser while keeping the required `cargo deny check` green when its published build dependency brings in `atomic-polyfill 1.0.3` (RUSTSEC-2023-0089)?
  - Context: `spec/24-open-source-stack.md` OSS-02 and this task require `phonenumber`; `cargo tree --target all -e normal,build -i atomic-polyfill` traces it through `phonenumber` → build dependency `postcard` → `heapless 0.7.17` → `atomic-polyfill`. `deny.toml` sets `unmaintained = "all"`; `cargo deny check` fails advisories. `cargo audit` exits successfully but reports the same unmaintained advisory as a warning. Removing it by ignoring the advisory would weaken the security gate.
  - Options: (a) update OSS-02 and this task to a maintained phone-number parser whose dependency tree passes the existing advisory policy; (b) identify and approve a compatible upstream release/patch that removes the build-time chain while preserving the required parser contract.
  - Recommendation: (a), unless a reviewed upstream fix for the mandated crate is available; keep the existing cargo-deny policy intact.

## Review
