---
id: T-001
title: Create the Cargo workspace, toolchain, lints and empty crates
milestone: M0
status: todo
depends_on: []
spec_refs: [ARC-01, ARC-02, ADR-0001]
estimated_size: S
---

## Goal
`cargo build --workspace` succeeds on a fresh clone. The workspace has every crate from the architecture with the correct dependency edges, shared lints and pinned dependency versions, so later tasks only add code.

## Context
- ARC-01: one binary `equalizer` (crate `eq-server`) that serves the API and the embedded SPA and runs jobs.
- ARC-02 dependency rules:
  - `eq-core` depends on no `eq-*` crate.
  - Always-on crates are layered core → audit → identity → scheduling.
  - Optional crates (`eq-enrollment`, `eq-billing`, `eq-attendance`, `eq-rental`, `eq-notify`, `eq-reports`, `eq-feedback`) may depend on `eq-core`, `eq-audit`, `eq-identity` and `eq-scheduling`, **never on each other**.
  - `eq-api` depends on all of them, with optional crates behind Cargo features of the same name, all default-on.
  - `eq-server` depends on `eq-api`.
  - `eq-testkit` is dev-only.

## Scope
### In
- Root `Cargo.toml` with `[workspace]`, `resolver = "3"`, `members = ["crates/*"]`, `[workspace.package]` (edition 2024, license "AGPL-3.0-only", repository URL), `[workspace.dependencies]` for **all** crates listed below with versions, and `[workspace.lints]`.
- `rust-toolchain.toml`: `channel = "stable"` pinned to a concrete version ≥ 1.85, components rustfmt and clippy.
- `rustfmt.toml` (`max_width = 110`, `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`; use only stable options).
- 14 crates, each with `Cargo.toml` (inheriting workspace package and lints) and `src/lib.rs` (or `src/main.rs` for eq-server), containing a crate doc comment with its purpose and a `pub fn crate_name() -> &'static str` placeholder **only if needed for compilation**. Prefer empty libs.
- `eq-server` produces `[[bin]] name = "equalizer"` with a `main` that prints the version.
- `scripts/check-deps.sh`: parses `cargo metadata --format-version 1 --no-deps` with `jq` and fails if any optional crate depends on another optional crate, or if `eq-core` depends on any `eq-*`.

### Out
- Any real logic (T-002+). CI workflow (T-006).

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.toml` | create | workspace |
| `rust-toolchain.toml`, `rustfmt.toml`, `.cargo/config.toml` | create | toolchain; `[alias] xtask`-free; set `SQLX_OFFLINE=true` in `[env]` |
| `crates/eq-{core,audit,identity,scheduling,enrollment,billing,attendance,rental,notify,reports,feedback,api,server,testkit}/Cargo.toml` + `src/lib.rs`/`main.rs` | create | crates |
| `scripts/check-deps.sh` | create | ARC-02 guard |

## Interfaces
`[workspace.dependencies]` (minimum versions; use the latest compatible patch):
```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "time", "sync"] }
axum = { version = "0.8", features = ["macros", "multipart"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "request-id", "compression-full", "limit", "cors", "set-header", "util"] }
tower_governor = "0.7"
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml_ng = "0.10"
schemars = { version = "1", features = ["chrono04", "uuid1"] }
utoipa = { version = "5", features = ["axum_extras", "uuid", "chrono"] }
utoipa-axum = "0.2"
jsonschema = "0.30"
json-patch = "4"            # RFC 7386 merge
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = { version = "0.10", features = ["serde"] }
cron = "0.15"
uuid = { version = "1", features = ["v7", "serde"] }
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
clap = { version = "4", features = ["derive", "env"] }
jsonwebtoken = "9"
argon2 = "0.5"
aes-gcm = "0.10"
chacha20poly1305 = { version = "0.10", features = ["stream"] }
hkdf = "0.12"
hmac = "0.12"
sha2 = "0.10"
rand = "0.8"
zeroize = { version = "1", features = ["derive"] }
base64 = "0.22"
rust-embed = { version = "8", features = ["mime-guess"] }
async-trait = "0.1"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
lettre = { version = "0.11", default-features = false, features = ["builder", "smtp-transport", "tokio1-rustls-tls", "hostname"] }
minijinja = "2"
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
infer = "0.16"
dashmap = "6"
moka = { version = "0.12", features = ["future"] }
proptest = "1"
insta = { version = "1", features = ["json", "yaml"] }
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
axum-test = "17"
serde_jcs = "0.1"
phonenumber = "0.3"
```
If any pinned major doesn't resolve, choose the nearest available major, record it in the Report, and **don't** change APIs that later tasks assume without noting it.

`[workspace.lints]`:
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_debug_implementations = "warn"
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "warn"
dbg_macro = "deny"
todo = "deny"
print_stdout = "warn"
```
`eq-billing/Cargo.toml` additionally sets `[lints.clippy] float_arithmetic = "deny"` (ARC-11). Since a crate can't both inherit and extend lints, put `#![deny(clippy::float_arithmetic)]` at the top of `eq-billing/src/lib.rs` instead.

Feature wiring in `eq-api/Cargo.toml`:
```toml
[features]
default = ["enrollment", "billing", "attendance", "rental", "notify", "reports", "feedback"]
enrollment = ["dep:eq-enrollment"]
billing = ["dep:eq-billing"]
attendance = ["dep:eq-attendance"]
rental = ["dep:eq-rental"]
notify = ["dep:eq-notify"]
reports = ["dep:eq-reports"]
feedback = ["dep:eq-feedback"]
```
`eq-server` re-exposes the same features, forwarding to `eq-api/<feature>`, with the same default.

## Business rules
None.

## Implementation notes
1. Optional crates' `Cargo.toml` depend on `eq-core`, `eq-audit`, `eq-identity` and `eq-scheduling` (path deps) even if unused yet, so the layering is fixed early. Suppress unused-crate warnings with `use eq_core as _;` in lib.rs if needed.
2. `eq-testkit` has `publish = false` and is used as a `[dev-dependencies]` of the other crates later.
3. `check-deps.sh` must be POSIX sh plus `jq`, with exit code 1 and a readable message on violation.

## Acceptance criteria
- AC1. Given a fresh clone, When `cargo build --workspace`, Then success and `target/debug/equalizer --version` prints `equalizer 0.1.0`.
- AC2. When `cargo build -p eq-server --no-default-features`, Then success (no optional crates compiled).
- AC3. When `scripts/check-deps.sh` runs, Then exit 0. If you temporarily add `eq-billing` as a dependency of `eq-rental`, Then exit 1 with a message naming both crates (verify manually and describe in the Report).

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/cli.rs` | `prints_version` (spawn the binary with `--version`, assert stdout) |

## Definition of done
- [ ] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` pass
- [ ] `cargo build -p eq-server --no-default-features` passes
- [ ] `scripts/check-deps.sh` passes
- [ ] Commit `chore(workspace): create cargo workspace and crates [T-001]`, signed off

## Report

## Questions
