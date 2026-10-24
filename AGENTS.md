# AGENTS.md — rules for coding agents (Codex & co.)

You are implementing **Ritma**, an open-source scheduling, billing and attendance engine for course schools.
The architecture is already decided. **Your job is to implement tasks exactly as written, not to design.**

## 0. The one rule

**Work only from a task file: `tasks/<milestone>/T-xxx-*.md`.** Each task is self-contained. It inlines the spec excerpts, interfaces, business rules and tests you need.
- Don't start work that has no task file.
- Don't implement beyond the task's **Scope → In**. Its **Scope → Out** is a hard boundary.
- If a task is ambiguous, contradicts the code or the spec, or needs a decision: **stop**. Write your question under `## Questions` in the task file, set `status: blocked`, update `tasks/INDEX.md`, and end your turn. Never guess on business rules, money, security or schema.

## 1. How to pick and run a task

1. Open `tasks/INDEX.md`. Pick the **lowest-ID** task with `status: todo` whose `depends_on` tasks are **all `done`**.
2. In the task file's frontmatter, set `status: in_progress`, and update the row in `tasks/INDEX.md`.
3. Read the whole task file. Read only the `spec/` sections it references, and only if the inlined context isn't enough.
4. Implement. Write the tests listed in **Tests to write**, with the exact names given.
5. Run every check in **Definition of done** and fix until they're all green.
6. Fill in `## Report` (what changed, files, deviations and why, follow-ups). Set `status: review` in the frontmatter and in INDEX.md.
7. Commit (one task = one commit, or one PR if large): `<type>(<scope>): <task title> [T-xxx]`. Add `Signed-off-by` (`git commit -s`).
8. Stop. A human reviews and sets `done`.

## 2. Repository map

```
crates/eq-*        Rust workspace (eq-core, eq-identity, eq-audit, eq-scheduling, eq-enrollment, eq-billing,
                   eq-attendance, eq-rental, eq-notify, eq-reports, eq-feedback, eq-api, eq-server, eq-testkit)
migrations/        ONE ordered sqlx migration set: NNNN_<module>_<desc>.sql (forward-only)
presets/           *.yaml school templates (data)
web/               React + Vite SPA (pnpm)
apps/desktop/      Tauri shell (M8)
deploy/            docker, compose, install.sh, 1-click templates
spec/              the source of truth (read-only for you)
tasks/             your work queue
brand/             branding & go-to-market (not code)
legacy/            v0 archive as a ZIP. DO NOT open, unzip or read it.
```

## 3. Commands

| Purpose | Command |
|---|---|
| Dev DB | `make dev-db` (postgres:16 on :5433) |
| Run everything (dev) | `make dev` |
| Rust checks | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` |
| sqlx offline data | `cargo sqlx prepare --workspace` (after changing any query) |
| OpenAPI + TS client | `make api` (= `cargo run -p eq-server -- openapi > web/openapi.json && pnpm -C web gen:api`) |
| Web checks | `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check` |
| E2E | `make e2e` |
| Guard scripts | `scripts/check-deps.sh && scripts/check-literals.sh` |

## 4. Non-negotiable engineering rules

1. **No hard-coded business values (POL-02).** Fees, days of the month, hours, durations, radii, percentages, timezones, currencies and locales come from policy structs or tenant data. If you need a new one, the task says which policy field to add. If it doesn't, stop and ask. Technical constants need `const` plus a `// technical:` comment.
2. **Money:** `eq_core::money::Money` (i64 minor units). No floats in billing. Percentages are `Bps`.
3. **Time:** no `Utc::now()` in `domain/`. Use the injected `Clock`. Local-calendar logic uses the tenant timezone. Ranges are half-open.
4. **Every state change:** inside `core::db::begin` (RLS tenant set) → domain → repo → `audit::record` → `outbox::emit` → commit. If audit fails, everything rolls back.
5. **Modules don't import each other.** Optional crates depend only on `eq-core` and the always-on crates (`eq-identity`, `eq-audit`, `eq-scheduling`). Cross-module communication goes through events or ports.
6. **Security:** never log PII (emails, phones, names beyond ids, tokens). Encrypt PII with `core::crypto`. Use parameterised SQL only (`sqlx::query!`). Every handler checks a permission.
7. **API:** every handler has `#[utoipa::path]`, errors are `problem+json` with stable `code`s, mutations of versioned aggregates need `If-Match`, and money and booking creation accept `Idempotency-Key`.
8. **UI:** no raw hex or Tailwind palette colours, only semantic tokens. Every string goes through i18n (`en` + `id`). Terminology goes through `t('term.*')`. Every list has empty, loading and error states. Meet WCAG 2.2 AA.
9. **Tests are part of the task.** A task without its listed tests passing isn't done.
10. **Don't touch** `spec/`, `brand/`, `legacy/` or other tasks' files, except the INDEX row and your own task file.

## 5. Style

- Rust: edition 2024, `thiserror` for library errors, `anyhow` only in binaries and jobs, no `unwrap()`/`expect()` outside tests and startup, and modules laid out per `spec/02-architecture.md` §3. Doc comments on public items cite spec IDs (`/// SCH-03: …`).
- SQL: snake_case, explicit column lists, no `SELECT *` in app code.
- TS: strict, no `any`, named exports, components in PascalCase files, hooks `useX`.
- Commits: Conventional Commits, imperative mood, task id in brackets.
