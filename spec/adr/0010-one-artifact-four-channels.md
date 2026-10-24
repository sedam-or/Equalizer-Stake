# ADR-0010: One release artifact feeds four distribution channels

- Status: Accepted (2026-10-25)

## Decision
- One binary per platform plus one container image per release.
- The managed cloud, the 1-click templates, the Docker one-liner and the desktop app all use it, unchanged.
- Every channel lands on the same `/setup` wizard.

## Consequences
- ✅ One code path to test, and bugs fixed once.
- ❌ Channel-specific UX (tray, QR codes, tunnel) lives outside the core binary, in the Tauri shell or the platform crate.
