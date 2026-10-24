---
id: T-802
title: Add embedded Postgres (postgresql_embedded) and OS-keychain secrets with the recovery kit
milestone: M8
status: draft
depends_on: [T-801]
spec_refs: [DST-20, SEC-31]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
No database to install: the app brings its own Postgres and keeps its secrets in the OS keychain.

## Context
- DST-20: PG 16 bundled (offline install default), random port, generated password in keychain (`keyring`), `EQ_SECRET_KEY` generated on first run, 'Export recovery kit' PDF/QR (SEC-31).

## Scope
### In
- PG lifecycle (start/stop/upgrade path), keychain, recovery kit

### Out
- —

## Acceptance criteria
- AC1. Fresh install on Windows/macOS/Linux CI runners boots and passes `/readyz` → `desktop::embedded_pg`
- AC2. Recovery kit restores on another machine → `recovery_kit`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): embedded Postgres and keychain [T-802]`, signed off

## Report

## Questions
