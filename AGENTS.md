# AGENTS.md — rules for coding agents (Codex & co.)

You are implementing **Ritma**, an open-source scheduling, billing and attendance engine for course schools.
The architecture is already decided. **Your job is to implement tasks exactly as written, not to design.**

## 0. The one rule

**Work only from a task file: `tasks/<milestone>/T-xxx-*.md`.** Each task is self-contained. It inlines the spec excerpts, interfaces, business rules and tests you need.
- Don't start work that has no task file.
- Don't implement beyond the task's **Scope → In**. Its **Scope → Out** is a hard boundary.
- If a task is ambiguous, contradicts the code or the spec, or needs a decision: **stop and block** following `WORKFLOW.md` §4. Never guess on business rules, money, security or schema.

**Behaviour guidelines are mandatory: [`CLAUDE.md`](CLAUDE.md)** (the Karpathy guidelines, kept verbatim from upstream). They apply to every change in this repository, by any agent or human: code, docs, tasks, CI and brand files.
1. **Think before coding.** State assumptions in the task Report (Step 4). If two readings exist, block instead of picking one.
2. **Simplicity first.** The minimum that satisfies Scope → In and the tests. No speculative options, abstractions or error handling for impossible cases.
3. **Surgical changes.** Every changed line traces to the task or issue. Don't reformat, rename or "improve" neighbouring code; list it under Follow-ups.
4. **Goal-driven execution.** Turn each AC into a failing test first, then make it pass; loop on the Definition of done until it is green.

Precedence: the task file decides *what* to build, this file and `spec/` decide the *rules*, and `CLAUDE.md` decides *how you work*. Where they seem to conflict, the stricter reading wins, and you note it in the Report.

## 1. How to pick and run a task

**Follow `WORKFLOW.md` exactly: Step 0 plus 12 steps, none skipped (write "n/a" where a step doesn't apply).** In short:

0. **Start:** clean tree, `scripts/tasks-index.sh --check`, WIP = 1. Pick a `changes_requested` task first, otherwise the lowest-ID `todo` whose `depends_on` are all `done`. Branch `task/T-xxx-<slug>`, set `status: in_progress`, run `scripts/tasks-index.sh` (never hand-edit `INDEX.md`).
1. **Read** the whole task and the spec IDs it cites. Note the business rules (policy keys) and NFR targets.
2. **Read the dependencies'** Reports (Deviations, Follow-ups) and the code they shipped.
3. **Read the design reference** for UI tasks (spec 09, states, i18n, WCAG 2.2 AA).
4. **Plan and test plan** in `## Report` before coding (AC → tests, edge, error and abuse cases), or block (WORKFLOW §4).
5. **Implement** only Scope → In, with every listed test under its exact name.
6. **Run it locally** (`make dev`): happy and error paths.
7. **Debug loop:** at most 3 attempts per failure, then block, with a Debug log.
8. **Self code review** against spec 16 §9, 17 and 19.
9. **Optimize** only what you measured.
10. **Security audit:** SBD-81 if the task has triggers.
11. **Revise and rerun every check.** Never tick a check you didn't run.
12. **Report** (WORKFLOW §3 Step 12 structure), set `status: review`, regenerate the index, make **one** signed commit per `COMMIT_CONVENTION.md`, and **stop**. One task per session. A human reviews and sets `done`.

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
- States, flows, validation, contracts: transitions follow `spec/20-state-machines.md` (STM-\*), multi-step behaviour follows `spec/21-data-flows.md` (FLW-\*), every input goes through a field kind and the pipeline in `spec/22-validation-and-sanitization.md` (VAL-\*), and every wire value uses its canonical form from `spec/23-contracts-and-canonical-forms.md` (CAN-\*).
- Dependencies: adopt the open-source project listed in `spec/24-open-source-stack.md` (OSS-\*) instead of writing your own. A dependency that isn't on that map needs a task that says so; otherwise block and ask.
- Auth, roles and caches: tokens, cookies and the RBAC matrix follow `spec/25-auth-and-access-control.md` (AUTH-\*); the container image and every cache follow `spec/26-containers-and-caching.md` (CTR-\*, CACHE-\*).
- UI feedback: only toasts, dialogs and banners listed in `spec/27-ux-states-and-feedback.md` (UX-\*) exist; a new one is added to that catalogue in the same PR. Journeys (`spec/28-user-journeys.md`, JRN-\*) are the scripts for the e2e tests.
- Branches and PRs: `BRANCHING.md` (trunk-based, `task/T-xxx-<slug>`, squash merge, `ci-ok`).
- Errors: `spec/19-error-handling.md` (codes registered in `errors.rs`, translated en+id, one log at the edge).
