---
id: T-210
title: Add the daily materialisation job with the cycle cap (5th week), closure handling, conflicts to attention items, and auto-complete
milestone: M2
status: todo
depends_on: [T-209]
spec_refs: [SCH-08, SCH-09, TST-15, TST-02]
estimated_size: M
---

## Goal
Recurring lessons appear on the calendar weeks ahead without anyone clicking. School rules for holidays and the "extra week of the month" are applied automatically, and anything that can't be booked lands in "Needs attention" instead of silently disappearing.

## Context
- Job `scheduling.materialize`:
  - Cron `0 1 * * *` local, dedupe `scheduling.materialize:{tenant}:{date}`.
  - Also enqueued on demand after series or member changes (dedupe `scheduling.materialize:{tenant}:series:{id}:{now_minute}`).
  - For each active series: generate up to `today_local + materialize_weeks_ahead weeks` in mode `Lenient` (conflicts are skipped and recorded). Update `series.materialized_until`.
- Idempotency: the unique index `(series_id, occurrence_date)` for non-cancelled sessions. Insert with `ON CONFLICT DO NOTHING` semantics: check existence first, and treat a unique violation as "exists". **A cancelled occurrence is not recreated.** Check for any session (any status) with that `(series_id, occurrence_date)` before planning.
- Participants: series members active on the date (`active_from <= d AND (active_to IS NULL OR active_to >= d)`) minus people on leave on that date, via the port `EnrollmentStatus::on_leave(tx, person_ids, date) -> HashSet<PersonId>`. Define the port in `eq_core::ports`; the default impl (enrollment disabled) returns an empty set.
- **Cycle cap** (SCH-08), when `recurrence.sessions_per_cycle = Some(n)`:
  - Count occurrences **per series per local calendar month**, by occurrence index in that month (1st Tuesday, 2nd Tuesday, …), counting skipped closure dates too, so "5th week" means the 5th weekday of the month.
  - Occurrence index > n follows `extra_occurrence`:
    - `Regular`: as normal.
    - `Skip`: no session.
    - `Substitute{offering_key, duration_minutes, consumes_credit}`: a session of that offering (looked up by key; missing → attention `substitute_offering_missing`), `kind = substitute`, the same resource and instructor if free; else the first alternative (T-207 finder); else skip and raise attention `materialize_conflict`.
    - Emit `scheduling.occurrence_skipped.v1 {series_id, occurrence_date, reason: "extra_occurrence", person_ids, credit_bps: 0}` for `Skip`.
- **Closures** (SCH-09). The date is in a closure (tenant-wide or the series location) → `holidays.on_closure`:
  - `Skip`: no session; emit `occurrence_skipped {reason: closure, credit_bps: 0}`.
  - `ShiftToNextWeek`: try the same weekday + 7 days at the same local time as a `makeup` session. If that is free (and not a closure), book it. Else apply `when_no_slot`:
    - `Skip` → occurrence_skipped credit 0.
    - `MakeupCredit` → occurrence_skipped with `credit_bps = Bps::FULL`.
    - `Substitute` → treat like the extra-occurrence substitute, using `recurrence.extra_occurrence` when it is a `Substitute`; otherwise fall back to MakeupCredit.
  - `MakeupCredit`: no session; occurrence_skipped with `credit_bps = Bps::FULL`.
- A conflict in Lenient mode: skip the occurrence and raise attention `materialize_conflict` (payload `{series_id, occurrence_date, conflicts, alternatives}`, dedupe `materialize_conflict:{series}:{date}`, severity `normal`).
- **Attention helper** (core): `eq_core::attention::{raise(tx, AttentionItem{module, kind, ref_type, ref_id, payload, severity, dedupe_key}), resolve(tx, dedupe_key, by)}`. The table already has `UNIQUE (tenant_id, dedupe_key)`. `raise` = `INSERT … ON CONFLICT (tenant_id, dedupe_key) DO UPDATE SET payload = EXCLUDED.payload WHERE attention_items.resolved_at IS NULL`, so an item the admin resolved is **never reopened** by later job runs.
- Job `scheduling.autocomplete` (cron `*/15 * * * *`): sessions with `status='scheduled' AND upper(time_range) < now` become `completed`. Emit `scheduling.session_completed.v1 {session_id, participant_ids (expected only)}` and audit with `ActorCtx::system`.
- **Reminders job: out of scope here** (it needs notify, M5).

## Scope
### In
- Extending `plan_occurrences` with closures and the cycle cap, the two jobs, the `EnrollmentStatus` port + default, the core attention helper, and the events.

### Out
- Credit creation (enrollment consumes the events in M4).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-scheduling/src/materialize.rs` | create or modify | Path named in this task's contract. |
| `crates/eq-scheduling/tests/materialize.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4, T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `cancelled_not_recreated` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. A Tuesday series, `sessions_per_cycle=4`, `Substitute{theory_class, 30, consumes_credit:false}`, materialising a month with 5 Tuesdays → 4 regular + 1 substitute 30-min `theory_class` session on the 5th Tuesday with `consumes_credit=false`. → `materialize::tests::fifth_week_substitute` (integration, TestClock)
- AC2. Running the job twice → no duplicates. → `idempotent_rerun`
- AC3. A closure on a series date with `on_closure=Skip` → no session that date, and one `occurrence_skipped` event with reason closure. → `closure_skip`
- AC4. `ShiftToNextWeek` with next week free → a makeup session next week. Next week occupied + `when_no_slot=MakeupCredit` → an event with credit 10000. → `closure_shift_and_fallback`
- AC5. An existing one-off session blocks week 3 → the job skips week 3 and creates one attention item, and a second run does not duplicate it. → `conflict_to_attention_once`
- AC6. A cancelled occurrence is not recreated by the next run. → `cancelled_not_recreated`
- AC7. Time travel: the clock passes a session's end → after `run_due` it is `completed` and the event is emitted. → `autocomplete`
- AC8. Property: for random series and months, the count of regular sessions per month ≤ n when the cap is set. → `proptest cycle_cap_holds`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/materialize.rs` | AC1–AC7 |
| `crates/eq-scheduling/src/materialize.rs` | `cycle_cap_holds` (proptest on `plan_occurrences`) |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `scripts/check-literals.sh` passes
- [ ] Commit `feat(scheduling): materialisation and auto-complete jobs [T-210]`, signed off

## Report

**Deviations** — RATIFY: TD-069, T-209 owns the base occurrence planner; T-210 extends it.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-069):** the task text assigns the materialisation core to T-210 and later assigns it to T-209 → T-209 defines the reusable core and T-210 extends it, because T-209 precedes its consumer and its own implementation notes name that ownership.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
