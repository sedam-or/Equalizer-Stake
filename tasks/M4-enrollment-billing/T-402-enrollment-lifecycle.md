---
id: T-402
title: Add enrollment create/activate/end with payer resolution, age and guardian rules, and the state machine
milestone: M4
status: todo
depends_on: [T-209, T-210, T-212, T-401]
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

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-enrollment/src/domain/*.rs` | create or modify | Path named in this task's contract. |
| `crates/eq-enrollment/tests/api_enrollments.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `activation_adds_member` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
