---
id: T-401
title: Add the enrollment schema, EnrollmentPolicy and the EnrollmentModule wiring
milestone: M4
status: draft
depends_on: [T-315]
spec_refs: [ENR-05, POL-01, POL-06, MOD-03]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Enrollment exists as a real module: its tables, its typed, validated rules with summaries, its permissions and its navigation.

## Context
- DDL: `spec/06-domain-model.md` §4.4 (registrations, enrollments, enrollment_changes, leaves, makeup_credits, eligibility snapshots), copied verbatim into `migrations/0020_enrollment.sql`, with RLS on every table.
- Policy: `EnrollmentPolicy` exactly as `spec/04-policy-engine.md` §6.4 (registration, guardians, age, leave, makeup_credits, eligibility), with the bounds, defaults and x-ui groups listed there, externally tagged enums (POL-13).
- Permissions and nav from `spec/modules/enrollment.md` §3. Dependency: requires_all scheduling (MOD-03).
- Summary lines `policy.enrollment.summary.*` for registration mode, guardian age, leave limit and eligibility; i18n labels for every schema path (TST-09).
- Re-run the preset tests (T-301): the enrollment patches in every preset must now validate.

## Scope
### In
- Migration, `eq_enrollment::policy`, `EnrollmentModule` (manifest, policy wiring, permissions, nav, `has_data` = exists(enrollments))
- Policy i18n strings in `web/src/locales/{en,id}/policy.json`

### Out
- Services (T-402+)

## Acceptance criteria
- AC1. Default policy and every preset's enrollment patch validate → `presets::all_presets_valid (re-run)`
- AC2. `leave.max_days_per_year` above its bound → pointer error → `enrollment::policy::tests::leave_bounds`
- AC3. Summary snapshot of the music preset policy → `enrollment::policy::tests::summary_snapshot`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/src/policy.rs` | `leave_bounds`, `summary_snapshot`, `schema_snapshot` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): schema and policy [T-401]`, signed off

## Report

## Questions
