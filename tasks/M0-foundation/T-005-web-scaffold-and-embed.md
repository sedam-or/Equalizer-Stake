---
id: T-005
title: Scaffold the web app (Vite/React/TS/Tailwind 4/Router) and embed it in the binary
milestone: M0
status: review
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
| `web/pnpm-lock.yaml` | create | pin resolved web dependencies |
| `web/src/brand.ts` | create | `export const BRAND = { name: "Ritma", tagline: "The rhythm of your school" } as const;` (rename point) |
| `Cargo.toml`, `Cargo.lock`, `crates/eq-api/Cargo.toml` | modify | add the specified embedded-asset dependencies |
| `crates/eq-api/src/{lib.rs,health.rs,spa.rs}`, `crates/eq-api/build.rs` | modify/create | register the fallback before shared middleware and embed assets |
| `crates/eq-core/src/{lib.rs,brand.rs}` | modify/create | export `pub const NAME: &str = "Ritma";` (rename point; used by later tasks) |

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
- [x] Rust checks pass; `pnpm -C web lint && pnpm -C web typecheck && pnpm -C web test && pnpm -C web build && pnpm -C web size` pass
- [x] Commit `feat(web): scaffold spa and embed in binary [T-005]`, signed off

## Report

**Plan** — (1) Add the pinned React/Vite/TypeScript/Tailwind/TanStack scaffold and the strict TS/Biome/Vitest/size-limit configuration → verify `pnpm -C web lint`, `typecheck`, `test`, `build`, and `size`. (2) Add the product brand and the minimal semantic token/style files → verify the required rendered-name test and Biome. (3) Add the `eq-core::brand::NAME` constant → verify Rust checks. (4) Add the RustEmbed build script and safe fallback handler with cache headers, MIME type and shell-file ETags → verify all five exact SPA integration tests. (5) Run a release binary with the built SPA and exercise root, a client route and an unmatched API path over HTTP → verify FE-01 behavior. (6) Run full Rust/web/guard/secret checks and record checks that do not yet exist in this milestone.

**Assumptions** — The package manager will be pinned to the preflight pnpm 9 release. Current patch releases will be pinned exactly within the task's required major versions. The dev server's `/api` proxy forwards to the already shipped server on port 8080. Cache behavior follows FE-01 and CACHE-20/21; shell resources that are not yet produced by this task remain eligible for the specified SPA fallback.

**Test plan** — Given built embedded assets, request `/` and assert 200, `text/html`, and `Cache-Control: no-cache` (`index_no_cache`); request a generated `/assets/*` file and assert the immutable cache directive (`asset_immutable`); request an unmatched client GET path and assert the embedded index (`spa_fallback`); request an unmatched `/api/v1/*` path and assert 404 `application/problem+json` with `not_found`, never HTML (`api_404_not_spa`); request the index with its ETag and assert a matching `If-None-Match` gets 304 (`etag_304`). In Vitest, render the home route component and assert the product name comes from `BRAND`. Edge cases: unknown/missing embedded files fall back only for GET requests outside `/api` and `/assets`; API paths never fall through to the SPA; ETag values are quoted and derived from the embedded SHA-256; paths remain within the embedded asset set. Security cases: static responses contain no API or tenant data; no raw HTML insertion or secret is introduced.

**Step 0 — Start** — T-004 is `review` at `2daed7862fdb17f70c1f478f03a0c9cbf0efca90`; its code and Report Deviations/Follow-ups/RATIFY were read. The tree was clean, the task index passed, and T-005 is the lowest eligible task. Status changed to `in_progress`, followed by `scripts/tasks-index.sh`.

**Step 1 — Task/spec** — Read this task and FE-01/02, ADR-0003, DS-02, CQ-60/61/62, CACHE-20/21, and the cross-cutting architecture/security/error rules. The business-rules section is empty; no domain behavior or schema is added.

**Step 2 — Dependencies** — Read T-004's shipped router, request-ID/trace middleware, API errors, and full Report. Its follow-up requires preserving request-ID middleware around the SPA and API. Its three prior self-resolutions are recorded in the dependency Report; they are inherited and do not require another decision here.

**Step 3 — Design reference** — Read `spec/09-design-system.md` DS-02 and the FE-01/FE-02 constraints. This scaffold displays only the brand name and tagline; it uses the four requested semantic color tokens and adds no product flow.

**Step 4 — Plan and test plan** — Recorded above before implementation. One mechanical mismatch between the implementation note and the normative cache inventory is recorded below for architect ratification.

**Summary** — Added the pinned React 19/Vite 6/TypeScript/Tailwind 4/TanStack web scaffold, strict TypeScript and Biome configuration, a Vitest-rendered branded home route, the initial-JavaScript gzip budget, the core product-name constant, and RustEmbed SPA delivery with MIME types, SPA fallback, API problem responses, cache policy and shell ETags. The build script creates and embeds the specified placeholder when `web/dist` is absent.

**Spec read** — `spec/08-frontend.md` FE-01/02; `spec/09-design-system.md` DS-02; `spec/16-code-quality.md` CQ-60/61/62/§12; `spec/17-clean-architecture.md` CA-01/40; `spec/18-security-by-design.md` SBD-45/81; `spec/19-error-handling.md` ERR-30/42; `spec/24-open-source-stack.md` OSS frontend/embed entries; `spec/26-containers-and-caching.md` CACHE-20/21; `spec/adr/0003-spa-embedded.md`.

**Files** — Created the scaffold under `web/`, including the package manifest and lockfile, strict compiler/linter/test/bundle-size configs, React/TanStack routes, the minimal color token and app styles, brand constants, and the exact product-name test. Added `crates/eq-api/build.rs` and `src/spa.rs`; wired the fallback before shared middleware in `src/health.rs`; registered the module in `src/lib.rs`; added RustEmbed/MIME dependencies; exported `eq_core::brand::NAME`; added `crates/eq-api/tests/spa.rs`; updated this task's file list/report and regenerated `tasks/INDEX.md`.

**Acceptance criteria → tests**
| AC | Exact test/check | Result |
|---|---|---|
| AC1 | `index_no_cache` | Passed; release binary returned the built `index.html`, `text/html`, `Cache-Control: no-cache`, and a quoted SHA-256 ETag. |
| AC2 | `asset_immutable` | Passed; generated JS served as `public, max-age=31536000, immutable`; no ETag was attached to a CACHE-20 asset. |
| AC3 | `spa_fallback` | Passed; `/schedule/week` returned the SPA shell with `no-cache`. |
| AC4 | `api_404_not_spa` | Passed; `/api/v1/nope` returned 404 `application/problem+json` with `not_found`. |
| AC4/validator | `etag_304` | Passed; matching `If-None-Match` returned an empty 304 with the same ETag and `no-cache`. |
| AC5 | `pnpm -C web size` | Passed; initial JS measured 98.77 KB gzip against 200 KB. |
| UI route | `renders product name` | Passed in Vitest; the page renders the name supplied by `BRAND`. |

**Checks run** — All listed Rust and web checks passed:
- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- `cargo test --workspace` — all workspace, PostgreSQL-backed, API, CLI, SPA and doc tests passed.
- `cargo build -p eq-server --release` — passed with real web assets; separately built after temporarily moving `web/dist` and confirmed both the generated placeholder file and the release binary contained the fallback message; restored the real assets and rebuilt.
- `pnpm -C web lint`; `pnpm -C web typecheck`; `pnpm -C web test`; `pnpm -C web build`; `pnpm -C web size` — all passed.
- `cargo audit`; `cargo deny check advisories` (advisories ok; Cargo Deny used its default config because the repo has no deny config); `scripts/check-deps.sh`; `python3 scripts/erd.py --check`; `scripts/tasks-index.sh --check`; `git diff --check`; `gitleaks detect --redact --no-banner` — all passed.
- The `pnpm-lock.yaml` and `.sqlx/` data are not affected by Rust queries; no migration changed, so `cargo sqlx prepare --workspace` and `python3 scripts/erd.py --migrations` are n/a.
- `pnpm -C web run i18n:check` was attempted and reported `ERR_PNPM_NO_SCRIPT`; T-005 adds no locale catalogs or localized application copy, and CQ-65 maps that checker to T-306. This remains a T-306 follow-up and was not reported as passed.
- `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, and `scripts/check-layers.sh` are not present yet (T-006 owns them). `make dev` and `make e2e` were attempted and report no targets; T-006 adds the Makefile and T-008 owns M0 E2E. Hosted GitHub Actions were not run.

**Manual checks** — Ran `pnpm -C web dev --host 127.0.0.1` and captured/inspected the rendered page at 390×844 and 1280×900 with local Playwright; Ritma and its tagline were visible without overflow at both widths. Ran the final release binary against the local PostgreSQL 16 dev container, then made real HTTP requests: `/` 200/no-cache/ETag, generated JS 200/immutable, `/schedule/week` 200 with the shell, `/api/v1/nope` 404 problem+json, and matching `If-None-Match` 304. Sent SIGTERM and confirmed graceful shutdown. Separately verified the placeholder path by building with `web/dist` absent, then restored the built SPA and rebuilt.

**Debug log** — The first Rust compile showed that the task's manifest-relative RustEmbed folder requires the `interpolate-folder-path` feature and that rust-embed exposes SHA-256 through `metadata.sha256_hash()` rather than a public field; enabled the feature and rendered the digest bytes as lowercase hex. Biome 2.6.3 moved the task's rules to `security`, `performance`, and `style`; configured the same named rules in those groups and noted the group changes. Enabled Biome's Tailwind CSS parser option, excluded the generated pnpm lock from formatting, and configured TanStack Router to ignore the required `index.test.tsx` file as a route. Applied `cargo fmt` after the first `cargo fmt --all --check` identified formatting-only changes. The follow-up Rust/web suites and live checks passed.

**Performance** — `pnpm -C web size` measured the initial JavaScript at 98.77 KB gzip (101.23 KB below the 200 KB ceiling); Vite reported 98.93 KB gzip for the generated JS. No route chunks or server queries were added.

**Security** — No authentication, PII, external request, or mutable domain data was added. The static handler reads only the embedded asset set; RustEmbed rejects paths outside its folder, missing `/assets/*` do not become HTML, and unmatched `/api/*` stays problem+json. Static shell content uses escaped React text and constant brand data; there is no raw HTML injection, and Biome enables SBD-45 `noDangerouslySetInnerHtml` at error severity. ETags derive from embedded SHA-256 metadata, and a matching validator returns an empty 304. The response layer keeps T-004's request-ID and trace middleware around the SPA fallback. `gitleaks` found no leaks.

**Deviations** — RATIFY: (1) Followed normative CACHE-20/21 by sending ETags only for `index.html`, `sw.js`, and `manifest.webmanifest`; hashed assets remain immutable without a validator despite the broader wording in the implementation note. (2) Added the directly affected manifest/lockfile and existing router-module paths to this task's Files table because Scope requires dependency setup/fallback wiring and T-004 explicitly requires preserving shared middleware. Biome 2.6.3 uses different rule groups than the task's examples; no named rule was dropped. The missing `i18n:check` and guard scripts are recorded above for their owning tasks, not as passed. Decision register references: TD-010, TD-011 (same order as the self-resolved bullets under Questions; see `tasks/DECISIONS.md`).

**Dependencies affected** — T-006 must build the SPA before compiling the release binary so RustEmbed sees the built files; it can add the dev/e2e targets and guard scripts. T-008 can use the embedded shell and exact SPA/API cache behavior for the M0 Playwright scenario. T-306 owns the i18n checker/API client later in the roadmap.

**Rollback plan** — Revert this task's commit to remove the web scaffold and embedded fallback. The build script may leave an ignored `web/dist` directory; it contains generated assets only and can be replaced by `pnpm -C web build`. No database or tenant state is changed.

**Follow-ups** — T-006: CI and Makefile must run `pnpm -C web build` before release compilation, and add the not-yet-present guards. T-008: run the M0 Playwright E2E in CI. T-306: add locale catalogs and `i18n:check`; the current scaffold has only locale-invariant brand identity. T-304 expands the minimal color tokens; T-307 adds PWA assets.

**Step 5 — Implement** — Implemented only the web scaffold, product rename points, and FE-01 embedded asset behavior in Scope → In.

**Step 6 — Run locally** — The `make dev` target does not exist until T-006, so started the Vite server and release binary directly. Exercised both responsive browser sizes and real release HTTP happy/error/cache paths as described above.

**Step 7 — Debug loop** — Fixed the RustEmbed feature/API mismatch, Biome rule-group/parser/config findings, route-plugin test-file warning, and Rust formatting findings; reran the relevant check after each correction. The final complete Rust and web checks are green.

**Step 8 — Self code review** — Reviewed the full task diff against CQ-60/61/62/§9, CA-01/40, ERR-30, and SBD-45. No `#[allow]`, secret, raw SQL, API policy change, untrusted HTML, or generated route tree is committed. The SPA fallback is installed before shared middleware so request IDs and tracing cover static and API fallback responses.

**Step 9 — Optimize** — Measured the initial bundle with the task's size-limit check; 98.77 KB gzip passes the 200 KB threshold. No speculative caching or code splitting was added.

**Step 10 — Security audit** — Reviewed static path selection, API isolation from the SPA, React text rendering, cache validators, and shared request middleware. SBD-81 auth/tenant/PII/external-call controls do not apply to this static scaffold; SBD-45 is enabled and gitleaks passed.

**Step 11 — Revise and verify** — Reran formatter, full clippy and workspace tests, all five SPA API tests, web lint/typecheck/test/build/size, release build and live HTTP smoke checks after corrections. Missing later-task scripts and hosted CI are listed accurately above.

**Step 12 — Report, commit** — Report complete; status is `review`, `tasks/INDEX.md` is regenerated through `scripts/tasks-index.sh`, and this task is prepared for one signed-off commit.

## Questions
- **Self-resolved (implementer, run 5, needs architect ratification):** The implementation note could be read to add ETags to every embedded file, but CACHE-20 specifies no validator for hashed assets and CACHE-21 specifies ETags for the index/service-worker/manifest shell files → add ETags and honor `If-None-Match` only for those shell files because the normative cache inventory is specific to each response class. [TD-010]
- **Self-resolved (implementer, run 5, needs architect ratification):** The Files table omitted required package manifests/lockfile and the existing router modules needed to wire the fallback inside the request-ID/trace stack → add the directly affected paths to the Files table because Scope explicitly requires dependency setup and fallback routing while T-004's Follow-ups require preserving middleware. [TD-011]

**RATIFY:** The two mechanical adjustments above follow CACHE-20/21 and the explicit FE-01 router/middleware requirements. No business, schema, permission or security policy is added.
