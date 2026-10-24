# Commit convention

Ritma uses **[Conventional Commits 1.0](https://www.conventionalcommits.org/en/v1.0.0/)**, with a few project rules. Commit messages feed the changelog and the version number (release-please, `RELEASING.md`), so their format is a tool contract, not a matter of taste.

This file is the full standard. `spec/16-code-quality.md` CQ-100/101 points here, and `WORKFLOW.md` uses it for every task.

## 1. Format

```
<type>(<scope>)<!>: <subject> [<refs>]

<body>

<footers>
```

- **Header:** at most **72 characters** in total. Required.
- **Body:** explains *why* and any trade-offs, wrapped at 72 characters. Required when the why isn't obvious from the subject.
- **Footers:** `Token: value` lines. `Signed-off-by` is required on every commit (DCO, `git commit -s`).

## 2. Types

| Type | Use for | Release effect | Changelog section |
|---|---|---|---|
| `feat` | A user- or API-visible capability | **minor** | Features |
| `fix` | A bug fix (behaviour now matches the spec) | **patch** | Bug fixes |
| `perf` | A faster or lighter implementation, no behaviour change | patch | Performance |
| `refactor` | A code change with no behaviour change (CQ-04: never mixed with feat/fix) | none | hidden |
| `test` | Adding or fixing tests only | none | hidden |
| `docs` | Docs, spec, tasks, brand, comments only | none | Documentation (user docs only) |
| `build` | Build system, Cargo/pnpm config, dependencies (`build(deps): …`) | none (patch if a vulnerable dependency is fixed, with `fix(deps)`) | hidden |
| `ci` | CI workflows and scripts used only by CI | none | hidden |
| `chore` | Repo housekeeping that fits nothing above (task status, tooling configs) | none | hidden |
| `revert` | Reverting a previous commit (§6) | depends on what's reverted | Reverts |

`style` isn't used: formatting is automatic (rustfmt, Biome), and formatting-only changes don't get their own commits.

## 3. Scopes

The scope is a **single** lowercase token from this list. Omit it only for repo-wide changes.

| Scope | Covers |
|---|---|
| `core`, `identity`, `audit`, `scheduling`, `enrollment`, `billing`, `attendance`, `rental`, `notify`, `reports`, `feedback` | That module's crate (`crates/eq-<scope>`) and its migrations |
| `api`, `server` | `eq-api` (router, middleware, OpenAPI), `eq-server` (CLI, config, main) |
| `web` | The SPA, when not specific to one module (module UI uses the module scope) |
| `ds` | Design system: tokens and `components/ui` |
| `i18n` | Translations |
| `presets` | `presets/*.yaml` |
| `desktop`, `deploy`, `cloud`, `platform` | `apps/desktop`, `deploy/`, cloud and platform code |
| `workspace`, `deps` | Cargo/pnpm workspace setup; dependency bumps |
| `spec`, `tasks`, `brand`, `adr` | Documentation areas |
| `security` | Cross-cutting security hardening that isn't owned by one module |

A change that touches several modules is either split, or scoped to the module that owns the behaviour. Use a broader scope (`api`, `web`) only when no module owns it.

## 4. Subject rules

- **Imperative mood, lowercase start (except proper nouns and ids such as `M1`, `OpenAPI`, `Tauri`), no trailing period:** `add late-fee job`, not `Added late fee job.` or `Adds…`.
- Say **what changes for the user or the system**, not what you did to files: `reject bookings outside opening hours`, not `update service.rs`.
- **Refs in brackets at the end:** the task id is required for task work, and spec IDs are optional. Examples: `[T-412]`, `[T-412, BIL-06]`. Work with no task (a hotfix, docs) may reference an issue: `[#123]`.
- No emoji, no ticket prefixes, and no PII (names, emails, phone numbers) anywhere in the message.

## 5. Breaking changes

A breaking change is any of these:
- removing or renaming an API field, endpoint or error code (ERR-23, API-13);
- a policy schema change without an automatic migration;
- a change to an env var's name or meaning;
- a DB change that requires manual steps.

It's marked **both** ways:
- with `!` after the scope: `feat(api)!: …`;
- with a `BREAKING CHANGE: <what breaks and how to migrate>` footer.

Before v1.0, release-please treats a breaking change as minor. After v1.0, it's a major bump and needs maintainer approval (GOVERNANCE.md).

## 6. Footers

| Footer | When |
|---|---|
| `Signed-off-by: Name <email>` | Always (`git commit -s`, DCO) |
| `BREAKING CHANGE: …` | §5 |
| `Refs: T-xxx, SCH-03` | When the refs don't fit in the header |
| `Fixes: #123` / `Closes: #123` | To close a GitHub issue on merge |
| `Co-authored-by: Name <email>` | Pair work, or an AI assistant that co-wrote the change |
| `Reverts: <sha>` | With `revert` commits |

Revert format: `revert: feat(billing): apply late fee from snapshot [T-412]`, with the body explaining why and the footer `Reverts: <sha>`.

## 7. Examples

```
feat(billing): apply late fee from snapshotted terms [T-412, BIL-06]

Late fees now use the terms captured on the invoice at issue time, so a
policy change in April doesn't alter March invoices (POL-05).

Signed-off-by: …
```

```
fix(scheduling): map exclusion violation to slot_taken with alternatives [T-203]
```

```
feat(api)!: rename problem code module_conflict to module_dependency [T-105]

BREAKING CHANGE: clients matching on `module_conflict` must use
`module_dependency` (422). The old code is no longer emitted.
Signed-off-by: …
```

```
docs(tasks): block T-001 with questions [T-001]
build(deps): bump axum from 0.8.1 to 0.8.3
ci: add gitleaks secret scan [T-006]
```

## 8. Task workflow mapping

| Situation (`WORKFLOW.md`) | Message |
|---|---|
| Task finished | The message given in the task's DoD, otherwise `<type>(<scope>): <task title> [T-xxx]` |
| Task blocked | `docs(tasks): block T-xxx with questions [T-xxx]` |
| Reviewer feedback | `docs(tasks): review feedback [T-xxx]` |
| Architect answers a block | `docs(tasks): resolve T-xxx questions [T-xxx]` |
| Promoting drafts | `docs(tasks): promote M<n> drafts to todo` |
| Local work-in-progress commits | Anything. They're squashed into the task commit before review |

## 9. Merge strategy and enforcement

- PRs are **squash-merged** (branch names, rebasing, stacking and protection: [`BRANCHING.md`](BRANCHING.md)). The **PR title becomes the commit header**, so it must follow this convention. The PR description's "What & why" becomes the body.
- `main` has one commit per task, linear history, and no merge commits.
- **[tool]** The CI `pr-title` job checks the PR title against:
  `^(feat|fix|perf|refactor|test|docs|build|ci|chore|revert)(\([a-z0-9-]+\))?!?: [A-Za-z0-9].{0,70}$` (the 72-character header limit is checked separately) (T-006). The DCO app checks the sign-off.
- **[review]** Type correctness (no `feat` for a refactor), scope from §3, imperative subject, refs present for task work.
- Optional local hook: `.githooks/commit-msg` runs the same regex. Enable it with `git config core.hooksPath .githooks`.
