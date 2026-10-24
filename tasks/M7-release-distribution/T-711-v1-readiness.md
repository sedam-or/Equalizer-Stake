---
id: T-711
title: v1.0 readiness — security review checklist, performance budget, README assets, and launch checklist
milestone: M7
status: draft
depends_on: [T-710]
spec_refs: [SEC-22, OPS-07, OPS-10, TST-05, PRD-10, PRD-11]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Ship v1.0 with evidence: every USP has its proof, every checklist item is green.

## Context
- `brand/GTM.md` §2–3 (README funnel order, badges including the concurrency proof), `brand/BRAND.md` §4 (USP proofs) and §8 (assets), `SECURITY.md` supported versions.
- OPS-10 upgrade path verified: replace binary/image, restart, migrations auto-apply (upgrade test from the previous tag).
- Performance: idle RSS ≤ 150 MB, p95 API latency targets from `spec/11-ops.md` OPS-07 under the 50-concurrent-user profile (k6 or oha script).

## Scope
### In
- `docs/release/v1-checklist.md` filled with links to evidence, load test script, README update

### Out
- Marketing site

## Acceptance criteria
- AC1. Load test meets OPS-07 targets → `perf::profile_50_users (CI artifact)`
- AC2. Every USP row links to a passing test or page → `manual review`

## Tests to write
| File | Test names |
|---|---|
| `scripts/loadtest/` | profile_50_users |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `chore(release): v1.0 readiness [T-711]`, signed off

## Report

## Questions
