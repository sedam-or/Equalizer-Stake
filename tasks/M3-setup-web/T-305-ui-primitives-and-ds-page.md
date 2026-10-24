---
id: T-305
title: Add the UI primitives (shadcn-based) and cross-domain components, with the /__ds demo route
milestone: M3
status: todo
depends_on: [T-304]
spec_refs: [DS-01, FE-34, FE-36, TST-12]
estimated_size: M
---

## Goal
A small, complete component kit that every screen composes. Each component shows every state on one dev page, which is visually snapshot-tested.

## Context
Component specs (spec 09 §4). Implement exactly these variants and states:

| Component | Variants / sizes | States / rules |
|---|---|---|
| Button | primary, secondary, ghost, destructive, link; sm/md/lg; icon-only requires `aria-label` (TS-enforced via a discriminated union) | default, hover, active, focus-visible, disabled, loading (spinner replaces icon, width locked) |
| Input, Textarea, Select, Combobox | md/lg | default, focus, invalid (danger border + message), disabled, read-only; label always visible; helper and error below, linked with `aria-describedby` |
| MoneyInput | — | currency prefix from `Intl`; stores minor units; paste-tolerant (`Rp 1.250.000` → 125000000 for IDR exponent 2) |
| Switch (Toggle row) | md | label left, switch right, whole row clickable |
| SegmentedControl | 2–4 options | radio-group semantics |
| StatusBadge | tone from the domain status tokens | icon + text, never colour only; prop `domain` + `status` → tone/icon/label key `status.<domain>.<status>` |
| Card | default, interactive, selected | padding `--space-6` (compact `--space-5`) |
| DataTable | compact/comfortable | loading (5 skeleton rows), empty, error; below md renders as a card list (primary column = title, 2–3 key fields, trailing action); sticky header ≥ md; row click |
| Dialog, Sheet | Sheet = bottom on mobile, right on desktop | focus trap; Esc closes unless dirty (then confirm) |
| Toast | success, error (with "Copy request id"), info | bottom-center mobile, bottom-right desktop; 5 s, pausable on hover/focus |
| PageHeader | title, description, primary action, secondary menu, breadcrumbs ≥ md | |
| EmptyState | icon, title, one line, primary action | |
| ErrorState | title, message, Retry, request id | |
| Skeleton | line, block, avatar | |
| Stepper | 3 steps | current/done/upcoming; mobile "Step 2 of 3" + progress bar |
| AvatarInitials | 24/32/40/64 | background from a deterministic hash of the id into 8 hues at L 0.85 (light) / 0.35 (dark), via CSS vars from tokens |
| AttentionList | — | severity dot, one-line text, primary action |
| Tooltip, Tabs, DropdownMenu, Popover, Checkbox, RadioGroup | shadcn defaults restyled with tokens | |

- Base on shadcn/ui (copy into `src/components/ui`, restyle with semantic tokens only) + Radix + lucide-react.
- **`/__ds` route** (included only when `import.meta.env.DEV` or the `VITE_DS=1` build flag): one section per component, every variant × state, a light/dark toggle, a density toggle, and a hue slider (applies `lib/theme.ts`).
- i18n keys for built-in copy (`common.retry`, `common.copy_request_id`, `common.loading`, …) go in `src/locales/{en,id}/common.json`. **i18n bootstrapping comes in T-306.** Here, use a minimal `t` shim that T-306 replaces (`src/lib/i18n-shim.ts`, deleted in T-306).
- Accessibility: every component passes `vitest-axe` with zero violations in its test. Touch targets ≥ 44 px in comfortable density.

## Scope
### In
- `src/components/ui/*`, `src/components/domain/{MoneyInput, MoneyText, StatusBadge, EmptyState, ErrorState, PageHeader, Stepper, AvatarInitials, AttentionList, DataTable}`, the `/__ds` route, and component tests.

### Out
- PolicyForm, ModuleCard and ScheduleGrid (later tasks).

## Acceptance criteria
- AC1. Every component has a Vitest + axe test, and all pass. → `*.test.tsx`
- AC2. `MoneyInput` paste of `Rp 1.250.000` in IDR → value 125000000. `$12.50` in USD → 1250. → `MoneyInput.test.tsx › paste tolerant`
- AC3. `DataTable` renders as a card list at 375 px width (matchMedia mock). → `DataTable.test.tsx › mobile card list`
- AC4. A Playwright visual snapshot of `/__ds` in light/dark × compact/comfortable (4 shots) is committed. → `web/e2e/ds.spec.ts`
- AC5. The raw-colour guard passes.

## Tests to write
| File | Test names |
|---|---|
| `web/src/components/**/*.test.tsx` | per component: `renders variants`, `a11y` (+ specific ones above) |
| `web/e2e/ds.spec.ts` | `design system snapshots` |

## Definition of done
- [ ] Web checks pass; the size-limit budget (initial JS ≤ 200 KB gzip) still passes
- [ ] Commit `feat(web): UI primitives and design-system page [T-305]`, signed off

## Report

## Questions
