---
id: T-704
title: Add the release pipeline: static binaries, multi-arch image, SBOM, checksums, cosign, release-please
milestone: M7
status: draft
depends_on: [T-703]
spec_refs: [DST-01, DST-05, SEC-22, OSS-20, OSS-21]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
One tag produces every artifact every channel needs, signed and reproducible.

## Context
- DST-05 steps 1–6 in `spec/14-distribution.md` §2 (targets, distroless image ≤ 60 MB with pg_dump, non-root uid 10001, SBOM CycloneDX, cosign keyless, `latest.json`).
- release-please config for Conventional Commits; `RELEASING.md` already describes the human steps.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `.github/workflows/release-please.yml`, `.github/workflows/release.yml`, `release-please-config.json` and `.release-please-manifest.json` already exist (OSS-20). This task completes them: write `deploy/docker/Dockerfile` (distroless, non-root uid 10001, copies the prebuilt `ritma` musl binary for `TARGETARCH` from `dist/`), verify the release-please `rust` + `cargo-workspace` config against the real workspace, add `latest.json` publishing, and do a dry run with `workflow_dispatch`.

## Scope
### In
- `deploy/docker/Dockerfile`; complete `.github/workflows/release.yml` and the release-please config (they exist)

### Out
- Desktop bundles (M8)

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

## Questions
