---
id: T-204
title: Add the locations, resources, closures, level tracks, offerings, price plans and public catalog APIs
milestone: M2
status: todo
depends_on: [T-202]
spec_refs: [SCH-04, API-05, API-07, API-11, POL-12]
estimated_size: M
---

## Goal
Admins can model their school (locations, rooms, closures, levels, courses and prices) through the API. Anyone can read the public catalog.

## Context
API (from `spec/modules/scheduling.md` §2; all under `/api/v1`):

| Method | Path | Perm | Notes |
|---|---|---|---|
| GET/POST | `/locations`, PATCH `/locations/{id}` | read: scheduling.catalog.read / write: scheduling.catalog.write | body `{name, address?, timezone?, geo?: {lat, lng}}`; archive with `POST /locations/{id}/archive` |
| GET/POST | `/resources`, PATCH `/resources/{id}`, POST `/resources/{id}/archive` | same | `{location_id, name, kind, capacity, rentable, color?, sort?}` |
| GET/POST | `/closures`, DELETE `/closures/{id}` | same | `{location_id?, date_from, date_to (inclusive in API, stored as [from, to+1) daterange), name: I18nText, kind}`; `POST /closures/import` takes CSV `date_from,date_to,name` (name applied to all locales) |
| GET/POST | `/catalog/level-tracks`, `/catalog/levels` | same | levels `{track_id, key, name, ordinal}` |
| GET/POST | `/catalog/offerings`, GET/PATCH `/catalog/offerings/{id}`, POST `/catalog/offerings/{id}/archive` | read: any authenticated; write: catalog.write | `{key, name, description?, level_track_id?, durations_minutes, capacity_min, capacity_max, min_age_months?, max_age_months?, internal, color?}` with If-Match on PATCH |
| GET/POST | `/catalog/offerings/{id}/price-plans` | catalog.write | `{level_id?, key, kind, amount: Money, sessions_included?, effective_from: date}`; a new effective_from adds a row; past rows are never edited |
| GET | `/public/catalog` | none | non-internal, non-archived offerings with **current** price plans (latest `effective_from <= today` local per (level, key)); I18nText resolved by `Accept-Language` (API-11) |

Rules:
1. `price_plans.currency` must equal `CorePolicy.currency` → otherwise 422 `policy_violation` rule `core.currency`.
2. `durations_minutes` values must each be a multiple of `slot_step_minutes`. If not, 422 `policy_violation` rule `scheduling.slot_step_minutes`.
3. Offering `key` is immutable after creation (PATCH with `key` → 422 `validation` code `immutable`).
4. Archiving a resource with future scheduled sessions → 409 `resource_in_use {count}`. The SPA offers to reschedule.
5. Location `timezone` must be a valid IANA name (`chrono_tz`). `geo` is both-or-none.
6. Closures may overlap each other (no constraint). The DTO returns `dates: {from, to}` inclusive.
7. **Has-data:** implement `SchedulingModule::has_data` = exists(sessions).
8. All writes are audited (`scheduling.<entity>.<verb>`). Lists are cursor-paginated (API-05).
9. Admin endpoints return I18nText as full maps (API-11).

## Scope
### In
- Handlers, services and repos for the above. The `/public/catalog` route sits under `/api/v1/public`, which skips the auth layer but **not** tenant resolution or rate limits.

### Out
- Sessions and series (T-206+). Import UI.

## Acceptance criteria
- AC1. CRUD happy paths for each entity, with audit rows. → `api_catalog::crud_*` (one test per entity)
- AC2. A price plan with a different currency → 422 `core.currency`. → `price_currency_must_match`
- AC3. A new price plan with a later effective_from makes `/public/catalog` show the old price before that date and the new one from that date (with `TestClock`). → `public_catalog_current_price`
- AC4. Archiving a room with a future session → 409 `resource_in_use`. (Insert the session row directly in the test, since the session API comes later.) → `archive_resource_in_use`
- AC5. Closure CSV import of 3 rows creates 3 closures. A malformed row → 422 with the row number in `errors[].params.row`. → `closure_import`
- AC6. A guardian calling POST `/catalog/offerings` → 403. → `catalog_write_forbidden_for_guardian`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/api_catalog.rs` | AC names above |

## Definition of done
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(scheduling): catalog and rooms API [T-204]`, signed off

## Report

## Questions
