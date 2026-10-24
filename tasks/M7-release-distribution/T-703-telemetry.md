---
id: T-703
title: Add opt-in anonymous telemetry with a plain-language toggle and documented payload
milestone: M7
status: draft
depends_on: [T-702]
spec_refs: [DST-40]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Measure adoption honestly: off by default, anonymous, and fully documented.

## Context
- DST-40 payload and rules in `spec/14-distribution.md` §7; `EQ_TELEMETRY` env plus owner toggle; weekly job; `docs/telemetry.md`.

## Scope
### In
- Job, toggle, docs

### Out
- The receiving endpoint (M9 platform)

## Acceptance criteria
- AC1. Off by default: no request sent (mock server) → `telemetry::off_by_default`
- AC2. Payload matches the documented schema and contains no PII → `payload_schema`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-server/tests/telemetry.rs` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(ops): opt-in telemetry [T-703]`, signed off

## Report

## Questions
