---
id: T-313
title: Add the admin pages — schedule (create/reschedule/cancel), series, catalog, rooms and closures, people and person detail
milestone: M3
status: todo
depends_on: [T-112b, T-204, T-208, T-306a, T-311, T-312]
spec_refs: [FE-30, FE-31, FE-34, SCH-07, SCH-10, SCH-12, IDN-10, VAL-33, VAL-34, UX-03, UX-05, UX-06]
estimated_size: M
---

## Goal
An admin runs the school's week from the browser: rooms, courses and prices, teachers and students, recurring lessons, and one-off changes, all on top of the M1/M2 APIs.

## Context
Pages (routes per spec 08 §6, all inside the shell with `requirePerm`):

| Route | Content | API |
|---|---|---|
| `/schedule` | PageHeader + ScheduleGrid. "New session" sheet: offering, duration (from the offering), resource, instructors (PersonPicker role instructor), participants (PersonPicker role student, multi), date + start (SlotPicker or time input), kind, notes, override hours checkbox (only with `scheduling.session.override_hours`) | `POST /sessions/check` live (debounced 300 ms) shows violations, warnings and ConflictPanel; submit `POST /sessions` with `idempotent: true` |
| `/sessions/:id` | Session detail: facts as chips, participants with status badges, instructors, actions: Reschedule (sheet with SlotPicker + If-Match), Cancel (dialog: scope, reason; the credit is decided by the server and shown in the success toast), Add participant, Substitute instructor; owner: Revert | T-208 endpoints |
| `/schedule/series`, `/schedule/series/:id` | List + create sheet (offering, weekday, start, duration, resource, instructor, capacity, valid from/to, members) → 409 shows `first_conflict_date` + ConflictPanel; detail: next occurrences, members (add/remove with dates), edit-from-date, end series | T-209 |
| `/catalog` | Tabs: Offerings (DataTable + editor sheet: key (immutable after create), name i18n, durations chips, capacity min/max, ages, internal, colour) · Levels (tracks + ordered levels) · Prices (per offering: plans with effective_from; "Set your prices" banner for `price_needs_review`) | T-204 |
| `/resources`, `/closures` | Rooms DataTable (name, kind, capacity, rentable, archive) · Closures list + add (date range, name i18n, location) + CSV import (FileDrop → `POST /closures/import`, errors per row) | T-204 |
| `/people` (+ `?role=`), `/people/:id` | Directory DataTable (search `q`, role filter, status) + "Add person" sheet (roles, contact, invite toggle with channel, guardian links) · Detail tabs: Profile (PII shown only if permitted), Roles, Links (guardian ↔ students), Schedule (`/sessions?person_id`) · actions: invite, deactivate/reactivate | T-112 |

- Every list has empty, loading and error states (FE-34). Empty-state copy explains why and what to do ("No rooms yet. Add the rooms you teach in.").
- Drafts (FE-30): the session create sheet `scheduling.session.new` and the series create sheet `scheduling.series.new`.
- Terminology everywhere (`useTerm`): "Add {term.instructor}".
- `PersonPicker` (domain component): a Combobox over `GET /people?role=&q=` with avatar initials, async search, multi mode.
- The Cancel dialog credit preview: **don't compute credits client-side** (business rule). After submit, the toast reads "Cancelled. Make-up credit: {percent}" from the response participants' `credit_bps`. **Backend addition:** make `POST /sessions/{id}/cancel` return the updated Session DTO including `participants[].credit_bps`.
- Feedback (spec 27, UX-01..07): use only these catalogue entries, through `useToast(id)`, `Dialog` and `Banner` (T-305). Screen states: the spec 27 §8 row for `/schedule`, `/schedule/series`, `/catalog`, `/resources`, `/closures`, `/people`, `/sessions/:id`. Toasts: `toast.scheduling.session_created`, `session_created_below_min`, `session_rescheduled`, `session_cancelled`, `series_created`, `series_ended`, `closure_added`, `closures_imported`, `session_reverted`, `toast.identity.person_created`, `invite_sent`. Dialogs: `dialog.scheduling.cancel_session`, `override_hours`, `end_series`, `revert_session`, `dialog.identity.deactivate_person`, `remove_role`, `dialog.generic.version_conflict`, `discard_changes`. Banners: `banner.enrollment_disabled_sessions`.

## Scope
### In
- The pages above, `PersonPicker`, `FileDrop`, the cancel response change (+ `make api`), and locale strings.

### Out
- Enrollment-driven flows (M4). Import wizard (M4).

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/e2e/admin.spec.ts` | test | Task-listed verification file. |
| `web/src/modules/scheduling/**/states.test.tsx`, `web/src/modules/identity/**/states.test.tsx` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T2, T3 (SBD-30).
- Data classes touched: S1/S2 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `web/e2e/admin.spec.ts › conflict alternative` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. E2E (dev seed + music preset): create a room, an offering with a price, an instructor and two students; create a one-off session; trigger a conflict by booking the same slot and pick an alternative chip, which succeeds. → `web/e2e/admin.spec.ts › conflict alternative`
- AC2. Create a series with 2 members and see the occurrences on `/schedule` in week view filtered by the instructor. → `series visible`
- AC3. Cancel a participation as admin with a reason, and the toast shows the credit from the server. → `cancel shows credit`
- AC4. Drag-reschedule a block to a free slot (green ghost), confirm, and the block moves. → `drag reschedule`
- AC5. Every page: empty, loading (delayed msw) and error (500 msw) states visible in unit tests. → `states.test.tsx`
- AC6. axe: 0 serious or critical on every page at 390 and 1280 px.

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/admin.spec.ts` | AC1–AC4, AC6 |
| `web/src/modules/scheduling/**/states.test.tsx`, `web/src/modules/identity/**/states.test.tsx` | AC5 |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks + E2E pass; `make api` regenerated
- [ ] Commit `feat(web): admin schedule, catalog and people pages [T-313]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
