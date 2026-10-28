---
id: T-213
title: M2 integration — the 50→1 concurrency proof (TST-05), series approval race, and the scheduling journey
milestone: M2
status: todo
depends_on: [T-117b, T-212]
spec_refs: [TST-05, SCH-03, DOM-10, PRD-12]
estimated_size: S
---

## Goal
Publicly provable: 50 simultaneous bookings of one slot produce exactly one booking. This test is the README badge and USP #2.

## Context
- TST-05: a tokio `JoinSet` of 50 tasks through the **HTTP layer** (axum-test or a real `reqwest` against `ritma serve` on a random port, against a real DB), each `POST /sessions` for the same resource and time with **different** Idempotency-Keys and different participants (so only the resource collides).
  - Expect exactly 1 × 201 and 49 × 409 `slot_taken`, and exactly 1 resource hold in `calendar_holds` for that range.
  - Run it 5 times in a loop in the same test (fresh slot each time) to catch flakiness.
- Variant B: 20 concurrent `POST /series/{id}/members` for a capacity-1 series. Exactly 1 succeeds; the rest get 422 `policy_violation` rule `scheduling.capacity` or 409. This relies on the `FOR UPDATE` lock on the series row in add_member (T-209).
- Variant C: 50 concurrent bookings where each request uses the same **instructor** but different rooms → exactly 1 × 201.
- CI: make sure the test runs in the `rust` job (not ignored). Add a separate CI step that runs only this test by name and writes the result into the job summary (`$GITHUB_STEP_SUMMARY`: "Concurrency proof: 50 → 1 ✅"). The README badge links to the workflow (README already has the badge placeholder; update the link only if the workflow name differs).
- Journey test `m2_journey`:
  1. Seed a tenant with the dev seed (T-117b).
  2. Create a location, 2 rooms, an offering and a price plan.
  3. Create an instructor and 2 students.
  4. Create a series.
  5. Materialise.
  6. A guardian cancels one participation (tier credit recorded).
  7. The admin reschedules one session.
  8. Availability reflects the changes.
  9. Auto-complete runs after time travel.
  10. Audit entries exist for each mutation.

## Scope
### In
- `crates/eq-server/tests/concurrency.rs`, `crates/eq-server/tests/m2_journey.rs`, and the CI summary step.

### Out
- Load and performance benchmarks.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-server/tests/concurrency.rs` | test | Task-listed verification file. |
| `crates/eq-server/tests/m2_journey.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `fifty_to_one_instructor` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. `concurrency::fifty_to_one_resource` passes 5/5 iterations.
- AC2. `concurrency::fifty_to_one_instructor` passes.
- AC3. `concurrency::series_capacity_race` passes (1 success).
- AC4. `m2_journey::m2_journey` passes.
- AC5. The CI summary shows the proof line.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/concurrency.rs` | `fifty_to_one_resource`, `fifty_to_one_instructor`, `series_capacity_race` |
| `crates/eq-server/tests/m2_journey.rs` | `m2_journey` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks pass, and the three concurrency tests pass in ten consecutive local runs (`for i in $(seq 10)`). Three consecutive green hosted CI runs are **environment-limited**: record them if you can observe them, otherwise list them under "Not run: <reason>"; this does not block the task
- [ ] Commit `test(scheduling): concurrency proof and M2 journey [T-213]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
