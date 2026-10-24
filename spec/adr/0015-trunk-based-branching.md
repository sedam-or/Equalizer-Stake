# ADR-0015: Trunk-based development with short-lived branches and squash merges

- Status: Proposed (2026-10-25), accepted on merge of the PR that adds `BRANCHING.md`

## Context
Ritma is built one task at a time (WORKFLOW.md, WIP = 1), mostly by agents and occasional contributors. Releases are cut by release-please from Conventional Commits on `main` (RELEASING.md), and SECURITY.md promises security fixes for the previous minor for 90 days. Branch naming was described in three files, with no rule for releases, hotfixes, backports or stacked PRs.

Options considered:
1. **GitFlow** (`develop`, `release/*`, `hotfix/*`, `main`). Built for scheduled releases of versioned products with long stabilisation. It doubles merge work, and release-please expects one trunk.
2. **GitHub Flow** (short branches off `main`, merge, deploy). Simple, but it says nothing about patching an older minor.
3. **Trunk-based with short-lived branches, squash merges, and maintenance branches only for supported old minors.**

## Decision
Option 3, specified in `BRANCHING.md`:
- `main` is the only long-lived branch and is always releasable. Every change arrives by PR with a green `ci-ok`.
- Branch names `<kind>/<ref>-<slug>` (`task/T-xxx-…` for task work). One task = one branch = one squashed commit.
- Rebase to stay current; squash-merge only; linear history.
- Releases are tags created by release-please on `main`. `release/vX.Y` exists only from v1.0 on, for 90 days, to backport fixes that landed on `main` first.
- Protection through a repository ruleset with the single required check `ci-ok`.

## Consequences
- ✅ The task model maps 1:1 to the history: each commit on `main` is one reviewed task, which makes `git bisect` and reverts precise.
- ✅ release-please needs no extra configuration for normal releases.
- ✅ Adding CI jobs never changes branch protection (only `ci-ok` is required).
- ❌ Squash merges lose WIP history. Accepted: the task Report and the PR hold the reasoning.
- ❌ Long-running work must be split into tasks of size ≤ M or hidden behind a module toggle. This is already required by WORKFLOW.md.
