---
id: T-311
title: Add the settings pages (school, features, rules list and per-module editor with effective date and history, admins, branding)
milestone: M3
status: todo
depends_on: [T-310, T-306a]
spec_refs: [PRD-05, PRD-06, POL-05, MOD-07, DS-10, IDN-11, VAL-33, VAL-34]
estimated_size: M
---

## Goal
After setup, owners change features and rules with the same components as the wizard, in at most 3 steps: open the card, edit and read the summary, save now or from a date.

## Context
- Routes (owner unless noted; under the shell, `requirePerm`):
  - `/settings/school`: name, country-derived locale/currency/timezone (= CorePolicy edit through the policy API), week start, and terminology overrides (a table of TermKey × en/id inputs with placeholders showing the defaults).
  - `/settings/features`: the ModuleCard grid + `useModuleToggles` → `PUT /settings/modules`. Success toast "Features updated".
  - `/settings/rules`: a list of module cards (`GET /settings/policies`) showing the version, the first 2 summary lines, and a `pending_future` chip "Changes from {date}".
  - `/settings/rules/:module`, the editor (PRD-05 / spec 04 §8):
    1. PolicyForm (Essentials + Advanced) with PolicySummary.
    2. Footer: "Apply from: [Now] [Start of next month] [Pick date]" (SegmentedControl + date picker; dates are local midnight in the tenant TZ, converted to an instant), an optional note, and **Save rules**.
    3. `PUT` with `If-Match: "v<version>"`. 409 `version_conflict` → a dialog "Someone else changed these rules" with [Reload] (shows the diff of the summary lines) [Cancel].
    - A History tab: versions (version, effective_from, author, note), clicking one shows read-only form + summary.
    - Drafts: form key `settings.rules.<module>` (FE-30).
  - `/settings/admins` (owner, `identity.admins.manage`): a list of admins and owners, "Invite admin" (person create + invite with channel link → copy the URL), and remove role (409 `last_owner` message).
  - `/settings/branding` (owner): brand hue slider 0–359 with the live preview (a button, a link, a badge, a selected nav item and a chart swatch) using `lib/theme.ts`; the "We darkened your colour slightly for readability" note when adjusted; corner style SegmentedControl; logo upload (`POST /files` purpose logo, SVG/PNG ≤ 512 KB) → saved via CorePolicy `branding` PUT.
  - `/settings/channels`: placeholder page "Coming with notifications" (M5).
- Nav: settings items come from core's nav declarations. **Add them to `CoreModule::nav()`** in the backend: `settings.school`, `settings.features`, `settings.rules`, `settings.admins`, `settings.branding` (section settings, perms `core.policy.write` / `core.modules.write` / `identity.admins.manage`).
- Admins (`core.policy.read`) can view rules read-only (form disabled, no footer).

## Scope
### In
- The pages above, the backend nav additions, and the locale strings.

### Out
- Channels (M5). Privacy settings UI (M6).

## Acceptance criteria
- AC1. E2E: change the scheduling buffer with "Start of next month" → the list shows the pending chip. History shows v2 with that date. The `/sessions/check` behaviour before that date still uses the old buffer (API assertion). → `web/e2e/settings.spec.ts › future dated rule`
- AC2. Two browser contexts edit the same module; the second save gets the conflict dialog. → `version conflict`
- AC3. Disabling feedback in Features removes its nav immediately (refetched `/meta/modules`). → `toggle feature`
- AC4. Branding hue 60 → the preview button uses dark text (`primary-fg` = brand-950). For a hue where `lib/theme.ts` reports `adjusted: true`, the adjustment note is visible. → `branding.test.tsx`
- AC5. An admin sees rules read-only. → `admin read only`
- AC6. axe: 0 serious or critical on all settings pages.

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/settings.spec.ts` | AC1–AC3, AC5, AC6 |
| `web/src/modules/settings/branding.test.tsx` | AC4 |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks + E2E pass; `make api` regenerated
- [ ] Commit `feat(web): settings for features, rules and branding [T-311]`, signed off

## Report

## Questions
