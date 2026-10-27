# 24 — Open-Source Stack: adopt, don't build

Ritma is built **by assembling proven open-source libraries and tools**. We write code only for what makes Ritma different: the scheduling ledger, the policy engine, and the course-school domain. Everything generic (HTTP, auth primitives, crypto, validation, forms, tables, i18n, files, email, testing, CI/CD) comes from a maintained project with a compatible license.

This file maps every concern to its chosen project, says what we wrap and what we must not re-implement, and lists the rejected alternatives. It extends the version pins in `02-architecture.md` §11 and `08-frontend.md` §1, which stay the minimum versions. Decision record: ADR-0014.

## 1. Rules

- **OSS-01. Adopt first.** Before writing generic code, check this map. If the concern is listed, use the listed project. If it isn't, prefer a widely used, maintained project over new code, and add it here in the same PR (CQ-27 review: maintenance, license, `build.rs`, transitive size).
- **OSS-02. Wrap at one seam.** Each external library is used behind **one** `eq_core` (or `web/src/lib`) module when it's security-relevant or likely to change: crypto (`eq_core::crypto`), outbound HTTP (`eq_core::outbound`), files (`eq_core::files`), fields (`eq_core::fields`), templates (`eq-notify` renderer). Modules call the seam, never the library (SBD-07, CA-31).
- **OSS-03. License gate.** Allowed: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, MPL-2.0, Unicode-3.0, CC0-1.0, and AGPL-3.0-only for our own crates (SEC-22). `cargo deny` and `pnpm licenses` enforce it in CI. Copyleft-strong or source-available licenses (SSPL, BSL, Elastic, Commons Clause) are not allowed, even for tools that ship in the image.
- **OSS-04. Pin and update.** Rust crates are pinned by `Cargo.lock`, npm by `pnpm-lock.yaml`, GitHub Actions by **commit SHA** with the tag in a comment, container images by digest in release builds. Dependabot (and Renovate if a maintainer prefers it, never both on the same ecosystem) opens grouped weekly updates (SBD-59).
- **OSS-05. No runtime services beyond PostgreSQL.** A library is preferred over a service. Ritma stays one binary plus Postgres (ARC-01); tools that need their own server (Keycloak, Redis, RabbitMQ, Elasticsearch) are rejected for the product. Dev-only containers (Mailpit, MinIO) are fine.
- **OSS-06. What we build ourselves** is listed in §5, with the reason. Adding to that list needs an ADR.

## 2. Backend map (Rust)

| Concern | Project | License | Our seam / notes |
|---|---|---|---|
| HTTP server, routing | `axum` 0.8, `tower` 0.5, `hyper` 1 | MIT | `eq-api` router assembly |
| Middleware (request id, trace, compression, CORS-off, timeouts, body limits, headers, panic catch) | `tower-http` 0.6 | MIT | ARC-04, SEC-20 |
| Rate limiting | `tower_governor` 0.7 (`governor`) | MIT/Apache | API-15 |
| Extra extractors (cookies, multi-value query, typed headers) | `axum-extra` | MIT | `ValidatedQuery` (VAL-40) |
| Async runtime | `tokio` 1 | MIT | |
| Database driver, migrations, compile-checked SQL | `sqlx` 0.9 + `sqlx-cli` 0.9 | MIT/Apache | `repo.rs` only (CA) |
| JSON | `serde`, `serde_json`, `serde_path_to_error` | MIT/Apache | pointers for V2 errors (VAL-07) |
| YAML presets | `serde_yaml_ng` | MIT | SET-01 |
| OpenAPI 3.1 | `utoipa` 6, `utoipa-axum` 0.3 (0.2 pulls unmaintained `paste`, RUSTSEC-2024-0436); `utoipa-scalar` 0.4 for the dev API explorer (`RITMA_ENV=dev` only) | MIT/Apache | API-02 |
| JSON Schema generation / validation | `schemars` 1 / `jsonschema` | MIT | POL-06/10 |
| JSON Merge Patch (presets, PATCH) | `json-patch` (`merge`) | MIT/Apache | POL-09, CAN-22 |
| Canonical JSON (RFC 8785) | `serde_jcs` | MIT/Apache | CAN-70 |
| Validated newtypes | `nutype` | MIT | `eq_core::fields` (VAL-20) |
| Unicode normalisation | `unicode-normalization` | MIT/Apache | VAL-11 |
| Email syntax | `email_address` | MIT | VAL-11 |
| Phone numbers | `rlibphonenumber` 2 (libphonenumber port; replaces `phonenumber` 0.3, whose build dependency `postcard` → `heapless 0.7` → `atomic-polyfill` fails the advisory gate, RUSTSEC-2023-0089) | Apache-2.0 | VAL-11 |
| Currency metadata (exponents) | `iso_currency` | MIT/Apache | `eq_core::money` |
| Country codes | `celes` | MIT/Apache | CAN-10 |
| Locale negotiation | `fluent-langneg` + `unic-langid` | MIT/Apache | CAN-35 |
| Time, time zones, cron expressions | `chrono` 0.4, `chrono-tz`, `cron` | MIT/Apache | ARC-09/10 |
| IDs | `uuid` 1 (`v7`) | MIT/Apache | DOM-01 |
| Password hashing | `argon2` 0.5 (`password-hash`) | MIT/Apache | IDN-02 |
| JWT | `jsonwebtoken` 9 | MIT | IDN-03 |
| TOTP + QR | `totp-rs` (with `qr`) | MIT | IDN-12 |
| AEAD, KDF, MAC, hashing | `aes-gcm`, `chacha20poly1305` (backups), `hkdf`, `hmac`, `sha2`, `aead::stream` | MIT/Apache | `eq_core::crypto` only (SBD-48) |
| Randomness, constant-time, secrets | `rand` (`OsRng`), `subtle`, `zeroize`, `secrecy` | MIT/Apache | SBD-49/50 |
| Common-password list | NCSC/SecLists top-10k list, embedded at build time | MIT (SecLists) | IDN-02 |
| In-memory cache | `moka` | MIT/Apache | MOD-05, POL-04, RPT-03 |
| Errors | `thiserror` 2, `anyhow` | MIT/Apache | ERR-40..42 |
| Logs, traces | `tracing`, `tracing-subscriber` (json), optional `tracing-opentelemetry` + `opentelemetry-otlp` | MIT | OPS-06 |
| Metrics | `metrics` + `metrics-exporter-prometheus` | MIT | `/metrics` (OPS-06) |
| Config, CLI | `figment` (env + `ritma.toml`), `clap` 4 | MIT/Apache | OPS-01/02 |
| Email (SMTP) | `lettre` 0.11 (rustls) | MIT | NTF-08 |
| Email markup | `mrml` (MJML in Rust, build-time) | MIT | NTF-07 |
| Templates (email, WhatsApp, printables) | `minijinja` (autoescape) | Apache-2.0 | SBD-45 |
| Outbound HTTP | `reqwest` 0.12 (rustls, no redirects) | MIT/Apache | `eq_core::outbound` (SBD-47) |
| Static SPA embedding | `rust-embed` 8, `mime_guess` | MIT | FE-01 |
| File type sniffing | `infer` | MIT | VAL-15 |
| Images (decode limits, resize, re-encode, EXIF strip) | `image` 0.25 | MIT/Apache | VAL-15 |
| SVG sanitise / rasterise | `usvg` / `resvg` | MIT/Apache | SEC-14 |
| File storage (local FS and S3 behind one API) | `object_store` | Apache-2.0 | `eq_core::files` (FLW-54) |
| Safe display filenames | `sanitize-filename` | MIT | VAL-16 |
| URLs | `url`, `percent-encoding` | MIT/Apache | CAN-80 |
| CSV / XLSX / ZIP | `csv` / `calamine` / `zip` | MIT/Unlicense | VAL-17, AUD-10 |
| iCal | `icalendar` | MIT/Apache | CAN-83 |
| Geo distance | `geo` (`Haversine`) | MIT/Apache | ATT-02 |
| Postgres advisory locks, LISTEN/NOTIFY | `sqlx::postgres::PgListener` | MIT/Apache | ARC-07 |
| Desktop shell, updater | `tauri` 2 + plugins | MIT/Apache | M8 |

## 3. Frontend map (web/)

| Concern | Project | License | Notes |
|---|---|---|---|
| Build, dev server | Vite 6, TypeScript 5 | MIT/Apache | FE-01 |
| UI framework | React 19 | MIT | |
| Styling | Tailwind CSS 4 with semantic tokens | MIT | DS |
| Component primitives | shadcn/ui (copied into `components/ui`) on Radix UI | MIT | DS; no other component library |
| Icons | `lucide-react` | ISC | |
| Toasts / command palette / date picker | `sonner` / `cmdk` / `react-day-picker` (shadcn defaults) | MIT | |
| Data tables | `@tanstack/react-table` (headless, inside `DataTable`) | MIT | sorting/filters map to API-06 params |
| Routing | TanStack Router (file-based, typed search params) | MIT | FE-15 |
| Server state | TanStack Query 5 | MIT | STM-91 |
| API types and client | `openapi-typescript` + `openapi-fetch` | MIT | API-02 |
| Generated validation schemas | `@hey-api/openapi-ts` (zod plugin only) + `zod` 4 | MIT | VAL-30 |
| Forms | `react-hook-form` + `@hookform/resolvers` | MIT | VAL-33 |
| Policy forms | custom `PolicyForm` on `ajv` 8 + `ajv-formats` (draft 2020-12) | MIT | FE-20, see §5 |
| Phone input normalisation | `libphonenumber-js` (min metadata) | MIT | VAL-32 |
| I18n | `i18next`, `react-i18next`, `i18next-icu`; key extraction with `i18next-cli` | MIT | TST-09 |
| Dates and time zones | `date-fns` 4 + `@date-fns/tz` | MIT | |
| Charts | Recharts | MIT | lazy |
| Maps | Leaflet, used directly through a small `MapPin` wrapper | BSD-2-Clause | lazy. `react-leaflet` is not used: its Hippocratic-2.1 license isn't on the allow-list [D-56] |
| Face detection | `@mediapipe/tasks-vision` | Apache-2.0 | lazy (ATT-06) |
| Complex UI flows | `xstate` 5 + `@xstate/react` | MIT | STM-96/97 only, lazy |
| IndexedDB | `idb` | ISC | offline queue (ATT-07) |
| PWA / offline | `vite-plugin-pwa` (Workbox, Background Sync) | MIT | FE-35 |
| Unit and component tests | Vitest, Testing Library, `vitest-axe`, MSW (API mocks from the OpenAPI types) | MIT | |
| E2E, accessibility, visual | Playwright + `@axe-core/playwright` | Apache/MPL-2.0 | TST-07/11/12 |
| Lint / format | Biome | MIT/Apache | |
| Dead code / unused deps | `knip` | ISC | CI `web` job |
| Bundle budget | `size-limit` | MIT | FE-02 |

## 4. Tooling, CI/CD and operations

| Concern | Project | Where |
|---|---|---|
| CI/CD platform | GitHub Actions | `.github/workflows/*` (§6) |
| Rust toolchain / caching | `dtolnay/rust-toolchain`, `Swatinem/rust-cache` | `ci.yml` |
| Install Rust CLI tools fast | `taiki-e/install-action` (`cargo-nextest`, `cargo-llvm-cov`, `cargo-deny`, `cargo-machete`, `sqlx-cli`) | `ci.yml` |
| Test runner / coverage | `cargo-nextest` / `cargo-llvm-cov` | TST-01 |
| Dependency policy, advisories | `cargo-deny`, `cargo-audit` (`rustsec/audit-check`), `pnpm audit`, OSV-Scanner | `ci.yml`, `security.yml` |
| Unused dependencies | `cargo-machete` | `ci.yml` |
| Secret scanning | gitleaks (official container image) | `ci.yml` (SBD-51) |
| Static analysis (SAST) | GitHub CodeQL (`actions`, `javascript-typescript`, `rust`) | `codeql.yml` |
| Supply-chain posture | OpenSSF Scorecard | `scorecard.yml` |
| Workflow linting / security | `actionlint`, `zizmor` | `ci.yml` `meta` job |
| Spelling (code + docs) | `typos` (`crate-ci/typos`) with `_typos.toml` (Indonesian domain words allowed) | `ci.yml` `docs` job |
| Markdown link check | `lychee` (offline, local links) | `ci.yml` `docs` job |
| API fuzzing from OpenAPI | Schemathesis | `ci.yml` `contract` job (TST-18) |
| Load tests | `k6` (scripts in `tests/load/`) | manual / release (BIL-02 target, TST-05 extended) |
| Release versioning and changelog | `release-please` (`googleapis/release-please-action`) | `release-please.yml` (RELEASING.md) |
| Cross-compiled static binaries | `cargo-zigbuild` (musl) | `release.yml` (DST-05) |
| Container build | Docker Buildx (`docker/*` actions), digest-pinned Alpine runtime base (ADR-0019, spec 26 CTR-02) | `release.yml` |
| Signing, SBOM, provenance | `cosign` keyless (Sigstore), Syft via `anchore/sbom-action` (CycloneDX), `actions/attest-build-provenance` (SLSA) | `release.yml` (SEC-22) |
| Image vulnerability scan | Trivy (`aquasecurity/trivy-action`) | `release.yml` |
| Dependency updates | Dependabot (`.github/dependabot.yml`) | weekly, grouped |
| Local hooks | `.githooks/commit-msg` (T-006); optional `lefthook` | CONTRIBUTING |
| Dev database / mail / S3 | `postgres:16`, Mailpit, MinIO (optional) | `deploy/dev/compose.yaml` |
| TLS reverse proxy | Caddy | `deploy/docker` (SEC-06) |
| Docs site | Astro Starlight | `docs/` (M7) |

## 5. What we build ourselves, and why

| Built in Ritma | Why not a library | Libraries it still stands on |
|---|---|---|
| Conflict ledger and alternatives (DOM-10, SCH-05) | It *is* the product guarantee; it's a Postgres exclusion constraint plus a pure search function | PostgreSQL `btree_gist`, `chrono` |
| Policy engine (POL) | Typed Rust structs are the design (ADR-0004); no rule engine fits "typed, versioned, effective-dated, snapshotted" | `schemars`, `jsonschema`, `json-patch` |
| Transactional outbox and job queue (ARC-07/09) | Enqueue must share the business transaction **and** the RLS tenant setting, with tenant-timezone cron and dedupe keys. That's ~400 lines over `sqlx` with `SKIP LOCKED`. Evaluated `apalis` and `underway`: neither gives same-tx enqueue with our RLS context plus per-tenant cron without adapters of similar size. Revisit by RFC if the queue grows | `sqlx`, `cron`, `PgListener` |
| Money (ARC-11) | A 100-line `i64` type is simpler and safer than a decimal library for minor units | `iso_currency` |
| Audit hash chain (AUD-04) | Domain-specific sealing and anonymisation proofs | `sha2`, `serde_jcs` |
| `PolicyForm` renderer (FE-20) | `react-jsonschema-form` and JSON Forms were evaluated: both add 60–120 KB gzip, fight shadcn styling, and don't support our widgets (`weekday-hours`, `tiers`, money/bps units) or externally tagged enums without heavy custom code. Our renderer is a recursive walker over a schema we control | `ajv`, `react-hook-form`, shadcn |
| Field kinds (VAL-20) | Glue that ties one rule to OpenAPI, the server parser and the client schema | `nutype`, `utoipa`, `zod` (generated) |

Rejected for the product (OSS-05): Keycloak / Ory / Zitadel (auth server), Redis (cache/queue), RabbitMQ/NATS (broker), Elasticsearch/Meilisearch (search: `pg_trgm` is enough), a separate PDF service (printables use the browser, BIL-11).

## 6. CI/CD pipeline (GitHub Actions)

**OSS-20.** Workflows (all with `permissions: contents: read` by default, SHA-pinned actions, `persist-credentials: false`, `concurrency` cancel-in-progress per ref):

| Workflow | Trigger | Jobs |
|---|---|---|
| `ci.yml` | PR, push to `main`, manual | `detect` (which parts exist) → `docs` (task index check, typos, links), `meta` (actionlint, zizmor), `pr-title` (COMMIT_CONVENTION §9 regex), `secrets` (gitleaks), `rust` (fmt, clippy, deny, machete, nextest, llvm-cov gate), `web` (biome, typecheck, vitest, i18n, knip, size-limit, build), `contract` (OpenAPI drift + Schemathesis), `scripts` (guard scripts), `e2e` (Playwright) → `ci-ok` (single required check) |
| `codeql.yml` | PR, push to `main`, weekly | CodeQL for `actions`, plus `javascript-typescript` and `rust` once code exists |
| `security.yml` | daily, manual | `cargo audit`, `pnpm audit --prod`, OSV-Scanner on lockfiles |
| `scorecard.yml` | weekly, push to `main` | OpenSSF Scorecard → code scanning |
| `release-please.yml` | push to `main` | release PR + tag once the Cargo workspace exists |
| `release.yml` | tag `v*` (from release-please), manual dry run | static binaries (linux amd64/arm64 musl, macOS, Windows), multi-arch Alpine-based image (ADR-0019) to GHCR, SBOM, Trivy, cosign signatures, provenance attestation, checksums, assets on the GitHub Release |

**OSS-21. Progressive activation.** The repository starts with specs only. Each job checks for its inputs (`Cargo.toml`, `web/package.json`, `web/openapi.json`, `web/e2e/`, guard scripts) in the `detect` job and **skips cleanly** when they don't exist yet, so `ci-ok` is green on a docs-only repo and becomes stricter automatically as T-001, T-005, T-006, T-114 and T-008 land. A skipped job is reported as skipped, never as passed silently: `ci-ok` fails if any job that *ran* failed or was cancelled.

**OSS-22. Branch protection** for `main` (repository settings, documented in `GOVERNANCE.md`): require `ci-ok`, CodeQL, one maintainer review, signed-off commits (DCO), linear history; no force pushes.

## 7. Decisions taken in this document

| ID | Decision |
|---|---|
| D-56 | `react-leaflet` is not used (Hippocratic license isn't on the allow-list); `MapPin` wraps Leaflet (BSD-2) directly |
| D-57 | Field kinds use `nutype`; client schemas are generated with `@hey-api/openapi-ts` (zod plugin) next to `openapi-typescript` |
| D-58 | Keep the in-house outbox/job queue over `sqlx` (evaluated `apalis`, `underway`) |
| D-59 | `PolicyForm` stays custom (evaluated `react-jsonschema-form`, JSON Forms) |
| D-60 | Actions are pinned by commit SHA; Dependabot updates them weekly |
| D-61 | CI jobs activate progressively (OSS-21) with one required aggregate check `ci-ok` |
