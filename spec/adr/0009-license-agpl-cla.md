# ADR-0009: License v2 under AGPL-3.0-only with a CLA; v0 stays MIT

- Status: Accepted (2026-10-25)

## Context
The sustainability plan is a managed cloud (15-cloud). Under a permissive license, competitors could run closed SaaS clones. The v0 code is MIT and co-authored by capstone teammates.

## Decision
- v2 code (everything outside `legacy/`) is **AGPL-3.0-only**.
- Contributors sign a **CLA** (CLA Assistant bot) granting the project the right to relicense, so a commercial license can be offered to organisations that can't use AGPL.
- `legacy/v0-capstone.zip` keeps its MIT license. v2 ports behaviour, never v0 code verbatim.
- The name and logo are protected separately by `TRADEMARK.md`.

## Consequences
- ✅ Self-hosting is free, and hosted forks must publish their changes.
- ❌ Some companies avoid AGPL dependencies. This is irrelevant for an end-user application. The CLA creates some friction for contributors, so it's explained clearly in CONTRIBUTING.
