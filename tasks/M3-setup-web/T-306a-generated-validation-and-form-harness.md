---
id: T-306a
title: Generate zod schemas from OpenAPI and add the shared form harness (sanitisers, error mapping, submit discipline)
milestone: M3
status: todo
depends_on: [T-306, T-114, T-102a]
spec_refs: [VAL-01, VAL-03, VAL-11, VAL-12, VAL-23, VAL-30, VAL-31, VAL-32, VAL-33, VAL-34, VAL-35, VAL-54, CAN-22, ERR-31, ERR-60, ERR-61, ERR-83, FE-34, FE-36, SBD-45, OSS-02]
estimated_size: M
---

## Goal
Every form in the SPA validates with schemas **generated** from the server's OpenAPI document, sanitises values exactly like the server, shows server errors on the right fields, and submits safely (no double submit, idempotency keys reused on retry). Client and server agree on the same test cases.

## Context
- VAL-01: the server is the authority; the client repeats structural rules for instant feedback only.
- VAL-30: `pnpm gen:api` runs (1) `openapi-typescript` → `src/api/schema.gen.ts` (existing, T-306), (2) `@hey-api/openapi-ts` with **only** the zod plugin → `src/api/zod.gen.ts`, (3) `scripts/gen-field-meta.ts` → `src/api/fields.gen.ts` (`component → { sanitize: Profile }` from `x-sanitize`). Generated files are committed and checked by the contract job (TST-10).
- VAL-11/12 profiles (`line`, `multiline`, `identifier`, `email`, `phone`, `secret`, `none`) and the forbidden-character classes, mirrored from T-102a. Use `String.prototype.normalize`, Unicode property regexes (`\p{Cc}`, `\p{M}`), explicit bidi ranges; phones with `libphonenumber-js` (`parsePhoneNumberWithError(input, tenantCountry)`, `.isValid()`, `.number`).
- VAL-31: `contracts/validation/cases/*.json` (from T-102a) must pass on the client too: same `ok`, same canonical value, same code.
- VAL-33: react-hook-form with `@hookform/resolvers/zod`, `mode: "onTouched"`, `reValidateMode: "onChange"`; sanitise on blur and on submit (never per keystroke); focus the first invalid field; error summary with links; `aria-invalid` + `aria-describedby`; messages `t('errors.field.<code>', params)`.
- zod 4 issue → VAL-03 code mapping (VAL-33): `too_small`/`too_big` → `too_short`/`too_long` (strings), `too_few_items`/`too_many_items` (arrays), `must_be_gte`/`must_be_lte` (numbers); `invalid_format` → `invalid_format`; `invalid_value` → `not_allowed`; `invalid_type` → `invalid_type` or `required` (undefined/null); `unrecognized_keys` → `not_allowed`; `custom` carries `params.code`.
- Server 422 → form (VAL-33): pointer `/students/0/name` → RHF path `students.0.name` with `setError`; unmatched pointers and `policy_violation` without a matching field → the error summary.
- VAL-34: submit disabled while pending; one `Idempotency-Key` per form instance (`crypto.randomUUID()`), reused on retry, rotated after success or an edit; mutations never auto-retry; client never sends totals/prices/status.
- VAL-23: policy-driven bounds (password minimum) come from `GET /meta/app` → `rules.password_min_length`.
- SBD-45: `href`/`src` from data only `https:`, `mailto:`, `tel:`, same-origin relative.

## Scope
### In
- `web/package.json` script `gen:api` extended as above; `@hey-api/openapi-ts`, `zod`, `react-hook-form`, `@hookform/resolvers`, `libphonenumber-js` added.
- `web/scripts/gen-field-meta.ts`.
- `web/src/lib/validation/`: `sanitize.ts` (profiles), `zodToFieldCode.ts`, `withSanitize.ts` (wraps a generated schema with `z.preprocess` per `fields.gen.ts`), `serverErrors.ts` (pointer → RHF path, `applyProblemToForm(problem, form)`), `cases.test.ts` (VAL-31 runner).
- `web/src/lib/forms/useRitmaForm.ts`: RHF + resolver + sanitise-on-blur + server-error mapping + `Idempotency-Key` lifecycle + submit discipline (STM-92).
- `web/src/components/domain/FormErrorSummary.tsx` and `FieldError.tsx` (a11y wiring).
- `web/src/lib/safeUrl.ts`.
- `errors.field.*` messages for every VAL-03 code in `src/locales/{en,id}/errors.json`.
- A Biome/grep guard in `scripts/check-web-boundaries.sh`: no `z.string().max(`/`.min(` literal bounds under `web/src/modules/**` (use generated schemas).

### Out
- Individual pages and forms (T-308 onward use the harness).
- `PolicyForm` (T-309 keeps ajv; it reuses `zodToFieldCode`-style mapping for ajv keywords per VAL-22).

## Files
| Path | Action |
|---|---|
| `web/package.json`, `web/openapi-ts.config.ts` | modify / create |
| `web/scripts/gen-field-meta.ts` | create |
| `web/src/api/zod.gen.ts`, `web/src/api/fields.gen.ts` | generated |
| `web/src/lib/validation/*.ts`, `web/src/lib/forms/useRitmaForm.ts`, `web/src/lib/safeUrl.ts` | create |
| `web/src/components/domain/FormErrorSummary.tsx`, `FieldError.tsx` | create |
| `web/src/locales/{en,id}/errors.json` | modify |
| `scripts/check-web-boundaries.sh` | modify |

## Interfaces
```ts
export type Profile = "line" | "multiline" | "identifier" | "email" | "phone" | "secret" | "none";
export function sanitize(profile: Profile, input: string, ctx: { country: string }): { ok: true; value: string | null } | { ok: false; code: FieldCode };

export function useRitmaForm<S extends z.ZodTypeAny>(opts: {
  schema: S;                       // generated, e.g. zPersonCreate
  defaultValues: z.input<S>;
  idempotent?: boolean;            // money/booking forms
  onSubmit: (values: z.output<S>, ctx: { idempotencyKey?: string }) => Promise<void>;
}): UseFormReturn<z.input<S>> & { submit: () => Promise<void>; state: FormSubmitState };

export function applyProblemToForm(problem: ApiProblem, form: UseFormReturn<any>): { unmatched: FieldError[] };
export function safeHref(url: string): string | undefined;
```

## Business rules
None (all rules come from the generated schemas or the server).

## Security
- Triggers: T1 (public forms `/join`, `/book` use the harness), T3 (forms handle S2 values).
- Threats → controls: **T**ampering via client-only checks → server is authoritative (VAL-01), client never sends totals (VAL-34); **I**nformation disclosure via `javascript:` links in data → `safeHref` (SBD-45); duplicate charges by double submit → idempotency key lifecycle (API-08).
- Abuse-case acceptance criteria: `safeHref("javascript:alert(1)")` → `undefined` → `safeUrl.test.ts: blocks_javascript_scheme`; double-clicking submit sends one request → `useRitmaForm.test.tsx: single_submit_while_pending`; a retry after a network error reuses the key → `useRitmaForm.test.tsx: retry_reuses_idempotency_key`.

## Implementation notes
- Configure `@hey-api/openapi-ts` to emit only the zod plugin output (no client), so `openapi-fetch` stays the HTTP client (D-57).
- Keep sanitisation pure and synchronous; libphonenumber-js "min" metadata keeps the bundle small (FE-02 budget: check with `size-limit`).
- In `cases.test.ts`, skip phone cases marked `"server_only": true` (metadata version differences), and fail if more than 10% of phone cases are skipped.

## Acceptance criteria
- AC1. `pnpm gen:api` regenerates `zod.gen.ts` and `fields.gen.ts` with no diff on a clean tree. → `CI contract job`
- AC2. All shared cases pass. → `cases.test.ts: cases_match_server`
- AC3. A server 422 with errors for `/display_name` and `/guardians/0/relationship` shows both messages under the right inputs and focuses the first. → `serverErrors.test.tsx: maps_pointers_to_fields`
- AC4. An unmatched pointer appears in the error summary. → `serverErrors.test.tsx: unmatched_to_summary`
- AC5. Typing `"  Ayu  "` and leaving the field shows `"Ayu"`; typing doesn't move the caret. → `useRitmaForm.test.tsx: sanitizes_on_blur_only`
- AC6. The abuse cases in Security pass.
- AC7. axe: the error summary and field errors have no serious/critical violations. → `FormErrorSummary.test.tsx: a11y`

## Tests to write
| File | Test names |
|---|---|
| `web/src/lib/validation/cases.test.ts` | `cases_match_server` |
| `web/src/lib/validation/serverErrors.test.tsx` | `maps_pointers_to_fields`, `unmatched_to_summary` |
| `web/src/lib/forms/useRitmaForm.test.tsx` | `sanitizes_on_blur_only`, `single_submit_while_pending`, `retry_reuses_idempotency_key`, `rotates_key_after_edit` |
| `web/src/lib/safeUrl.test.ts` | `blocks_javascript_scheme`, `allows_https_mailto_tel_relative` |
| `web/src/components/domain/FormErrorSummary.test.tsx` | `a11y`, `links_focus_fields` |

## Definition of done
- [ ] `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web i18n:check` pass
- [ ] `make api` produces no diff; size budget holds
- [ ] Guard scripts pass
- [ ] Commit `feat(web): generate zod schemas and add the form harness [T-306a]`, signed off

## Report

## Questions

## Review
