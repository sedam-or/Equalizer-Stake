---
id: T-102a
title: Add field kinds, text sanitisers, limits and the ValidatedJson/ValidatedQuery extractors
milestone: M1
status: todo
depends_on: [T-002, T-102]
spec_refs: [VAL-03, VAL-04, VAL-05, VAL-06, VAL-07, VAL-10, VAL-11, VAL-12, VAL-18, VAL-19, VAL-20, VAL-21, VAL-31, VAL-40, VAL-41, VAL-50, CAN-10, CAN-21, CAN-22, CAN-32, CAN-33, SBD-20, SBD-44, ERR-31, OSS-02]
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
  - `phone`: trim, `phonenumber::parse(Some(ctx.country), s)`, `is_valid()`, output E.164; letters rejected.
  - `secret`: NFKC only; no trim; reject C0 controls; never echo the value or put it in `params`.
- **Forbidden characters** (VAL-12, all profiles except `secret`): C0 except `\n` in multiline, DEL, C1, U+202A–U+202E, U+2066–U+2069, noncharacters (U+FDD0–U+FDEF and U+xFFFE/U+xFFFF), private use (U+E000–U+F8FF, planes 15–16), more than 8 consecutive combining marks. LRM/RLM allowed in `line`/`multiline`, stripped in `identifier`/`email`.
- **Canonical forms** (CAN-10): `Id` lowercase hyphenated UUID only; `Instant` RFC 3339 **with offset** (missing offset rejected), ≤ 6 fractional digits; `LocalDate` `YYYY-MM-DD`; `Geo` rounded to 6 decimals; booleans only `true`/`false` (also in query strings); `Money` within ±10^15 (`MONEY_MAX_ABS_MINOR`, technical).
- **Requests** (CAN-21/22): unknown fields rejected → `{pointer:"/field", code:"not_allowed"}`; duplicate JSON keys rejected (D-17); depth ≤ 32; on create, absent = `null`; on PATCH, JSON Merge Patch semantics (absent = unchanged, `null` = clear, arrays replaced).
- **Query and path** (CAN-32/33): repeated params for lists (`axum_extra::extract::Query`); unknown query params → 400 `bad_request`; `limit` 1..=200; a malformed path id → 404 `not_found`.
- **Rejections** (VAL-07 V0–V2): missing/wrong `Content-Type` → 400 `bad_request` (not 415, D-16); JSON syntax → 400 `bad_request`; a shape error → 422 `validation` with the pointer from `serde_path_to_error`.

## Scope
### In
- `eq_core::text`: the six profiles as pure functions (`sanitize_line`, `sanitize_multiline`, …) returning `Result<String, TextIssue>`.
- `eq_core::limits`: technical constants of VAL-18 (body sizes are configured in T-115; here only JSON depth, default array length 100, `q` 1–100, page size 1–200, cursor ≤ 512, `MONEY_MAX_ABS_MINOR`), each with `// technical:`.
- `eq_core::fields`: `FieldKind`, `Raw<K>`, `FieldIssue`, `FieldCtx { country: Country, tenant_currency: Currency }`, `Collector`, and these kinds from VAL-21: `DisplayName`, `LegalName`, `TenantName`, `PlaceName`, `ShortText`, `Address`, `Note`, `Comment`, `I18nLine<MAX>`, `Email`, `Phone`, `Password`, `Key`, `Slug`, `SearchQuery`, `IdField`, `LocalDateField`, `BirthDate`, `LocalTimeField`, `InstantField`, `Minutes<MIN,MAX>`, `MoneyAmount`, `CurrencyField`, `BpsField`, `CountryField`, `TimeZoneField`, `LocaleField`, `GeoField`, `Hue`, `Stars`, `IdList<MAX>`. Kinds wrap the T-102 types where they exist (don't duplicate `Money`, `Bps`, `LocalTime`, `GeoPoint`, `Locale`, `Currency`).
- Each kind's `schema()` emits OpenAPI constraints from the same constants plus `x-sanitize` and `x-field-kind`; `Raw<K>` implements `ToSchema` as a `$ref` to the kind's component.
- `eq_core::http::{ValidatedJson<T>, ValidatedQuery<T>}` extractors with the rejection mapping above.
- `contracts/validation/cases/<kind>.json` for every kind in Scope, and the Rust runner `eq_core::fields::tests::cases_match` (VAL-31).
- `errors.field.*` code constants registered per ERR-21 (`eq_core::errors::FIELD_CODES`).

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
| `crates/eq-core/Cargo.toml` | modify | `nutype`, `unicode-normalization`, `email_address`, `phonenumber`, `serde_path_to_error`, `axum-extra` (query), `celes` |

## Interfaces
```rust
// eq_core::fields
pub enum Profile { Line, Multiline, Identifier, Email, Phone, Secret, None }

pub struct FieldIssue { pub code: &'static str, pub params: serde_json::Map<String, serde_json::Value> }

pub trait FieldKind: Sized {
    type Input: serde::de::DeserializeOwned + utoipa::ToSchema;
    const NAME: &'static str;
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
pub struct ValidatedQuery<T>(pub T);
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
None. All limits here are technical (`// technical:`) or come from the DDL. Policy-driven bounds (password minimum) are passed in by the caller.

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
- AC2. A request with 3 invalid fields returns one 422 with 3 errors in body order. → `http::validated::tests::collects_all_field_errors`
- AC3. `"Budi@Example.COM "` → `"budi@example.com"`; `"budi@[127.0.0.1]"` → `invalid_format`. → `fields::tests::email_canonical`
- AC4. `"0812-3456-7890"` with country ID → `"+6281234567890"`. → `fields::tests::phone_national_to_e164`
- AC5. `"2026-11-05T15:00:00"` (no offset) → `invalid_format`. → `fields::tests::instant_requires_offset`
- AC6. Every kind's OpenAPI schema carries the same bounds as its `parse` (generated test over all kinds). → `fields::tests::schema_matches_parse_bounds`
- AC7. Every case file passes. → `fields::tests::cases_match`
- AC8. A malformed path UUID on a test route → 404 `not_found`. → `http::validated::tests::bad_path_id_is_404`
- AC9. The four abuse cases in Security pass.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/text.rs` | `line_profile_*`, `multiline_profile_*`, `identifier_profile_*`, `secret_profile_nfkc_no_trim`, `prop_sanitize_idempotent` |
| `crates/eq-core/src/fields/*.rs` | `display_name_collapses_whitespace`, `rejects_bidi_override`, `email_canonical`, `phone_national_to_e164`, `instant_requires_offset`, `schema_matches_parse_bounds`, `cases_match` |
| `crates/eq-core/src/http/validated.rs` | `rejects_unknown_field`, `rejects_duplicate_keys`, `rejects_deep_json`, `collects_all_field_errors`, `wrong_content_type_is_400`, `bad_path_id_is_404`, `unknown_query_param_is_400` |

## Definition of done
- [ ] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` pass
- [ ] Guard scripts pass: `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed; no new `#[allow]`
- [ ] Docs/comments cite spec IDs (`/// VAL-11: …`)
- [ ] Commit `feat(core): add field kinds and validated extractors [T-102a]`, signed off

## Report

## Questions

## Review
