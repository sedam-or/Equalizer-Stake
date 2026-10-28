---
id: T-310
title: Add the 3-step setup wizard UI (school + template, features, rules with live summary) with resume and back navigation
milestone: M3
status: todo
depends_on: [T-302b, T-306a, T-307, T-308, T-309]
spec_refs: [PRD-04, PRD-07, SET-10, SET-11, SET-12, SET-13, MOD-07, FE-21, FE-30, VAL-33, VAL-34, UX-01, UX-03, UX-06]
estimated_size: M
---

## Goal
A non-technical owner opens the app for the first time and has a working school in three screens, under 10 minutes, without reading docs. This is USP #1 and the hero GIF.

## Context
- Route `/setup` (spec 05 §2). Guard: `GET /setup/status`. `completed` → redirect `/dashboard`. `needs_token` → read `?token=` from the URL (the desktop and installer channels pass it), keep it in memory, and if absent show a token field with help "Find it in the server log or in setup-token.txt".
- Layout: a centred column (max 880 px). Stepper at the top (desktop: 3 labelled steps; mobile: "Step 2 of 3" + progress bar). Step 3 is 2-column ≥ lg (form 7/12, summary 5/12 sticky). Primary button bottom-right; sticky bottom bar on mobile.
- **Step 1 — Your school:**

| Field | Widget | Rule |
|---|---|---|
| School name | text 2–80 | required |
| Country | searchable Combobox from `/setup/countries` | default from `navigator.language` region |
| Language / Currency / Timezone | 3 read-only chips + "Edit" link that expands to 3 selects | derived from country |
| Template | 5 large cards from `/setup/presets` (icon, name, description, "Includes: …" module names) | **none pre-selected**; Next disabled until chosen |
| Number of rooms | stepper 1–50 (shown when `ask_room_count`), label uses `term.resource` plural of the chosen preset's terminology (preview) | default `default_room_count` |
| Your name, email, password, confirm | only when `needs_token` (self-host first run) | password min from `/meta/app.auth` |

  - Next → `POST /setup/school` (+ token). Success: adopt the returned access token (`auth.adopt`), then step 2.
  - After step 1 committed, "Back" from step 2 shows the same form in edit mode bound to `PATCH /setup/school`. The template cards are disabled with the note "To use a different template, reset setup", plus a "Reset setup" text button → confirm → `POST /setup/reset` (409 shows `errors.setup_reset_blocked`).
- **Step 2 — Features:** a grid of `ModuleCard`s from `/meta/modules` (compiled only), prefilled from the server state; dependency dialogs via `useModuleToggles`. Next → `PUT /setup/modules`.
- **Step 3 — Rules:** `GET /setup/rules`. One accordion card per enabled module, in server order. Each card shows:
  - a header (module name + first 2 summary lines),
  - **Essentials** (open),
  - **Advanced** (collapsed).

  Focus on a card makes it the "focused module" for the summary column. Edits keep a local document per module and call preview (debounced).
  - Inline helpers per spec 05 §2.4. In M3 only **attendance** (map pin writing `PATCH /locations/{id}` geo) and **notify** ("Connect channels" → link to `/settings/channels`, opening in a drawer later; for now a disabled button with the tooltip "Available after setup") apply. Billing's payment-instructions editor is a `textarea-i18n` widget rendered by PolicyForm. The helpers render only if the module is enabled **and** its policy is real (not a stub).
  - **Finish** → `POST /setup/complete {policies: {module: doc}}` (send only modules the user touched). 422 → scroll to the first error field and expand its section. Success → `/dashboard?welcome=1`.
- **Resume (SET-12):** a refresh at any step resumes at `current_step` from `/setup/status`. Step-3 edits that weren't submitted are kept in drafts (`eq:draft` localStorage + `PUT /me/drafts/setup.rules`) and restored with a prompt.
- **Welcome:** the dashboard shows a one-time welcome banner when `?welcome=1` (T-314 renders it; this task adds the query handling contract).
- Copy (en + id) in `modules.setup`, in the voice of spec 09 §6. Example step-1 title: "Tell us about your school" / "Ceritakan tentang sekolah Anda".
- Measured target (PRD-07 / TST-07): ≤ 10 user actions per step on the default path.
- Feedback (spec 27, UX-01..07): use only these catalogue entries, through `useToast(id)`, `Dialog` and `Banner` (T-305). Screen states: the spec 27 §8 row for `/setup`. Toasts: `toast.generic.network`, `toast.core.draft_restored`. Dialogs: `dialog.core.module_enable_deps`, `module_disable_dependents`, `module_disable_with_data`, `reset_setup`, `dialog.generic.draft_restore`.

## Scope
### In
- `src/modules/setup/*` (pages, step components, hooks), and the route `/setup` outside the authenticated shell.

### Out
- Sample data. Channels UI (M5).

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/e2e/setup.spec.ts` | test | Task-listed verification file. |
| `web/src/modules/setup/**/*.test.tsx` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T1, T3, T4 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- S/E: spoofed identity, unauthorized role or tenant scope → auth mode and permission checks precede data access; scope in SQL and RLS remain mandatory (SBD-40/41/42, AUTH-05, SEC-05).
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `validation blocks finish` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Playwright on a fresh DB: open `/setup?token=…` → step 1 (name, country Indonesia → id/IDR/Asia/Jakarta chips, Music school, 3 rooms, owner fields) → step 2 (unchanged) → step 3 (change buffer 10 → 15, see the summary update) → Finish → the dashboard welcome. Total user actions per step ≤ 10, and the run takes < 60 s in CI. → `web/e2e/setup.spec.ts › music school happy path`
- AC2. "Next → Next → Finish" without changes works for **each** of the 5 presets (parametrised). → `next next finish all presets`
- AC3. A reload on step 3 resumes at step 3 with unsaved edits offered for restore. → `resume`
- AC4. Turning on billing alone on step 2 shows the dependency dialog. → `dependency dialog`
- AC5. An invalid essentials value (buffer 999) blocks Finish, scrolls to the field and shows the error. → `validation blocks finish`
- AC6. axe: 0 serious or critical on every step, at 390 px and 1280 px.

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/setup.spec.ts` | AC1–AC6 |
| `web/src/modules/setup/**/*.test.tsx` | unit tests for the step-1 derivations and button enablement |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks + E2E pass; record the action counts per step in the Report
- [ ] Commit `feat(web): 3-step setup wizard [T-310]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
