---
id: T-002
title: Add config loading, the Clock abstraction and AppError → problem+json in eq-core
milestone: M0
status: review
depends_on: [T-001]
spec_refs: [ERR-10, ERR-20, ERR-21, ERR-30, ERR-31, CQ-21, CQ-22, OPS-01, ARC-10, ARC-12, API-04, VAL-03, CAN-24]
estimated_size: M
---

## Goal
Every later crate can read a typed `Config`, get time from an injectable `Clock`, and return `AppError`, which renders as RFC 9457 `application/problem+json` with stable codes.

## Context
- OPS-01 env vars:
  - Required: `DATABASE_URL`, `RITMA_SECRET_KEY` (base64, ≥ 32 bytes decoded), `RITMA_PUBLIC_URL`.
  - Optional:
    - `RITMA_DATABASE_OWNER_URL` (default = DATABASE_URL), `RITMA_SECRET_KEY_PREVIOUS`
    - `RITMA_MODE` (`single`|`multi`, default single), `RITMA_LISTEN` (default `0.0.0.0:8080`)
    - `RITMA_DATA_DIR` (default `./data` in debug builds, `/var/lib/ritma` in release builds)
    - `RITMA_FILES_BACKEND` (`local`|`s3`), `RITMA_SETUP_TOKEN`, `RITMA_TRUSTED_PROXIES` (CSV of CIDRs)
    - `RITMA_LOG` (default `info`), `RITMA_LOG_FORMAT` (`json`|`pretty`)
    - `RITMA_BACKUP_SCHEDULE` (`off`|`daily`), `RITMA_BACKUP_KEEP` (default 14), `RITMA_SMTP_URL`
    - `RITMA_CAPTCHA_PROVIDER`/`_SITE_KEY`/`_SECRET`, `RITMA_TILE_URL`, `RITMA_TELEMETRY` (`off`|`on`), `RITMA_UPDATE_CHECK` (`on`|`off`)
    - `RITMA_METRICS_TOKEN`, `RITMA_DB_POOL` (default 10), `RITMA_DEMO` (bool)
  - Unknown `RITMA_*` variables produce a warning listing them.
- Data classes (SBD-10, spec 18 §2): **S3** are `DATABASE_URL`, `RITMA_DATABASE_OWNER_URL`, `RITMA_SECRET_KEY`, `RITMA_SECRET_KEY_PREVIOUS`, `RITMA_SETUP_TOKEN`, `RITMA_SMTP_URL`, `RITMA_CAPTCHA_SECRET`, `RITMA_METRICS_TOKEN` (a URL counts because it can embed a password). Everything else in `Config` is S0/S1.
- ARC-10: `domain/` code never reads the clock. Services use `Clock`.
- ARC-12 `AppError` variants and API-04 mapping (HTTP status → `code`), shown in Interfaces.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `FieldError.code` uses only the closed VAL-03 set (`required`, `invalid_type`, `too_short`, `too_long`, `too_few_items`, `too_many_items`, `must_be_gte`, `must_be_lte`, `invalid_format`, `invalid_characters`, `not_allowed`, `not_unique`, `unknown_reference`). Declare them as constants in `eq_core::errors` (T-102a adds the parsers that emit them).
  - Multipart field errors point at `/$files/<part>` for file parts (CAN-24, D-08).

## Scope
### In
- `eq_core::config::{Config, Mode, FilesBackend, LogFormat}` with `Config::from_env() -> Result<Config, ConfigError>` and `Config::for_tests()`.
- `eq_core::clock::{Clock (trait), SystemClock, Timestamp}` + `eq_core::clock::test::ManualClock` behind `#[cfg(any(test, feature = "testkit"))]`.
- `eq_core::error::{AppError, FieldError, Problem}` + `impl IntoResponse for AppError` (axum) + `From<sqlx::Error>` mapping (RowNotFound → NotFound, pool timeout or connection loss → `Unavailable { retry_after_s: DB_UNAVAILABLE_RETRY_AFTER_S }`, others → Internal; the full SQLSTATE mapping in ERR-43 is added by the tasks that need it).
- Technical constant (AGENTS §4.1, not a business value, so no policy field): `pub const DB_UNAVAILABLE_RETRY_AFTER_S: u64 = 5; // technical: client back-off hint when the pool times out or the connection drops`. The `Retry-After` header of an `Unavailable` error is always its `retry_after_s`. A `RateLimited` error carries the value its caller computed (T-115's limiter); T-002 only renders it.
- ERR-21 registry: `pub struct ErrorCodeDef { pub code: &'static str, pub status: u16, pub title: &'static str }` and `eq_core::error::codes` with the core codes (`bad_request`, `unauthenticated`, `forbidden`, `not_found`, `module_disabled`, `validation`, `policy_violation`, `precondition_required`, `payload_too_large`, `locked`, `rate_limited`, `unavailable`, `setup_required`, `internal`) plus `pub const CODES: &[&ErrorCodeDef]`. `Problem.title` comes from the `ErrorCodeDef` that the `AppError` variant carries (`code: &'static ErrorCodeDef`), so a module crate passes its own registered definition and `eq-core` needs no registry injection or global state. Core codes are `pub static` items in `eq_core::error::codes` (use `&codes::NOT_FOUND`); module crates declare theirs as `pub static` in their `errors.rs` (ERR-21) and pass `&codes::SLOT_TAKEN`. A helper `pub fn validate_code(code: &str) -> bool` checks the ERR-20 format `^[a-z][a-z0-9_]{2,47}$`.
- The problem `type` URI base is the constant `PROBLEM_BASE = "https://ritma.dev/problems/"` + code with `_` replaced by `-`.

### Out
- Exclusion-violation mapping (T-203), and the tracing setup (T-004).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/config.rs` | create | Config |
| `crates/eq-core/src/config/tests.rs` | create | Config tests kept in a separate source file to meet CQ-11. |
| `crates/eq-core/src/clock.rs` | create | Clock |
| `crates/eq-core/src/error.rs` | create | AppError/Problem |
| `crates/eq-core/src/error/tests.rs` | create | AppError tests kept in a separate source file to meet CQ-11. |
| `crates/eq-core/src/lib.rs` | edit | `pub mod config; pub mod clock; pub mod error;` |
| `crates/eq-core/Cargo.toml` | edit | deps: axum, serde, serde_json, thiserror, anyhow, chrono, base64, sqlx, tracing, zeroize, secrecy, figment + add `url = "2"` and `ipnet = { version = "2", features = ["serde"] }` to `[workspace.dependencies]`; feature `testkit` |
| `Cargo.toml` | edit | Update the workspace SQLx version to the approved SQLx 0.9 dependency baseline. |
| `spec/24-open-source-stack.md` | edit | Record the matching SQLx and SQLx CLI versions, as authorized to resolve Q4. |

## Interfaces
```rust
pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub trait Clock: Send + Sync + std::fmt::Debug + 'static { fn now(&self) -> Timestamp; }
#[derive(Debug, Default, Clone)] pub struct SystemClock;
// test
#[derive(Debug, Clone)] pub struct ManualClock(std::sync::Arc<std::sync::Mutex<Timestamp>>);
impl ManualClock { pub fn new(t: Timestamp) -> Self; pub fn set(&self, t: Timestamp); pub fn advance(&self, d: chrono::Duration); }

// SBD-50: every S3 value is a `secrecy` type (redacting Debug, zeroized on drop). `Config` holds secrets, so it has a
// hand-written `Debug` (below) and no `Clone`, `Serialize` or `Deserialize`. It is shared as `Arc<Config>` (T-004).
pub struct Config {
    pub database_url: SecretString, pub database_owner_url: SecretString,          // may embed a password
    pub secret_key: SecretSlice<u8>, pub secret_key_previous: Option<SecretSlice<u8>>,
    pub public_url: url::Url, pub mode: Mode, pub listen: std::net::SocketAddr, pub data_dir: std::path::PathBuf,
    pub files_backend: FilesBackend, pub setup_token: Option<SecretString>,
    // OPS-01 loopback CIDRs are the default; extra proxies require explicit configuration.
    pub trusted_proxies: Vec<ipnet::IpNet>,
    pub log: String, pub log_format: LogFormat, pub backup_schedule: BackupSchedule, pub backup_keep: u16,
    pub smtp_url: Option<SecretString>,                                           // may embed a password
    pub captcha: Option<CaptchaConfig>, pub tile_url: String,
    pub telemetry: bool, pub update_check: bool, pub metrics_token: Option<SecretString>, pub db_pool: u32, pub demo: bool,
}
pub enum CaptchaProvider { Turnstile, Hcaptcha }              // RITMA_CAPTCHA_PROVIDER = turnstile | hcaptcha (OPS-01)
pub struct CaptchaConfig { pub provider: CaptchaProvider, pub site_key: String, pub secret: SecretString }
// `impl Debug for Config` and `for CaptchaConfig`: print every non-secret field normally, and each secret field as
// `[REDACTED]` when set or `None` when unset. Callers read a secret with `.expose_secret()` at the one place that needs it.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError { #[error("missing required env var {0}")] Missing(&'static str), #[error("invalid {var}: {reason}")] Invalid { var: &'static str, reason: String } }

#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldError { pub pointer: String, pub code: String, #[serde(skip_serializing_if = "serde_json::Map::is_empty")] pub params: serde_json::Map<String, serde_json::Value> }

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")] NotFound { entity: &'static str },
    #[error("conflict: {code}")] Conflict { code: &'static ErrorCodeDef, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("validation failed")] Validation { errors: Vec<FieldError> },
    #[error("forbidden")] Forbidden { perm: &'static str },
    #[error("forbidden: {code}")] ForbiddenRule { code: &'static ErrorCodeDef, detail: String },
    #[error("unauthenticated")] Unauthenticated { code: &'static ErrorCodeDef },
    #[error("module disabled")] ModuleDisabled { module: String },
    #[error("unprocessable: {code}")] Unprocessable { code: &'static ErrorCodeDef, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("policy violation: {rule}")] PolicyViolation { rule: String, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("precondition required")] PreconditionRequired,
    #[error("gone: {code}")] Gone { code: &'static ErrorCodeDef },
    #[error("locked")] Locked { until: Option<Timestamp> },
    #[error("setup required")] SetupRequired,
    #[error("rate limited")] RateLimited { retry_after_s: u64 },
    #[error("payload too large")] PayloadTooLarge,
    #[error("bad request: {code}")] BadRequest { code: &'static ErrorCodeDef, detail: String },
    #[error("unavailable")] Unavailable { retry_after_s: u64 },
    #[error(transparent)] Internal(#[from] anyhow::Error),
}
```
Status/code mapping:
- NotFound → 404 `not_found`
- Conflict → 409 `<code.code>` (status is fixed by the variant; title from the carried `ErrorCodeDef`)
- Validation → 422 `validation` (+`errors`)
- Forbidden → 403 `forbidden` (+`perm`)
- ForbiddenRule → 403 `<code>`
- Unauthenticated → 401 `<code>`
- ModuleDisabled → 404 `module_disabled` (+`module`)
- Unprocessable → 422 `<code>` (+extras)
- PolicyViolation → 422 `policy_violation` (+`rule`, extras)
- PreconditionRequired → 428 `precondition_required`
- Gone → 410
- Locked → 423 `locked` (+`until` when `Some`)
- SetupRequired → 503 `setup_required` (no tenant exists yet; T-104 raises it, T-302 clears it)
- RateLimited → 429 `rate_limited` + `Retry-After` header
- PayloadTooLarge → 413 `payload_too_large`
- BadRequest → 400
- Unavailable → 503 `unavailable` + `Retry-After` header
- Internal → 500 `internal` (detail "Internal error"; log the error with `tracing::error!` including the chain)

Body fields: `type`, `title` (human title per code, English), `status`, `code`, `detail`, `request_id` (from a task-local set by T-004; use `Option` and omit if absent), plus extension members flattened. Header `Content-Type: application/problem+json`.

## Business rules
None (technical only).

## Security
- Triggers: T3 (reads S3 values), T4 (keys).
- Data classes touched: S3 (the list in Context).
- Threats → control:
  - **I**nformation disclosure: a startup log, a panic message or an `{:?}` of `Config` leaks a key, token or URL password → SBD-50 `secrecy` types with redacting `Debug`, a hand-written `Debug` on `Config` and `CaptchaConfig`, no `Clone`/`Serialize`/`Deserialize` on them, `Config` shared as `Arc<Config>`.
  - **I**: a `ConfigError` echoing the offending value → `Invalid { reason }` never contains the value of an S3 variable, only what is wrong with it ("must decode to at least 32 bytes").
  - **T**/**E**: a short or non-base64 `RITMA_SECRET_KEY` is accepted → rejected at startup (AC2).
- Abuse-case acceptance criteria: AC7 (`debug_redacts_secrets`), AC8 (`error_does_not_echo_secret`).

## Implementation notes
- CQ-21 forbids `chrono::Utc::now` and `std::env::var` workspace-wide. `SystemClock::now` is the one allowed caller of `Utc::now`: annotate it with `#[expect(clippy::disallowed_methods, reason = "ARC-10: SystemClock is the only wall-clock reader")]`. Config loading uses figment/envy; if you call `std::env::var` directly, do it only inside `eq_core::config` with the same kind of `#[expect]`.
- Decode `RITMA_SECRET_KEY` as standard or URL-safe base64. If it decodes to < 32 bytes → `Invalid`.
- `RITMA_PUBLIC_URL` must parse as http(s). A trailing slash is normalised away.
- Wrap `Config::from_env` around a `fn from_map(HashMap<String,String>)` so tests don't touch the process env.
- Build `Config` by hand from the map (or from Figment values into a private raw type with no `Debug` derive), moving each S3 value straight into its `secrecy` type and dropping the raw copy. Enable the `secrecy` `serde` feature only if the extraction needs it, and never on `Config` itself.
- The request-id task-local: `tokio::task_local! { pub static REQUEST_ID: uuid::Uuid; }` in `error.rs`, read with `REQUEST_ID.try_with(|id| *id).ok()`.

## Acceptance criteria
- AC1. Given a map without `DATABASE_URL`, When `from_map`, Then `Missing("DATABASE_URL")`. → `config::tests::missing_database_url`
- AC2. Given a 16-byte key, Then `Invalid{var:"RITMA_SECRET_KEY"}`. → `config::tests::short_secret_rejected`
- AC3. Given defaults, Then `listen=0.0.0.0:8080`, `mode=Single`, `backup_keep=14`. → `config::tests::defaults`
- AC4. `AppError::Conflict{code:&SLOT_TAKEN,…}` (the test declares `static SLOT_TAKEN: ErrorCodeDef = ErrorCodeDef{code:"slot_taken",status:409,title:"Slot already taken"}`, as eq-scheduling will) renders 409 with JSON `code=="slot_taken"`, `title=="Slot already taken"` and `type` ending in `/slot-taken`, and the extension members appear at top level. → `error::tests::conflict_problem_json`
- AC5. `AppError::Internal` renders 500 and its detail doesn't contain the inner error message. → `error::tests::internal_hides_detail`
- AC6. `ManualClock::advance` changes `now()`. → `clock::tests::manual_clock_advances`
- AC7. Given a config built with fake secrets (`test-secret-…`, a URL `postgres://u:test-secret-pw@h/db`, a captcha secret and tokens), When formatted with `{:?}` and `{:#?}`, Then none of the fake values appear and `[REDACTED]` does. → `config::tests::debug_redacts_secrets`
- AC8. Given an invalid `RITMA_SECRET_KEY`, Then the `ConfigError` text contains no part of the supplied value. → `config::tests::error_does_not_echo_secret`
- AC9. Given an `RITMA_PUBLIC_URL` with userinfo, When configuration is parsed, Then it is rejected because this URL is a non-secret Config field and SBD-50 requires credential-bearing URLs to be treated as secrets. → `config::tests::public_url_rejects_credentials`
- AC10. Given only some of the CAPTCHA provider, site key and secret, When configuration is parsed, Then it is rejected rather than silently disabling or constructing an incomplete CAPTCHA config. → `config::tests::captcha_configuration_requires_all_fields`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/config/tests.rs` (`config::tests`) | `missing_database_url`, `short_secret_rejected`, `defaults`, `unknown_vars_reported`, `debug_redacts_secrets`, `error_does_not_echo_secret`, `public_url_rejects_credentials`, `captcha_configuration_requires_all_fields` |
| `crates/eq-core/src/error/tests.rs` (`error::tests`) | `conflict_problem_json`, `internal_hides_detail`, `rate_limited_sets_retry_after`, `validation_lists_errors`, `unavailable_sets_retry_after`, `core_codes_valid_and_unique` |
| `crates/eq-core/src/clock.rs` (mod tests) | `manual_clock_advances` |

## Definition of done
- [x] Rust checks pass
- [x] Commit `feat(core): config, clock and problem+json errors [T-002]`, signed off

## Report

**Plan** — Follow the resolved Config, Clock and AppError contracts. Add and run the named tests first; update the approved SQLx baseline without suppressing advisory checks; implement Config, Clock and problem responses; then run the full Rust, security, guard and runtime checks.

**Assumptions** — Q4 was resolved under the maintainer's run-5 delegation: use SQLx 0.9 and its matching CLI baseline, keep the Postgres-only workspace features, and keep `cargo audit` and `cargo deny` enabled. `spec/24-open-source-stack.md` records the dependency choice. Use only defaults specified by OPS-01/07/10 or this task. A Config instance remains shared through `Arc<Config>`; this task itself does not introduce a server or a SQL query.

**Test plan** —
- Required Config entries and key length → `missing_database_url`, `short_secret_rejected`, `defaults`.
- Unknown `RITMA_*` keys are named in the warning without their values → `unknown_vars_reported`.
- Debug formatting and invalid-key errors do not expose fake S3 values → `debug_redacts_secrets`, `error_does_not_echo_secret`.
- Public URLs reject credentials and normalize path slashes; partial CAPTCHA settings fail closed → `public_url_rejects_credentials`, `public_url_trailing_path_slash_is_normalized`, `captcha_configuration_requires_all_fields`.
- Loopback proxy defaults and explicit CIDR additions → `defaults`, `explicit_trusted_proxies_extend_loopback_defaults`.
- Conflict, internal, rate-limited, validation and unavailable responses match RFC 9457, with extensions and retry headers → the corresponding named `error::tests`.
- Request IDs are omitted outside middleware scope and included inside it → `request_id_is_omitted_without_scope`, `request_id_is_included_when_scoped`.
- Core error codes and field error codes use the specified closed sets; SQL row-not-found, connection-loss, timeout and other errors map as specified → `core_codes_valid_and_unique`, `field_error_codes_match_val_03`, `sqlx_errors_map_to_core_errors`.
- Edge cases include standard and URL-safe base64, 16-byte key rejection, unknown environment keys, URL credential rejection, explicit proxy extensions, empty validation params, and each AppError mapping.
- Abuse cases cover fake DB URL passwords, key material, owner URL, setup token, SMTP URL, CAPTCHA secret and metrics token in compact and alternate Debug output, plus no invalid-key echo.

**Step 0 — Start** — Resumed on `milestone/m0-foundation` at `d4404a8`; the tree was clean and the task index was current. T-001 is `review`, so T-002 is eligible under the run's `review`-counts-for-ordering rule. The maintainer delegated handling the Q4 blocker for relevant dependency files. T-002 was set to `in_progress` and the index regenerated.

**Step 1 — Task/spec** — Read the complete T-002 task and historical Questions. The active interfaces use secrecy-backed Config secrets, hand-written redacted Debug, `ErrorCodeDef` references, the named 5-second technical retry hint and the local `SLOT_TAKEN` test definition. Read OPS-01/07/10, OSS-04, ERR-10/20/21/30/31/43, CQ-11/21/22/23/27/28 and §9/§12, CA-01/31 and §4, SBD-10/20/30/50/52 and §8, CAN-24, VAL-03, and `spec/29-erd.md` §§1/4/5/7. No migration or ERD change is part of this task.

**Step 2 — Dependencies** — Read T-001's Report, Deviations, Follow-ups and shipped workspace scaffold. T-001's lockfile records the dependency resolution; it has no outstanding interface conflict with T-002. Its PostgreSQL availability follow-up was checked during this run: Docker is reachable and the `postgres:16-alpine` image reports PostgreSQL 16.15.

**Step 3 — Design reference** — n/a; this is a backend task with no UI.

**Step 4 — Plan and test plan** — The plan and AC-to-test mapping above were recorded before implementation. The implementation follows the resolved ErrorCodeDef and Config security contracts. The previously blocked SQLx advisory was resolved by changing the shared SQLx version to 0.9, updating the dependency map, and retaining both audit tools without an ignore rule.

**Step 5 — Implement** — Added Config parsing via Figment's raw environment provider and a testable map parser. S3 values enter secrecy wrappers before parsing, keys are decoded into `SecretSlice`, temporary values zeroize on drop, and Config/CaptchaConfig/SensitiveMap use hand-written redacted Debug with no secret-bearing Clone or serialization. Added Clock/SystemClock/ManualClock, the core ErrorCodeDef registry, VAL-03 field-code constants, AppError-to-Problem rendering, request ID task-local lookup, retry headers and SQLx error mapping. Split module tests into the task-listed test subfiles so each source file remains under CQ-11's 600-line review limit.

**Step 6 — Run locally** — `make dev` and `make e2e` both returned `No rule to make target` because T-006 has not added the Makefile yet. The task has no live endpoint or migration to exercise. Axum response bodies and Config happy/error cases were exercised through the unit tests. PostgreSQL is available but is not needed by this change.

**Step 7 — Debug loop** — The initial test scaffold ran red for all 16 named tests before implementation. During implementation, the compiler caught a URL borrow conflict; the path was copied before mutation. Clippy required a collapsed `if let` for Retry-After insertion. The first request-ID test evaluated `into_response()` before entering task-local scope; moving construction inside the scoped future fixed it. Each failure was corrected and the relevant tests/checks rerun successfully.

**Step 8 — Self-review** — Reviewed the diff against CQ-11/§9/§12, CA-01/31, ERR-10/20/21/30/31/42/43 and SBD-81. New source files are within the 600-line limit. SQL is not added, modules are not cross-imported, and no unrelated crates or endpoints are changed. The source has no `dbg!`, `todo!`, production `unwrap()`, `#[allow]`, user data or real credentials. The only `expect` is in the test-only `Config::for_tests` constructor and carries a reason.

**Step 9 — Optimize** — n/a; no runtime query or endpoint was added, and no performance path exists to measure.

**Step 10 — Security audit** — T3/T4 apply. S3 values use `secrecy` wrappers, zeroization and redacted Debug; parse errors do not echo supplied values; unknown-variable warnings include names only; public URLs with credentials are rejected; internal responses hide the cause; SQL errors are parameter-free in client output. Auth-chain, tenant/RLS, exported PII, external calls and migration controls are n/a because this task adds none. Fake-secret abuse tests pass. `cargo audit`, `cargo deny check advisories` and `gitleaks detect --redact --no-banner` passed. No security waiver was added.

**Step 11 — Revise and full verification** — All Rust, security and available guard checks passed after the final edits. Web, hosted CI, Makefile runtime/E2E targets and later T-006 guard scripts were not available at this task boundary; details are recorded below.

**Step 12 — Report, commit** — T-002 is set to `review`, the task index is regenerated, and the signed task commit uses the exact message required by the Definition of done.

**Summary** — Config, Clock and RFC 9457 error handling are implemented. All 26 `eq-core` tests pass, the full Rust workspace suite passes, and dependency audits pass on SQLx 0.9 without suppressions. `make dev` and `make e2e` are unavailable until T-006 adds those targets.

**Spec read** — `spec/02-architecture.md` §§2–3; `spec/11-ops.md` OPS-01/07/10; `spec/16-code-quality.md` CQ-11/21/22/23/27/28, §§9/12; `spec/17-clean-architecture.md` CA-01/31 and §4; `spec/18-security-by-design.md` SBD-10/20/30/50/52 and §8; `spec/19-error-handling.md` ERR-10/20/21/30/31/42/43 and §10; `spec/20-state-machines.md`; `spec/21-data-flows.md`; `spec/22-validation-and-sanitization.md` VAL-03; `spec/23-contracts-and-canonical-forms.md` CAN-24; `spec/24-open-source-stack.md` OSS-04; `spec/29-erd.md` §§1/4/5/7.

**Files** — `Cargo.toml`, `Cargo.lock`, `spec/24-open-source-stack.md`, `crates/eq-core/Cargo.toml`, `crates/eq-core/src/lib.rs`, `crates/eq-core/src/config.rs`, `crates/eq-core/src/config/tests.rs`, `crates/eq-core/src/clock.rs`, `crates/eq-core/src/error.rs`, `crates/eq-core/src/error/tests.rs`, `crates/eq-core/src/errors.rs`, `tasks/M0-foundation/T-002-core-config-clock-errors.md`, and `tasks/INDEX.md` regenerated through `scripts/tasks-index.sh`.

**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `config::tests::missing_database_url` | Passed |
| AC2 | `config::tests::short_secret_rejected` | Passed |
| AC3 | `config::tests::defaults` | Passed |
| AC4 | `error::tests::conflict_problem_json` | Passed |
| AC5 | `error::tests::internal_hides_detail` | Passed |
| AC6 | `clock::tests::manual_clock_advances` | Passed |
| AC7 | `config::tests::debug_redacts_secrets` | Passed for compact and alternate Debug formats with fake secrets |
| AC8 | `config::tests::error_does_not_echo_secret` | Passed |
| AC9 | `config::tests::public_url_rejects_credentials` | Passed |
| AC10 | `config::tests::captcha_configuration_requires_all_fields` | Passed |
| Additional | Unknown vars, URL normalization, proxy extension, request IDs, error mappings, SQLx mapping and VAL-03 constants | Passed |

**Checks run** — ✅ `cargo fmt --all --check`; ✅ `cargo clippy --workspace --all-targets -- -D warnings`; ✅ `cargo test --workspace` (26 eq-core tests and CLI/workspace/doc tests); ✅ `cargo test -p eq-core --features testkit` (26 tests); ✅ `cargo clippy -p eq-core --features testkit --all-targets -- -D warnings`; ✅ `cargo build --workspace`; ✅ `cargo build -p eq-server --no-default-features`; ✅ `cargo audit`; ✅ `cargo deny check advisories` (default config; repository has no `deny.toml`); ✅ `gitleaks detect --redact --no-banner`; ✅ `scripts/check-deps.sh`; ✅ `scripts/tasks-index.sh --check`; ✅ `python3 scripts/erd.py --check`; ✅ `git diff --check`; ✅ `cargo sqlx --version` (0.9.0). Not run: web checks because `web/` does not exist; `make dev` and `make e2e` because the Makefile is introduced by T-006; `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh` and `scripts/check-layers.sh` because T-006 has not created them; `make api` and `cargo sqlx prepare --workspace` because no API or query macro changed; `scripts/erd.py --migrations` because no migration changed; hosted CI because it requires GitHub Actions.

**Manual checks** — Unit tests exercised Config defaults and invalid inputs, response status/body/content type, extension fields, retry headers and request ID scope. No server request could be sent because T-004/T-006 have not added a runnable server target or Makefile.

**Debug log** — Resolved one borrow-checker error while normalizing a URL, one clippy `collapsible_if` finding, and one test setup error that constructed a response before entering the request-ID scope. The full relevant test/check commands passed after each correction.

**Performance** — n/a; no query or endpoint was introduced.

**Security** — The SBD-81 controls relevant to config input and secrets were reviewed. All config map values are wrapped in secrecy-backed storage before parsing. Debug output omits fake secret values; invalid key and public URL failures omit supplied values. Problem responses never include an internal error's cause. No SQL query, permission check, tenant table or external call is introduced.

**Deviations** — **RATIFY:** (1) OPS-01's Docker private-proxy default has no CIDRs or detection contract, so use only the specified loopback CIDRs by default and require explicit proxy entries; the architect already ratified this in Q3. (2) Reject credential-bearing `RITMA_PUBLIC_URL` values so the non-secret URL field cannot expose credentials through Debug. (3) Reject partial CAPTCHA provider/site-key/secret tuples instead of silently disabling or accepting incomplete configuration. Items (2) and (3) remain the Run-4 self-resolutions recorded in Questions. Q3's retry constant and Q2's ErrorCodeDef interface are architect-resolved. Q4 was resolved under the maintainer's run-5 delegation by updating the shared SQLx baseline and spec map to 0.9; both advisory checks remain enabled and green. Moving tests to subfiles is a CQ-11 file-size implementation detail and does not change interfaces or behavior.

**Dependencies affected** — T-003 and later crates consume `Config`, SQLx and AppError. The SQLx 0.9 workspace baseline is inherited by later milestones; those tasks must continue to run their own Rust and DB checks. T-203 retains `slot_taken` registration ownership.

**Rollback plan** — Reverting the signed T-002 commit restores the prior blocked task state and SQLx 0.8 dependency baseline; no database schema or persistent data is changed.

**Follow-ups** — Review the delegated SQLx 0.9 dependency-map update as part of T-002. Confirm downstream tasks against the inherited SQLx 0.9 API during their implementation. T-203 remains responsible for shared `slot_taken` registration.

## Questions
- **Resolved (architect, 2026-10-25):** Q1 — option (a), as recommended. The Interfaces section now uses `secrecy` types for every S3 field, a hand-written redacting `Debug`, no `Clone`, a defined `CaptchaConfig`, and a `## Security` section with AC7 and AC8. Nothing else in M0–M3 reads the secret fields directly (T-003 `connect` takes `&Config`, T-004 holds `Arc<Config>`), so no other task changes.
- **Q1 (blocking):** May T-002 amend the `Config` interface so every S3 value (including keys, setup/metrics tokens, captcha secrets, and any credential-bearing database/SMTP URLs) uses `secrecy::SecretString`/`SecretBox`, with a custom redacted `Debug` implementation and an abuse-case test proving the formatted output omits fake secret values?
  - Context: T-002 lines 63–71 require `#[derive(Debug, Clone)]` and plain `String` fields for values that can hold credentials or tokens. `spec/18-security-by-design.md` SBD-10 requires S3 field classification/protection, and SBD-50 requires redacting, zeroizing wrappers and prohibits plain `Debug` on a struct holding secrets.
  - Options: (a) amend the interface as stated above and classify each S3 field; (b) define a different explicit secret-bearing `Config` shape that meets SBD-10/SBD-50 and update the task tests/interfaces to match.
  - Recommendation: (a), because it keeps the current `Config` layout while changing only the sensitive field types and debug behavior needed to meet the security spec.
- **Resolved (architect, 2026-10-25):** Q2 — option (a). `AppError` variants that take a code now carry `&'static ErrorCodeDef` (Interfaces, status mapping and AC4 updated). `Problem.title` is read from that definition; no registry injection. `slot_taken` is shared by scheduling and rental, so T-203 registers it in `eq_core::error::codes` (409, "Slot already taken"); T-002 only uses a test-local copy in AC4. Callers elsewhere pass `&codes::X` (T-105 and spec 19 ERR-21 updated).
- **Q2 (historical; resolved):** How must `AppError` expose an error-code title to the `eq-core` response renderer when the code is defined by another crate?
  - Context: T-002 defines `AppError` code fields as `&'static str` and requires `Problem.title` to come from the matching `ErrorCodeDef`. ERR-21 says each crate owns its `CODES`, while `IntoResponse` is implemented in `eq-core`; the task defines no registry injection or title-bearing code reference. AC4 requires `slot_taken`, but the listed core `CODES` omit it (ERR-30 gives its title).
  - Options: (a) change code fields in `AppError` to carry `&'static ErrorCodeDef` and update the task interface/tests; (b) specify a registry-injection API that makes module registries available to the core renderer.
  - Recommendation: (a), because it keeps code/status/title together without global mutable registry state and lets each crate pass its own registered definition.

- **Self-resolved (implementer, run 3, needs architect ratification):** OPS-01 says the trusted-proxy default includes private ranges in Docker but supplies no CIDRs or Docker-detection contract → when no explicit proxy list is configured, use only OPS-01's exact loopback CIDRs and require explicit `RITMA_TRUSTED_PROXIES` for additional proxies, because SBD-20's deny-by-default reading is safer than trusting an unspecified range.
- **Resolved (architect, run-3 follow-up):** Q3 — option (a). The retry duration is the named technical constant `DB_UNAVAILABLE_RETRY_AFTER_S = 5` (Context). It is a back-off hint, not a business rule, and AGENTS §4.1 allows technical constants with a `// technical:` comment. Also ratified: the loopback-only default for `RITMA_TRUSTED_PROXIES` (explicit entries add proxies; SBD-20).
- **Q3 (historical; resolved):** What retry duration or policy source must `From<sqlx::Error>` use for `PoolTimedOut`/connection-loss errors mapped to `Unavailable { retry_after_s }`?
  - Context: T-002 requires both this SQL mapping and a `Retry-After` response header for `Unavailable`, but specifies no retry duration or policy field. The Run 3 hard limit forbids inventing a duration.
  - Options: (a) amend the task/spec with an approved retry policy field or named technical value and use it in the SQL mapping; (b) change the SQL conversion contract so it can return `Unavailable` without an invented delay, then amend the required header behavior consistently in the task/spec.
  - Recommendation: (a), if the `Retry-After` requirement is retained, because it provides a traceable duration while preserving ERR-10 and the task's current response contract.
- **Self-resolved (implementer, run 4, needs architect ratification):** `RITMA_PUBLIC_URL` must parse as HTTP(S), while Config classifies its non-S3 fields as S0/S1 and SBD-50 treats credential-bearing URLs as secrets → reject a public URL containing username or password because a public origin is not specified as an S3 secret and must not leak credentials through Config Debug.
- **Self-resolved (implementer, run 4, needs architect ratification):** the three optional CAPTCHA environment values map to `Option<CaptchaConfig>`, but no partial-configuration behavior is given → reject a partial provider/site-key/secret tuple because the option represents the complete configuration and silent disabling would hide invalid operator input.

- **Q4 (blocking):** May T-002 ship with SQLx 0.8.6 while `cargo audit` reports RUSTSEC-2023-0071 for locked `rsa 0.9.10`, even though the MySQL driver is optional and absent from the active workspace feature graph?
  - Context: T-002 requires `From<sqlx::Error>`; SQLx 0.8.6's lock graph includes `rsa 0.9.10` through the optional MySQL driver. `cargo tree --workspace -i rsa` reports no active path, but `cargo audit` scans Cargo.lock and fails with “No fixed upgrade is available.” `cargo deny check advisories` passed using its default config; there is no `deny.toml`. SBD-75 lists both audit tools, and SBD-64 prohibits waiving a security rule without an approved spec change.
  - Options: (a) approve an explicit policy for this inactive optional dependency and its audit handling; (b) change the dependency/task contract to a dependency graph that passes the audit without an exception; (c) keep T-002 blocked until SQLx/RustSec publishes a fixed dependency path.
  - Recommendation: (c), because I cannot waive or alter the security gate, and the task does not authorize replacing SQLx with an unlisted dependency.
  - **Resolved (maintainer delegation, 2026-10-25):** option (b). Use SQLx 0.9 with the existing Postgres-only feature set and matching SQLx CLI, and update `spec/24-open-source-stack.md`; do not suppress or ignore the audit. The maintainer delegated resolution of this blocker for relevant task and dependency files.
