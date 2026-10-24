---
id: T-001
title: Create the Cargo workspace, toolchain, lints and empty crates
milestone: M0
status: review
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
- [x] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` pass
- [x] `cargo build -p eq-server --no-default-features` passes
- [x] `scripts/check-deps.sh` passes
- [x] Commit `chore(workspace): create cargo workspace and crates [T-001]`, signed off

## Report
**Plan** —
1. Add the root Cargo workspace, pinned stable toolchain and formatting/lint/editor configuration → verify with Cargo metadata and the required format check.
2. Add the 14 crate manifests with only the dependency edges and feature wiring specified by T-001 → verify with `cargo metadata` and `scripts/check-deps.sh`.
3. Add crate docs, the `ritma --version` CLI and `prints_version` integration test → verify with the test and binary output.
4. Build the full workspace and the server without optional features → verify both acceptance commands.
5. Exercise the dependency guard against a deliberately invalid optional-crate edge, then restore the valid manifest → verify the guard fails with both crate names and passes after restoration.

**Assumptions** — Use the repository URL reported by the checked-out `origin` remote. Pin the locally installed stable compiler version if it satisfies the task's minimum; the current compiler is 1.99.1. No business rules or schema assumptions.

**Test plan** —
- AC1: Given the workspace, when `cargo build --workspace` runs and `ritma --version` is invoked, then the build succeeds and prints `ritma 0.1.0` → `crates/eq-server/tests/cli.rs::prints_version` plus the build command.
- AC2: Given optional features, when `cargo build -p eq-server --no-default-features` runs, then it succeeds without compiling optional module crates → the exact build command.
- AC3: Given valid dependency edges, when `scripts/check-deps.sh` runs, then it exits 0; when an optional crate depends on another optional crate, then it exits 1 and names both crates → positive and negative manual guard checks.
- Edge/error cases: validate both forbidden graph shapes (optional-to-optional and eq-core-to-eq-*); no business input or HTTP error cases exist in this scaffold task.
- Abuse cases: n/a; T-001 declares `subtle`/`secrecy` dependencies but creates or processes no tokens, secrets, PII or request input.

**Step 0 — Start** — Clean start commit `1c1cbbf790d13943003cbef640bb2e679d61326c`; all 56 M0–M3 tasks are `todo`; this task has no dependencies. Branch and index checks are recorded in `RUN_LOG.md`.
**Step 1 — Task/spec** — Read the full task and the sections listed below. No policy values are introduced.
**Step 2 — Dependencies** — n/a; `depends_on` is empty.
**Step 3 — Design reference** — n/a; no UI files.

**Step 5 — Implement** — Added the shared workspace/toolchain configuration, 14 layered crates, optional feature forwarding, a Clap version command and the dependency-edge guard.

**Summary** — The Rust workspace now builds all 14 crates and exposes the `ritma` binary. The CLI reports the workspace version, optional modules can be disabled from `eq-server`, and the guard rejects forbidden dependency edges.

**Spec read** — `spec/02-architecture.md` §1–2; `spec/adr/0001-rust-modular-monolith.md`; `spec/16-code-quality.md` CQ-14/20/21, §9 and §12; `spec/17-clean-architecture.md` §1 and §4; `spec/18-security-by-design.md` §5.3; `spec/19-error-handling.md` §10.

**Files** —
- `.cargo/config.toml`
- `.editorconfig`
- `Cargo.toml`
- `Cargo.lock` (generated)
- `clippy.toml`
- `rust-toolchain.toml`
- `rustfmt.toml`
- `scripts/check-deps.sh`
- `crates/eq-core/Cargo.toml`
- `crates/eq-core/src/lib.rs`
- `crates/eq-audit/Cargo.toml`
- `crates/eq-audit/src/lib.rs`
- `crates/eq-identity/Cargo.toml`
- `crates/eq-identity/src/lib.rs`
- `crates/eq-scheduling/Cargo.toml`
- `crates/eq-scheduling/src/lib.rs`
- `crates/eq-enrollment/Cargo.toml`
- `crates/eq-enrollment/src/lib.rs`
- `crates/eq-billing/Cargo.toml`
- `crates/eq-billing/src/lib.rs`
- `crates/eq-attendance/Cargo.toml`
- `crates/eq-attendance/src/lib.rs`
- `crates/eq-rental/Cargo.toml`
- `crates/eq-rental/src/lib.rs`
- `crates/eq-notify/Cargo.toml`
- `crates/eq-notify/src/lib.rs`
- `crates/eq-reports/Cargo.toml`
- `crates/eq-reports/src/lib.rs`
- `crates/eq-feedback/Cargo.toml`
- `crates/eq-feedback/src/lib.rs`
- `crates/eq-api/Cargo.toml`
- `crates/eq-api/src/lib.rs`
- `crates/eq-server/Cargo.toml`
- `crates/eq-server/src/main.rs`
- `crates/eq-server/tests/cli.rs`
- `crates/eq-testkit/Cargo.toml`
- `crates/eq-testkit/src/lib.rs`
- `tasks/M0-foundation/T-001-cargo-workspace.md`
- `tasks/INDEX.md` (generated by `scripts/tasks-index.sh`)

**Acceptance criteria → tests** —
| AC | Test | Result |
|---|---|---|
| AC1 | `prints_version`; `cargo build --workspace`; `ritma --version` | ✅ Build and exact version output passed. The test failed against an empty main (expected red), then passed after version handling was added. |
| AC2 | `cargo build -p eq-server --no-default-features`; `cargo tree` feature check | ✅ Build passed; tree contained no optional crates with defaults disabled and all seven with defaults enabled. |
| AC3 | `scripts/check-deps.sh`; temporary optional-to-optional and core-to-eq dependency probes | ✅ Valid graph passed; each invalid edge failed and named both crates. Manifests were restored. |

**Step 8 — Self-review** — Reviewed the staged change against `spec/16-code-quality.md` §9, `spec/17-clean-architecture.md` and `spec/19-error-handling.md`. The 14 package edges match ARC-02; no business logic or HTTP handlers were introduced. The no-default feature tree excludes all optional crates.

**Step 11 — Revise and full verification / Checks run** —
- ✅ `cargo fmt --all --check`
- ✅ `cargo clippy --workspace --all-targets -- -D warnings`
- ✅ `cargo test --workspace`
- ✅ `cargo build --workspace`
- ✅ `cargo build -p eq-server --no-default-features`
- ✅ `scripts/check-deps.sh` (valid graph and both negative probes)
- ✅ `cargo metadata --format-version 1 --no-deps` (14 workspace packages)
- ✅ Cargo metadata assertions for all 14 dependency edges, default features, server feature forwarding and `eq-testkit` `publish = false`
- ✅ Temporary dependency-resolution manifest: `cargo generate-lockfile` resolved every external workspace dependency and feature set.
- ✅ `sh -n scripts/check-deps.sh`, `git diff --check`, `scripts/tasks-index.sh --check`, required-file and hygiene scan
- n/a: web checks; no web files are in this task.
- n/a: `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`; these scripts are introduced by T-006 and WORKFLOW.md §3 Step 11 allows skipping scripts that do not exist yet.
- n/a: `make api` and `cargo sqlx prepare --workspace`; no API or SQL query changed.

**Step 6 — Manual checks** — `target/debug/ritma --version` and `cargo run -q -p eq-server --no-default-features -- --version` both printed `ritma 0.1.0`. `make dev` is not applicable: this task has only CLI behavior, and the Makefile/server runtime arrive in later tasks.

**Debug log** — One ad hoc feature-tree assertion initially parsed the tree's branch glyphs as package names. The verifier was corrected to match package-version lines; it then confirmed both default and no-default feature graphs. No implementation failure remained.

**Step 9 — Optimize / Performance** — n/a; no runtime path was added.

**Step 10 — Security** — No SBD-30 trigger or user data handling. `subtle` and `secrecy` are declared for the future token/secret seams in SBD-49/50; this task adds no token or secret processing. No secret literals were introduced. `gitleaks` is not installed, so that optional local scan was unavailable.

**Deviations** — Added the generated `Cargo.lock` to record resolved versions under `spec/24-open-source-stack.md` OSS-04; the task's Files table did not list it explicitly. No other deviations.

**Dependencies affected** — None beyond the dependency graph specified by T-001.

**Rollback plan** — Revert the signed T-001 commit; there are no database or external-service changes.

**Follow-ups** — PostgreSQL 16 availability will be checked when T-006 adds the dev database configuration. No T-001 follow-up is required.

## Questions
- **Resolved (maintainer, 2026-10-25):** Q1 — yes. Stable rustfmt only. The nightly-only options were removed from Scope → In and replaced by CQ-20's stable options. Import grouping is now a review rule (spec/16 CQ-20). The lint set was extended per CQ-21. Restart from Scope → In.
