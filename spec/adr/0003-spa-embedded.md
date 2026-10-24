# ADR-0003: React + Vite SPA embedded in the binary (no Next.js/SSR)

- Status: Accepted (2026-10-25)

## Context
v0 used Next.js 15 with SSR. The app sits behind a login (SEO is irrelevant) and must ship inside one binary and the desktop app.

## Decision
A **React 19 + Vite + TanStack Router/Query + Tailwind 4 + shadcn/ui** SPA, built to static files and embedded with `rust-embed`. The marketing site is a separate Astro project.

## Consequences
- ✅ No Node runtime in production. The same assets serve the cloud, docker and desktop.
- ✅ Most v0 components, tokens and know-how carry over. It has the largest UI contributor pool.
- ❌ The first load relies on the client bundle. Mitigations: a 200 KB gzip budget and route-level code splitting.
