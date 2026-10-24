---
id: T-002
title: Add config loading, the Clock abstraction and AppError → problem+json in eq-core
milestone: M0
status: todo
depends_on: [T-001]
spec_refs: [OPS-01, ARC-10, ARC-12, API-04]
estimated_size: M
---

## Goal
Every later crate can read a typed `Config`, get time from an injectable `Clock`, and return `AppError`, which renders as RFC 9457 `application/problem+json` with stable codes.

## Context
- OPS-01 env vars:
  - Required: `DATABASE_URL`, `EQ_SECRET_KEY` (base64, ≥ 32 bytes decoded), `EQ_PUBLIC_URL`.
  - Optional:
    - `EQ_DATABASE_OWNER_URL` (default = DATABASE_URL), `EQ_SECRET_KEY_PREVIOUS`
    - `EQ_MODE` (`single`|`multi`, default single), `EQ_LISTEN` (default `0.0.0.0:8080`)
    - `EQ_DATA_DIR` (default `./data` in debug builds, `/var/lib/equalizer` in release builds)
    - `EQ_FILES_BACKEND` (`local`|`s3`), `EQ_SETUP_TOKEN`, `EQ_TRUSTED_PROXIES` (CSV of CIDRs)
    - `EQ_LOG` (default `info`), `EQ_LOG_FORMAT` (`json`|`pretty`)
    - `EQ_BACKUP_SCHEDULE` (`off`|`daily`), `EQ_BACKUP_KEEP` (default 14), `EQ_SMTP_URL`
    - `EQ_CAPTCHA_PROVIDER`/`_SITE_KEY`/`_SECRET`, `EQ_TILE_URL`, `EQ_TELEMETRY` (`off`|`on`), `EQ_UPDATE_CHECK` (`on`|`off`)
    - `EQ_METRICS_TOKEN`, `EQ_DB_POOL` (default 10), `EQ_DEMO` (bool)
  - Unknown `EQ_*` variables produce a warning listing them.
- ARC-10: `domain/` code never reads the clock. Services use `Clock`.
- ARC-12 `AppError` variants and API-04 mapping (HTTP status → `code`), shown in Interfaces.

## Scope
### In
- `eq_core::config::{Config, Mode, FilesBackend, LogFormat}` with `Config::from_env() -> Result<Config, ConfigError>` and `Config::for_tests()`.
- `eq_core::clock::{Clock (trait), SystemClock, Timestamp}` + `eq_core::clock::test::ManualClock` behind `#[cfg(any(test, feature = "testkit"))]`.
- `eq_core::error::{AppError, FieldError, Problem}` + `impl IntoResponse for AppError` (axum) + `From<sqlx::Error>` mapping (RowNotFound → NotFound, others → Internal).
- The problem `type` URI base is the constant `PROBLEM_BASE = "https://equalizer.dev/problems/"` + code with `_` replaced by `-`.

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
    #[error("policy violation: {rule}")] PolicyViolation { rule: String, detail: String, extra: serde_json::Map<String, serde_json::Value> },
    #[error("precondition required")] PreconditionRequired,
    #[error("gone: {code}")] Gone { code: &'static str },
    #[error("locked")] Locked { until: Timestamp },
    #[error("rate limited")] RateLimited { retry_after_s: u64 },
    #[error("payload too large")] PayloadTooLarge,
    #[error("bad request: {code}")] BadRequest { code: &'static str, detail: String },
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
- PolicyViolation → 422 `policy_violation` (+`rule`, extras)
- PreconditionRequired → 428 `precondition_required`
- Gone → 410
- Locked → 423 `locked` (+`until`)
- RateLimited → 429 `rate_limited` + `Retry-After` header
- PayloadTooLarge → 413 `payload_too_large`
- BadRequest → 400
- Internal → 500 `internal` (detail "Internal error"; log the error with `tracing::error!` including the chain)

Body fields: `type`, `title` (human title per code, English), `status`, `code`, `detail`, `request_id` (from a task-local set by T-004; use `Option` and omit if absent), plus extension members flattened. Header `Content-Type: application/problem+json`.

## Business rules
None (technical only).

## Implementation notes
- Decode `EQ_SECRET_KEY` as standard or URL-safe base64. If it decodes to < 32 bytes → `Invalid`.
- `EQ_PUBLIC_URL` must parse as http(s). A trailing slash is normalised away.
- Wrap `Config::from_env` around a `fn from_map(HashMap<String,String>)` so tests don't touch the process env.
- The request-id task-local: `tokio::task_local! { pub static REQUEST_ID: uuid::Uuid; }` in `error.rs`, read with `REQUEST_ID.try_with(|id| *id).ok()`.

## Acceptance criteria
- AC1. Given a map without `DATABASE_URL`, When `from_map`, Then `Missing("DATABASE_URL")`. → `config::tests::missing_database_url`
- AC2. Given a 16-byte key, Then `Invalid{var:"EQ_SECRET_KEY"}`. → `config::tests::short_secret_rejected`
- AC3. Given defaults, Then `listen=0.0.0.0:8080`, `mode=Single`, `backup_keep=14`. → `config::tests::defaults`
- AC4. `AppError::Conflict{code:"slot_taken",…}` renders 409 with JSON `code=="slot_taken"` and `type` ending in `/slot-taken`, and the extension members appear at top level. → `error::tests::conflict_problem_json`
- AC5. `AppError::Internal` renders 500 and its detail doesn't contain the inner error message. → `error::tests::internal_hides_detail`
- AC6. `ManualClock::advance` changes `now()`. → `clock::tests::manual_clock_advances`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/config.rs` (mod tests) | `missing_database_url`, `short_secret_rejected`, `defaults`, `unknown_vars_reported` |
| `crates/eq-core/src/error.rs` (mod tests) | `conflict_problem_json`, `internal_hides_detail`, `rate_limited_sets_retry_after`, `validation_lists_errors` |
| `crates/eq-core/src/clock.rs` (mod tests) | `manual_clock_advances` |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `feat(core): config, clock and problem+json errors [T-002]`, signed off

## Report

## Questions
