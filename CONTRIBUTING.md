# Contributing

Thank you for helping small schools run better. 💙
There's a way to help at every skill level. Pick the one that suits you:

| You can… | Start here | Rust needed? |
|---|---|---|
| Translate the app | Weblate project (link in README) | No |
| Add a template for your kind of school | [Add a preset](#add-a-preset) | No (YAML) |
| Improve docs or screenshots | `docs/` | No |
| Fix a bug or build a small feature | Issues labelled `good first issue` (each links a task file in `tasks/`) | Yes |
| Build a new module | Write an RFC first ([GOVERNANCE.md](GOVERNANCE.md)) | Yes |

## Ground rules

- Be kind. Read the [Code of Conduct](CODE_OF_CONDUCT.md).
- **Security issues:** never open a public issue. See [SECURITY.md](SECURITY.md).
- **Specs come first.** Behaviour is defined in `spec/`, and work is broken into `tasks/**/T-*.md`. If code and spec disagree, the spec wins. To change the spec, open a PR to `spec/` (and an RFC for big changes).
- **Code quality:** follow [`spec/16-code-quality.md`](spec/16-code-quality.md): lints, layering, naming, tests, PR size (≤ ~400 lines) and the review checklist. CI enforces most of it.
- **Never hard-code a business value** (a fee, a day of the month, opening hours, a radius, a timezone, a currency). Add a policy field instead (`spec/04-policy-engine.md`). CI checks this.

## Development setup

Requirements: Rust stable (see `rust-toolchain.toml`), Node 22 + pnpm 9, Docker (for Postgres and tests).

```bash
git clone https://github.com/ritmahq/ritma.git && cd ritma
make dev-db        # starts postgres:16 in docker on :5433
make dev           # runs the server (:8080) and the Vite dev server (:5173) with hot reload
make test          # cargo test + pnpm test (uses testcontainers)
make e2e           # Playwright against a release build
```

## Workflow

1. Find or open an issue. For anything non-trivial, comment first so we can agree on the approach.
2. Branch from `main`: `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>`.
3. Make your change with tests. Follow the task file's Definition of Done if there is one.
4. Commit using [Conventional Commits](https://www.conventionalcommits.org), and reference the task or spec IDs, e.g.
   `feat(billing): apply late fee from snapshotted terms [T-412, BIL-06]`
5. Sign off every commit (DCO): `git commit -s`. Sign the CLA once when the bot asks (see "License" below).
6. Open a PR using the template. CI must be green. A maintainer reviews within 5 days.

## Add a preset

A preset is a YAML file in `presets/` that sets up a type of school: which features are on, default rules, terminology and a sample catalog.

1. Copy `presets/blank.yaml` to `presets/<your_id>.yaml`.
2. Fill in `name`, `description`, `modules`, `terminology`, and the `policies` you want to change (only list what differs from the defaults; see `spec/04-policy-engine.md` §6 for every field).
3. Add catalog entries (levels, offerings, sample prices per currency).
4. Run `cargo test -p eq-core presets`. It validates your file.
5. Open a PR with a short description of the kind of school it is for.

## Add a module

Follow the checklist in `spec/03-module-system.md` §7. An RFC is required first.

## Add a translation

UI strings live in `web/src/locales/<lang>/*.json`, and notification templates live in `crates/eq-notify/templates/<lang>/`. Translations are managed in Weblate, so please don't edit non-English JSON directly in PRs unless you're adding a new language.

## License

By contributing you agree that your contributions are licensed under **AGPL-3.0-only**, and you sign our **Contributor License Agreement** (CLA Assistant bot on your first PR). The CLA lets the project offer a commercial license to organisations that can't use AGPL, which funds development. You keep your copyright. See `spec/adr/0009-license-agpl-cla.md` for the reasoning.
