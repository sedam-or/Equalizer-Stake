# AGENTS.md — rules for coding agents (Codex & co.)

You are implementing **Ritma**, an open-source scheduling, billing and attendance engine for course schools.
The architecture is already decided. **Your job is to implement tasks exactly as written, not to design.**

## 0. The one rule

**Work only from a task file: `tasks/<milestone>/T-xxx-*.md`.** Each task is self-contained. It inlines the spec excerpts, interfaces, business rules and tests you need.
- Don't start work that has no task file.
- Don't implement beyond the task's **Scope → In**. Its **Scope → Out** is a hard boundary.
- If a task is ambiguous, contradicts the code or the spec, or needs a decision: **stop and block** following `WORKFLOW.md` §4. Never guess on business rules, money, security or schema.

## 1. How to pick and run a task

**Follow `WORKFLOW.md` exactly. It's the standard procedure for every task.** In short:

1. **Preflight:** clean tree, `scripts/tasks-index.sh --check`, and no other task `in_progress` (WIP = 1).
2. **Pick:** a `changes_requested` task first, otherwise the lowest-ID `todo` whose `depends_on` are all `done`.
3. **Claim:** branch `task/T-xxx-<slug>`, set `status: in_progress` in the frontmatter, run `scripts/tasks-index.sh`. Never hand-edit `INDEX.md`.
4. **Understand:** read the whole task, write a short Plan in `## Report`, or block (§4 of WORKFLOW).
5. **Implement** only Scope → In, with every listed test under its exact name.
6. **Verify:** every DoD check plus a self-review. Never tick a check you didn't run.
7. **Report:** use the WORKFLOW §3 Step 6 structure, set `status: review`, regenerate the index.
8. **Commit** once, signed off: `<type>(<scope>): <task title> [T-xxx]`.
9. **Stop.** One task per session. A human reviews and sets `done`.

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
| Guard scripts | `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh` |

## 4. Non-negotiable engineering rules

1. **No hard-coded business values (POL-02).** Fees, days of the month, hours, durations, radii, percentages, timezones, currencies and locales come from policy structs or tenant data. If you need a new one, the task says which policy field to add. If it doesn't, stop and ask. Technical constants need `const` plus a `// technical:` comment.
2. **Money:** `eq_core::money::Money` (i64 minor units). No floats in billing. Percentages are `Bps`.
3. **Time:** no `Utc::now()` in `domain/`. Use the injected `Clock`. Local-calendar logic uses the tenant timezone. Ranges are half-open.
4. **Every state change:** inside `core::db::begin` (RLS tenant set) → domain → repo → `audit::record` → `outbox::emit` → commit. If audit fails, everything rolls back.
5. **Modules don't import each other.** Optional crates depend only on `eq-core` and the always-on crates (`eq-identity`, `eq-audit`, `eq-scheduling`). Cross-module communication goes through events or ports.
6. **Security (full standard: `spec/18-security-by-design.md`):** deny by default; tenant only from context; classify every new field (SBD-10); follow the task's `## Security` section and write its abuse-case tests; never weaken a control to make something work. If a task hits a security trigger (SBD-30) without saying how it's protected, block and ask. Never log PII (emails, phones, names beyond ids, tokens). Encrypt PII with `core::crypto`. Use parameterised SQL only (`sqlx::query!`). Every handler checks a permission.
7. **API:** every handler has `#[utoipa::path]`, errors are `problem+json` with stable `code`s, mutations of versioned aggregates need `If-Match`, and money and booking creation accept `Idempotency-Key`.
8. **UI:** no raw hex or Tailwind palette colours, only semantic tokens. Every string goes through i18n (`en` + `id`). Terminology goes through `t('term.*')`. Every list has empty, loading and error states. Meet WCAG 2.2 AA.
9. **Tests are part of the task.** A task without its listed tests passing isn't done.
10. **Don't touch** `spec/`, `brand/`, `legacy/` or other tasks' files. Exceptions: your own task file, and `tasks/INDEX.md` via `scripts/tasks-index.sh`.

## 5. Style

**The full code-quality standard is `spec/16-code-quality.md` (CQ-\*), and the architecture rules are `spec/17-clean-architecture.md` (CA-\*): business decisions only in pure `domain/`, services as recipes, SQL only in `repo.rs` on the module's own tables, cross-module only via events/ports.** It applies to every task even when the task doesn't cite it, and its §12 Definition of done is added to every task's own list. Lint exceptions use `#[expect(lint, reason = "…")]` only, never `#[allow]`. The summary:

- Rust: edition 2024, `thiserror` for library errors, `anyhow` only in binaries and jobs, no `unwrap()`/`expect()` outside tests and startup, and modules laid out per `spec/02-architecture.md` §3. Doc comments on public items cite spec IDs (`/// SCH-03: …`).
- SQL: snake_case, explicit column lists, no `SELECT *` in app code.
- TS: strict, no `any`, named exports, components in PascalCase files, hooks `useX`.
- Commits: `COMMIT_CONVENTION.md` (Conventional Commits, scopes, `[T-xxx]` refs, sign-off).
- Errors: `spec/19-error-handling.md` (codes registered in `errors.rs`, translated en+id, one log at the edge).
