---
id: T-704
title: Add the release pipeline: static binaries, multi-arch image, SBOM, checksums, cosign, release-please
milestone: M7
status: todo
depends_on: [T-703]
spec_refs: [DST-01, DST-05, SEC-22, OSS-20, OSS-21]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
One tag produces every artifact every channel needs, signed and reproducible.

## Context
- DST-05 steps 1–6 in `spec/14-distribution.md` §2 (targets, Alpine image ≤ 60 MB with pg_dump (ADR-0019), non-root uid 10001, SBOM CycloneDX, cosign keyless, `latest.json`).
- release-please config for Conventional Commits; `RELEASING.md` already describes the human steps.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `.github/workflows/release-please.yml`, `.github/workflows/release.yml`, `release-please-config.json` and `.release-please-manifest.json` already exist (OSS-20). This task completes them: write `deploy/docker/Dockerfile` exactly as `spec/26-containers-and-caching.md` §2 (digest-pinned Alpine + `postgresql16-client`, ADR-0019; non-root uid 10001, copies the prebuilt `ritma` musl binary for `TARGETARCH` from `dist/`), verify the release-please `rust` + `cargo-workspace` config against the real workspace, add `latest.json` publishing, and do a dry run with `workflow_dispatch`.

- **Maintenance releases (BRANCHING.md §5, RELEASING.md):** `release-please.yml` already accepts `target-branch`. When the released tag is not the highest semver tag (a patch from `release/vX.Y`), `release.yml` must not mark the GitHub release as latest, must not push the image `latest` tag (`docker/metadata-action` `flavor: latest=false`) and must not update `latest.json`.

## Scope
### In
- `deploy/docker/Dockerfile`; complete `.github/workflows/release.yml` and the release-please config (they exist)

### Out
- Desktop bundles (M8)

## Files
| Path | Action | Purpose |
|---|---|---|
| `paths explicitly named by Scope and Tests to write` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T6 (SBD-30).
- Data classes touched: S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- S/T/I: forged or replayed external events → verify signatures in constant time before parsing, deduplicate provider event IDs, and never log raw bodies (SBD-63).
- Abuse-case acceptance criterion: `CI check` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. A dry-run release on a fork produces all assets and verifiable signatures → `manual: documented in Report`
- AC2. Image size ≤ 60 MB → `CI check`

## Tests to write
| File | Test names |
|---|---|
| `CI` | release dry run |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `build: release pipeline [T-704]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
