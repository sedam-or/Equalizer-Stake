# GTM — launch, growth, community, sustainability

> GitHub discoverability, repo setup and the launch-day runbook: [`GITHUB.md`](GITHUB.md).

## 1. North-star and metrics

- **North-star:** **weekly active schools**, meaning schools with ≥ 1 session scheduled and ≥ 1 non-owner login in the week, counted across cloud plus opt-in telemetry.
- **Leading indicators:**

| Metric | Source | Target 6 months after v1.0 |
|---|---|---|
| GitHub stars | GitHub | 2,000 |
| Deploys (opt-in pings + cloud signups) | DST-40, CLD | 300 installs, 100 cloud tenants |
| Setup completion rate | Telemetry/cloud: `setup_completed / step1_started` | ≥ 70% |
| Median time-to-first-session | Cloud events | < 10 min |
| Paying cloud schools | Billing | 30 |
| External contributors (merged PRs) | GitHub | 25, including ≥ 5 presets and translations |

## 2. Launch sequence (tied to milestones in `tasks/INDEX.md`)

| Phase | When | What | Exit criteria |
|---|---|---|---|
| **0. Build in public** | M0–M3 | Public repo from day 1 with the roadmap, weekly progress posts (X/LinkedIn/Threads, ID + EN), and a "design notes" blog series (policy-as-data, the 50→1 concurrency proof, Rust modular monolith). Collect a waitlist on the landing page | 300 waitlist emails, 150 stars |
| **1. Private beta** | M4–M6 | 3–5 real schools: the original music school (anonymised case study), plus one each of dance, tutoring and studio rental. Weekly calls, and a shared WhatsApp group for support | 4 schools running real billing for 1 cycle; top-10 issues fixed |
| **2. Public launch v1.0** | M7 | Same day: Show HN ("Show HN: an open-source OS for music and dance schools, one Rust binary"), r/selfhosted, r/rust (the technical angle), Product Hunt (Tuesday), and Indonesian communities (Kaskus/IndieHackers ID/Telegram dev groups, music-teacher Facebook groups). PRs to `awesome-selfhosted` (Booking and Scheduling), `awesome-rust` (Applications) and `awesome-indonesia` | 500 stars in the launch week; 50 deploys |
| **3. Desktop launch** | M8 | "Runs on your front-desk PC": video tutorials in Bahasa on YouTube; outreach to school associations | 100 desktop installs |
| **4. Cloud GA** | M9 | Pricing live, plus migration offers ("we import your Excel for free in month 1") | 30 paying schools |

## 3. README as a conversion funnel (order is normative)

1. Banner (logo, tagline) and a one-line value statement.
2. **Hero GIF** of the 3-step setup.
3. Three buttons: **Try the demo**, **Start free in the cloud**, **Self-host** (anchor), with **Download for Windows/macOS** underneath.
4. "Why" in 5 bullets (the USPs from BRAND §4), each with a tiny proof link.
5. Feature grid by module, with a screenshot thumbnail each.
6. Self-host section: deploy buttons, the Docker one-liner, desktop downloads.
7. Badges: CI, **"double-booking test: passing (50→1)"**, license AGPL, latest release, translation %.
8. "Built with" (Rust, Postgres, React) plus links to the architecture docs, for developer credibility.
9. Roadmap, contributing ("add a preset in YAML, no Rust needed"), community links.
10. The origin story (2 lines), license and trademark.

A Bahasa Indonesia README (`README.id.md`) is linked at the top.

## 4. Community playbook

- **Channels:** GitHub Discussions (Q&A, Ideas, Show and tell), Discord (EN), Telegram group (ID), and monthly community calls.
- **Contribution ladders** (easiest first):
  1. Translate (Weblate, project `ritma/app`).
  2. Add or improve a **preset** (YAML).
  3. Docs and screenshots.
  4. `good first issue` (labelled; each one has a task file).
  5. Module work through an RFC.
- **Labels:** `good first issue`, `help wanted`, `preset`, `i18n`, `module:<id>`, `rfc`, `needs-repro`, `security` (private).
- **Recognition:** all-contributors bot, a monthly "contributor spotlight", and a CONTRIBUTORS section in release notes.
- **Response SLAs** (maintainer): issues triaged ≤ 3 days; PR first review ≤ 5 days; security ≤ 48 h.
- **Honest growth only:** no star buying, no fake reviews, no spam DMs. Stars come from the demo, the GIF, the concurrency story and helpful posts.

## 5. Local partner program ("Mitra")

Indonesian freelancers and IT shops already sell setups to schools. Give them a reason to choose us:
- A "Certified Mitra" listing on the website after they complete a free setup course (docs + quiz).
- Mitra get a referral share of the cloud subscription (20% for 12 months), or they can sell self-hosted setup and support themselves. AGPL allows this, and we encourage it.
- Mitra materials: a one-page brochure (ID), a demo script and a migration checklist.

## 6. Pricing and packaging

Cloud plans are in `spec/15-cloud.md` §3. Guidance:
- Self-host is **free with all features**. We never cripple self-host.
- Paid services (cloud, setup/migration help, priority support, WhatsApp credit bundles through partners, custom presets) are services, not code.
- Annual cloud plans get 2 months free. Schools are seasonal, so offer "pause during holidays" (read-only, 50% off) on Standard.

## 7. Content plan (first 12 posts)

1. Why we rebuilt a capstone project as open source (story).
2. Policy-as-data: how a school configures 40 business rules in 3 steps.
3. The one Postgres constraint that makes double booking impossible.
4. A Rust modular monolith: one binary from desktop to cloud.
5. Designing for parents on phones: lessons from v0.
6. Late fees without drama: snapshots and effective-dated rules.
7. Running your school on a $5 server (tutorial, ID + EN).
8. From Excel to Ritma in 15 minutes (import tutorial).
9. "The rhythm of your school": why we named it Ritma (from *ritme*, rhythm).
10. Privacy for children's data under UU PDP (practical guide).
11. Case study: beta school #1.
12. How to write a preset for your kind of school.

## 8. Funding options parallel to revenue

- GitHub Sponsors and Open Collective (transparent budget).
- Grants: NLnet NGI Zero (digital commons, EU-friendly scope: privacy and self-hosting), and Indonesian digital-SME programmes (e.g. via ministry or incubator calls). The grant-finder workflow can scan these quarterly.
- University partnership: continued as a capstone or teaching project, with students contributing modules under mentorship.
