---
id: T-706
title: Add the Docker one-liner installer (install.sh) and the documented compose stack with Caddy
milestone: M7
status: draft
depends_on: [T-705]
spec_refs: [DST-30, DST-31, DST-02]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
A school's IT friend runs one command on a $5 server and gets HTTPS and a setup link.

## Context
- Runtime hardening for the app service exactly as `spec/26-containers-and-caching.md` CTR-10 (read-only root FS, tmpfs `/tmp`, uid 10001, `cap_drop: [ALL]`, `no-new-privileges`, limits, healthcheck); `postgres` and `caddy` pinned by digest; Caddy without a response cache (CACHE-24..26).
- DST-30 behaviour 1–5 (detect, ask domain + email, write `/opt/ritma/` with compose, `.env` 600, Caddyfile, wait for `/readyz`, print the setup URL, idempotent re-run) and DST-31 in `spec/14-distribution.md` §6.
- POSIX `sh`, shellcheck clean; non-interactive flags for automation.

## Scope
### In
- `deploy/install.sh`, `deploy/docker/docker-compose.yml`, `deploy/docker/Caddyfile`, docs

### Out
- —

## Files
| Path | Action | Purpose |
|---|---|---|
| `deploy/docker/docker-compose.yml` | create or modify | Path named in this task's contract. |
| `deploy/install.sh` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4, T6 (SBD-30).
- Data classes touched: S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- S/T/I: forged or replayed external events → verify signatures in constant time before parsing, deduplicate provider event IDs, and never log raw bodies (SBD-63).
- Abuse-case acceptance criterion: `idempotent` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. In a CI VM/container with Docker: install, then the setup URL responds 200 → `installer::smoke (CI job)`
- AC2. Re-run keeps `.env` secrets → `idempotent`
- AC3. shellcheck passes → `lint`

## Tests to write
| File | Test names |
|---|---|
| `CI job `installer-smoke`` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(deploy): installer and compose [T-706]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
