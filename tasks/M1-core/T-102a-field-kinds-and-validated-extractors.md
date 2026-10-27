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
  - `phone`: trim, `PHONE_NUMBER_UTIL.parse(s, Some(region))` (`region` from `ctx.country`), `is_valid()`, output E.164 via `format_as(PhoneNumberFormat::E164)`; letters rejected.
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
- `eq_core::fields`: `FieldKind`, `Raw<K>`, `FieldIssue` (code, safe params, optional relative pointer), `FieldCtx { country: Country, tenant_currency: Currency, tenant_locale: Locale, today: NaiveDate, password_min_length: Option<usize> }`, `Collector`, and these kinds from VAL-21: `DisplayName`, `LegalName`, `TenantName`, `PlaceName`, `ShortText`, `Address`, `Note`, `Comment`, `I18nLine<MAX>`, `Email`, `Phone`, `Password`, `Key`, `Slug`, `SearchQuery`, `IdField`, `LocalDateField`, `BirthDate`, `LocalTimeField`, `InstantField`, `Minutes<MIN,MAX>`, `MoneyAmount`, `CurrencyField`, `BpsField`, `CountryField`, `TimeZoneField`, `LocaleField`, `GeoField`, `Hue`, `Stars`, `IdList<MAX>`. `Country` is `celes::Country`. Kinds wrap the T-102 types where they exist (don't duplicate `Money`, `Bps`, `LocalTime`, `GeoPoint`, `Locale`, `Currency`).
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
- AC2. A request with 3 invalid fields returns one 422 with 3 errors in `Collector` invocation order. The DTO interface discards source-object ordering before V3 collection. → `http::validated::tests::collects_all_field_errors`
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

**Plan** — 1. Add pure text profiles and exact profile/property tests in `text.rs` → verify the named tests. 2. Add technical limits and the shared field model, collector and schemas → verify each kind's boundaries and generated schema. 3. Add validation case files before wiring the corpus runner → verify every case. 4. Add the listed dependencies and canonical wrappers for the VAL-21 kinds → verify country, currency, time, money and identifier edge cases. 5. Add JSON/query extractors with duplicate-key, depth, shape, content-type and path rejection handling → verify every named extractor test. 6. Export the modules and field error registry → verify workspace compilation. 7. Run runtime/manual checks where applicable, self-review, SBD-81 and secret scanning → record what is not applicable. 8. Run the complete Rust and guard suites, regenerate the task index, write this Report, and commit the task with sign-off.

**Assumptions** — Use `celes::Country` as the canonical ISO 3166-1 country value because CAN-10 defines that representation and OSS-02 maps `celes`; convert its alpha-2 code to `phonenumber::country::Id` only at the phone parser boundary. Use T-102's validated `Currency`, `Money`, `Bps`, `GeoPoint`, `LocalTime`, and `Locale` APIs directly. The context carries tenant default locale, tenant-local `today`, and an optional caller-supplied password minimum because VAL-21/23 require these inputs. Password input/output use secrecy-backed storage and redacted formatting per SBD-50. No business bounds are inferred beyond VAL-21.

**Test plan** — Given the task's canonical and invalid inputs, when each profile/kind parses them, then it returns its canonical value or only the closed VAL-03 error codes (`text.rs` profile tests; `fields::tests::display_name_collapses_whitespace`, `rejects_bidi_override`, `email_canonical`, `phone_national_to_e164`, `instant_requires_offset`, `schema_matches_parse_bounds`, `cases_match`). Given body shape, duplicate keys, excess depth, wrong content type or invalid path/query input, when `ValidatedJson`/`ValidatedQuery` or the path helper receives the request, then it returns the specified 400/404/422 problem and preserves `Collector` invocation order (`http::validated::tests::*`). Edge coverage includes min−1/min/max/max+1, blank and Unicode whitespace, each forbidden character class, allowed emoji ZWJ and LRM/RLM cases, 8/9 combining marks, IDN/email boundaries, country-specific phone parsing, UUID/date/time/offset forms, money cap and numeric bounds, duplicate list IDs, query repetition and unknown keys. Abuse coverage is the four task cases: bidi override, mass assignment, duplicate JSON keys and depth 33; password errors and Debug output must not reveal a fake secret.

**Step 0 — Start** — Resumed on `milestone/m1-core` at `8537c14`; the tree was clean and `scripts/tasks-index.sh --check` passed. T-102a is the lowest-ID eligible M1 task because T-002 and T-102 are `review`; T-101 is blocked, but is not a dependency of this task. Set T-102a to `in_progress` and regenerated the index.

**Step 1 — Task/spec** — Read the complete task, its resolved dependency interface, and the cited validation, canonical-form, error, code-quality, architecture and security rules. The task has no business rules; technical bounds are named constants and comments. T-102a has no UI or schema-migration scope.

**Step 2 — Dependencies** — Read T-002's Report, RATIFY items and follow-ups, and its shipped `errors`/`AppError` interface. Read T-102's Report, RATIFY items and follow-ups, and its shipped currency and policy value types. Confirmed T-102a uses the shipped `Currency`, `Money`, `Bps`, `LocalTime`, `GeoPoint` and `Locale` APIs, and T-002 exports the closed field-code registry and `AppError::Validation`.

**Step 3 — Design reference** — n/a; this is a backend validation task with no UI.

**Step 4 — Plan and test plan** — The plan, assumptions and AC/edge/error/abuse mapping above were recorded before implementation. Scope gives the full field context and country type; the deviations recorded below cover the AC2 ordering mismatch, secret handling and workspace dependency policy.

**Summary** — A prototype implemented the text sanitizers, technical limits, typed field kinds, shared validation corpus, and JSON/query extractors. Its Rust tests, guard scripts, and M1 E2E smoke suite passed. The task is blocked because the task-mandated `phonenumber` dependency pulls an unmaintained build-time dependency that the repository's required `cargo deny check` rejects; the code prototype was reverted as WORKFLOW §4 requires when blocking.

**Spec read** — `spec/22-validation-and-sanitization.md` §§1–4 (VAL-03 through VAL-21); `spec/23-contracts-and-canonical-forms.md` §§1–3 (CAN-01, CAN-10, CAN-21/22, CAN-32/33); `spec/16-code-quality.md` §§9 and 12 (CQ-27); `spec/17-clean-architecture.md` §§1, 4 and 7; `spec/18-security-by-design.md` §§4 and 8 (SBD-30, SBD-50, SBD-81); `spec/19-error-handling.md` §10; `spec/24-open-source-stack.md` OSS-02.

**Files** — During the prototype, created the requested `text.rs`, `limits.rs`, field kinds, extractor, and 31 contract case files, and changed workspace dependency manifests/lockfile and the field error registry. These implementation changes were reverted after the required dependency-advisory check failed. The blocked commit contains this task file and regenerated `tasks/INDEX.md` only.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `fields::tests::display_name_collapses_whitespace` | Passed against prototype; implementation reverted pending dependency decision |
| AC2 | `http::validated::tests::collects_all_field_errors` | Passed against prototype; implementation reverted pending dependency decision |
| AC3 | `fields::tests::email_canonical` | Passed against prototype; implementation reverted pending dependency decision |
| AC4 | `fields::tests::phone_national_to_e164` | Passed against prototype; implementation reverted pending dependency decision |
| AC5 | `fields::tests::instant_requires_offset` | Passed against prototype; implementation reverted pending dependency decision |
| AC6 | `fields::tests::schema_matches_parse_bounds` | Passed against prototype; implementation reverted pending dependency decision |
| AC7 | `fields::tests::cases_match` | Passed against prototype; implementation reverted pending dependency decision |
| AC8 | `http::validated::tests::bad_path_id_is_404` | Passed against prototype; implementation reverted pending dependency decision |
| AC9 | `rejects_bidi_override`, `password_debug_is_redacted`, `rejects_unknown_field`, `rejects_duplicate_keys`, `rejects_deep_json` | Passed against prototype; implementation reverted pending dependency decision |

**Step 5 — Implement** — Implemented the scoped prototype and its exact named tests. The first `cargo deny check` found that the required `phonenumber` crate's build dependency `postcard` brings in `heapless 0.7.17` → `atomic-polyfill 1.0.3` (RUSTSEC-2023-0089). `deny.toml` sets `unmaintained = "all"`; the CI check therefore fails. I inspected the crate tree and advisory; no code-only fix is available without changing the approved dependency stack or weakening the advisory gate. Per WORKFLOW §4, reverted the prototype instead of leaving unreviewed code on the blocked branch.

**Step 6 — Run locally** — `make e2e` built the web app and release server, started the local PostgreSQL service, exercised live HTTP/browser behavior in Chromium and WebKit mobile, and passed all 6 Playwright cases (home, health, and SPA fallback in both projects). The task adds extractors but no route that uses them yet, so field request behavior was exercised through the in-process Axum extractor tests; `make dev` was n/a.

**Step 7 — Debug log** — `cargo deny check` failed on RUSTSEC-2023-0089. Attempt 1: ran the required check and confirmed the failure. Alternative investigation: inspected `deny.toml`, `cargo tree --target all -e normal,build -i atomic-polyfill`, the `phonenumber` registry manifest, and `cargo audit`; the dependency is required by the parser's build chain, and `cargo audit` reports the same advisory as an allowed warning while `cargo deny` rejects it. Removing the failure would require either an approved dependency-stack change or a narrowly scoped advisory exception; the latter would weaken the configured security gate and is not permitted. No retry or unrelated policy change was made.

**Step 8 — Self code review** — Reviewed the prototype against `spec/16-code-quality.md` §9, `spec/17-clean-architecture.md` §7, and `spec/19-error-handling.md` §10. The full workspace Clippy and all four guard scripts passed; no `#[allow]` was added. The typed values kept fields private, the password path used `SecretString` and redacted `Debug`, and validation errors contained only safe codes/parameters. The prototype was then reverted because of the unresolved dependency gate.

**Step 9 — Optimize** — n/a; this task adds no database queries or public endpoint to measure. Structural work is bounded by `JSON_MAX_DEPTH` and named array/query/page/cursor limits.

**Step 10 — Security audit** — Triggers: T1 (request boundary), T3 (S2 name/email/phone/birth-date/address inputs), and T4 (S3 password). SBD-81: no auth/data-access handler or table was added (items 1–2, 7–9 n/a); the prototype applied server-side bounds and closed field/error kinds (item 4); it introduced no output sink or external call (items 5, 7); password values used `SecretString`, and its abuse test checked redaction (item 6); the bidi, mass-assignment/unknown-field, duplicate-key, depth-33, and password-redaction tests passed (item 10). `gitleaks detect --redact --no-banner` returned one `curl-auth-user` finding at pre-existing `Makefile:80` (commit `c83e55d`); inspection showed the line contains only the templated E2E readiness URL and no credential. I did not edit the unrelated Makefile or weaken the scanner.

**Step 11 — Revise and verify** — All task implementation checks listed below were run against the prototype before it was reverted. The required Cargo advisory gate did not pass, so no DoD item is marked complete and the task remains blocked. Final task metadata/index checks are rerun after this report is written.

**Step 12 — Report and commit** — Reported as `blocked` with the dependency question below; implementation files were reverted, and the task metadata plus index will be committed with the required signed `docs(tasks): block T-102a with questions [T-102a]` commit.

**Checks run**
- ✅ `cargo fmt --all --check` (prototype)
- ✅ `cargo clippy --workspace --all-targets -- -D warnings` (prototype)
- ✅ `cargo test --workspace` (prototype; all tests passed, including 72 `eq-core` unit tests and the Docker-backed database/integration tests)
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` (prototype)
- ✅ `make e2e` (6/6 browser tests)
- ✅ `scripts/tasks-index.sh --check` (before report status update; rerun after update)
- ✅ `python3 scripts/erd.py --check` (no schema changes; rerun not required)
- ✅ `git diff --check` (before report status update; rerun after update)
- ✅ `cargo audit` (exit 0; it reports RUSTSEC-2023-0089 as an allowed warning)
- ❌ `cargo deny check` (`advisories FAILED`: unmaintained `atomic-polyfill 1.0.3`, RUSTSEC-2023-0089, introduced through the task-mandated `phonenumber` build dependency)
- ❌ `gitleaks detect --redact --no-banner` (one pre-existing false positive at `Makefile:80`, documented under Security)
- n/a `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check` (no web source changed; E2E ran `pnpm -C web build` and browser tests)
- n/a `python3 scripts/erd.py --migrations`, database migration checks, and `make api` (no migration, SQL query, or API contract change)
- Not run: hosted GitHub Actions (requires the hosted CI environment)

**Manual checks** — `make e2e` passed 6 live browser cases against the release server and local PostgreSQL. No task-specific endpoint exists yet to call with `curl`; extractor success/error behavior was verified in Axum router tests.

**Debug log** — The advisory failure and the dependency-tree investigation are recorded in Step 7. No other failure required more than one attempt.

**Performance** — n/a; no endpoint or query was added. The E2E release server became ready within the Makefile's configured readiness window.

**Security** — Triggers T1, T3, T4; controls and abuse checks are recorded in Step 10. The prototype's password Debug/redaction and validation errors did not expose the fake secret. The required cargo-deny advisory check remains unresolved and blocks completion.

**Deviations** — RATIFY (choices in the prototype, not shipped): (1) AC2 body-key order → `Collector` invocation order because DTO deserialization discards source order before collection (VAL-04); (2) password input/output → secrecy-backed wrappers with hand-written redacting `Debug`, no `Clone`/`Serialize`/plain `Debug` derive (SBD-50); (3) dependency declarations → central workspace dependencies because CQ-27 requires one declaration per dependency. These three self-resolutions remain in `## Questions` for architect review. The implementation was reverted while the dependency gate is unresolved.

**Dependencies affected** — Direct dependants not ready to start until this task returns to `review`/`done`: T-110, T-112, T-116a, T-303, and T-306a.

**Rollback plan** — The implementation prototype has already been reverted. After the architect resolves Q1, resume T-102a from a clean branch, reapply the accepted dependency choice, and rerun all checks; the blocker commit can be reverted as part of that resumed task.

**Follow-ups** — Resolve the approved phone-parser dependency and rerun `cargo deny check`. Naming audit: the canonical display mapping is `Equalizer-Stake` → **Ritma**. The pre-report case-insensitive scan found 15 old-name matches across 9 files: 6 operational URL/repository-slug occurrences (Cargo metadata, BRANCHING helper, README CI badges), 6 historical/provenance references in brand/spec records, and 3 prior task-report audit mentions. `gh repo view` confirms the active GitHub slug remains `sedam-or/Equalizer-Stake`; product-facing labels are already Ritma. Keep those operational URLs until the repository is actually renamed/transferred, then update them; historical mentions should remain clearly marked as history. No branding, spec, remote, or repository setting was changed in this task.

## Questions
- **Self-resolved (implementer, run 4, needs architect ratification):** AC2 requires source body-key ordering, while the specified DTO plus `Collector::field` interface retains only the adapter's later call order → define the returned error order as `Collector` invocation order and test that contract; source key order cannot be recovered after DTO deserialization without changing the interface (VAL-04).
- **Self-resolved (implementer, run 4, needs architect ratification):** The generic `Raw<K>`/`Password` path carries a secret although only the profile behavior is specified → use secrecy-backed input/output wrappers with hand-written redacted `Debug` and no `Clone`/`Serialize`/plain `Debug` derive (SBD-50).
- **Self-resolved (implementer, run 4, needs architect ratification):** The task lists only `crates/eq-core/Cargo.toml` for new dependencies, while CQ-27 requires a single workspace dependency declaration → add those declarations to the workspace `Cargo.toml` and reference them with `.workspace = true` in `eq-core` (CQ-27).
- **Resolved (architect, 2026-10-27):** Q1 — option (a): `phonenumber` is replaced by `rlibphonenumber` 2 (Apache-2.0, maintained libphonenumber port). Probed with the repository `deny.toml`: advisories, bans and licenses pass; the gate is unchanged and no exception is added. Spec 10, 22 and 24 are updated. In the workspace `Cargo.toml` remove `phonenumber` and add `rlibphonenumber = "2"`. API: `rlibphonenumber::PHONE_NUMBER_UTIL.parse(input, Some(region))` returns a `PhoneNumber`; `is_valid()`; `format_as(PhoneNumberFormat::E164)`. `region` is `rlibphonenumber::enums::Region`, built from the tenant's ISO alpha-2 country (a `Region` has no string constructor, so convert through a small private mapping from the alpha-2 code; an unknown code is a validation error, not a panic). Letters (vanity numbers) stay rejected, as before.
- **Q1 (blocking):** How should T-102a use the task-mandated `phonenumber` parser while keeping the required `cargo deny check` green when its published build dependency brings in `atomic-polyfill 1.0.3` (RUSTSEC-2023-0089)?
  - Context: `spec/24-open-source-stack.md` OSS-02 and this task require `phonenumber`; `cargo tree --target all -e normal,build -i atomic-polyfill` traces it through `phonenumber` → build dependency `postcard` → `heapless 0.7.17` → `atomic-polyfill`. `deny.toml` sets `unmaintained = "all"`; `cargo deny check` fails advisories. `cargo audit` exits successfully but reports the same unmaintained advisory as a warning. Removing it by ignoring the advisory would weaken the security gate.
  - Options: (a) update OSS-02 and this task to a maintained phone-number parser whose dependency tree passes the existing advisory policy; (b) identify and approve a compatible upstream release/patch that removes the build-time chain while preserving the required parser contract.
  - Recommendation: (a), unless a reviewed upstream fix for the mandated crate is available; keep the existing cargo-deny policy intact.

## Review
