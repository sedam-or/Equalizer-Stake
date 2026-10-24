# 01 — Product

## 1. Vision

**PRD-01.** Equalizer is an open-source operating system for independent course schools and studios: music, dance, martial arts, tutoring, art, swimming, and rehearsal or recording studios. It replaces the usual mix of WhatsApp, spreadsheets and paper with one system that is:
- **correct by construction**: double booking is impossible, and money is exact;
- **configured, not customised**: every business rule is data, set in a 3-step wizard;
- **modular**: a school turns on only what it uses;
- **owned by the school**: the same AGPL code runs in our cloud, on a $5 VPS, or on the school's own PC.

## 2. Problem (first principles)

A small school's operations come down to five recurring jobs:

| Job | Today | Failure cost | Module |
|---|---|---|---|
| Put a student, a teacher and a room in the same time slot | Admin brain + WhatsApp | 3–5 double bookings/month; lost trust | scheduling (core) |
| Know who is enrolled, at what level, under which guardian | Excel | Wrong invoices, lost leads | enrollment |
| Collect monthly fees on time | Manual reminders, bank-transfer screenshots | Late payments, 2–3 h/day admin | billing + notify |
| Prove lessons happened | Paper sign-in | 20–30% no-show disputes, unpaid teachers | attendance |
| Rent the studio by the hour | WhatsApp | Double bookings, unpaid holds | rental |

The owner also needs one read-only view of health (reports) and protection against misuse (audit).

## 3. Personas and roles

| Role | Who | Primary device | Top tasks |
|---|---|---|---|
| **owner** | Business owner | Desktop/phone | Setup wizard, rules, modules, admins, dashboard, overrides, audit |
| **admin** | Front-desk/operations | Desktop | Students, schedule, invoices, payment verification, attendance review |
| **instructor** | Teacher/coach/tutor | Phone | My schedule, check-in, mark student attendance, progress feedback |
| **guardian** | Parent paying for 1..N students | Phone (60%+) | Children's schedule, invoices, pay or upload proof, leave requests, ratings |
| **student** | Adult learner (self-paying), or a minor with view-only access | Phone | Own schedule, invoices (if payer), bookings (rental) |

**PRD-02.** Roles are fixed in v1 (no custom roles). Their *labels* are terminology and can be renamed by presets (e.g. instructor → "Coach", guardian → "Parent").

**PRD-03.** One Person can hold several roles in one tenant (an instructor may also be a guardian). The UI shows a role switcher when a person has more than one.

## 4. The core product contract — setup in 3 steps

**PRD-04.** A brand-new install or cloud signup reaches a fully working school in **at most 3 wizard steps**. Each step has safe defaults, so pressing *Next → Next → Finish* without changing anything produces a valid, usable configuration.

| Step | Title (EN / ID) | The user does | The system does |
|---|---|---|---|
| 1 | **Your school** / *Sekolah Anda* | School name; country (auto-sets currency, timezone, language); pick a template; on self-host first run, also the owner email and password | Loads the preset, creates the tenant, owner, default location and N rooms |
| 2 | **Features** / *Fitur* | Toggle module cards | Resolves dependencies live and explains locked toggles |
| 3 | **Rules** / *Aturan* | Review one card per enabled module ("Essentials" open, "Advanced" collapsed), prefilled from the template | Validates, shows a plain-language summary, saves policy version 1, goes to the dashboard |

**PRD-05.** The same screens are reused after setup: *Settings → Features* (step 2) and *Settings → Rules* (step 3). Changing a rule creates a new policy version that takes effect now or on a chosen future date (POL-05).

**PRD-06.** Every rule screen shows a live **plain-language summary** next to the form, for example: "Invoices are issued on the 5th and due on the 10th. From the 11th, a Rp100.000 late fee is added and the household is frozen until paid in full." The summary is generated from the policy, never hand-written per tenant.

**PRD-07.** Success metric: median time from first page load to the first scheduled session is **under 10 minutes** for the music-school template. This is measured by the e2e test (TST-07) and by opt-in telemetry.

## 5. Modules (v1)

| Module | Kind | Depends on | One-liner |
|---|---|---|---|
| `core` | always on | — | Tenants, locations, resources, people, policies, calendar holds, events, jobs, files |
| `identity` | always on | core | Login, sessions, RBAC, invites, password reset |
| `audit` | always on | core | Immutable hash-chained log, retention, anonymisation |
| `scheduling` | always on | core | Offerings, sessions, closures/holidays, conflict-free scheduling |
| `enrollment` | optional | scheduling | Students, guardians, registration, levels, recurring patterns, leave, make-up credits |
| `billing` | optional | enrollment **or** rental | Invoices, payments, late fees, freeze, payment references |
| `attendance` | optional | scheduling | Instructor check-in (GPS/time/photo/face), student attendance marks, review queue |
| `rental` | optional | scheduling | Hourly resource reservations with approval and holds |
| `notify` | optional | core | Email, WhatsApp, in-app; templates; retries, fallback, circuit breaker |
| `reports` | optional | core | Owner dashboard, eligibility, exports |
| `feedback` | optional | enrollment | Instructor→guardian progress notes, guardian→instructor ratings |

## 6. Non-goals for v1

- Native mobile apps (the PWA covers this).
- An automatic payment gateway. v1 does manual verification only, behind a `PaymentProvider` trait; gateways come in v1.x.
- Custom roles or permission editing.
- Custom rule scripting (no DSL, no user-written code). Rules are parameters of known mechanisms (ADR-0004).
- Third-party runtime plugins (WASM). Modules are first-party crates.
- Accounting and general ledger, payroll, inventory, LMS/content, video lessons.
- Multi-location is **modelled** (Location entity) but the v1 UI is optimised for one location. A second location is allowed but there are no cross-location reports.

## 7. Quality targets (from the v0 NFRs, kept)

| ID | Target |
|---|---|
| PRD-10 | p95 page interactive ≤ 2 s on 4G; API p95 ≤ 300 ms for reads, ≤ 1 s for a schedule save |
| PRD-11 | Runs on 1 vCPU / 1 GB RAM with ≤ 500 students. App RSS ≤ 150 MB idle |
| PRD-12 | Zero double bookings under concurrency (DB-enforced, TST-05) |
| PRD-13 | Notifications dispatched ≤ 15 s after the triggering event (when the channel is healthy) |
| PRD-14 | WCAG 2.2 AA, mobile-first from 320 px, keyboard navigable |
| PRD-15 | Languages: `en`, `id` at launch. All strings in catalogs |
| PRD-16 | Backups daily, RPO ≤ 24 h self-hosted (≤ 1 h cloud), restore tested in CI |
