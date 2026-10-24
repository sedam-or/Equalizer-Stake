<p align="center">
  <!-- banner: brand/assets/banner-{light,dark}.png (see brand/BRAND.md §8) -->
  <b>Ritma</b> · <i>the rhythm of your school</i><br/>
  <b>The open-source operating system for course schools & studios.</b><br/>
  Scheduling · enrollment · monthly fees · attendance · parent messages. Set up in 3 steps.
</p>

<p align="center"><a href="README.id.md">🇮🇩 Baca dalam Bahasa Indonesia</a></p>

> **🚧 Status: v2 is being rebuilt from the spec.** The original capstone (v0) is archived in `legacy/`.
> Follow progress in [ROADMAP.md](ROADMAP.md) and [tasks/INDEX.md](tasks/INDEX.md).

<!-- HERO GIF: 3-step setup (brand/BRAND.md §8) -->

**Try the demo** · **Start free in the cloud** · **[Self-host](#self-host)** · Download for Windows / macOS / Linux
*(links go live at v1.0)*

## Why

- **Set up your school in 3 steps.** Pick a template, switch on features, review the rules. Done in under 10 minutes.
- **Zero double bookings, guaranteed by the database.** Not by a hopeful check in code: a PostgreSQL exclusion constraint, proven by a 50-requests → 1-booking test on every commit.
- **Your rules, not ours.** Fees, due dates, late fees, opening hours, cancellation credits and check-in radius are all settings, with a plain-language summary of what each one does.
- **Built for how small schools really work.** WhatsApp-first reminders, one invoice per parent for all their children, bank-transfer reference codes, and Bahasa Indonesia and English.
- **Your data, your choice of home.** The same software on your front-desk PC, your own server, or our cloud. Export everything anytime.

## Features (modules, switch on what you need)

| Module | What it does |
|---|---|
| Scheduling *(always on)* | Rooms, courses, recurring weekly slots, conflict-free booking with smart alternatives, holidays |
| Enrollment | Online registration, parents with many children, levels, leave, make-up credits |
| Billing | Monthly invoices, payment codes, proof upload and verification, late fees, freeze/unfreeze |
| Attendance | Teacher check-in (GPS / time / photo / face), student attendance |
| Studio rental | Hourly bookings with approval, deposits and expiring holds |
| Notifications | Email + WhatsApp + in-app, with retries and fallback |
| Reports | Owner dashboard, utilisation, receivables, printable reports |
| Feedback | Progress notes to parents, teacher ratings |

## Self-host

One binary plus PostgreSQL. Choose what suits you (available from v1.0):

- **Docker one-liner** (any Linux server): `curl -fsSL https://get.<domain>/ | sh`
- **1-click**: Railway · Render · DigitalOcean · Coolify · CasaOS · YunoHost
- **Desktop app** for the school's PC (Windows / macOS / Linux), with secure access from phones

## Built with

Rust (axum, sqlx) · PostgreSQL · React + Vite + Tailwind · Tauri (desktop).
Architecture and decisions: [`spec/`](spec/00-index.md) · [ADRs](spec/adr/).

## Contributing

Add a template for your kind of school **in YAML, no Rust needed**, translate, or pick a `good first issue`.
See [CONTRIBUTING.md](CONTRIBUTING.md) and [GOVERNANCE.md](GOVERNANCE.md).

## Story

Born as a university capstone for a real Indonesian music school, and now rebuilt as open source so that any course school can use it.

## License

[AGPL-3.0](LICENSE) for v2. The archived v0 in `legacy/` is MIT. The name and logo are covered by [TRADEMARK.md](TRADEMARK.md).
