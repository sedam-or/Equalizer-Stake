# 26 — Containers and Caching

Two operational concerns that were spread across specs 08, 11, 14, 21, 23, 24 and several tasks. This document is the single place for **how the container image is built and run** (CTR-\*) and for **every cache in the system** (CACHE-\*): what it holds, its key, lifetime, invalidation, and what must never be cached. It follows the existing rules (OSS-05: PostgreSQL is the only stateful service; D-10: `/api` is `no-store`) and records the few new decisions in §7.

Decisions: [ADR-0018](adr/0018-in-process-caches-no-redis.md) (caching), [ADR-0019](adr/0019-small-alpine-runtime-image.md) (image).

## Part A — Containers

### 1. One image, every container channel

**CTR-01.** There is exactly one application image, `ghcr.io/<owner>/ritma:<version>`, multi-arch (`linux/amd64`, `linux/arm64`), built by `release.yml` from the release binaries (DST-05). Docker one-liner, compose, 1-click templates and the cloud all run this image. The desktop app doesn't use containers (DST-20).

**CTR-02. The image contains:**

| Item | Why | Source |
|---|---|---|
| `/usr/local/bin/ritma` | The static musl binary with the SPA, migrations and presets embedded | `release.yml` binaries job |
| `pg_dump` / `pg_restore` (PostgreSQL 16 client) | `ritma backup` / restore (OPS-05) | Alpine `postgresql16-client` |
| Nothing else from the build | No compiler, no Node, no source | multi-stage build |

TLS trust roots are compiled into the binary (`rustls` with bundled web PKI roots), and time zones come from `chrono-tz`, so the image needs neither `ca-certificates` nor `tzdata` for Ritma itself.

### 2. Dockerfile (normative shape for T-704)

```dockerfile
# syntax=docker/dockerfile:1
# Runtime: Alpine (digest-pinned) because pg_dump is dynamically linked (ADR-0019).
FROM alpine:<current stable>@sha256:<pinned> AS runtime
ARG TARGETARCH
RUN apk add --no-cache postgresql16-client \
 && addgroup -g 10001 -S ritma && adduser -u 10001 -S -G ritma -H -s /sbin/nologin ritma \
 && mkdir -p /data && chown 10001:10001 /data
COPY --chmod=0555 dist/ritma-linux-musl-${TARGETARCH} /usr/local/bin/ritma
ENV RITMA_DATA_DIR=/data RITMA_LISTEN=0.0.0.0:8080 RITMA_LOG_FORMAT=json
USER 10001:10001
VOLUME ["/data"]
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD ["/usr/local/bin/ritma", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/ritma"]
CMD ["serve"]
```

- **CTR-03. Base image pinned by digest.** Dependabot (`docker` ecosystem, OSS-04) proposes digest bumps. The binary is built in CI, not in the Dockerfile, so the image build is a copy plus one `apk add` and takes seconds.
- **CTR-04. Non-root.** UID/GID 10001, no login shell, no `sudo`. `/data` is the only writable path the app needs.
- **CTR-05. Healthcheck without curl.** `ritma healthcheck` is a CLI subcommand that calls `http://127.0.0.1:$PORT/healthz` with a 2 s timeout and exits 0/1. It has no network access beyond loopback. (Added to T-004's CLI.)
- **CTR-06. Labels** come from `docker/metadata-action`: `org.opencontainers.image.{title,description,version,revision,source,licenses=AGPL-3.0-only,created}`.
- **CTR-07. Size budget** ≤ 60 MB compressed per architecture (DST-05). CI fails the release dry run above it.
- **CTR-08. Signals and shutdown.** `ritma serve` is PID 1 and handles SIGTERM itself: stop accepting, drain requests and jobs ≤ 25 s (OPS-03). No init wrapper is needed because the binary spawns no child processes except `pg_dump` during backups, which it waits for.

### 3. Runtime hardening (compose, 1-click, cloud)

**CTR-10.** Every deployment template runs the app container with:

| Setting | Value | Why |
|---|---|---|
| `read_only: true` | root filesystem read-only | Nothing outside `/data` and `/tmp` is written |
| `tmpfs: [/tmp]` | size 64 MiB | Upload spooling and pg_dump temp files |
| `user` | `10001:10001` | CTR-04 |
| `cap_drop: [ALL]` | no Linux capabilities | Listens on 8080, needs none |
| `security_opt: [no-new-privileges:true]` | | Blocks setuid escalation |
| `pids_limit` | 256 | Fork-bomb guard |
| `mem_limit` / `cpus` | 512 MiB / 1.0 (documented defaults, adjustable) | OPS-07 idle RSS ≤ 150 MB leaves headroom |
| `restart` | `unless-stopped` | DST-30 |
| `depends_on` | `postgres: condition: service_healthy` | Start order |
| `healthcheck` | from the image | CTR-05 |
| Ports | only Caddy publishes 80/443; the app listens on the internal network | SEC-06 |
| Secrets | `.env` mode 600 (`RITMA_SECRET_KEY`, `POSTGRES_PASSWORD`, `RITMA_SETUP_TOKEN`) | DST-30; never baked into the image |

**CTR-11. Compose stacks.**

| File | Services | Purpose | Task |
|---|---|---|---|
| `deploy/dev/compose.yaml` | `postgres:16` on :5433 | Local development only (`make dev-db`), weak dev credentials allowed | T-006 |
| `deploy/docker/docker-compose.yml` | `app` (this image), `postgres:16-alpine` (volume, healthcheck `pg_isready`), `caddy:2` (automatic HTTPS) | Production self-host (DST-30/31) | T-706 |
| Test containers | `postgres:16` via `testcontainers` | Integration tests in CI (TST) | T-007 |

Third-party images (`postgres`, `caddy`) are pinned by digest in production compose and updated by Dependabot.

### 4. Supply chain of the image

**CTR-20.** On every release (`release.yml`, OSS §6):
1. Build with Buildx for both architectures, **without restoring any shared build cache** (CACHE-40).
2. Generate the SBOM and SLSA provenance (`sbom: true`, `provenance: mode=max`).
3. Scan with Trivy and fail on **CRITICAL** fixable vulnerabilities.
4. Sign the image digest keylessly with cosign and push an attestation (`actions/attest-build-provenance`).
5. Publish tags `X.Y.Z` and `X.Y`, and `latest` only for the highest release (BRANCHING.md §5, T-704).

Users verify with:

```sh
cosign verify ghcr.io/<owner>/ritma:1.2.3 \
  --certificate-identity-regexp 'https://github.com/<owner>/.+/.github/workflows/release.yml@refs/tags/v.*' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
gh attestation verify oci://ghcr.io/<owner>/ritma:1.2.3 --owner <owner>
```

## Part B — Caching

### 5. Rules

- **CACHE-01. In-process only.** Server caches are `moka` caches inside the `ritma` process. There is no Redis or Memcached (OSS-05, ADR-0018). Each instance has its own cache. Correctness never depends on a cache: every cached value is either invalidated by a PostgreSQL `NOTIFY` after commit or bounded by a short TTL that is acceptable for that data.
- **CACHE-02. Keys always start with the tenant id** (except process-global data such as compiled schemas). A cache lookup without a tenant id for tenant data is a bug (SBD-42).
- **CACHE-03. No S2/S3 data in shared caches.** Decrypted PII, tokens and money details are never put in an HTTP-shared cache, a CDN, or a server cache that outlives the request, except the key material in CACHE-15, which is zeroized on eviction. Authenticated API responses are `Cache-Control: no-store` (D-10).
- **CACHE-04. Invalidate, then expire.** Writes that change cached data `NOTIFY` a channel after commit (FLW-05). Every process `LISTEN`s and evicts. On a reconnect of the listener connection, the process **clears all NOTIFY-backed caches**, because notifications sent while it was disconnected are lost.
- **CACHE-05. Every cache is listed here.** A task that adds a cache adds a row to §6 in the same PR, with key, TTL, invalidation and the data class (SBD-10).

### 6. Cache inventory

#### 6.1 Server (`moka`, per process)

| ID | What | Key | TTL / size | Invalidation | Data class | Owner |
|---|---|---|---|---|---|---|
| CACHE-10 | Tenant resolution (`TenantInfo`: id, status, locale, currency, timezone) | host slug | 60 s / 10 000 | TTL only; a suspended tenant may be served ≤ 60 s [D-66] | S0/S1 | T-104 |
| CACHE-11 | Enabled modules | `tenant_id` | 30 s | `NOTIFY eq_modules_changed` (MOD-05) | S0 | T-105 |
| CACHE-12 | Policy versions `Vec<Versioned<Value>>` | `(tenant_id, module)` | 30 s safety TTL | `NOTIFY eq_policy_changed` (POL-04, FLW-05) | S0/S1 | T-106 |
| CACHE-13 | Compiled policy JSON Schemas | `module` | process lifetime | none (static per binary) | S0 | T-106 |
| CACHE-14 | Auth checks: `ver` per credential; permission set per `(tenant, person, role)` | as stated | 60 s / 30 s | in-process eviction on password change; TTL for deactivation (D-65) | S1 | T-110 |
| CACHE-15 | Unwrapped tenant DEKs (`Keyring`) | `tenant_id` | 10 min | TTL; cleared on key rotation; values `Zeroizing` | **S3 key material** | T-103 |
| CACHE-16 | Report widget results | `(tenant_id, widget, from, to, role)` | 60 s | `core.module_toggled.v1` evicts the tenant (RPT-03) | S1 aggregates only (RPT-02) | T-606 |
| CACHE-17 | Migration status for `/readyz` | — | 10 s | TTL | S0 | T-004 |
| CACHE-18 | Rate-limiter buckets (`governor`) | IP / user | rolling window | none | S0 | T-115 |

Not caches (durable state that looks like one): `idempotency_keys` (24 h replay store, API-08), `drafts` (7 days, FE-30), `jobs`/`outbox` (queues), `event_receipts`. They live in PostgreSQL and survive restarts.

**Multi-instance note.** CACHE-18 limits are per instance, so with N instances behind a load balancer the effective limit is up to N×. That is accepted for v1. Cloud deployments set limits per instance accordingly (spec 15).

#### 6.2 HTTP caching

| ID | Response | `Cache-Control` | Validator | Source |
|---|---|---|---|---|
| CACHE-20 | Hashed SPA assets `/assets/*` | `public, max-age=31536000, immutable` | — | FE-01, T-005 |
| CACHE-21 | `index.html`, `sw.js`, `manifest.webmanifest` | `no-cache` | `ETag` (304) | FE-01, T-005 |
| CACHE-22 | Every `/api/v1/*` response by default | `no-store` | `ETag` on versioned aggregates is for `If-Match`, not caching | D-10, API-07 |
| CACHE-23 | `GET /api/v1/meta/app` | `public, max-age=60` | — | D-10 (no PII: name, locale, brand) |
| CACHE-24 | Tenant logo `GET /api/v1/files/{id}` (purpose `logo`) | `public, max-age=3600` | `ETag` | T-303 (shown on the login page) |
| CACHE-25 | Avatars (purpose `avatar`) | **`private, max-age=3600`** | `ETag` | T-303 [D-67] |
| CACHE-26 | Signed file downloads (`?exp&sig`) | `private, max-age=<seconds until exp>`, never beyond 300 | — | SEC-15, CAN-81 |
| CACHE-27 | `GET /calendar.ics?token=` | `no-store` (calendar apps poll on their own schedule) | — | SCH-16 |
| CACHE-28 | Printable reports and invoice documents (HTML) | `no-store` | — | RPT-04, BIL-11 |

Caddy (CTR-11) is configured as a reverse proxy only, with no response cache, so `public` responses are cached by browsers, never by a shared server cache that could mix tenants.

#### 6.3 Browser

| ID | What | Where | Lifetime | Cleared on | Source |
|---|---|---|---|---|---|
| CACHE-30 | Server data (TanStack Query) | memory | `staleTime` 30 s, `gcTime` 5 min | logout, role switch, tenant change; targeted `invalidateQueries` after each mutation (keys per resource, ERR-61 for 409 refetch) | T-306 |
| CACHE-31 | App shell precache | Service worker (Workbox) | until the next deploy (`sw.js` revision) | SW update | T-307, FE-35 |
| CACHE-32 | `/api/v1/me/schedule` (read-only offline) | SW runtime cache, `NetworkFirst` | 14 days | logout (`caches.delete`), role switch | T-307, FE-35 |
| CACHE-33 | All other `/api` | — | `NetworkOnly`, never cached by the SW | — | T-307 |
| CACHE-34 | Form drafts | `localStorage` `eq:draft:{tenant}:{person}:{form_key}` | until submit; server copy 7 days | logout (SBD-58) | FE-30 |
| CACHE-35 | Offline check-in queue | IndexedDB, AES-GCM with a non-extractable device key | until 2xx | logout, after send | ATT-07, T-511 |
| CACHE-36 | Tenant theme CSS | memory (regenerated from `meta/app.brand`) | page lifetime | — | DS-10 |
| CACHE-37 | UI preferences (density, theme, collapsed groups, last tab) | `localStorage` | indefinite | never contains data from the API | FE-30, DS |

Tokens are never cached anywhere in the browser except in memory (FE-12, AUTH-03).

#### 6.4 CI/CD

| ID | What | Where | Rule |
|---|---|---|---|
| CACHE-40 | Cargo build cache (`Swatinem/rust-cache`), pnpm store (`actions/setup-node` `cache: pnpm`) | GitHub Actions cache | **CI only.** Release workflows never restore these caches, so a cache written by a PR run can't end up in a signed artifact (cache poisoning; zizmor `cache-poisoning`) [D-68] |
| CACHE-41 | Docker layer cache | — | Not used for releases (CTR-20). The image build is a copy plus one `apk add` and doesn't need it |
| CACHE-42 | sqlx offline data (`.sqlx/`) | repository | Committed; regenerated with `cargo sqlx prepare` (AGENTS §3) |

## 7. Decisions taken in this document

| ID | Decision | Why |
|---|---|---|
| D-66 | Tenant-resolution cache stays TTL-only (60 s) | Tenant status changes are rare and come from the platform; ≤ 60 s to suspend is acceptable |
| D-67 | Avatars use `private, max-age=3600` (T-303 said `public`) | Avatars can show children; `private` keeps them out of shared proxies. The logo stays `public` |
| D-68 | Release builds don't restore Actions caches; `release.yml` updated | Prevents cache poisoning of signed artifacts |
| D-69 | The runtime image is digest-pinned Alpine with `postgresql16-client`, not distroless (ADR-0019) | `pg_dump` is dynamically linked; copying it with its libraries into distroless is fragile |
| D-70 | `ritma healthcheck` subcommand for container health | No `curl`/`wget` dependency; works with a read-only root FS |
| D-71 | On `LISTEN` reconnect, NOTIFY-backed caches are cleared | Notifications sent during the disconnect are lost |
