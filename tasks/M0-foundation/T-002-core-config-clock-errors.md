---
id: T-002
title: Add config loading, the Clock abstraction and AppError → problem+json in eq-core
milestone: M0
status: todo
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
- ARC-10: `domain/` code never reads the clock. Services use `Clock`.
- ARC-12 `AppError` variants and API-04 mapping (HTTP status → `code`), shown in Interfaces.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `FieldError.code` uses only the closed VAL-03 set (`required`, `invalid_type`, `too_short`, `too_long`, `too_few_items`, `too_many_items`, `must_be_gte`, `must_be_lte`, `invalid_format`, `invalid_characters`, `not_allowed`, `not_unique`, `unknown_reference`). Declare them as constants in `eq_core::errors` (T-102a adds the parsers that emit them).
  - Multipart field errors point at `/$files/<part>` for file parts (CAN-24, D-08).

## Scope
### In
- `eq_core::config::{Config, Mode, FilesBackend, LogFormat}` with `Config::from_env() -> Result<Config, ConfigError>` and `Config::for_tests()`.
- `eq_core::clock::{Clock (trait), SystemClock, Timestamp}` + `eq_core::clock::test::ManualClock` behind `#[cfg(any(test, feature = "testkit"))]`.
- `eq_core::error::{AppError, FieldError, Problem}` + `impl IntoResponse for AppError` (axum) + `From<sqlx::Error>` mapping (RowNotFound → NotFound, pool timeout → Unavailable, others → Internal; the full SQLSTATE mapping in ERR-43 is added by the tasks that need it).
- ERR-21 registry: `pub struct ErrorCodeDef { pub code: &'static str, pub status: u16, pub title: &'static str }` and `eq_core::error::codes` with the core codes (`bad_request`, `unauthenticated`, `forbidden`, `not_found`, `module_disabled`, `validation`, `policy_violation`, `precondition_required`, `payload_too_large`, `locked`, `rate_limited`, `unavailable`, `internal`) plus `pub const CODES: &[ErrorCodeDef]`. `Problem.title` comes from the matching `ErrorCodeDef`. A helper `pub fn validate_code(code: &str) -> bool` checks the ERR-20 format `^[a-z][a-z0-9_]{2,47}$`.
- The problem `type` URI base is the constant `PROBLEM_BASE = "https://ritma.dev/problems/"` + code with `_` replaced by `-`.

### Out
- Exclusion-violation mapping (T-203), and the tracing setup (T-004).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/config.rs` | create | Config |
| `crates/eq-core/src/clock.rs` | create | Clock |
| `crates/eq-core/src/error.rs` | create | AppError/Problem |
| `crates/eq-core/src/lib.rs` | edit | `pub mod config; pub mod clock; pub mod error;` |
| `crates/eq-core/Cargo.toml` | edit | deps: axum, serde, serde_json, thiserror, anyhow, chrono, base64, sqlx, tracing, zeroize + add `url = "2"` and `ipnet = { version = "2", features = ["serde"] }` to `[workspace.dependencies]`; feature `testkit` |

## Interfaces
```rust
pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub trait Clock: Send + Sync + std::fmt::Debug + 'static { fn now(&self) -> Timestamp; }
#[derive(Debug, Default, Clone)] pub struct SystemClock;
// test
#[derive(Debug, Clone)] pub struct ManualClock(std::sync::Arc<std::sync::Mutex<Timestamp>>);
impl ManualClock { pub fn new(t: Timestamp) -> Self; pub fn set(&self, t: Timestamp); pub fn advance(&self, d: chrono::Duration); }

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String, pub database_owner_url: String,
    pub secret_key: zeroize::Zeroizing<Vec<u8>>, pub secret_key_previous: Option<zeroize::Zeroizing<Vec<u8>>>,
    pub public_url: url::Url, pub mode: Mode, pub listen: std::net::SocketAddr, pub data_dir: std::path::PathBuf,
    pub files_backend: FilesBackend, pub setup_token: Option<String>, pub trusted_proxies: Vec<ipnet::IpNet>,
    pub log: String, pub log_format: LogFormat, pub backup_schedule: BackupSchedule, pub backup_keep: u16,
    pub smtp_url: Option<String>, pub captcha: Option<CaptchaConfig>, pub tile_url: String,
    pub telemetry: bool, pub update_check: bool, pub metrics_token: Option<String>, pub db_pool: u32, pub demo: bool,
}
#[derive(Debug, thiserror::Error)]
pub enum ConfigError { #[error("missing required env var {0}")] Missing(&'static str), #[error("invalid {var}: {reason}")] Invalid { var: &'static str, reason: String } }

#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldError { pub pointer: String, pub code: String, #[serde(skip_serializing_if = "serde_json::Map::is_empty")] pub params: serde_json::Map<String, serde_json::Value> }

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")] NotFound { entity: &'static str },
    #[error("conflict: {code}")] Conflict { code: &'static str, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("validation failed")] Validation { errors: Vec<FieldError> },
    #[error("forbidden")] Forbidden { perm: &'static str },
    #[error("forbidden: {code}")] ForbiddenRule { code: &'static str, detail: String },
    #[error("unauthenticated")] Unauthenticated { code: &'static str },
    #[error("module disabled")] ModuleDisabled { module: String },
    #[error("unprocessable: {code}")] Unprocessable { code: &'static str, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("policy violation: {rule}")] PolicyViolation { rule: String, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("precondition required")] PreconditionRequired,
    #[error("gone: {code}")] Gone { code: &'static str },
    #[error("locked")] Locked { until: Timestamp },
    #[error("rate limited")] RateLimited { retry_after_s: u64 },
    #[error("payload too large")] PayloadTooLarge,
    #[error("bad request: {code}")] BadRequest { code: &'static str, detail: String },
    #[error("unavailable")] Unavailable { retry_after_s: u64 },
    #[error(transparent)] Internal(#[from] anyhow::Error),
}
```
Status/code mapping:
- NotFound → 404 `not_found`
- Conflict → 409 `<code>`
- Validation → 422 `validation` (+`errors`)
- Forbidden → 403 `forbidden` (+`perm`)
- ForbiddenRule → 403 `<code>`
- Unauthenticated → 401 `<code>`
- ModuleDisabled → 404 `module_disabled` (+`module`)
- Unprocessable → 422 `<code>` (+extras)
- PolicyViolation → 422 `policy_violation` (+`rule`, extras)
- PreconditionRequired → 428 `precondition_required`
- Gone → 410
- Locked → 423 `locked` (+`until`)
- RateLimited → 429 `rate_limited` + `Retry-After` header
- PayloadTooLarge → 413 `payload_too_large`
- BadRequest → 400
- Unavailable → 503 `unavailable` + `Retry-After` header
- Internal → 500 `internal` (detail "Internal error"; log the error with `tracing::error!` including the chain)

Body fields: `type`, `title` (human title per code, English), `status`, `code`, `detail`, `request_id` (from a task-local set by T-004; use `Option` and omit if absent), plus extension members flattened. Header `Content-Type: application/problem+json`.

## Business rules
None (technical only).

## Implementation notes
- CQ-21 forbids `chrono::Utc::now` and `std::env::var` workspace-wide. `SystemClock::now` is the one allowed caller of `Utc::now`: annotate it with `#[expect(clippy::disallowed_methods, reason = "ARC-10: SystemClock is the only wall-clock reader")]`. Config loading uses figment/envy; if you call `std::env::var` directly, do it only inside `eq_core::config` with the same kind of `#[expect]`.
- Decode `RITMA_SECRET_KEY` as standard or URL-safe base64. If it decodes to < 32 bytes → `Invalid`.
- `RITMA_PUBLIC_URL` must parse as http(s). A trailing slash is normalised away.
- Wrap `Config::from_env` around a `fn from_map(HashMap<String,String>)` so tests don't touch the process env.
- The request-id task-local: `tokio::task_local! { pub static REQUEST_ID: uuid::Uuid; }` in `error.rs`, read with `REQUEST_ID.try_with(|id| *id).ok()`.

## Acceptance criteria
- AC1. Given a map without `DATABASE_URL`, When `from_map`, Then `Missing("DATABASE_URL")`. → `config::tests::missing_database_url`
- AC2. Given a 16-byte key, Then `Invalid{var:"RITMA_SECRET_KEY"}`. → `config::tests::short_secret_rejected`
- AC3. Given defaults, Then `listen=0.0.0.0:8080`, `mode=Single`, `backup_keep=14`. → `config::tests::defaults`
- AC4. `AppError::Conflict{code:"slot_taken",…}` renders 409 with JSON `code=="slot_taken"` and `type` ending in `/slot-taken`, and the extension members appear at top level. → `error::tests::conflict_problem_json`
- AC5. `AppError::Internal` renders 500 and its detail doesn't contain the inner error message. → `error::tests::internal_hides_detail`
- AC6. `ManualClock::advance` changes `now()`. → `clock::tests::manual_clock_advances`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/config.rs` (mod tests) | `missing_database_url`, `short_secret_rejected`, `defaults`, `unknown_vars_reported` |
| `crates/eq-core/src/error.rs` (mod tests) | `conflict_problem_json`, `internal_hides_detail`, `rate_limited_sets_retry_after`, `validation_lists_errors`, `unavailable_sets_retry_after`, `core_codes_valid_and_unique` |
| `crates/eq-core/src/clock.rs` (mod tests) | `manual_clock_advances` |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `feat(core): config, clock and problem+json errors [T-002]`, signed off

## Report

## Questions
