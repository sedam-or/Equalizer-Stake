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
| `crates/eq-core/src/clock.rs` | create | Clock |
| `crates/eq-core/src/error.rs` | create | AppError/Problem |
| `crates/eq-core/src/lib.rs` | edit | `pub mod config; pub mod clock; pub mod error;` |
| `crates/eq-core/Cargo.toml` | edit | deps: axum, serde, serde_json, thiserror, anyhow, chrono, base64, sqlx, tracing, zeroize, secrecy + add `url = "2"` and `ipnet = { version = "2", features = ["serde"] }` to `[workspace.dependencies]`; feature `testkit` |

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

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/src/config.rs` (mod tests) | `missing_database_url`, `short_secret_rejected`, `defaults`, `unknown_vars_reported`, `debug_redacts_secrets`, `error_does_not_echo_secret` |
| `crates/eq-core/src/error.rs` (mod tests) | `conflict_problem_json`, `internal_hides_detail`, `rate_limited_sets_retry_after`, `validation_lists_errors`, `unavailable_sets_retry_after`, `core_codes_valid_and_unique` |
| `crates/eq-core/src/clock.rs` (mod tests) | `manual_clock_advances` |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `feat(core): config, clock and problem+json errors [T-002]`, signed off

## Report

**Plan** — Follow the architect-resolved interfaces in this task:
1. Add the exact named tests for Config, Clock, and AppError first; run them against the scaffold and record the expected failures.
2. Implement Config with the OPS-01 Figment configuration source and a testable map parser; convert S3 fields directly into secrecy wrappers, redact Debug manually, and validate required values without echoing inputs.
3. Implement the Clock abstraction and test-only ManualClock, keeping the only wall-clock read in SystemClock with the CQ-21-scoped lint expectation.
4. Implement ErrorCodeDef and the core registry, carrying `&'static ErrorCodeDef` through code-bearing AppError variants so response titles come from the definition; use AC4's local SLOT_TAKEN definition and do not register it in core.
5. Implement Problem serialization, extension flattening, request-id task-local lookup, Retry-After headers, and the specified SQL error mappings; run the exact response tests.
6. Wire the modules and permitted dependencies, then run every applicable Rust, guard, security, and runtime check; record unavailable checks as Not run with their reason.

**Assumptions** — Use the resolved ErrorCodeDef-carrying AppError interface and Config/CaptchaConfig security contract as written. Figment is the configuration source named by OSS-04/OPS-01; only defaults explicitly specified by OPS-01/07/10 or this task are applied. T-002 uses a local `SLOT_TAKEN` definition for AC4; T-203 owns its shared registration.

**Test plan** —
- Given a map without `DATABASE_URL`, when parsed, then `Missing("DATABASE_URL")` → `config::tests::missing_database_url`.
- Given a 16-byte decoded key, when parsed, then it is rejected as `RITMA_SECRET_KEY` → `config::tests::short_secret_rejected`.
- Given valid required values and omitted optional values, when defaults are applied, then the specified listen, mode, and backup retention are used → `config::tests::defaults`.
- Given an unknown `RITMA_*` variable, when configuration loads, then a warning identifies its name → `config::tests::unknown_vars_reported`.
- Given fake credentials and tokens, when Config is formatted with both Debug modes, then no fake secret appears and `[REDACTED]` does → `config::tests::debug_redacts_secrets`.
- Given an invalid key, when parsing fails, then the error contains no part of the supplied value → `config::tests::error_does_not_echo_secret`.
- Given a `slot_taken` conflict with extensions, when rendered, then status/code/type/extensions match the problem contract → `error::tests::conflict_problem_json`.
- Given an internal error, when rendered, then status is 500 and detail hides the cause → `error::tests::internal_hides_detail`.
- Given rate-limited or unavailable errors, when rendered, then `Retry-After` is set → `rate_limited_sets_retry_after` and `unavailable_sets_retry_after`.
- Given field validation errors, when rendered, then the field error list is present → `validation_lists_errors`.
- Given the core code registry, when validated, then every code matches ERR-20 and codes are unique → `core_codes_valid_and_unique`.
- Given a manual clock, when advanced, then `now()` reflects the duration → `clock::tests::manual_clock_advances`.
- Edge cases: missing/empty config entries; 31-byte versus 32-byte keys; standard and URL-safe base64; invalid URL scheme/trailing slash; unknown prefixed variables; absent request ID; empty/populated field error lists; retry values; SQL row-not-found, pool timeout, and other SQL errors.
- Error cases: ConfigError missing/invalid, all AppError mappings, internal detail hiding, invalid code syntax, retry headers, and SQL error mapping.
- Abuse cases: fake database URL password, key, CAPTCHA secret, setup token, SMTP URL, and metrics token never appear in compact/alternate Debug; invalid key input is not echoed.

**Step 0 — Start** — Run 3 started at `f15dc08fc1880ab82af623238f32d90427f2a5e6` on `milestone/m0-foundation`; the preflight tree was clean, the ancestry check and index check passed, and T-001 is `review`. T-002 is the lowest eligible task because its sole dependency is T-001. It was claimed as `in_progress` and the index regenerated before Step 1.
**Step 1 — Task/spec** — Read the full T-002 task including the architect's Q1 and Q2 resolutions. The current interface uses secrecy-backed S3 fields, redacted Debug, and `&'static ErrorCodeDef` for code-bearing AppError variants; AC4 uses a local `SLOT_TAKEN`, and T-203 owns its shared registration. Read OPS-01/07/10, OSS-04, ERR-10/20/21/30/31/43, CQ-21/22/23/27/28, CA-01/31, and SBD-10/20/30/50/52. The missing SQL pool retry duration is not specified in those contracts.
**Step 2 — Dependencies** — Read T-001's Report, Deviations and Follow-ups, plus its shipped workspace and `eq-core` scaffold. T-001 is `review` under Run 3 ordering. Its `Cargo.lock` deviation follows OSS-04, and its PostgreSQL follow-up is satisfied by the Run 3 preflight image check and running PostgreSQL 16 container; neither changes T-002's contract.
**Step 3 — Design reference** — n/a; no UI work.
**Step 4 — Plan and test plan** — Recorded above. The resolved Q1/Q2 interfaces remove the prior blockers. Before implementation, the required `From<sqlx::Error>` mapping is still underspecified: `PoolTimedOut`/connection-loss errors must become `Unavailable { retry_after_s }` and the response must set `Retry-After`, but no duration or policy source is defined. The Run 3 hard limit forbids inventing a duration. The task is blocked with options and a recommendation in Q3 below.
**Step 5 — Implement** — n/a; Step 4 is blocked on a missing retry policy value. No production code or tests were added.
**Step 6 — Run locally** — n/a; this task has no executable server yet, and `make dev` is introduced by T-006. No application runtime path was changed.
**Step 7 — Debug loop** — n/a; implementation did not start and no implementation failure occurred.
**Step 8 — Self-review** — n/a; no implementation diff.
**Step 9 — Optimize** — n/a; no runtime path was added.
**Step 10 — Security audit** — The task triggers T3 and T4. The resolved task contract requires secrecy wrappers, redacted Debug, no serialization/Clone for secret-bearing Config types, safe config errors, and abuse tests. These controls were not implemented or audited because the task is blocked before implementation.
**Step 11 — Revise and full verification** — The required format, clippy, and workspace test commands ran against the existing scaffold and passed; they contain no T-002 implementation or named T-002 tests. The task-index check, dependency guard, diff check, and secret scans also passed. Web, SQLx, API-generation, and runtime checks do not apply before implementation; the later T-006 scripts and Makefile are not present yet.

**Summary** — No production implementation was added. Q1 and Q2 are resolved and the plan now follows those interfaces, but SQL pool errors cannot be mapped to the required `Unavailable { retry_after_s }` without an approved retry duration. The task is `blocked` pending the Q3 contract answer.

**Spec read** — `spec/02-architecture.md` §§2–3; `spec/11-ops.md` OPS-01/07/10; `spec/16-code-quality.md` CQ-21/22/23/27/28, §§9/12; `spec/17-clean-architecture.md` CA-01/31 and §4; `spec/18-security-by-design.md` SBD-10/20/30/50/52 and §8; `spec/19-error-handling.md` ERR-10/20/21/30/31/43 and §10; `spec/20-state-machines.md`; `spec/21-data-flows.md`; `spec/22-validation-and-sanitization.md` VAL-03; `spec/23-contracts-and-canonical-forms.md` CAN-24; `spec/24-open-source-stack.md` OSS-04.

**Files** — Modified only `tasks/M0-foundation/T-002-core-config-clock-errors.md` (Run 3 plan, report and blocking question) and regenerated `tasks/INDEX.md` through `scripts/tasks-index.sh`.

**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `config::tests::missing_database_url` | Not run: blocked before implementation. |
| AC2 | `config::tests::short_secret_rejected` | Not run: blocked before implementation. |
| AC3 | `config::tests::defaults` | Not run: blocked before implementation. |
| AC4 | `error::tests::conflict_problem_json` | Not run: blocked before implementation. |
| AC5 | `error::tests::internal_hides_detail` | Not run: blocked before implementation. |
| AC6 | `clock::tests::manual_clock_advances` | Not run: blocked before implementation. |
| AC7 | `config::tests::debug_redacts_secrets` | Not run: blocked before implementation. |
| AC8 | `config::tests::error_does_not_echo_secret` | Not run: blocked before implementation. |
| Additional listed tests | `unknown_vars_reported`, `rate_limited_sets_retry_after`, `validation_lists_errors`, `unavailable_sets_retry_after`, `core_codes_valid_and_unique` | Not run: blocked before implementation. |

**Checks run** — ✅ `cargo fmt --all --check`; ✅ `cargo clippy --workspace --all-targets -- -D warnings`; ✅ `cargo test --workspace` (scaffold only: `eq-core` contains 0 tests; the existing `prints_version` test passed); ✅ `scripts/check-deps.sh`; ✅ `scripts/tasks-index.sh --check`; ✅ `git diff --check`; ✅ `gitleaks dir --redact --no-banner tasks/M0-foundation/T-002-core-config-clock-errors.md`; ✅ `gitleaks dir --redact --no-banner tasks/INDEX.md`. Not run: the named T-002 tests do not exist because implementation is blocked; web checks (`web/` is absent); API generation/SQLx prepare (no API or query change); `make dev`/`make e2e` (no Makefile until T-006); `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh` (introduced by T-006).
**Manual checks** — n/a; there is no task runtime to exercise.
**Debug log** — None; the block is due to a missing interface value, not an implementation failure.
**Performance** — n/a; no runtime path was added.
**Security** — Triggers T3 and T4 apply. No secret-bearing code or fixtures were added. On resumption, verify SBD-81, redacted Debug and non-echoing config errors, including the listed abuse tests.
**Deviations** — **RATIFY:** OPS-01 names loopback trusted proxies plus unspecified private ranges in Docker. If the task resumes without enumerated Docker CIDRs, default only to the exact loopback CIDRs given by OPS-01; require explicit `RITMA_TRUSTED_PROXIES` configuration for additional proxies, preserving the stricter deny-by-default behavior in SBD-20. The task interface comment now records this choice; no runtime code was changed.
**Dependencies affected** — T-003 depends directly on T-002; dependent tasks remain ineligible until T-002 returns to `todo` and reaches `review`/`done`.
**Rollback plan** — Revert this signed task-block commit; it changes only the task's status/report/questions and the generated index.
**Follow-ups** — Architect must specify the retry duration/policy source for SQL pool timeout and connection-loss mappings before T-002 can be resumed. T-203 remains responsible for the shared `slot_taken` registration after T-002 is completed.

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
