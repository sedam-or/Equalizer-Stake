# Roadmap

The detailed, task-level plan lives in [`tasks/INDEX.md`](tasks/INDEX.md). The milestones are:

| Milestone | Theme | User-visible result |
|---|---|---|
| **M0** Foundation | Workspace, CI, core plumbing | `equalizer serve` boots, `/healthz` works |
| **M1** Core | Tenants, identity, policies, modules, audit | Log in; policies and modules exist via the API |
| **M2** Scheduling | Catalog, sessions, series, zero double-booking | The 50→1 concurrency badge goes green |
| **M3** Setup & web app | SPA shell, design system, 3-step wizard, schedule UI | **A school is set up in 3 steps** and can schedule lessons |
| **M4** Enrollment & billing | Registration, guardians, invoices, payments, late fees, freeze | Monthly fees run themselves |
| **M5** Attendance & notifications | Check-in, marks, email/WhatsApp | Parents get reminders; teachers check in |
| **M6** Rental, reports, feedback, privacy | Studio rental, dashboard, progress notes, data rights | The full v1 feature set |
| **M7** Release v1.0 | Docker one-liner, 1-click templates, docs, demo | **Public launch** |
| **M8** Desktop | Tauri app, embedded Postgres, tunnel | Install on the front-desk PC |
| **M9** Cloud | Multi-tenant platform, plans, subscriptions | Sign up and go |

Later (v1.x and beyond): payment gateways (Xendit, Midtrans, Stripe), template overrides for notifications, multi-location reports, custom roles, public API tokens and webhooks, and more presets from the community.
