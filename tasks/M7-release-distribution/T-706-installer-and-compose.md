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
- DST-30 behaviour 1–5 (detect, ask domain + email, write `/opt/ritma/` with compose, `.env` 600, Caddyfile, wait for `/readyz`, print the setup URL, idempotent re-run) and DST-31 in `spec/14-distribution.md` §6.
- POSIX `sh`, shellcheck clean; non-interactive flags for automation.

## Scope
### In
- `deploy/install.sh`, `deploy/docker/docker-compose.yml`, `deploy/docker/Caddyfile`, docs

### Out
- —

## Acceptance criteria
- AC1. In a CI VM/container with Docker: install, then the setup URL responds 200 → `installer::smoke (CI job)`
- AC2. Re-run keeps `.env` secrets → `idempotent`
- AC3. shellcheck passes → `lint`

## Tests to write
| File | Test names |
|---|---|
| `CI job `installer-smoke`` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(deploy): installer and compose [T-706]`, signed off

## Report

## Questions
