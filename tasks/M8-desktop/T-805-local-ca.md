---
id: T-805
title: Add the local-only HTTPS mode (local CA, mDNS name, guided certificate install)
milestone: M8
status: draft
depends_on: [T-804]
spec_refs: [DST-22]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Schools without reliable internet still get HTTPS on the LAN.

## Context
- DST-22 local mode: `rcgen` CA, cert for LAN IP + `ritma.local` (`mdns-sd`), HTTPS on 8443, QR to download the CA with per-OS instructions.

## Scope
### In
- CA generation and storage, HTTPS listener, guide screens

### Out
- —

## Acceptance criteria
- AC1. A client trusting the CA connects over TLS 1.2+ → `desktop::local_ca_tls`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): local CA mode [T-805]`, signed off

## Report

## Questions
