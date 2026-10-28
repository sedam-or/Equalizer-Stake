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
- DDL: `spec/06-domain-model.md` §4.4 (registrations, enrollments, leaves, makeup_credits) plus `enrollment_changes` from §4.12, copied verbatim. Eligibility (ENR-09) is computed on read and has no table. All of it goes into `migrations/0020_enrollment.sql`, with RLS on every table.
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/src/policy.rs` | create or modify | Path named in this task's contract. |
| `migrations/0020_enrollment.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T4 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `all_presets_valid (re-run)` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Default policy and every preset's enrollment patch validate → `presets::all_presets_valid (re-run)`
- AC2. `leave.max_days_per_year` above its bound → pointer error → `enrollment::policy::tests::leave_bounds`
- AC3. Summary snapshot of the music preset policy → `enrollment::policy::tests::summary_snapshot`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/src/policy.rs` | `leave_bounds`, `summary_snapshot`, `schema_snapshot` |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): schema and policy [T-401]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
