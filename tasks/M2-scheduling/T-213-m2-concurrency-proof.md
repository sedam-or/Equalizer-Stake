---
id: T-213
title: M2 integration — the 50→1 concurrency proof (TST-05), series approval race, and the scheduling journey
milestone: M2
status: todo
depends_on: [T-212]
spec_refs: [TST-05, SCH-03, DOM-10, PRD-12]
estimated_size: S
---

## Goal
Publicly provable: 50 simultaneous bookings of one slot produce exactly one booking. This test is the README badge and USP #2.

## Context
- TST-05: a tokio `JoinSet` of 50 tasks through the **HTTP layer** (axum-test or a real `reqwest` against `equalizer serve` on a random port, against a real DB), each `POST /sessions` for the same resource and time with **different** Idempotency-Keys and different participants (so only the resource collides).
  - Expect exactly 1 × 201 and 49 × 409 `slot_taken`, and exactly 1 resource hold in `calendar_holds` for that range.
  - Run it 5 times in a loop in the same test (fresh slot each time) to catch flakiness.
- Variant B: 20 concurrent `POST /series/{id}/members` for a capacity-1 series. Exactly 1 succeeds; the rest get 422 `policy_violation` rule `scheduling.capacity` or 409. This relies on the `FOR UPDATE` lock on the series row in add_member (T-209).
- Variant C: 50 concurrent bookings where each request uses the same **instructor** but different rooms → exactly 1 × 201.
- CI: make sure the test runs in the `rust` job (not ignored). Add a separate CI step that runs only this test by name and writes the result into the job summary (`$GITHUB_STEP_SUMMARY`: "Concurrency proof: 50 → 1 ✅"). The README badge links to the workflow (README already has the badge placeholder; update the link only if the workflow name differs).
- Journey test `m2_journey`:
  1. Seed a tenant with the dev seed (T-117).
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
- [ ] All checks pass, three consecutive green CI runs
- [ ] Commit `test(scheduling): concurrency proof and M2 journey [T-213]`, signed off

## Report

## Questions
