# 11 — Operations

## 1. Configuration (environment variables)

**OPS-01.** All configuration comes from env vars or an optional `ritma.toml` (env wins). Parsing uses `figment` or a hand-written struct with `envy`. Unknown `RITMA_*` variables produce a startup warning.

| Variable | Required | Default | Purpose |
|---|---|---|---|
| `DATABASE_URL` | ✅ | — | Postgres URL for role `eq_app` |
| `RITMA_DATABASE_OWNER_URL` | | = DATABASE_URL | Role used for migrations (`eq_owner`) |
| `RITMA_SECRET_KEY` | ✅ | — | 32-byte base64 master secret (SEC-01) |
| `RITMA_SECRET_KEY_PREVIOUS` | | — | For rotation |
| `RITMA_PUBLIC_URL` | ✅ | — | e.g. `https://school.example.com` (links in emails, cookies, CSP) |
| `RITMA_MODE` | | `single` | `single` \| `multi` (cloud) |
| `RITMA_LISTEN` | | `0.0.0.0:8080` | |
| `RITMA_DATA_DIR` | | `/var/lib/ritma` (docker), OS app-data dir (desktop) | Files, backups, setup token |
| `RITMA_FILES_BACKEND` | | `local` | `local` \| `s3` (`RITMA_S3_ENDPOINT`, `RITMA_S3_BUCKET`, `RITMA_S3_REGION`, `RITMA_S3_ACCESS_KEY`, `RITMA_S3_SECRET_KEY`) |
| `RITMA_SETUP_TOKEN` | | random on first boot | Pre-seed the setup token (1-click templates) |
| `RITMA_TRUSTED_PROXIES` | | `127.0.0.1/32,::1/128` + private ranges in docker | For X-Forwarded-* |
| `RITMA_LOG` | | `info` | `tracing` env filter |
| `RITMA_LOG_FORMAT` | | `json` (docker) / `pretty` (dev) | |
| `RITMA_BACKUP_SCHEDULE` | | `daily` | `off` \| `daily` (03:00 tenant TZ) |
| `RITMA_BACKUP_KEEP` | | `14` | Number of local backups kept |
| `RITMA_BACKUP_S3_*` | | — | Optional off-site copy |
| `RITMA_SMTP_URL` | | — | Optional **platform default** sender (cloud) used when a tenant hasn't configured email |
| `RITMA_CAPTCHA_PROVIDER/SITE_KEY/SECRET` | | — | turnstile \| hcaptcha |
| `RITMA_TILE_URL` | | OSM standard | Map tiles for the map pin |
| `RITMA_TELEMETRY` | | `off` | `off` \| `on` (DST-40) |
| `RITMA_METRICS_TOKEN` | | — | Enables `/metrics` with Bearer auth |

## 2. CLI

**OPS-02.** The binary `ritma` has these subcommands (`clap`):

| Command | Purpose |
|---|---|
| `serve` (default) | Run migrations (advisory lock 7_000_001), then start HTTP, the dispatcher, job workers and the cron planner |
| `migrate` | Run migrations only |
| `openapi` | Print the OpenAPI JSON (TST-10) |
| `backup --out <path>` / `restore --in <path> [--force]` | OPS-05 |
| `doctor` | Check DB connectivity, extensions, RLS enabled on all tenant tables, secret strength, public URL reachability, clock skew vs DB, disk space, pending migrations. Prints ✅/⚠️/❌ with fixes |
| `create-owner --email --name` | Recovery: create an owner and print an invite link (requires DB access) |
| `rotate-keys [--master] [--tenant-dek]` | SEC-01/02 |
| `seed-demo` | Demo data (DST-30); refuses unless `RITMA_DEMO=1` |
| `version` | Version, commit, build features |
| `healthcheck` | Exit 0 if `GET http://127.0.0.1:<port>/healthz` answers 200 within 2 s, else 1. Used by the container `HEALTHCHECK` (CTR-05) |

## 3. Runtime

- **OPS-03. Workers.**
  - Job workers: N = `min(4, cpus)` tokio tasks, each claiming one job at a time. Long jobs (cycle billing) check a cancellation token. A graceful shutdown on SIGTERM waits ≤ 25 s.
  - The dispatcher is one task.
  - The cron planner is one task per process. Multiple processes are safe thanks to dedupe keys (ARC-09).
- **OPS-04. Migrations.**
  - Forward-only, expand → migrate → contract across releases.
  - A migration never drops a column in the same release that stops using it.
  - `sqlx migrate` runs with checksums verified, and startup refuses to run on a DB with **newer** migrations than the binary knows (a downgrade guard).
- **OPS-05. Backups.**
  - `ritma backup` writes `{RITMA_DATA_DIR}/backups/eq-{tenant|all}-{YYYYMMDD-HHmm}.eqbak`, an encrypted (SEC-31) tar stream of:
    - `manifest.json` (app version, schema version, created_at, sha256 of parts)
    - `db.dump` (`pg_dump -Fc`; the docker image includes `postgresql-client` matching the server major; the desktop bundles it)
    - `files/`
  - `restore` refuses if the manifest schema is newer than the binary, and requires an empty DB unless `--force`.
  - The daily job is configured by `RITMA_BACKUP_SCHEDULE` and pruned to `RITMA_BACKUP_KEEP`.
  - Settings → Backups (owner) lists backups, and offers "Back up now", "Download" (signed URL) and restore instructions.
  - CI runs a backup → restore → checksum round trip (TST-13).
- **OPS-06. Observability.**
  - Structured logs with `request_id`, `tenant_id`, `actor_id` (ids only).
  - `/healthz` (process up) and `/readyz` (DB ping ≤ 500 ms, migrations current).
  - Optional `/metrics` (Prometheus): HTTP latency histograms per route, job counts and durations by kind, outbox lag, notification failures by channel, DB pool usage.
- **OPS-07. Resource budget.** Idle RSS ≤ 150 MB. DB pool default 10 (env `RITMA_DB_POOL`). Request timeout 30 s (uploads 120 s).
- **OPS-08. Dead letters and alerts.** Dead-lettered events and jobs create `attention_items` (high severity) and appear in Settings → System health, with retry and discard buttons (owner).
- **OPS-09. Time.** The server compares its clock to the DB's `now()` at startup and hourly, and logs an error if skew > 2 s. Business logic uses DB-consistent UTC from `Clock` (ARC-10).

## 4. Upgrades

**OPS-10.** An upgrade is "replace the binary/image and restart". Migrations run automatically. The release notes flag any manual step, and the goal is none. The in-app "Update available" notice (owner only) polls `https://releases.<domain>/latest.json` once a day **only if** `RITMA_TELEMETRY` or `RITMA_UPDATE_CHECK=on` (the default is on for the update check, which sends no data beyond the request itself). Desktop uses the Tauri updater (DST-24).
