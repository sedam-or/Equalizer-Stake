# ADR-0019: A digest-pinned Alpine runtime image with the PostgreSQL client

- Status: Proposed (2026-10-25), accepted on merge of the PR that adds `spec/26-containers-and-caching.md`. Supersedes the "distroless" wording in DST-05 and OSS §4.

## Context
The `ritma` binary is a static musl build with the SPA, migrations and presets embedded. The image also needs `pg_dump`/`pg_restore` for `ritma backup` (OPS-05). DST-05 asked for `gcr.io/distroless/static` plus a static `pg_dump`, with Alpine as the fallback.

`pg_dump` is dynamically linked against libpq, OpenSSL, zlib, lz4 and zstd. There is no maintained static build. Copying it with its loader and libraries into a distroless image would be a hand-made, unscanned bundle that breaks silently on updates.

Options considered:
1. **Distroless static + copied `pg_dump` and libraries.** Smallest attack surface on paper, fragile in practice, and invisible to scanners.
2. **Distroless base for the app and `pg_dump` run from the `postgres` container** (`docker exec`). Couples the app to the compose layout and doesn't work on 1-click hosts with a managed database.
3. **Digest-pinned Alpine with `postgresql16-client` from the distro, non-root, read-only root FS.**

## Decision
Option 3 (spec 26 §2–§4): Alpine pinned by digest and bumped by Dependabot, `apk add postgresql16-client` only, UID 10001, `ritma healthcheck` instead of curl, read-only root filesystem with `cap_drop: [ALL]` and `no-new-privileges` in every template, Trivy gate on CRITICAL, cosign signature, SBOM and provenance.

## Consequences
- ✅ `pg_dump` comes from a maintained distro package and is covered by Trivy and the SBOM.
- ✅ Size stays within the 60 MB budget (CTR-07).
- ❌ The image contains BusyBox and `apk`. Mitigated by running non-root on a read-only filesystem with no capabilities; nothing in the runtime path executes a shell.
