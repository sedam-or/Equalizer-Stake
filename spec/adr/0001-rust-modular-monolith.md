# ADR-0001: Rust modular monolith in a single binary

- Status: Accepted (2026-10-25)
- Deciders: maintainer

## Context
v0 claimed three stacks: FastAPI (working), Go (empty skeleton) and Next.js. The product must be self-hostable by non-technical schools on a $5 VPS or on a school PC (14-distribution), must guarantee zero double bookings and must handle money exactly.

## Decision
Build the backend as a **modular monolith in Rust**: a Cargo workspace with one crate per module, compiled into one `equalizer` binary that also serves the embedded SPA and runs jobs in-process.

## Consequences
- ✅ Single static binary (about 20–40 MB RAM idle) that the desktop app (Tauri, Rust) can embed as a library. Strong typing for state machines, money and policies. Compile-time module features.
- ✅ No service mesh, broker or Redis, so each install has fewer moving parts.
- ❌ A smaller contributor pool and slower early velocity than Go or Python. Mitigations: strict task files (tasks/), a simple crate layout and a "presets are YAML" contribution path for non-Rust contributors.
- ❌ Compile times. Mitigations: workspace layering, `sccache` in CI, `cargo check` in dev.

## Alternatives considered
- Go (the existing skeleton): faster onboarding, weaker domain modelling, and no in-process embedding with Tauri.
- Keep FastAPI: needs a runtime, makes single-binary distribution hard, and the domain model would have to be rewritten anyway.
