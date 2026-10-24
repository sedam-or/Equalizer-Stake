---
id: T-804
title: Add the secure-link mode (Cloudflare tunnel via broker) so phones get HTTPS for camera, GPS and PWA
milestone: M8
status: draft
depends_on: [T-803]
spec_refs: [DST-22, SEC-06]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Check-in with camera and GPS works on phones even though the server is a PC in the office.

## Context
- DST-22 secure link mode: `cloudflared` sidecar, hostname from the tunnel broker API (M9 T-908), fallback user token; `RITMA_PUBLIC_URL` set to the tunnel URL.

## Scope
### In
- Tunnel lifecycle, first-run dialog, settings, status indicator

### Out
- Broker service (T-908)

## Acceptance criteria
- AC1. With a mock broker, the tunnel starts and `meta/app` reports the https public URL → `desktop::tunnel_mock`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | AC names |

## Definition of done
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): secure tunnel [T-804]`, signed off

## Report

## Questions
