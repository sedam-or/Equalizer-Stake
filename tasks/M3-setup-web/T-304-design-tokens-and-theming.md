---
id: T-304
title: Add the full design tokens (3 layers), Tailwind 4 bridge, dark mode, density, self-hosted fonts and tenant theming
milestone: M3
status: todo
depends_on: [T-302]
spec_refs: [DS-01, DS-02, DS-03, DS-04, DS-10]
estimated_size: M
---

## Goal
Every visual value comes from one token file. Tenants can safely change hue, corners and logo, and every combination stays WCAG AA. Raw colours can't sneak into components.

## Context (normative values: copy exactly)
- **Primitive OKLCH table** (spec 09 §2.1). Neutral, brand (hue 255) and accent:

| Step | Neutral | Brand | Accent |
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

  - `gray-0` = white. Status ramps green 150, amber 75, red 25, blue 240, violet 300 use the brand L/C ramp with C capped at 0.17.
- **Semantic tokens:** the complete table in spec 09 §2.2 (bg, surface, surface-sunken, surface-raised, border, border-strong, text, text-muted, text-subtle, primary, primary-hover, primary-fg, primary-subtle, accent, success/warning/danger/info with `-subtle` and `-fg`, focus-ring, overlay), each in light and dark. **Copy that table into `tokens.css` comments and implement every row.**
- **Domain status tokens** (spec 09 §2.2): `--status-<domain>-<status>-{bg,fg,border}`, generated from the tone table: session, participant, invoice, payer, check-in, enrollment, reservation. Tones: info, neutral, neutral-muted, success, warning, danger.
- **Chart palette:** `--chart-1..8` at L 0.62–0.72, C 0.14, hues [255, 150, 75, 300, 25, 195, 120, 340].
- **Typography** (spec 09 §2.3):
  - Sizes: `--text-display 36/44 700 Jakarta`, `--text-h1 28/36 700`, `--text-h2 22/30 600`, `--text-h3 18/26 600`, `--text-body-lg 16/24 400 Inter`, `--text-body 14/20`, `--text-sm 13/18`, `--text-xs 12/16 500`.
  - Fonts are self-hosted woff2 (latin + latin-ext) in `web/public/assets/fonts/` from the official OFL releases, with OFL license files included. `font-display: swap`. **No Google Fonts at runtime.**
- **Space** `--space-0..12` = 0, 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 64 px. **Controls** 32/40/48. **Radius** sm 6, md 8, lg 12, xl 16, full 9999, multiplied by `--radius-scale` (sharp 0.5, default 1, round 1.5). **Shadows** 1/2/3 as specified; dark mode uses borders instead. **Motion** fast 120, base 200, slow 320, ease-standard `cubic-bezier(.2,0,0,1)`, ease-exit `cubic-bezier(.4,0,1,1)`; reduced motion → 0 ms except opacity (100 ms). **Z** sticky 10, nav 20, overlay 40, modal 50, toast 60, tooltip 70.
- **Density:** `<html data-density="comfortable|compact">`. Comfortable = body-lg + 48 px controls; compact = body + 40 px. The default comes from the active role (guardian, student, instructor → comfortable; owner, admin → compact on ≥ lg, comfortable below).
- **Theme:** `<html data-theme="light|dark">`, following `prefers-color-scheme` unless the user overrides it (stored in localStorage `eq:theme`; this is a UI preference, not a token).
- **Tailwind 4 bridge** (spec 09 §2.5): `@theme { --color-*: initial; }`, then `@theme inline { …every semantic token… }`, plus `@custom-variant dark (&:where([data-theme=dark], [data-theme=dark] *));`. Raw palette utilities must not exist.
- **Tenant theming algorithm** (DS-10), in `web/src/lib/theme.ts` using `culori`:
  1. Generate brand-50…950 from `brand_hue` using the L/C ramp, clamped into the sRGB gamut (`toGamut('rgb')`).
  2. `primary-fg` = white if contrast(white, brand-600) ≥ 4.5, else brand-950.
  3. Verify text-on-primary ≥ 4.5, primary-on-surface ≥ 3, and link-on-bg ≥ 4.5 (WCAG 2 contrast). On failure, lower brand-600 L by 0.02 steps until it passes, and return `{adjusted: true, deltaL}`.
  4. Inject `<style id="tenant-theme">` with the overrides. Inputs come from `meta/app.brand` (`brand_hue`, `corner_style`).
- **Guard:** `web/scripts/check-raw-colors.mjs` fails on `#[0-9a-f]{3,8}\b`, `rgb(`, `hsl(`, `oklch(` in `src/**/*.{ts,tsx,css}` outside `src/styles/tokens.css` and `src/lib/theme.ts`, and on Tailwind palette classes (`(bg|text|border|ring|fill|stroke)-(red|blue|green|gray|slate|zinc|neutral|stone|amber|yellow|lime|emerald|teal|cyan|sky|indigo|violet|purple|fuchsia|pink|rose|orange)-\d`). Add it to `pnpm lint`.

## Scope
### In
- `web/src/styles/tokens.css`, `app.css`, the fonts, `lib/theme.ts` (+ tests), `lib/density.ts`, the guard script, and the provider in `main.tsx` that applies theme and density.

### Out
- Components (T-305).

## Acceptance criteria
- AC1. Vitest: for every hue 0..359 step 1, the generated theme passes all three contrast checks after adjustment. → `theme.test.ts › all hues accessible`
- AC2. Hue 90 (yellow) → `primary-fg` is brand-950 (dark text). Hue 255 → white. → `theme.test.ts › fg choice`
- AC3. The guard script fails on a fixture file containing `bg-blue-600` and `#fff`. → `check-raw-colors.test.ts`
- AC4. Light and dark semantic tokens resolve (snapshot of computed styles in jsdom for 5 tokens). → `tokens.test.ts`

## Tests to write
| File | Test names |
|---|---|
| `web/src/lib/theme.test.ts` | `all hues accessible`, `fg choice`, `corner style scale` |
| `web/scripts/check-raw-colors.test.ts` | `detects raw colors` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks pass (`pnpm -C web lint && typecheck && test`)
- [ ] Commit `feat(web): design tokens and tenant theming [T-304]`, signed off

## Report

## Questions
