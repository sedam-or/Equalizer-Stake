# ADR-0011: Desktop = Tauri shell embedding the server + embedded Postgres; HTTPS via tunnel by default

- Status: Accepted (2026-10-25)

## Context
Many small schools have one front-desk PC and poor IT. Phones need HTTPS for camera, GPS and PWA install.

## Decision
- A Tauri 2 app runs `eq_server` in-process plus `postgresql_embedded`.
- The default phone access mode is a **secure tunnel** with an issued `<slug>.<tunnel-domain>` hostname.
- An advanced local-CA mode is available for offline use.

## Consequences
- ✅ Double-click install with real HTTPS on phones.
- ❌ Running the tunnel broker is an operational cost. Mitigations: rate limits and the abuse policy. A user can also bring their own tunnel token.
