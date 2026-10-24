---
id: T-803
title: Add the system tray menu and LAN access with the phones-and-tablets QR code
milestone: M8
status: draft
depends_on: [T-802]
spec_refs: [DST-20, DST-21]
estimated_size: S
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
The app keeps running in the tray and phones on the same Wi-Fi can open it by scanning a code.

## Context
- DST-20 tray items and DST-21 binding/port rules.

## Scope
### In
- Tray, QR dialog, start at login

### Out
- HTTPS (T-804/T-805)

## Acceptance criteria
- AC1. Closing the window keeps the server running; Quit stops it cleanly → `desktop::tray_lifecycle`

## Tests to write
| File | Test names |
|---|---|
| `apps/desktop/tests` | AC names |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(desktop): tray and LAN access [T-803]`, signed off

## Report

## Questions
