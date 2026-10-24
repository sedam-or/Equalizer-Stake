---
id: T-307
title: Add the app shell (sidebar, top bar, bottom nav), module registry, route guards, ModuleDisabled page and the PWA
milestone: M3
status: todo
depends_on: [T-306]
spec_refs: [FE-15, FE-16, FE-35, MOD-09, PRD-03, STM-90, STM-91]
estimated_size: M
---

## Goal
After login, every role lands in a shell whose navigation is built from the server's enabled modules and the caller's permissions. Turning a module off removes it from the UI with no client change.

## Context
- FE-15: navigation comes **only** from `GET /meta/modules → nav` (already filtered). `src/modules/registry.ts` maps `nav.id` → `{icon component, route}` and module id → `{routes (lazy), widgets}`. An unknown `nav.id` is skipped with a `console.warn` in dev.
- Guards in `src/app/guards.ts` as TanStack Router `beforeLoad`:
  - `requireAuth`: `useSession` anonymous → redirect `/login?next=`.
  - `requirePerm(perm)`: missing → render `<Forbidden/>`.
  - `requireModule(id)`: disabled → render `<ModuleDisabled module=id/>`. Owners see a link "Turn on in Settings → Features"; others see "Not available at your school".
- Layout (spec 09 §5):
  - ≥ lg: a 264 px sidebar (collapsible to 72 px, state in localStorage `eq:sidebar`) with sections main/admin/settings, plus a top bar (tenant name + logo, a search placeholder, the notifications bell placeholder (hidden unless notify is enabled), the role switcher (if more than one role), and an avatar menu: Profile, Theme, Log out).
  - < lg: a top bar plus a **bottom navigation** of max 5 items per role (FE §6):
    - Guardian: Home · Schedule · Invoices · Messages · Me.
    - Instructor: Today · Schedule · Check-in · Students · Me.
    - Admin and owner: Home · Schedule · People · Billing · More (More opens a sheet with the rest of the nav).
    - Items whose module is disabled or whose nav id isn't present are dropped, and the bar never shows empty slots.
  - Content max-width 1280 px (admin) / 720 px (portal roles).
- RoleSwitcher calls `auth.switchRole`, then invalidates all queries and re-fetches `/me` and `/meta/modules`.
- Density follows the active role (T-304 rule) unless the user overrides it in `/me`.
- Placeholder routes: `/dashboard` (T-314 fills it), `/me` (T-308).
- PWA (FE-35) with `vite-plugin-pwa`:
  - Manifest: name from `brand.ts`, icons from `web/public/icons/` (192/512 maskable placeholders generated from the brand mark; real assets come later), `display: standalone`, theme colour from tokens at build time (light primary).
  - Workbox: precache the app shell. `NetworkFirst` for `/api/v1/me/schedule` with a 14-day max-age cache (read-only offline). Everything else under `/api` is `NetworkOnly`.
  - The app-level `banner.offline` while offline and the `toast.generic.back_online` toast on reconnect (spec 27 §5, §7), via `navigator.onLine` events, and a `useOnline()` hook for disabling buttons.
- The shell has skip-to-content, landmark roles, and focus moves to the page `h1` on navigation.

## Scope
### In
- `src/app/shell/*`, `src/app/guards.ts`, `src/modules/registry.ts`, the router setup with lazy routes, `ModuleDisabled`, `Forbidden`, `NotFound`, the PWA config and `useOnline`.

### Out
- Page contents.

## Acceptance criteria
- AC1. With `/meta/modules` mocked (msw) to exclude billing, no billing nav appears, and navigating to `/billing/invoices` shows ModuleDisabled with the owner link for owners only. → `shell.test.tsx › module disabled`
- AC2. A guardian at 390 px width sees a bottom bar of ≤ 5 items, and no sidebar. → `shell.test.tsx › guardian bottom nav`
- AC3. RoleSwitcher triggers `/auth/switch-role` and refetches modules. → `role switch refetch`
- AC4. A Lighthouse PWA "installable" check passes in CI (Playwright + `@lhci/cli` or the manifest + SW registration assertion). → `web/e2e/pwa.spec.ts`
- AC5. axe: 0 serious or critical violations on the shell in both layouts.

## Tests to write
| File | Test names |
|---|---|
| `web/src/app/shell/shell.test.tsx` | AC1–AC3, AC5 |
| `web/e2e/pwa.spec.ts` | `installable` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks pass; the bundle budget passes
- [ ] Commit `feat(web): app shell, module-aware nav and PWA [T-307]`, signed off

## Report

## Questions
