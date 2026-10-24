---
id: T-000
title: Imperative title that fits one PR
milestone: M0
status: todo            # todo | in_progress | blocked | review | done | draft
depends_on: []
spec_refs: []
estimated_size: S       # S (<150 LOC) | M (<400 LOC) | L = must be split before starting
---

## Goal
2–3 sentences: the outcome, observable by a user or a test.

## Context
The minimal excerpt of the spec, pasted inline, so that this file is enough to do the work.

## Scope
### In
- Exactly what to build.
### Out
- Explicit non-goals (handled by other tasks, named by id).

## Files
| Path | Action | Purpose |
|---|---|---|

## Interfaces
Exact Rust signatures / SQL DDL / endpoints (method, path, request, response, errors) / TS types.

## Business rules
1. Rule, with policy key and default. Never a literal in code.

## Security
<!-- Required when a trigger from spec/18 SBD-30 applies (T1–T8); otherwise write "No triggers." -->
- Triggers: T…
- Data classes touched: S…
- Threats (STRIDE, relevant letters only) → control (spec/SBD id):
- Abuse-case acceptance criteria (with test names):

## Implementation notes
Ordered steps, pitfalls, crates and versions.

## Acceptance criteria
- AC1. Given … When … Then …  → test `crate::module::tests::name`

## Tests to write
| File | Test names |
|---|---|

## Definition of done
- [ ] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` pass
- [ ] Web (if touched): `pnpm -C web lint typecheck test i18n:check` pass
- [ ] Guard scripts pass: `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed; decisions in `domain/`, own tables only, no new `#[allow]`
- [ ] OpenAPI and TS client regenerated if the API changed (`make api`); sqlx offline data updated
- [ ] Docs/comments cite spec IDs
- [ ] Commit `<type>(<scope>): <title> [T-000]`, signed off

## Report
_Filled in by the implementer: what changed, deviations and why, follow-ups._

## Questions
_Filled in by the implementer if blocked._
