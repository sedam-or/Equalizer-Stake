---
id: T-001
title: Create the Cargo workspace, toolchain, lints and empty crates
milestone: M0
status: todo
depends_on: []
spec_refs: [ARC-01, ARC-02, ADR-0001, CQ-14, CQ-20, CQ-21, SBD-49, SBD-50]
estimated_size: S
---

## Goal
`cargo build --workspace` succeeds on a fresh clone. The workspace has every crate from the architecture with the correct dependency edges, shared lints and pinned dependency versions, so later tasks only add code.

## Context
- ARC-01: one binary `ritma` (crate `eq-server`) that serves the API and the embedded SPA and runs jobs.
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
- `rustfmt.toml` with **stable options only** (CQ-20): `max_width = 110`, `use_field_init_shorthand = true`, `use_try_shorthand = true`. (Import grouping is a review rule, not a rustfmt setting.)
- `clippy.toml` exactly as in CQ-21 (below), and `.editorconfig` per CQ-14.
- 14 crates, each with `Cargo.toml` (inheriting workspace package and lints) and `src/lib.rs` (or `src/main.rs` for eq-server), containing a crate doc comment with its purpose and a `pub fn crate_name() -> &'static str` placeholder **only if needed for compilation**. Prefer empty libs.
- `eq-server` produces `[[bin]] name = "ritma"` with a `main` that prints the version.
- `scripts/check-deps.sh`: parses `cargo metadata --format-version 1 --no-deps` with `jq` and fails if any optional crate depends on another optional crate, or if `eq-core` depends on any `eq-*`.

### Out
- Any real logic (T-002+). CI workflow (T-006).

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.toml` | create | workspace |
| `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.editorconfig`, `.cargo/config.toml` | create | toolchain; `[alias] xtask`-free; set `SQLX_OFFLINE=true` in `[env]` |
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
subtle = "2"
secrecy = "0.10"
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

`[workspace.lints]` (CQ-21; CI runs clippy with `-D warnings`):
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_debug_implementations = "warn"
unused_qualifications = "warn"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "warn"
panic = "warn"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
print_stdout = "warn"
print_stderr = "warn"
cast_possible_truncation = "warn"
cast_possible_wrap = "warn"
cast_sign_loss = "warn"
cast_lossless = "warn"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
too_many_lines = "warn"
semicolon_if_nothing_returned = "warn"
redundant_else = "warn"
manual_let_else = "warn"
explicit_iter_loop = "warn"
```
`clippy.toml`:
```toml
too-many-lines-threshold = 80
too-many-arguments-threshold = 7
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-print-in-tests = true
disallowed-methods = [
  { path = "chrono::Utc::now", reason = "use the injected eq_core::clock::Clock (ARC-10)" },
  { path = "chrono::Local::now", reason = "use Clock + the tenant timezone" },
  { path = "std::time::SystemTime::now", reason = "use Clock" },
  { path = "std::env::var", reason = "read config only through eq_core::config (OPS-01)" },
  { path = "std::thread::sleep", reason = "blocks the runtime; use tokio::time::sleep" },
]
```
Exceptions use `#[expect(lint, reason = "…")]` only; `#[allow]` is forbidden (CQ-110). The `main` that prints the version needs `#[expect(clippy::print_stdout, reason = "CLI output")]`. If a dependency's derive or proc macro trips `allow_attributes`, stop and ask (don't weaken the lint).
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
- AC1. Given a fresh clone, When `cargo build --workspace`, Then success and `target/debug/ritma --version` prints `ritma 0.1.0`.
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
- Analysis: T-001 is the lowest-ID `todo` task with no dependencies. The repository has no v2 Cargo workspace or application crates yet; this task establishes that foundation before later backend and web tasks.
- Blocker: Scope → In requires stable Rust and only stable rustfmt options, but also explicitly requires `imports_granularity = "Crate"` and `group_imports = "StdExternalCrate"`. Both options require nightly rustfmt.
- Verification: installed `rustc 1.99.1` and `rustfmt 1.10.0`; a temporary `/tmp` fixture with the requested configuration produced warnings that both settings are unavailable on stable. Although `rustfmt --check` exited 0, it ignored those settings. Official documentation confirms both are unstable: https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#imports_granularity and https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#group_imports.
- Files changed: this task's status, Report and Questions; the T-001 status in `tasks/INDEX.md`.
- Implementation and Definition of done checks have not started. Per `AGENTS.md` §0, implementation stops until the conflicting requirements are resolved.
- Recommended resolution: retain pinned stable Rust and `max_width = 110`; omit the two nightly-only settings. No scope or requirement changes have been applied.

## Questions
- **Resolved (maintainer, 2026-10-25):** Q1 — yes. Stable rustfmt only. The nightly-only options were removed from Scope → In and replaced by CQ-20's stable options. Import grouping is now a review rule (spec/16 CQ-20). The lint set was extended per CQ-21. Restart from Scope → In.
- Q1 (blocking): May T-001 omit `imports_granularity` and `group_imports` from `rustfmt.toml` to satisfy its stable-only requirement? Recommended: yes. Retaining both settings would require an explicit exception for nightly formatting and a revised toolchain/check requirement.
