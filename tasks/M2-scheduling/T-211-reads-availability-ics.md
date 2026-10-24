---
id: T-211
title: Add session lists and calendar views, /me/schedule, availability, attention list, instructor availability and iCal feeds
milestone: M2
status: todo
depends_on: [T-210]
spec_refs: [SCH-14, SCH-15, SCH-16, IDN-10, API-05, API-09]
estimated_size: M
---

## Goal
Everyone sees the schedule that's theirs: admins the whole school, instructors their teaching, guardians their children's lessons. Free slots are computed in one place, and calendars can be subscribed to from phones.

## Context
- `GET /sessions?from&to&resource_id&instructor_id&person_id&status&view=calendar|list&cursor`:
  - Scoped (IDN-10): admin/owner all; instructor teaching; guardian sessions with any linked student participating; student own.
  - `view=calendar` returns every session in `[from, to)` with no pagination; the range is ≤ 42 days (technical const), else 422 `validation /to range_too_large`.
  - `view=list` is cursor-paginated by `starts_at, id`.
  - DTO as T-206 (participants filtered: guardians see only their linked students among participants; other participants are counted as `others_count`).
- `GET /me/schedule?from&to`: the same as `/sessions?view=calendar` with the caller's scope, but for **every role** (owner included: owner → empty unless they also teach).
- `GET /availability?date=&location_id=&duration=&offering_id?&instructor_id?&resource_id?`:
  - Perm: staff, or guardian/student when `resource_id` is rentable (used by rental in M6). Otherwise 403.
  - Per resource at the location (or the one given): free start times on that local date. A start is free when:
    - `[start, start+duration)` fits the opening hours, aligned to `slot_step_minutes`, the date is not a closure, and the booking window applies for non-staff;
    - the resource's `[start, start+duration+buffer)` doesn't overlap any resource hold;
    - with `instructor_id`, the instructor's range (with buffer) doesn't overlap that instructor's holds.
  - Response `{date, duration, resources: [{id, name, slots: ["HH:MM", ...]}]}`. Advisory (DOM-11).
  - Implemented as the pure function `domain::availability::free_starts(...)` reusing `domain::slot` and the hold-overlap helper from T-207.
- `GET /attention?module&kind&cursor` (staff): open attention items, newest first. `POST /attention/{id}/resolve` (staff) → 204, audited. **Core endpoint** (not scheduling), since other modules raise items too. Put it in `eq-api` core routes with perm `core.attention.read` (owner, admin) and `core.attention.resolve` (admin). Add these permissions to core's list.
- SCH-15: `GET/PUT /instructors/{id}/availability` (staff, or the instructor themselves) → `[{weekday, intervals: [{start, end}]}]`. PUT replaces all rows for that person.
- SCH-16 iCal:
  - `GET /calendar.ics?token=` (no auth header). `token = base64url(person_id (16 bytes) || HMAC-SHA256(Keys.ics, tenant_id || person_id || ics_secret_version_be32)[0..16])`. A bad or old token → 404.
  - Add the column `persons.ics_secret_version int NOT NULL DEFAULT 1` in `migrations/0012_ics.sql`. `POST /me/calendar-token` returns the URL. `DELETE /me/calendar-token` increments the version, which revokes the old URL.
  - The feed covers sessions from −30 to +90 days in the person's scope (instructor/guardian/student), with RFC 5545 VEVENTs: UID `session-{id}@{host}`, DTSTART/DTEND in UTC, SUMMARY = offering name (tenant locale) + resource, STATUS CANCELLED for cancelled sessions. Use the `icalendar` crate.
  - Response `text/calendar; charset=utf-8`. Rate limit as public.

## Scope
### In
- The endpoints above, `domain::availability`, the migration `0012`, and the core attention endpoints and permissions.

### Out
- The SPA (M3).

## Acceptance criteria
- AC1. A guardian sees only sessions with their linked students, and other participants appear only as `others_count`. → `api_reads::guardian_calendar_scope`
- AC2. Room 1, Mon 14:30–17:00, step 15, duration 45, buffer 10, one existing session 15:00–15:45 (resource hold `[15:00, 15:55)`) → slots exactly `["16:00", "16:15"]`. 14:30 fails because `[14:30, 15:25)` overlaps the hold, 15:55 is not aligned to the step, and 16:30 + 45 exceeds 17:00. → `availability_exact`
- AC3. Property: every start returned by `free_starts` can be booked (pure check against the same holds). → `proptest availability_bookable`
- AC4. `view=calendar` with a 60-day range → 422. → `calendar_range_limit`
- AC5. An iCal token works, and after `DELETE /me/calendar-token` the old URL → 404. → `ics_token_revocable`
- AC6. The ics output parses with the `icalendar` parser and contains the cancelled session with `STATUS:CANCELLED`. → `ics_contents`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/api_reads.rs` | AC1, AC2, AC4–AC6 |
| `crates/eq-scheduling/src/domain/availability.rs` | `availability_bookable` |

## Definition of done
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(scheduling): schedule views, availability and iCal [T-211]`, signed off

## Report

## Questions
