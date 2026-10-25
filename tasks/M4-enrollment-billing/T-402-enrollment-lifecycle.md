---
id: T-402
title: Add enrollment create/activate/end with payer resolution, age and guardian rules, and the state machine
milestone: M4
status: draft
depends_on: [T-401]
spec_refs: [ENR-01, ENR-02, ENR-05, DOM-01]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Staff enroll a student in an offering with the right payer, and the system enforces age and guardian rules from the policy and walks the enrollment state machine.

## Context
- ENR-01 payer resolution, ENR-02 age check (Block | Warn), and the guardian-required rule (`guardians.required_under_age_years`), from `spec/modules/enrollment.md` §1.
- State machine `spec/06-domain-model.md` §5.2 (pending → active → on_leave → ended). Unlisted transitions → 409 `invalid_transition`.
- Activation snapshots `leave_terms` onto the enrollment (POL-05) and emits `enrollment.enrollment_activated.v1`. Ending emits `enrollment.enrollment_ended.v1` (payload structs already in `eq_core::events`, T-212).
- Series membership goes through the `SeriesMembership` port (T-209), never scheduling tables directly.
- API rows for enrollments from `spec/modules/enrollment.md` §2 (create, get, list scoped, activate, end).

## Scope
### In
- Pure `domain::{payer, age, transitions}`, services, endpoints, audit, events, `EnrollmentLookup` and `EnrollmentStatus` port implementations (replacing the defaults from T-210/T-212)

### Out
- Registration (T-403/T-404), leave (T-406), credits (T-407)

## Acceptance criteria
- AC1. 10-year-old without a guardian with `required_under_age_years=18` → 422 (module AC1) → `api_enrollments::guardian_required`
- AC2. min age 84 months and a 6-year-old → 422 with Block, warning with Warn (module AC2) → `age_block_and_warn`
- AC3. Activation adds the series membership through the port and the student appears in materialised sessions → `activation_adds_member`
- AC4. Ending removes future participations (via T-212 consumer) → `end_removes_future`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-enrollment/tests/api_enrollments.rs` | AC names |
| `crates/eq-enrollment/src/domain/*.rs` | pure unit tests + `proptest transitions_closed` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(enrollment): enrollment lifecycle [T-402]`, signed off

## Report

## Questions
