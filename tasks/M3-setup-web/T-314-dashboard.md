---
id: T-314
title: Add the role dashboards (owner/admin today + attention + setup checklist, instructor today, guardian schedule) and the welcome banner
milestone: M3
status: todo
depends_on: [T-313]
spec_refs: [FE-16, SET-13, PRD-04]
estimated_size: S
---

## Goal
Right after setup, the owner sees exactly what to do next. Every role's home screen shows today at a glance.

## Context
- FE-16: when `reports` is enabled, widgets come from `/reports/widgets` (M6). **Until then (and whenever reports is disabled), render this fallback dashboard.**
- **Setup checklist** (spec 05 §2.5): generated from state, not stored. **Backend addition:** `GET /api/v1/setup/checklist` (owner, admin) → `[{key, done: bool, href}]`, computed in core from each module's `Module::checklist(tx, tenant)` (spec 03 §3; **add this trait method in code** with a default empty vec, plus `ChecklistItem {key, done, href, order}`). Items, in order:
  1. `set_prices`: any offering with `price_needs_review` → `/catalog?tab=prices` (scheduling).
  2. `add_instructors`: no active instructor membership → `/people?role=instructor` (identity).
  3. `add_students`: no student membership → `/people?role=student` (identity; enrollment adds "enable registration link" in M4).
  4. `connect_channels`: notify enabled and no channel configured → `/settings/channels` (notify, M5; until then the notify stub returns nothing).
  5. `location_pin`: attendance GPS on and no `geo` → `/settings/school` (attendance, M5).
  6. `add_closures`: no closure in the current local year → `/closures` (scheduling).
  7. `invite_admin`: no admin other than the owner → `/settings/admins` (identity).
  - A widget with a progress bar ("3 of 7 done"); done items are struck through and collapse after all are done. It is dismissible per user (localStorage `eq:checklist-dismissed:{tenant}`).
- **Welcome banner** (`?welcome=1`): "Your school is ready." with 3 shortcuts (Add a teacher, Create a schedule, Invite an admin). Dismiss removes the query param.
- **Owner/admin:** the checklist (if not complete), Today (sessions today grouped by resource with status badges; links to `/sessions/:id`), and the AttentionList (`GET /attention`, actions per kind: `materialize_conflict` → open the series date with alternatives).
- **Instructor:** "Today" (their sessions in order, the next one highlighted), and a check-in shortcut placeholder (M5).
- **Guardian/student:** next 7 days of their sessions (`/me/schedule`), grouped by day, with participant names for guardians.
- Empty, loading and error states for every widget.

## Scope
### In
- `/dashboard` page and widgets, the checklist endpoint, the `Module::checklist` trait method, and implementations for identity and scheduling.

### Out
- Reports widgets (M6).

## Acceptance criteria
- AC1. After setup (music preset, IDR) the checklist shows `add_instructors`, `add_students`, `add_closures`, `invite_admin` not done, and `set_prices` done. With SGD, `set_prices` is not done. → `api_setup::checklist_states` + `web/e2e/dashboard.spec.ts › checklist`
- AC2. After adding an instructor, that item is done on reload. → `checklist progresses`
- AC3. A guardian sees only their children's sessions. → `guardian home`
- AC4. A materialisation conflict appears in the admin's AttentionList with an action. → `attention visible`
- AC5. axe: 0 serious or critical for each role's dashboard.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/api_setup.rs` | `checklist_states` |
| `web/e2e/dashboard.spec.ts` | AC1–AC5 |

## Definition of done
- [ ] Rust + web checks + E2E pass; `make api` regenerated
- [ ] Commit `feat(web): role dashboards and setup checklist [T-314]`, signed off

## Report

## Questions
