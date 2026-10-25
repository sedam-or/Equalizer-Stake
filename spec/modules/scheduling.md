# Module: scheduling (always on)

Purpose: the catalog (offerings, levels, price plans), locations/resources UI, closures, recurring series, sessions and conflict-free booking. This is where the "zero double-booking" guarantee lives.
Policy: `SchedulingPolicy` (04 §6.3). Tables: 06 §4.3, plus `calendar_holds` and `closures` from core.
Legacy reference: `v0-capstone:srs/02-scheduling.md`, `backend/app/services/scheduling.py`, `scheduler.py`.

## 1. Rules

- **SCH-01. Time model.**
  - A session's `time_range` is `[starts_at, starts_at + duration)`.
  - `duration` MUST be one of the offering's `durations_minutes`. Staff may also pick any multiple of `slot_step_minutes` with `scheduling.session.custom_duration`.
  - `starts_at` in local time MUST align to `slot_step_minutes`.
- **SCH-02. Opening hours.** The whole `[start, end)` range (without buffer) MUST fit inside one interval of `opening_hours` for the local weekday at the location's timezone. Otherwise the result is 422 `PolicyViolation rule=scheduling.opening_hours`, unless the actor has `scheduling.session.override_hours` **and** sends `override_hours: true`. Overrides are audited.
- **SCH-03. Conflicts.** Creating or rescheduling a session writes calendar holds (DOM-10) in the same tx. The exclusion constraint decides. On `23P01` the service:
  1. rolls back,
  2. re-queries the overlapping holds (read-only, new tx) to find which subject collided,
  3. computes alternatives (SCH-05),
  4. returns 409 `slot_taken` with `{conflicts: [{subject_kind, subject_id, subject_name, owner_kind, range}], alternatives: [...]}`.
- **SCH-04. Closures.** A session whose local date falls in a closure (tenant-wide or same location) is rejected with 422 `rule=scheduling.closure`, unless overridden. Materialisation applies `holidays.on_closure` (SCH-09).
- **SCH-05. Alternatives.**
  - Up to `alternatives.count` free options, searched in this priority order (v0 SRS-08 §4.2):
    1. same time, other suitable resource (capacity ≥ participants, same location);
    2. same resource, nearest start within ±`radius_minutes` stepping by `slot_step_minutes`;
    3. other resource, nearest start within the radius.
  - Each candidate must satisfy opening hours, closures and **all** subjects' holds (resource, instructors, participants).
  - The search is a pure function over the holds loaded for the day window: `domain::alternatives::find(...)`.
- **SCH-06. Capacity.** Participants are ≤ `offering.capacity_max` and ≤ `resource.capacity`. Fewer than `capacity_min` gives a **warning** in the response (`warnings: ["below_min_capacity"]`), not an error.
- **SCH-07. Series (recurring slots).**
  - A series is a weekly slot.
  - Creating one validates the first `materialize_weeks_ahead` occurrences atomically: all of them are free, or 409 is returned with the first conflicting date.
  - Members come from `series_members`. The enrollment module adds and removes them, and admins can add them manually when enrollment is disabled.
- **SCH-08. Materialisation.**
  - The job `scheduling.materialize` runs daily at 01:00 tenant TZ and on demand after series or member changes.
  - For each active series it generates sessions up to `today + materialize_weeks_ahead weeks`. Sessions are idempotent per `(series_id, occurrence_date)`.
  - Participants are the members active on that date, excluding people whose enrollment is `on_leave` covering the date (via the `EnrollmentStatus` port).
  - **Billing cycle cap:** if `recurrence.sessions_per_cycle = Some(n)`, occurrences are counted per local calendar month. Occurrence n+1 and later follow `extra_occurrence`:
    - `Regular`: no change.
    - `Skip`: no session.
    - `Substitute`: a session of the offering `offering_key` with `duration_minutes`, `kind=substitute`, `consumes_credit` as configured, same participants. The resource and instructor are the same as the series if free; otherwise the first alternative; otherwise it is skipped and the admin is notified.
  - **Conflicts during materialisation** (e.g. someone booked a one-off session) are skipped. The job records a `materialize_conflict` item that appears in the admin "Needs attention" list with alternatives. Materialisation never overwrites.
- **SCH-09. Closures vs. series.**
  - `on_closure = Skip`: no session that week.
  - `ShiftToNextWeek`: an extra session is booked on the same weekday next week at the same time if free (`kind=makeup`). If it isn't free, `when_no_slot` applies:
    - `Skip`
    - `MakeupCredit`: emits an event so enrollment grants a credit
    - `Substitute`: as in SCH-08 extra occurrence
  - `MakeupCredit`: a credit is granted directly to each participant.
- **SCH-10. Cancellation.**
  - `POST /sessions/{id}/cancel {reason, scope: "session"|"participant", participant_id?}`. The reason is required if `reason_required`.
  - Who: staff with `scheduling.session.cancel`, or a payer/guardian for their own student's participation when `payer_can_cancel`. Payers can only cancel their participant, not the whole session.
  - **Credit computation** (pure function `domain::cancellation::credit_bps`):
    - Staff cancelling the whole session: `staff_cancel_credit_bps`.
    - A payer cancelling a participant: `h = hours_before_start` (floored). The result is the `makeup_credit_bps` of the first tier with `min_hours_before <= h`, or 0.
    - An instructor absence (staff cancel with `cause: instructor_absence`) follows `instructor_absence`:
      - `SubstituteThenReschedule`: the response suggests substitute instructors free in that slot. If staff choose none, the system tries to reschedule to the same time next week. If that fails, a credit of 10000.
      - `Reschedule`
      - `MakeupCredit` (10000)
  - The participant's `credit_bps` is stored, and `scheduling.session_cancelled.v1` is emitted. Enrollment creates `makeup_credits` if > 0 and `makeup_credits.enabled`.
  - Holds: a whole-session cancel deletes all holds. A participant cancel deletes only that participant's hold.
- **SCH-11. No-show.** Not decided here. Attendance marks `absent`, and enrollment applies `student_no_show` (Forfeit gives no credit; MakeupCredit grants 10000).
- **SCH-12. Reschedule.** `POST /sessions/{id}/reschedule {starts_at, resource_id?, instructor_ids?, reason}` is cancel-free. It updates the range, rewrites holds, increments `version`, emits `session_rescheduled`, and requires `If-Match`.
- **SCH-13. Frozen payers.** On `billing.payer_frozen.v1`, all `expected` participants of the payer's students in future sessions become `blocked`. On `payer_unfrozen`, they go back to `expected`. Blocked participants keep their holds.
- **SCH-14. Availability view.** `GET /availability?date=&location_id=&duration=&offering_id=&instructor_id=` returns, per resource, the free start times for that local date: opening hours minus closures minus holds (including buffer), aligned to the step. It is used by the schedule UI and the rental booking UI. It is advisory (DOM-11).
- **SCH-15. Instructor availability (optional data).** `instructor_availability(person_id, weekday, intervals)` is a soft constraint. Booking outside it gives a warning `outside_instructor_availability`, not an error. *(The table is added in this module's migration: `instructor_availability(tenant_id, person_id, weekday, start_local, end_local)`.)*
- **SCH-16. Calendar feeds.** `GET /calendar.ics?token=` gives a per-person signed, read-only iCal feed of their sessions (instructor, guardian, student). The token is HMAC(person_id, rotating secret) and can be revoked from the profile.

## 2. API (all under `/api/v1`)

| Method | Path | Perm | Notes |
|---|---|---|---|
| GET/POST | `/locations`, PATCH `/locations/{id}` | read: any staff / write: `scheduling.catalog.write` | includes `geo` pin |
| GET/POST | `/resources`, PATCH/DELETE(archive) `/resources/{id}` | same | |
| GET/POST | `/closures`, DELETE `/closures/{id}` | same | `POST /closures/import` accepts CSV `date_from,date_to,name` |
| GET/POST | `/catalog/level-tracks`, `/catalog/levels` | same | |
| GET/POST | `/catalog/offerings`, GET/PATCH/DELETE(archive) `/catalog/offerings/{id}` | read: any authenticated (public subset at `/public/catalog`) | |
| GET/POST | `/catalog/offerings/{id}/price-plans` | `scheduling.catalog.write` | new effective_from creates a new row; never edit past rows |
| GET | `/public/catalog` | none | Non-internal, non-archived offerings with current prices, for `/join` and the landing page |
| GET | `/availability` | staff, or guardian/student for rental | SCH-14 |
| GET/POST | `/series`, GET/PATCH `/series/{id}`, POST `/series/{id}/end` | `scheduling.series.write` | PATCH changes take effect from `effective_date` (future occurrences regenerated) |
| POST/DELETE | `/series/{id}/members` | `scheduling.series.write` | manual membership when enrollment is disabled |
| GET | `/sessions` | scoped | `?from&to&resource_id&instructor_id&person_id&status&view=calendar\|list` |
| POST | `/sessions` | `scheduling.session.write` | one-off session `{offering_id, resource_id, instructor_ids, participant_ids, starts_at, duration_minutes, kind, override_hours?}` → 201 Session \| 409 `slot_taken` \| 422 |
| POST | `/sessions/check` | `scheduling.session.write` | Same body as POST; dry run → `{ok, conflicts, alternatives, warnings}` (no writes) |
| GET | `/sessions/{id}` | scoped | includes participants, instructors, attendance summary if enabled |
| POST | `/sessions/{id}/reschedule` | `scheduling.session.write` | If-Match |
| POST | `/sessions/{id}/cancel` | staff or payer (participant scope) | SCH-10 |
| POST | `/sessions/{id}/participants` / DELETE `/sessions/{id}/participants/{pid}` | `scheduling.session.write` | add make-up/trial participant (writes hold) |
| POST | `/sessions/{id}/instructors` | `scheduling.session.write` | substitute |
| GET | `/me/schedule` | any | own sessions (instructor: teaching; guardian: linked students; student: own) |
| GET | `/attention` | staff | "Needs attention" items: materialisation conflicts, sessions without instructor, below min capacity |
| GET/PUT | `/instructors/{id}/availability` | staff or self | SCH-15 |
| GET | `/calendar.ics` | token | SCH-16 |

`Session` DTO: `{id, offering:{id,key,name}, kind, status, starts_at, ends_at, local_date, local_start, duration_minutes, resource:{id,name}, location_id, instructors:[{id,display_name,role}], participants:[{id,display_name,status}], series_id?, consumes_credit, cancelled?:{at,by_role,reason}, version}`.

## 3. Permissions

| Permission | owner | admin | instructor | guardian | student |
|---|---|---|---|---|---|
| scheduling.catalog.read | ✅ | ✅ | ✅ | ✅ | ✅ |
| scheduling.catalog.write | — | ✅ | — | — | — |
| scheduling.series.write | — | ✅ | — | — | — |
| scheduling.session.read | ✅ all | ✅ all | own | linked | own |
| scheduling.session.write | — | ✅ | — | — | — |
| scheduling.session.cancel | — | ✅ | — | participant-only (policy) | participant-only if payer |
| scheduling.session.override_hours | — | ✅ | — | — | — |
| scheduling.session.custom_duration | — | ✅ | — | — | — |
| scheduling.override.revert | ✅ | — | — | — | — |

The owner override `POST /sessions/{id}/revert` restores a cancelled session to scheduled. It succeeds only if all holds can be re-acquired; otherwise it returns 409.

## 4. Events and jobs

- **Emits:** `session_scheduled`, `session_rescheduled`, `session_cancelled`, `session_completed`.
- **Consumes:**
  - `billing.payer_frozen/unfrozen` (SCH-13)
  - `enrollment.enrollment_on_leave` (cancel the participant's future sessions in range with credit 0, reason `on_leave`)
  - `enrollment.enrollment_ended` (remove the member from the series from `ended_on`, and delete participant holds after that date)
- **Jobs:**
  - `scheduling.materialize`: daily at 01:00 plus on demand.
  - `scheduling.autocomplete`: every 15 min.
  - `scheduling.reminders`: every 5 min. It finds sessions starting within each `notify.session_reminders_minutes_before` window and emits a notification request with dedupe key `reminder:{session}:{minutes}`. It is planned only if notify is enabled.

## 5. Acceptance criteria

1. **Concurrency (TST-05).** Given one free slot, When 50 concurrent POST /sessions target the same resource and time, Then exactly 1 returns 201 and 49 return 409 `slot_taken`, and `calendar_holds` contains exactly 1 resource hold for that range.
2. **Buffer.** Given `buffer_minutes=10` and a session 15:00–15:45 in Room 1, When booking Room 1 at 15:50, Then 409. When booking at 15:55, Then 201.
3. **Instructor clash.** Given teacher T teaching 15:00–15:45 in Room 1, When booking T in Room 2 at 15:30, Then 409 with conflict subject = T.
4. **Student clash without buffer.** Given student S in a 15:00–15:45 session, When adding S to a 15:45 session, Then 201 (no buffer for students).
5. **Opening hours.** Given Mon 14:30–17:00, When booking Mon 16:30 for 45 min, Then 422 `scheduling.opening_hours`. With an admin override flag, Then 201 plus an audit entry with `override_hours`.
6. **Fifth week.** Given a Tuesday series, `sessions_per_cycle=4` and `Substitute{theory_class,30}`, When materialising a month with 5 Tuesdays, Then 4 regular sessions plus 1 substitute 30-min theory session on the 5th Tuesday with `consumes_credit=false`.
7. **Cancellation tiers.** Given tiers `[{24 h → 10000}]`, When a guardian cancels 30 h before, Then credit 10000. When they cancel 3 h before, Then credit 0. Given `tiers=[]`, then 0 in both cases.
8. **Alternatives.** Given a conflict, Then the alternatives are ≤ `alternatives.count`, every one is bookable (verified by booking it in the test), and they are ordered by SCH-05 priority.
9. **Materialisation idempotency.** Running materialise twice creates no duplicates (unique index).
10. **Closure skip.** A closure on a series date with `on_closure=Skip` creates no session for that date.
