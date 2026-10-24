---
id: T-005
title: Scaffold the web app (Vite/React/TS/Tailwind 4/Router) and embed it in the binary
milestone: M0
status: todo
depends_on: [T-004]
spec_refs: [FE-01, FE-02, ADR-0003, DS-02, CQ-60, CQ-61, CQ-62, CACHE-20, CACHE-21]
estimated_size: M
---

## Goal
`pnpm -C web build` produces `web/dist`, and the release binary serves it at `/` with an SPA fallback and correct cache headers. Dev mode proxies `/api` to the Rust server.

## Context
- FE-01 serving rules:
  - `/assets/*` (hashed): `Cache-Control: public, max-age=31536000, immutable`.
  - `index.html`, `sw.js`, `manifest.webmanifest`: `no-cache`.
  - Any non-`/api`, non-asset GET that doesn't match a file falls back to `index.html` (200).
  - `/api/*` that doesn't match a route → 404 problem+json, never index.html.
  - These headers are CACHE-20/21 in spec 26; the full cache inventory (HTTP, browser, server, CI) is spec 26 Part B.
- FE-02: initial JS ≤ 200 KB gzip, checked with `size-limit`.
- Stack: React 19, Vite 6, TypeScript 5 strict (`noUncheckedIndexedAccess`), Tailwind 4 (`@tailwindcss/vite`), TanStack Router (file-based, `@tanstack/router-plugin`), TanStack Query 5, Biome, Vitest.
- DS-02: tokens live in `web/src/styles/tokens.css`. This task creates a **minimal** token file (colour bg/surface/text/primary only). The full system comes in T-304.

## Scope
### In
- `web/` scaffold: `package.json` (scripts `dev`, `build`, `preview`, `lint`, `typecheck`, `test`, `size`), `vite.config.ts` (react, tailwind, router plugin, `server.proxy['/api'] = 'http://localhost:8080'`), `tsconfig.json`, `biome.json`, `index.html` (lang from meta later; `<div id="root">`), `src/main.tsx`, `src/routes/__root.tsx`, `src/routes/index.tsx` (a "Hello" page showing the product name from `src/brand.ts`), `src/styles/{tokens.css,app.css}`, `.size-limit.json`.
- `crates/eq-api/src/spa.rs`: `#[derive(RustEmbed)] #[folder = "$CARGO_MANIFEST_DIR/../../web/dist"] struct Assets;` with a handler implementing FE-01. If `web/dist` is missing at compile time, build with an embedded placeholder `index.html` that says "Web UI not built. Run pnpm -C web build". Use a `build.rs` that creates `web/dist/index.html` placeholder if absent, so `cargo build` never fails on a fresh clone.
- Wire it into `router()` as a fallback service after the `/api` and health routes.

### Out
- Design tokens beyond the minimum (T-304), the API client (T-306), the PWA (T-307).

## Files
| Path | Action | Purpose |
|---|---|---|
| `web/**` | create | scaffold |
| `web/src/brand.ts` | create | `export const BRAND = { name: "Ritma", tagline: "The rhythm of your school" } as const;` (rename point) |
| `crates/eq-api/src/spa.rs`, `crates/eq-api/build.rs` | create | embed |
| `crates/eq-core/src/brand.rs` | create | `pub const NAME: &str = "Ritma";` (rename point; used by later tasks) |

## Interfaces
`GET /` → 200 text/html (`no-cache`)
`GET /assets/index-abc123.js` → 200 (immutable)
`GET /schedule/week` → 200 index.html
`GET /api/v1/nope` → 404 problem `not_found`

## Business rules
None.

## Implementation notes
- `tsconfig.json` (CQ-60): `strict`, `noUncheckedIndexedAccess`, `noImplicitOverride`, `noFallthroughCasesInSwitch`, `verbatimModuleSyntax`, `forceConsistentCasingInFileNames`; path alias `@/` → `src/` (also in `vite.config.ts`).
- `biome.json` (CQ-61): `recommended` + at `error`: `noExplicitAny`, `noNonNullAssertion`, `useExhaustiveDependencies`, `noUnusedImports`, `noUnusedVariables`, `useImportType`, `noConsole`, `noDefaultExport` (override: allowed in `*.config.ts`), `noBarrelFile`, `noExcessiveCognitiveComplexity` (max 15), `noDangerouslySetInnerHtml` (SBD-45). Formatter: 2 spaces, width 100, double quotes, semicolons, trailing commas `all`. Ignore generated files (`routeTree.gen.ts`, `src/api/schema.d.ts`). Use the rule names of the installed Biome version and note any renames in the Report.
- Pin exact versions in `package.json` (no `^`). Commit `pnpm-lock.yaml`.
- `packageManager: "pnpm@9.x"` in package.json.
- MIME types come from `mime_guess`. Add `ETag` from the embedded file's hash (`rust-embed` metadata sha256) and honour `If-None-Match` with 304.

## Acceptance criteria
- AC1. After `pnpm -C web build && cargo build -p eq-server --release`, `GET /` returns the built index with `Cache-Control: no-cache`. → `eq-api/tests/spa.rs::index_no_cache`
- AC2. A hashed asset returns `immutable`. → `asset_immutable`
- AC3. `GET /some/client/route` → index.html. → `spa_fallback`
- AC4. `GET /api/v1/unknown` → 404 problem+json. → `api_404_not_spa`
- AC5. `pnpm -C web size` passes the budget (≤ 200 KB gzip initial JS).

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/spa.rs` | `index_no_cache`, `asset_immutable`, `spa_fallback`, `api_404_not_spa`, `etag_304` |
| `web/src/routes/index.test.tsx` | `renders product name` |

## Definition of done
- [ ] Rust checks pass; `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web build && pnpm -C web size` pass
- [ ] Commit `feat(web): scaffold spa and embed in binary [T-005]`, signed off

## Report

## Questions
