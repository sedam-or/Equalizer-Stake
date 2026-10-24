---
id: T-309
title: Add PolicyForm (schema-driven renderer with every widget), PolicySummary and ModuleCard
milestone: M3
status: todo
depends_on: [T-306]
spec_refs: [FE-20, FE-21, FE-22, POL-06, POL-07, POL-10, POL-13, MOD-07, PRD-06]
estimated_size: M
---

## Goal
Any module's rules render as a friendly form from the server's JSON Schema, with no per-module UI code. The plain-language summary updates as the owner types. This is the heart of "your rules, not ours".

## Context
- Inputs: `GET /settings/policies/{module}` gives `{schema, document, summary, version}`. Live: `POST /settings/policies/{module}/preview {document}` → `{valid, errors, summary}`, debounced 300 ms (technical const). In setup, the same through `/setup/rules`.
- `PolicyForm` props (FE-20): `{module, schema, value, onChange, errors, group: 'essentials'|'advanced', currency, locale, enabledModules}`.
- Rendering rules:
  - Walk `object.properties` in `x-ui.order` order (missing → schema order). Show only fields whose `x-ui.group` equals the `group` prop. Nested objects inherit the parent's group unless they set their own.
  - Widget choice: `x-ui.widget` first; else boolean → toggle; enum with ≤ 4 options → SegmentedControl; enum → Select; integer/number → number input (`unit` suffix); string → input; array of numbers → chips; array of objects → `tiers`-like table.
  - **Rust enums (POL-13):** a `oneOf` of string consts → select. A `oneOf` mixing strings and single-key objects → a variant select plus the nested fields of the chosen data variant. Switching variants resets the nested value to the schema `default` (or an empty object filled with defaults).
  - `x-ui.visible_if {path, equals}` (JSON Pointer relative to the module doc) hides the field when false. `x-ui.requires_module` hides it when that module isn't in `enabledModules`. `widget: hidden` is never rendered.
  - Labels and help: `t('policy.<module>.<dotted path>.label')` / `.help`, where array indices are dropped from the path (e.g. `policy.billing.payment_methods.instructions.label`). Enum option labels are `t('policy.<module>.<path>.options.<value>')`.
  - Field anatomy (spec 09 §4): label · control · unit suffix · help (muted) · error.
  - The Advanced section is collapsed by default and remembers its open state per module (localStorage `eq:policy-adv:{module}`).
- Widgets (FE-22), each as its own component with tests:
  - `toggle`, `number`, `money` (MoneyInput, minor units, tenant currency), `percent` (bps ↔ %, step 0.5 → 50 bps), `duration` (minutes, with "1 h 30 min" helper text), `time` ("HH:MM"), `day-of-month` (select 1–28 with ordinals).
  - `weekday-hours`: 7 rows (week start from CorePolicy), each with an open toggle and 1..3 interval rows (start/end time inputs), "Copy Monday to weekdays", and inline overlap errors.
  - `tiers`: an editable table with add/remove, enforcing the sort on blur (descending by the first column), with column widgets from `items.properties` x-ui.
  - `chips`: a number list with add (enter) and remove (×).
  - `select`, `textarea-i18n` (tabs per locale: en, id), `map-pin` (lazy Leaflet: a map with a draggable marker + "Use my current location"; tile URL from `meta/app`), `offering-ref` (select from `GET /catalog/offerings`, value = key).
- Validation: ajv 2020-12 + ajv-formats compiled per schema (memoised). Field errors appear on blur or after the first submit. Server preview errors (cross-field) map by JSON Pointer to fields; unmatched errors show at the top.
- `PolicySummary` (FE-21): renders `SummaryLine[]` via `renderSummary` (T-306), with a module icon. States: updating (subtle pulse while a preview is in flight), invalid (shows the first error in place of the summary). Layout: a sticky column ≥ lg, a bottom sheet with a "Summary" handle on mobile.
- `ModuleCard` (MOD-07): icon tile 40 px, name, two-line description (`module.<id>.name/description`), switch; states on/off/locked (lock icon + tooltip "Always included")/has-data chip.
  - `useModuleToggles(modules)` implements the dependency dialogs:
    - Turning on X with unmet `requires_all` → dialog "X needs A. Turn on A too?" [Turn on both] [Cancel]. For `requires_any`, offer the first option.
    - Turning off X needed by Y… → "Turning off X also turns off Y and Z." [Turn off all] [Cancel].
    - Has data → an extra line: "Your data is kept. Turn it back on any time."
  - It returns the next enabled set. The caller persists it.
- i18n keys for scheduling, identity, audit and core policy paths in `src/locales/{en,id}/policy.json` (TST-09 requires every schema path). **Backend addition in this task:** a CLI subcommand `ritma policy-schemas --out <dir>` writes `<module>.schema.json` and `<module>.default.json` for every compiled module. `make api` also runs it into `web/src/policy-schemas/`, and CI's contract job checks it for drift. The script `pnpm i18n:policy-keys` reads those files and lists the required keys. **Write real human copy for both languages** (no placeholders).

## Scope
### In
- The `policy-schemas` CLI + make/CI wiring, `src/components/domain/policy/*` (PolicyForm, the widgets, PolicySummary), `ModuleCard`, `useModuleToggles`, the policy locale strings for the modules that exist, and the key-generation script.

### Out
- The wizard and settings pages (T-310, T-311).

## Acceptance criteria
- AC1. Given the SchedulingPolicy schema fixture (`web/src/policy-schemas/scheduling.schema.json`), the essentials group renders opening hours, buffer, durations, cancellation tiers and no-show, and advanced renders the rest. → `PolicyForm.test.tsx › groups`
- AC2. Changing `extra_occurrence` to `substitute` reveals offering_key, duration and consumes_credit. Switching back to `skip` removes them from the value. → `enum variants`
- AC3. `weekday-hours`: "Copy Monday to weekdays" copies to Tue–Fri only. An overlap shows an inline error. → `WeekdayHours.test.tsx`
- AC4. A server preview error at `/cancellation/notice_tiers/1/min_hours_before` highlights row 2 of the tiers table. → `server errors map to fields`
- AC5. `useModuleToggles`: enabling billing alone opens the dialog, and "Turn on both" yields `{enrollment, billing}`. → `useModuleToggles.test.ts`
- AC6. `pnpm i18n:check` passes, including every policy path for core, identity, audit and scheduling.
- AC7. axe: 0 violations for a form with every widget (a fixture schema `all-widgets.schema.json`).

## Tests to write
| File | Test names |
|---|---|
| `web/src/components/domain/policy/*.test.tsx` | AC1–AC4, AC7 |
| `web/src/components/domain/ModuleCard/useModuleToggles.test.ts` | AC5 |

## Definition of done
- [ ] Web checks pass; bundle budget passes (Leaflet lazy)
- [ ] Commit `feat(web): schema-driven policy forms [T-309]`, signed off

## Report

## Questions
