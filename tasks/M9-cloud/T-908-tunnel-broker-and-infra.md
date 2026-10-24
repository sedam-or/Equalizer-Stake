---
id: T-908
title: Add the tunnel broker service for desktop secure links and the cloud infrastructure docs
milestone: M9
status: draft
depends_on: [T-907]
spec_refs: [DST-22, CLD-30, CLD-31, CLD-32, CLD-34]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Desktop schools get free, rate-limited HTTPS hostnames, and the cloud is documented well enough for someone else to run it.

## Context
- DST-22 broker contract (issue `<slug>.<tunnel-domain>` per install, rate limits, abuse policy); CLD-30..34 regions, infra, email, SLA in `docs/cloud/`.

## Scope
### In
- Broker endpoints in eq-platform, infra docs and runbooks

### Out
- Actual cloud provisioning

## Acceptance criteria
- AC1. Broker issues one hostname per install id and rate-limits re-registration → `platform::broker`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-platform/tests` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(platform): tunnel broker and infra docs [T-908]`, signed off

## Report

## Questions
