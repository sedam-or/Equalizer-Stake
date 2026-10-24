# 08 — Frontend (SPA)

## 1. Stack (normative)

| Concern | Choice |
|---|---|
| Package manager | pnpm 9 (`web/` is its own workspace root) |
| Build | Vite 6, TypeScript 5 (`strict`, `noUncheckedIndexedAccess`) |
| UI | React 19, Tailwind CSS 4, shadcn/ui (Radix primitives, copied into `src/components/ui`), lucide-react |
| Routing | TanStack Router (file-based, type-safe search params) |
| Server state | TanStack Query 5 |
| API client | `openapi-fetch` + `openapi-typescript` types generated from `web/openapi.json` (`pnpm gen:api`) |
| Forms | react-hook-form + `@hookform/resolvers` + zod 4 schemas **generated** from `web/openapi.json` by `@hey-api/openapi-ts` (zod plugin only), wrapped by `useRitmaForm` (VAL-30..34); a custom `PolicyForm` renderer + `ajv` (policy forms, FE-20) |
| Phone input | `libphonenumber-js` (min metadata), same canonical E.164 as the server (VAL-32) |
| Complex UI flows | `xstate` 5 (check-in capture and offline queue only, STM-96/97) |
| I18n | i18next + react-i18next + i18next-icu (ICU MessageFormat), catalogs in `src/locales/{en,id}/*.json` |
| Dates | `date-fns` 4 + `@date-fns/tz` (tenant timezone aware) |
| Charts | Recharts |
| Map pin | Leaflet used directly through a small `MapPin` wrapper (not `react-leaflet`, D-56), tile URL from `meta/app` (default OSM, configurable) |
| Face detection | `@mediapipe/tasks-vision` (lazy, check-in route only) |
| PWA | `vite-plugin-pwa` (Workbox): installable, offline shell, read-only cached schedule |
| Tests | Vitest + Testing Library + `@axe-core/react` (dev) / `vitest-axe`; Playwright e2e (`web/e2e`) |
| Lint/format | Biome; `knip` for dead code; `size-limit` for FE-02 |

The full frontend map with licenses is `24-open-source-stack.md` §3. Client UI state machines (auth session, view states, form submission, drafts, wizard, check-in) are STM-90..97.

**FE-01.** No SSR. The SPA is built to `web/dist` and embedded into the binary (`rust-embed`) by `eq-api`. Serving rules:
- `/assets/*` (hashed) get `Cache-Control: public, max-age=31536000, immutable`.
- `index.html`, `sw.js` and `manifest.webmanifest` get `no-cache`.
- Any non-`/api`, non-asset GET falls back to `index.html`.

During development, Vite on :5173 proxies `/api` to :8080.

**FE-02. Bundle budget.** Initial JS ≤ 200 KB gzip. Route chunks are lazy. MediaPipe, Leaflet and Recharts load only on the routes that use them. CI checks the budget with `size-limit`.

## 2. Source layout

```
web/src/
├── main.tsx                 # providers: QueryClient, i18n, Router, Theme, Auth
├── brand.ts                 # product name, logo paths (rename point)
├── app/
│   ├── router.tsx           # TanStack Router instance
│   ├── shell/               # AppShell, Sidebar, BottomNav, TopBar, RoleSwitcher, NotificationsBell
│   └── guards.ts            # requireAuth, requirePerm, requireModule loaders
├── routes/                  # file-based routes (thin: compose module pages)
├── modules/
│   ├── registry.ts          # module id → { nav, routes, widgets } (mirrors backend ids)
│   ├── setup/  scheduling/  enrollment/  billing/  attendance/  rental/  notify/  reports/  feedback/  settings/  identity/
│   │   ├── api.ts           # typed query/mutation hooks for this module
│   │   ├── pages/           # page components
│   │   └── components/      # module-specific components
├── components/
│   ├── ui/                  # shadcn primitives (Button, Input, Dialog, Sheet, Select, Tabs, Toast, Tooltip, ...)
│   └── domain/              # cross-module: ScheduleGrid, SlotPicker, PersonPicker, MoneyInput, MoneyText, StatusBadge, EmptyState, ErrorState, DataTable, PageHeader, PolicyForm, PolicySummary, ModuleCard, MapPin, FileDrop, Stepper, AttentionList
├── lib/
│   ├── api.ts               # openapi-fetch client + auth middleware + problem → Error mapping
│   ├── auth.ts              # in-memory token, refresh scheduling, role switch
│   ├── format.ts            # money, date, time, duration, ordinal via Intl (tenant locale/tz)
│   ├── i18n.ts              # i18next init; terminology interpolation ({{term.instructor}})
│   ├── drafts.ts            # FE-30
│   └── offline/             # IndexedDB queue (check-in), crypto helpers
├── locales/{en,id}/{common,errors,policy,modules.<id>}.json
└── styles/{tokens.css, app.css}
```

## 3. Bootstrapping and auth

- **FE-10.** On load:
  1. `GET /meta/app` (branding, locale, tenant).
  2. `GET /setup/status`. If setup isn't completed, redirect to `/setup`.
  3. `POST /auth/refresh` (cookie). On success, store the access token in memory and fetch `/me` and `/meta/modules`. On failure, go to the login route.
- **FE-11.** Access tokens are refreshed 60 s before `exp` with a single-flight refresh. A 401 on any call triggers one refresh, then a retry. A failed refresh logs out. Tabs share auth through `BroadcastChannel('eq-auth')`, so one tab refreshes and the others receive the token.
- **FE-12.** The SPA never stores tokens in localStorage. Logout calls `/auth/logout`, clears the Query cache, broadcasts, and redirects.

## 4. Module awareness

- **FE-15.** Navigation is built **only** from `GET /meta/modules → nav`, already permission-filtered, mapped through `modules/registry.ts` for icons and route components. A route whose module is disabled renders `<ModuleDisabled/>` with a link to Settings → Features for owners. Route loaders call `requireModule(id)` and `requirePerm(perm)`.
- **FE-16.** Dashboard widgets are listed from `/reports/widgets`, or from a fallback minimal dashboard (today + attention) when reports is disabled.

## 5. Policy forms (the 3-step promise)

- **FE-20. `PolicyForm` renderer.** Props: `{ module, schema, value, onChange, errors, group: 'essentials'|'advanced', currency, locale }`.
  - It walks the JSON Schema (object/properties, arrays, enums, `oneOf` for Rust enums with data). It picks widgets from `x-ui.widget`, then from the type: boolean → toggle, enum → select (≤ 4 options → segmented control), integer → number, string → input.
  - It handles `x-ui.visible_if` and `requires_module`.
  - Labels and help come from `t('policy.<module>.<path>.label|help')`.
  - Validation: `ajv` (draft 2020-12, `ajv-formats`) on change for field errors. Server preview (`POST …/preview`, debounced 300 ms) gives cross-field errors and the summary.
  - Rust enum representations: schemars emits externally tagged enums. The renderer supports:
    - unit variants as strings, e.g. `"freeze"`;
    - single-key objects for data variants, e.g. `{ "flat": { "amount": 100 } }`, rendered as a select of the variant plus the nested fields.
- **FE-21. `PolicySummary`.** It renders `SummaryLine[]` with ICU and the formatters (`$money`, `$minutes`). It is shown live next to the form, in a sticky column on desktop or a bottom sheet on mobile.
- **FE-22. Widgets** (implement all listed in POL-06):
  - `weekday-hours`: 7 rows, each with an open toggle and 1..3 time-range inputs, plus "Copy Monday to weekdays".
  - `tiers`: a table with add/remove rows and sort enforcement.
  - `money`: a locale-formatted input that stores minor units.
  - `percent`: stores bps (shows %, step 0.5).
  - `day-of-month`: a select 1–28 with ordinals.
  - `chips`: a number list with add/remove.
  - `textarea-i18n`: tabs per locale.
  - `map-pin`: Leaflet plus "Use my current location".
  - `offering-ref`: a select from `/catalog/offerings`.

## 6. Route map (v1)

`/:lang?` is not used. The locale comes from the person or tenant, with a switch in the profile. Public routes are prefixed as shown.

| Route | Module | Roles | Page |
|---|---|---|---|
| `/setup` (steps 1–3) | core | owner / setup token | Setup wizard (05 §2) |
| `/login`, `/forgot`, `/reset/:token`, `/invite/:token` | identity | public | Auth pages |
| `/join` | enrollment | public | Public registration (ENR-03) |
| `/book` | rental | public (if guests) | Public booking (RNT-10) |
| `/dashboard` | core/reports | all | Role dashboard (owner/admin widgets; instructor today + check-in; guardian students + invoices) |
| `/schedule` | scheduling | staff | ScheduleGrid (day/week, by resource or instructor) + create/reschedule/cancel |
| `/schedule/series`, `/schedule/series/:id` | scheduling | admin | Recurring slots |
| `/catalog` (offerings, levels, prices) | scheduling | admin | Catalog editor |
| `/resources`, `/closures` | scheduling | admin | Rooms, holidays (CSV import) |
| `/people` (+ `?role=`) , `/people/:id` | identity | staff | Directory and person detail (tabs per enabled module) |
| `/registrations`, `/registrations/:id` | enrollment | admin | Review and approve |
| `/students/:id` | enrollment | staff, guardian (linked) | Student profile: enrollments, schedule, attendance, credits, leave, feedback, eligibility |
| `/import` | enrollment | admin | CSV/XLSX import wizard |
| `/billing/invoices`, `/billing/invoices/:id` | billing | admin, payer (own) | Invoices |
| `/billing/payments` | billing | admin | Verification queue |
| `/billing/payers` | billing | admin | Restricted/frozen payers |
| `/checkin` | attendance | instructor | Today's sessions + camera/GPS check-in |
| `/attendance/review` | attendance | admin | Pending check-ins |
| `/sessions/:id` | scheduling | scoped | Session detail + marks (attendance) |
| `/rental`, `/rental/reservations` | rental | members / admin | Book a room / manage reservations, rate cards |
| `/feedback` | feedback | instructor, guardian | Notes and ratings |
| `/reports`, `/reports/print/:report` | reports | owner/admin | Dashboard, printables |
| `/audit` | audit | owner/admin | Audit log + verify |
| `/privacy` | audit | all | My data: export, deletion request |
| `/notifications` | notify | all | Inbox |
| `/settings/school`, `/settings/features`, `/settings/rules`, `/settings/rules/:module`, `/settings/channels`, `/settings/admins`, `/settings/branding` | core | owner (channels: admin too) | Settings (reusing setup components) |
| `/me` | identity | all | Profile, password, TOTP, sessions, notification prefs, calendar feed link |

**Guardian mobile navigation (bottom bar, max 5):** Home · Schedule · Invoices · Messages · Me.
**Instructor:** Today · Schedule · Check-in · Students · Me.
**Admin and owner** use the sidebar on desktop, and on mobile a bottom bar with Home · Schedule · People · Billing · More.

## 7. UX rules

- **FE-30. Drafts.**
  - Long forms (session create, invoice manual, registration review, policy forms) autosave to `localStorage` under `eq:draft:{tenant}:{person}:{form_key}` on change (debounced 2 s), and to the server with `PUT /me/drafts/{form_key}` every 30 s while dirty.
  - On open, a draft newer than the loaded entity prompts "Restore unsaved changes?".
  - Submitting deletes both copies.
  - A `DraftIndicator` shows Saved, Saving, or Not synced.
  - *(Core endpoints: `GET/PUT/DELETE /me/drafts/{form_key}`, TTL 7 days via a daily cleanup job.)*
- **FE-31. Conflict UX.** A 409 `slot_taken` opens a panel with the conflict explanation and alternatives as clickable chips. Clicking one fills in the form and resubmits.
- **FE-32. Frozen UX.** Payer state from `/me` (`payer_state`) shows a persistent banner. Disabled actions explain why in a tooltip, with a link to the invoice.
- **FE-33. Optimistic UI** only for marks and notification reads. Everything else waits for the server, because correctness comes before speed for scheduling and money.
- **FE-34. Empty, loading and error states** are mandatory for every list and widget (`EmptyState` with a primary action, `Skeleton`, `ErrorState` with retry plus the request id). Which error shows where is ERR-61 (`19-error-handling.md`).
- **FE-35. Offline.** The app shell and the last-fetched `/me/schedule` (14 days) are cached for read-only offline use. The check-in offline queue is ATT-07. Other mutations need connectivity, and the UI shows an offline toast with buttons disabled.
- **FE-36. Accessibility.** Every interactive element is reachable by keyboard and has a visible focus ring. Dialogs trap focus. Forms link errors with `aria-describedby`. Colour is never the only signal (status badges include text and an icon). Target size is ≥ 44×44 px on touch. axe has zero serious or critical violations in the Playwright smoke run of every page.
- **FE-37. Terminology.** UI copy never hard-codes "Teacher" or "Student". It uses `t('term.instructor')` and similar, resolved from `meta/app.tenant.terminology` with fallback to the locale catalog.
