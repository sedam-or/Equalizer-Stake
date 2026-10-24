# 12 — Testing Strategy

## 1. Layers

| ID | Layer | Tooling | Scope | Gate |
|---|---|---|---|---|
| TST-01 | Domain unit | `cargo test`, `insta` | Pure `domain/` functions: state machines, credit tiers, proration, alternatives, check-in evaluation, summaries | ≥ 90% line coverage of `domain/` (cargo-llvm-cov) |
| TST-02 | Property | `proptest` | Money arithmetic; opening-hours fit; alternatives are always bookable; materialisation idempotency; tier lookup monotonic | Required for the listed functions |
| TST-03 | Integration (DB) | `testcontainers` Postgres 16, one container per test binary, one schema per test via template DB | Repos and services with real SQL, RLS, triggers, constraints | Every service use-case has ≥ 1 test |
| TST-04 | API | `axum-test` against the full router with a seeded tenant | Status codes, problem codes, permissions per role, module gating | Every endpoint × (allowed role, forbidden role) |
| TST-05 | **Concurrency proof** | tokio `JoinSet` of 50 tasks through the HTTP layer, against a real DB | 50 concurrent bookings of the same slot → exactly 1 × 201, 49 × 409, 1 resource hold. Also: 20 concurrent approvals into a series with capacity 1 → 1 success | Required, and runs in CI on every PR. Badge in README |
| TST-06 | Tenant isolation | Integration | Two tenants seeded. For every tenant table: query as tenant A returns 0 rows of B; INSERT with B's id under A's setting fails; unset setting returns 0 rows | Required; the table list comes from `pg_class` automatically, so new tables are covered |
| TST-07 | E2E | Playwright (Chromium + WebKit mobile viewport 390×844) | Fresh install → setup (3 steps, music preset) → add instructor → approve registration → series → invoice cycle (time travel) → guardian pays → admin verifies. Separate specs per module | The setup flow takes ≤ 10 user actions per step and completes in < 60 s in CI |
| TST-08 | Preset validation | `cargo test -p eq-core presets` | Every preset parses against `preset.schema.json`; every module patch merges and passes `validate()`; offering keys referenced by policy exist in the catalog | Required |
| TST-09 | I18n completeness | Rust test + `pnpm i18n:check` | Every policy schema path has `en`/`id` label keys; every error `code` has `errors.<code>`; every notify template exists for `en`/`id` × routed channels; no unused keys | Required |
| TST-10 | Contract drift | CI step | `ritma openapi` equals the committed `web/openapi.json`; `pnpm gen:api` produces no diff; `pnpm typecheck` passes | Required |
| TST-11 | Accessibility | Playwright + `@axe-core/playwright` | Every route in the route map | 0 serious or critical |
| TST-12 | Visual | Playwright screenshots of `/__ds` (light/dark, compact/comfortable) | Design-system regressions | Reviewed diff |
| TST-13 | Backup round trip | Integration (docker) | seed → backup → wipe → restore → row counts and file hashes equal | Required on release branches |
| TST-14 | No hard-coded business values | `scripts/check-literals.sh` | In `crates/*/src/{domain,service}*.rs`, grep for timezone names, `WIB`, currency codes, money-looking integer literals ≥ 1000 not in `const` with `// technical:`, `day() == <n>`, coordinate literals | Required (POL-02) |
| TST-15 | Time travel | `Clock` mock + `JobRunner::run_due(now)` | Month-end, overdue, reminders, materialisation, review SLA, retention jobs | Every cron job has a time-travel test |
| TST-16 | Log hygiene | `tracing-test` capture | No PII field names or values in logs during the e2e-like API suite | Required |

## 2. Fixtures

- **TST-20.** `eq-testkit` (dev-only crate):
  - `TestDb::new()` creates a fresh DB from a template with migrations applied.
  - `Seed::tenant(preset)` runs the real setup service.
  - Builders: `PersonBuilder`, `SessionBuilder`, …
  - `TestClock` (settable, advancable).
  - `ApiClient::as_role(role)`.
  - Fake channels `MockSmtp` and `MockWhatsApp` that record sends and can fail on demand.
  - All seeds are deterministic (fixed RNG seed), and `Clock` starts at `2026-01-05T00:00:00+07:00`, a Monday.

## 3. CI pipeline (`.github/workflows/ci.yml`)

1. `rust`: fmt check → clippy `-D warnings` → `cargo deny check` → `cargo test --workspace` (includes TST-01..06, 08, 09, 13–16) → coverage report.
2. `web`: `pnpm install --frozen-lockfile` → biome → typecheck → vitest → `i18n:check` → `size-limit` → build.
3. `contract`: TST-10.
4. `e2e`: build the binary with the embedded web → start Postgres service → Playwright (TST-07, 11, 12).
5. `scripts`: `check-deps.sh` (ARC-02), `check-literals.sh` (TST-14), `check-web-boundaries.sh` (CQ-64), `check-layers.sh` (CA-01).
6. `pr-title`: Conventional Commits title check (CQ-100/101).
7. `security`: `gitleaks` secret scan on every PR. A scheduled daily workflow runs `cargo audit` and `pnpm audit --prod` (SBD-74/75). The security test catalogue is in `18-security-by-design.md` §7.

The `rust` job also enforces the TST-01 coverage gate on `domain/` with `cargo llvm-cov`.

PRs must pass all jobs. `main` is protected.
