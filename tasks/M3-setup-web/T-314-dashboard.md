---
id: T-314
title: Add the role dashboards (owner/admin today + attention + setup checklist, instructor today, guardian schedule) and the welcome banner
milestone: M3
status: todo
depends_on: [T-313]
spec_refs: [FE-16, SET-13, PRD-04, UX-01, UX-07]
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
- Feedback (spec 27, UX-01..07): use only these catalogue entries, through `useToast(id)`, `Dialog` and `Banner` (T-305). Screen states: the spec 27 §8 row for `/dashboard` (per-widget empty and error states). Toasts: `toast.core.attention_resolved`. Dialogs: none. Banners: `banner.setup_checklist`, `banner.notify_in_app_only`.

## Scope
### In
- `/dashboard` page and widgets, the checklist endpoint, the `Module::checklist` trait method, and implementations for identity and scheduling.

### Out
- Reports widgets (M6).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-api/tests/api_setup.rs` | test | Task-listed verification file. |
| `web/e2e/dashboard.spec.ts` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust + web checks + E2E pass; `make api` regenerated
- [ ] Commit `feat(web): role dashboards and setup checklist [T-314]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
