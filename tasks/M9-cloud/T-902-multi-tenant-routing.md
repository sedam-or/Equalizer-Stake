---
id: T-902
title: Add multi-mode routing hardening and custom domains with on-demand TLS checks
milestone: M9
status: draft
depends_on: [T-901]
spec_refs: [CLD-01, CLD-02, CLD-03]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Each school gets its own address, optionally its own domain, safely.

## Context
- CLD-01/02: subdomain routing (T-104), unknown host → platform 404 page; custom domains via `GET /platform/domain-check?domain=` for Caddy `on_demand_tls`.

## Scope
### In
- Domain table and verification flow, check endpoint

### Out
- —

## Acceptance criteria
- AC1. Unverified domain → check endpoint 404; verified → 200 → `platform::domain_check`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): routing and custom domains [T-902]`, signed off

## Report

## Questions
