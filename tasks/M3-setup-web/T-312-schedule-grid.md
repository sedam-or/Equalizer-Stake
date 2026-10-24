---
id: T-312
title: Add the ScheduleGrid (day by resource, week by day), SlotPicker and the conflict/alternatives panel
milestone: M3
status: todo
depends_on: [T-307]
spec_refs: [DS-01, FE-31, FE-33, FE-36, SCH-05, SCH-14]
estimated_size: M
---

## Goal
Staff see the school's week at a glance, see **why** a slot isn't free (the buffer is drawn), and resolve conflicts with one click on a suggested alternative.

## Context (normative, spec 09 §4.1)
- **Axes:**
  - The vertical time axis runs from the earliest opening to the latest closing among the visible days (from `SchedulingPolicy.opening_hours`, read through `GET /settings/policies/scheduling` for staff, or a lightweight `GET /meta/app`-cached copy). Lines every 15 min, hour labels `HH:mm` in the tenant TZ.
  - Closed times are shaded `--color-surface-sunken` with a diagonal hatch (CSS `repeating-linear-gradient` using token colours).
  - Closures: full-column shading with the name label.
- **Views:**
  - Day: resources as columns.
  - Week: days as columns, filtered by one resource **or** one instructor (a select in the toolbar).
  - Toolbar: date navigation (prev/today/next), a date picker, a view switch, a filter.
- **Session block:**
  - Colour from `offering.color`, else the chart palette by offering index. A 4 px left border, a tinted fill (colour-mix with the surface), title = offering name, subtitle = participants (or `n/capacity`) and instructor.
  - Status styles: cancelled = hatched + line-through time; completed = muted; any blocked participant → lock icon.
  - **Buffer tail:** the hatched area after the block, of length `buffer_minutes` (from the session DTO; add `buffer_minutes` to the Session DTO in the backend if missing).
- **Now line:** 2 px `--color-danger` with a dot, auto-scrolled into view on open.
- **Interactions:**
  - Desktop: click an empty cell → `onCreate({resourceId, start})`. Drag a block (pointer events) → ghost at the 15-min snap (`slot_step_minutes`), calling `POST /sessions/check` (debounced 150 ms) to colour the ghost green or red. Drop → confirm dialog → `onReschedule`.
  - Mobile: day view only, resources as horizontal swipe tabs; tap a slot → `onCreate`.
  - Keyboard: the grid is `role="grid"`, arrow keys move the focused cell, Enter activates, and blocks are buttons with `aria-label="{offering}, {instructor}, {start} to {end}, {resource}, {status}"`.
- **Density:** 1 h = 64 px desktop, 72 px mobile.
- **Performance:** virtualise columns beyond 12 resources. ≤ 300 blocks render without jank (a perf test with 500 seeded sessions, asserting render < 100 ms in jsdom as a smoke check, plus a Playwright trace in CI as an artifact).
- **SlotPicker:** a mobile-first list of free starts grouped Morning (< 12:00), Afternoon (12–17), Evening (≥ 17) from `GET /availability`, with chips as buttons.
- **ConflictPanel** (FE-31): for an `ApiProblem` with `code === 'slot_taken'`, it shows each conflict as a sentence ("Room 2 is booked 15:00–15:45", "Ayu is teaching 15:00–15:45") and the alternatives as clickable chips ("Room 3 · 15:00", "Room 2 · 15:55"). Clicking calls `onPick(alt)`.
- Data: `GET /sessions?view=calendar&from&to&resource_id|instructor_id` via TanStack Query (key includes the range and filter). Closures from `GET /closures?from&to`.

## Scope
### In
- `src/components/domain/ScheduleGrid/*`, `SlotPicker`, `ConflictPanel`, and the `buffer_minutes` DTO field if missing (+ `make api`). A `/__ds` section with seeded fake data.

### Out
- The page wiring the create/reschedule sheets (T-313).

## Acceptance criteria
- AC1. Given sessions 15:00–15:45 with buffer 10, the block spans 45 min and the hatched tail spans 10 min (DOM measurements in px with 64 px/h). → `ScheduleGrid.test.tsx › buffer tail`
- AC2. Closed hours are shaded and a closure column shows its name. → `closed and closures`
- AC3. Keyboard: arrow navigation moves focus and Enter on an empty cell calls `onCreate` with the right start. → `keyboard`
- AC4. 500 sessions over 20 resources render with column virtualisation (only visible columns are in the DOM). → `virtualization`
- AC5. ConflictPanel renders the sentences and the alternatives, and the click callback fires. → `ConflictPanel.test.tsx`
- AC6. axe: 0 violations.

## Tests to write
| File | Test names |
|---|---|
| `web/src/components/domain/ScheduleGrid/*.test.tsx` | AC1–AC4, AC6 |
| `web/src/components/domain/ConflictPanel.test.tsx` | AC5 |

## Definition of done
- [ ] Web checks pass; bundle budget passes
- [ ] Commit `feat(web): schedule grid and conflict panel [T-312]`, signed off

## Report

## Questions
