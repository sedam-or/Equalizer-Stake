# 09 — Design System

Register: **product UI**. The design serves daily operations. It is calm, dense where admins work, and large and friendly where parents tap on phones. Marketing visuals (landing page) live outside the app and follow `brand/BRAND.md`.

## 1. Principles

- **DS-01. Clarity over decoration.** No glassmorphism, gradients-as-surfaces, parallax or 3D in the app. (v0 shipped three.js/ogl effects in the app bundle; those are removed.)
- **DS-02. One source of truth.** Every colour, size, radius, shadow and duration is a CSS custom property defined in `web/src/styles/tokens.css`, in three layers: **primitive → semantic → component**. Components use **only** semantic or component tokens, never primitives or hex values. A Biome/stylelint rule plus a grep in CI fails on raw hex in `src/**/*.tsx|css` outside `tokens.css`.
- **DS-03. Themeable per tenant, safely.** A tenant can change the **brand hue**, the logo and the corner style. Nothing else changes, so every tenant theme stays accessible.
- **DS-04. Light and dark** from day one, because tokens make it cheap. v0 dropped dark mode only because it lacked tokens.

## 2. Tokens

### 2.1 Primitive colour (OKLCH)

The neutral scale `gray-{0,50,100,200,300,400,500,600,700,800,900,950}` uses hue 255 and chroma 0.005–0.02. The values below are normative for the default theme:

| Step | Neutral (L C H) | Brand (default hue 255) | Accent (hue 75) |
|---|---|---|---|
| 50 | 0.985 0.003 255 | 0.97 0.02 255 | 0.98 0.03 75 |
| 100 | 0.965 0.005 255 | 0.93 0.04 255 | 0.95 0.06 75 |
| 200 | 0.925 0.008 255 | 0.87 0.07 255 | 0.90 0.10 75 |
| 300 | 0.870 0.010 255 | 0.78 0.11 255 | 0.85 0.14 75 |
| 400 | 0.710 0.015 255 | 0.66 0.16 255 | 0.80 0.16 75 |
| 500 | 0.550 0.018 255 | 0.56 0.19 255 | 0.75 0.16 70 |
| 600 | 0.450 0.020 255 | 0.49 0.20 255 | 0.67 0.15 65 |
| 700 | 0.370 0.020 255 | 0.42 0.18 255 | 0.56 0.13 60 |
| 800 | 0.280 0.018 255 | 0.35 0.14 255 | 0.46 0.10 55 |
| 900 | 0.210 0.015 255 | 0.28 0.10 255 | 0.38 0.08 55 |
| 950 | 0.150 0.012 255 | 0.20 0.07 255 | 0.27 0.06 55 |

Status primitives use fixed hues: green 150, amber 75, red 25, blue 240, violet 300. They have the same L/C ramp as brand, with C capped at 0.17.

### 2.2 Semantic colour

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-bg` | gray-50 | gray-950 | Page |
| `--color-surface` | white (gray-0) | gray-900 | Cards, sheets, inputs |
| `--color-surface-sunken` | gray-100 | gray-950 | Table headers, wells |
| `--color-surface-raised` | white | gray-800 | Popovers, dialogs |
| `--color-border` | gray-200 | gray-800 | Default borders |
| `--color-border-strong` | gray-300 | gray-700 | Inputs, dividers that matter |
| `--color-text` | gray-900 | gray-50 | Body |
| `--color-text-muted` | gray-600 | gray-400 | Secondary (≥ 4.5:1 on bg/surface) |
| `--color-text-subtle` | gray-500 | gray-500 | Placeholders, meta (large text or non-essential only) |
| `--color-primary` | brand-600 | brand-400 | Primary actions, links, focus |
| `--color-primary-hover` | brand-700 | brand-300 | |
| `--color-primary-fg` | white or brand-950 (computed, §3) | brand-950 | Text on primary |
| `--color-primary-subtle` | brand-50 | brand-950 | Selected rows, active nav bg |
| `--color-accent` | accent-500 | accent-400 | Highlights, "new" chips, sparingly |
| `--color-success` / `-subtle` / `-fg` | green-600 / green-50 / white | green-400 / green-950 / green-950 | |
| `--color-warning` / … | amber-600 / amber-50 / gray-950 | amber-400 / amber-950 / amber-950 | |
| `--color-danger` / … | red-600 / red-50 / white | red-400 / red-950 / red-950 | |
| `--color-info` / … | blue-600 / blue-50 / white | blue-400 / blue-950 / blue-950 | |
| `--color-focus-ring` | brand-500 @ 40% | brand-400 @ 50% | 3 px ring + 1 px offset |
| `--color-overlay` | gray-950 @ 50% | black @ 60% | Modal scrim |

**Domain status tokens.** Each status maps to `{bg, fg, border, icon}`. `StatusBadge` uses them:

| Domain | Status → tone |
|---|---|
| Session | scheduled → info · completed → neutral · cancelled → danger (strikethrough time) |
| Participant | expected → neutral · blocked → warning (lock icon) · cancelled → neutral-muted |
| Invoice | draft → neutral · issued → info · partially_paid → warning · paid → success · overdue → danger · cancelled → neutral-muted · needs_manual_reference → warning |
| Payer | normal → (none) · restricted → warning · frozen → danger (snowflake icon) |
| Check-in | verified → success · pending_review → warning · rejected → danger |
| Enrollment | pending → info · active → success · on_leave → warning · ended → neutral-muted |
| Reservation | requested → info · pending_payment → warning · confirmed → success · completed → neutral · cancelled/expired/rejected → neutral-muted |

**Chart palette.** 8 categorical colours at L 0.62–0.72 and C 0.14, hues [255, 150, 75, 300, 25, 195, 120, 340], checked for ≥ 3:1 against the surface. Sequential palettes for heatmaps use the brand ramp 100→700.

### 2.3 Typography

- Families: **Plus Jakarta Sans** (display/headings; designed in Indonesia, a nod to the project's origin) and **Inter** (UI/body), both SIL OFL. They are **self-hosted** as woff2 subsets (latin + latin-ext) under `/assets/fonts`, with no runtime Google Fonts (privacy, offline). `font-display: swap`. Monospace is `ui-monospace, "JetBrains Mono"` (fallback stack only).
- Numbers: `font-variant-numeric: tabular-nums` on money, times, tables and counters.

| Token | Size / line-height | Weight | Family | Use |
|---|---|---|---|---|
| `--text-display` | 36/44 | 700 | Jakarta | Setup welcome, empty hero |
| `--text-h1` | 28/36 | 700 | Jakarta | Page titles |
| `--text-h2` | 22/30 | 600 | Jakarta | Section titles |
| `--text-h3` | 18/26 | 600 | Jakarta | Card titles |
| `--text-body-lg` | 16/24 | 400 | Inter | Portal (guardian/instructor) body, mobile default |
| `--text-body` | 14/20 | 400 | Inter | Admin body, tables |
| `--text-sm` | 13/18 | 400 | Inter | Meta, helper text |
| `--text-xs` | 12/16 | 500 | Inter | Badges, captions (never for essential info) |

**Density rule:**
- Portal roles (guardian, student, instructor) use `--text-body-lg` and a 48 px control height on mobile.
- Admin screens on ≥ lg use `--text-body` and 40 px controls.
- This is set by a `data-density="comfortable|compact"` attribute on `<html>`, chosen from the active role. Users can override it in their profile.

### 2.4 Space, size, radius, elevation, motion

- **Space** (4 px base): `--space-0..12` = 0, 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 64 px.
- **Control heights:** `--control-sm` 32, `--control-md` 40, `--control-lg` 48 px. The touch target minimum is 44 px.
- **Radius** (the tenant "corner style" = `sharp | default | round` multiplies these by 0.5 / 1 / 1.5): `--radius-sm` 6, `--radius-md` 8, `--radius-lg` 12, `--radius-xl` 16, `--radius-full` 9999.
- **Elevation:**
  - `--shadow-1`: 0 1px 2px oklch(0 0 0 / .06)
  - `--shadow-2`: 0 4px 12px oklch(0 0 0 / .08)
  - `--shadow-3`: 0 12px 32px oklch(0 0 0 / .12)
  - Dark mode replaces shadows with `--color-border` plus a 1-step lighter surface.
- **Motion:**
  - `--duration-fast` 120 ms, `--duration-base` 200 ms, `--duration-slow` 320 ms.
  - `--ease-standard` cubic-bezier(.2, 0, 0, 1), `--ease-exit` cubic-bezier(.4, 0, 1, 1).
  - `prefers-reduced-motion: reduce` sets every duration to 0 ms except opacity fades (100 ms).
- **Breakpoints:** sm 640, md 768, lg 1024, xl 1280, 2xl 1536. The layout is mobile-first and works from 320 px.
- **Z-index:** `--z-sticky` 10, `--z-nav` 20, `--z-overlay` 40, `--z-modal` 50, `--z-toast` 60, `--z-tooltip` 70.

### 2.5 Tailwind 4 bridge

`tokens.css` defines the variables, and `app.css` maps them in `@theme`:

```css
@import "tailwindcss";
@import "./tokens.css";
@theme inline {
  --color-bg: var(--color-bg);
  --color-surface: var(--color-surface);
  --color-primary: var(--color-primary);
  /* …every semantic colour… */
  --radius-md: var(--radius-md);
  --font-sans: "Inter", ui-sans-serif, system-ui, sans-serif;
  --font-display: "Plus Jakarta Sans", var(--font-sans);
}
@custom-variant dark (&:where([data-theme=dark], [data-theme=dark] *));
```

Utilities like `bg-surface text-text-muted rounded-md` are then valid, and raw palette utilities (`bg-blue-600`) are disabled by not importing Tailwind's default colours (`@theme { --color-*: initial; }` before the inline block).

## 3. Tenant theming algorithm

**DS-10.** Inputs: `brand_hue` (0–359, default 255), `corner_style`, and the logo (SVG or PNG ≤ 512 KB, sanitised: SVG through an allow-list sanitiser and no scripts).

1. Generate brand-50…950 using the L/C ramp in §2.1 with the tenant hue. Clamp chroma per step to stay inside the sRGB gamut (`culori` `toGamut('rgb')`).
2. Choose `--color-primary-fg`: white if contrast(white, brand-600) ≥ 4.5, otherwise brand-950. For hues 60–120 (yellows) this picks the dark text automatically.
3. Verify: text-on-primary ≥ 4.5:1, primary-on-surface ≥ 3:1 (UI component), and link colour on bg ≥ 4.5:1. If any check fails, shift brand-600 L down in 0.02 steps until it passes, and report the adjustment to the owner in Settings → Branding ("We darkened your colour slightly for readability").
4. Output a `<style id="tenant-theme">` with overrides, generated client-side from `meta/app.brand` and cached.

The settings screen shows a live preview of a button, a link, a badge, a selected nav item and a chart swatch.

## 4. Components (specs)

Each component has a Storybook-like **demo route** `/__ds` (dev builds only) that shows every variant and state, and a Playwright visual snapshot (TST-12).

| Component | Variants / sizes | States | Notes |
|---|---|---|---|
| **Button** | primary, secondary (border), ghost, destructive, link; sm/md/lg; icon-only (requires `aria-label`) | default, hover, active, focus-visible, disabled, loading (spinner replaces icon, width locked) | One primary per view region. Destructive actions confirm via Dialog |
| **Input / Textarea / Select / Combobox** | md/lg | default, focus, invalid (border danger + message), disabled, read-only | Label always visible (no placeholder-as-label). Helper and error text below |
| **MoneyInput** | — | as Input | Shows the currency symbol prefix from Intl. Stores minor units. Paste-tolerant (`Rp 1.250.000` → 125000000) |
| **Toggle (Switch)** | md | on/off/disabled | Label left, switch right. Whole row clickable |
| **SegmentedControl** | 2–4 options | | Replaces select for short enums |
| **StatusBadge** | tone from §2.2 | | Icon + text. Never colour-only |
| **Card** | default, interactive (hover border-strong), selected | | Padding `--space-6` (compact `--space-5`) |
| **DataTable** | compact/comfortable | loading (5 skeleton rows), empty, error | Below md it renders as a **card list**, with primary column as title, 2–3 key fields and a trailing action. Sticky header ≥ md. Row click opens detail. Bulk select only where an API exists |
| **Dialog / Sheet** | Dialog ≤ md content; Sheet (bottom on mobile, right on desktop) for forms | | Focus trap, Esc closes unless dirty (then confirm) |
| **Toast** | success, error (with request id copy), info | | Bottom-center mobile, bottom-right desktop. 5 s, pausable |
| **PageHeader** | title, description, primary action, secondary menu, breadcrumbs ≥ md | | |
| **EmptyState** | icon, title, one-line explanation, primary action | | Copy explains *why* it's empty and *what to do* |
| **Stepper** | 3 steps (setup) | current, done, upcoming | Mobile shows "Step 2 of 3" + progress bar |
| **ModuleCard** | on/off/locked/has-data | | Icon tile 40 px, name, 2-line description, toggle; locked shows a lock and the tooltip "Always included" |
| **PolicyForm field** | anatomy: label · control · unit suffix · help (muted) · error | | Advanced group collapsed by default and remembers its state per module |
| **PolicySummary** | list of sentences with an icon per module | updating (subtle pulse), invalid (shows the first error instead) | |
| **ScheduleGrid** | day (resources as columns) / week (days as columns, filter one resource or instructor) | | §4.1 |
| **SlotPicker** | mobile-first list of free starts grouped by morning/afternoon/evening | | Used by booking, make-up, reschedule |
| **AvatarInitials** | 24/32/40/64 | | Background from a deterministic hash of the person id into 8 hues at L 0.85 (light) / 0.35 (dark) |
| **AttentionList** | — | | Items with severity dot, one-line text, primary action button |

### 4.1 ScheduleGrid (detailed)

- **Axes.**
  - Vertical time axis from the earliest opening to the latest closing of the visible days, with 15-minute minor lines and hour labels (`HH:mm`, tenant TZ).
  - Closed times are shaded with `--color-surface-sunken` plus a diagonal hatch.
  - Closures (holidays) are full-column shaded with the name label.
- **Session block.**
  - Colour comes from `offering.color`, or else from the category palette by offering index.
  - Layout: a 4 px left border, a light tinted fill, the title (offering), and a subtitle (participants or "3/6") and instructor.
  - Status styles: cancelled is hatched and line-through, completed is muted, and a block containing blocked participants shows a lock icon.
  - The **buffer** is drawn as a hatched tail after the block, so staff can see why the next slot isn't free.
- **Now line:** a 2 px `--color-danger` line with a dot, auto-scrolled into view on open.
- **Interactions:**
  - Desktop: click an empty cell to open the "New session" sheet pre-filled; drag a block to reschedule (calls `/sessions/check` live while dragging and shows red/green ghosting; drop then confirms).
  - Mobile: a day view with resources as horizontal swipe tabs; tap a slot to open a sheet.
  - Keyboard: arrow keys move the focused cell, Enter opens it.
- **Density:** 1 h = 64 px on desktop and 72 px on mobile.
- **Performance:** virtualise columns beyond 12 resources. Render ≤ 300 blocks per view without jank (≤ 16 ms per frame on mid-range Android). Test with 500 seeded sessions.
- **A11y:** each block is a button with `aria-label="Piano, Ayu, 15:00 to 15:45, Room 2, scheduled"`. The grid has `role="grid"` with row and column headers.

## 5. Layout patterns

- **App shell:**
  - ≥ lg: a 264 px sidebar (collapsible to 72 px) plus a top bar (tenant name, search, notifications, role switcher, avatar).
  - < lg: a top bar plus a **bottom navigation** of 5 items (FE §6).
  - Content max-width 1280 px for admin and 720 px for the portal.
- **Detail pages:** a header with key facts as chips, then tabs per enabled module (e.g. a person page has Profile, Enrollments, Schedule, Invoices, Attendance, Feedback).
- **Forms:** a single column ≤ 640 px wide. Related fields go in 2-column grids ≥ md. Primary submit at bottom-right on desktop, and a sticky bottom bar on mobile.
- **Setup wizard:** a centred column of 880 px max. Step 3 uses a 2-column layout ≥ lg (form 7/12, summary 5/12 sticky).

## 6. Content style (UI copy)

- Plain words, sentence case, no jargon. "Late fee" instead of "Penalty surcharge". In Bahasa Indonesia, everyday terms: "Denda keterlambatan", "SPP", "Wali murid".
- Errors say what happened and what to do, e.g. "Room 2 is taken at 15:00. Try 15:50 or Room 3."
- Buttons are verbs: "Save rules", "Send invoice", "Approve".
- Never blame the user. Never expose internal codes except the request id in error toasts.
- Numbers, dates and money always go through formatters (locale + tenant TZ + currency).
