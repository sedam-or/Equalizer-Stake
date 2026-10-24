# BRAND — strategy, positioning, naming, identity

> Status: **name decided: Ritma** (2026-10-25). Formerly the *Equalizer-Stake* capstone. Trademark and domain checks are still open (§5.5).

## 1. The problem (in the customer's words)

> "Every month I spend days chasing transfers on WhatsApp. Two teachers ended up in the same room again last week. Parents ask me the schedule at 10 pm."
> — the owner of a 150-student music school (composite of v0 discovery interviews)

Small course schools (music, dance, martial arts, tutoring/bimbel, art, swimming) and rehearsal studios run on WhatsApp, Excel and paper. Their costs: double bookings (3–5 per month in v0's case), late payments, 2–3 hours a day of admin, and disputes about who attended.

## 2. Ideal customer profile (ICP)

| Attribute | Value |
|---|---|
| Business | Independent course school or studio, 1–3 locations |
| Size | 20–500 active students, 2–30 instructors, 1–3 admin staff |
| Geography | Indonesia first (Bahasa, IDR, WhatsApp-native, bank-transfer culture), then SEA, then global |
| Buyer | Owner (often also teaching). Not technical. Price-sensitive. Values trust and "it just works" |
| Daily user | Front-desk admin (desktop) and parents (phone) |
| Trigger to switch | A painful double booking, a cash-flow month, growth past ~80 students, or a new admin hire |

Secondary ICP: **developers and local IT partners** who set up systems for schools. They star, deploy, contribute presets and resell setup services.

## 3. Positioning

**For** independent course schools and studios **who** juggle schedules, fees and attendance across WhatsApp and spreadsheets, **Ritma is** an open-source school operating system **that** sets up in three steps and makes double bookings impossible. **Unlike** global SaaS tools priced in dollars and built for someone else's market, or booking tools that stop at appointments, **Ritma** handles the whole student lifecycle (enrollment, recurring lessons, monthly fees, attendance and parent communication), runs on your own PC, server or our cloud, and your data is always yours.

- **Category:** open-source school management for course businesses ("school OS").
- **Developer one-liner:** *"Cal.com for course schools."*

## 4. USPs (each one must be provable; the proof is linked)

| # | Claim (EN) | Klaim (ID) | Proof in product | Spec |
|---|---|---|---|---|
| 1 | **Set up your school in 3 steps** | Siap dipakai dalam 3 langkah | Setup wizard (template → features → rules), < 10 min, e2e-timed | PRD-04/07, SET-*, TST-07 |
| 2 | **Zero double bookings. Guaranteed by the database.** | Dijamin tidak ada jadwal bentrok | DB exclusion constraint + public concurrency test badge (50 → 1) | DOM-10, TST-05 |
| 3 | **Your rules, not ours** | Aturan sekolah Anda, bukan aturan kami | Every fee, date, radius and hour is a setting with a live plain-language summary | POL-*, PRD-06 |
| 4 | **Built for how small schools really work** | Dibuat untuk cara kerja sekolah kecil | WhatsApp-first reminders, one invoice per parent for all children, bank-transfer codes, late fee and freeze, Bahasa Indonesia | NTF-08, BIL-04/08 |
| 5 | **Your data, your choice of home** | Data Anda, Anda yang pilih tempatnya | The same software on your PC, your server or our cloud. Export everything anytime | DST-*, DST-42 |
| 6 | **Open source, forever** | Open source, selamanya | AGPL-3.0, public roadmap, no closed modules | ADR-0009 |

Messaging rules: lead with **outcomes** (time saved, money collected, no clashes), not features. Never claim "AI". Never claim compliance certifications we don't hold. Say "designed with UU PDP in mind" until audited.

## 5. Naming

### 5.1 Why rename

1. "Stake" strongly evokes **Stake.com** (a crypto casino) and staking (crypto). That is a trust and SEO risk with parents and school owners, and a likely ad-platform policy flag.
2. "Equalizer-Stake" is two words, hyphenated, hard to say in Indonesian, and doesn't suggest schools, time or classes.
3. The name is the first impression in the README, app stores, WhatsApp messages ("Pesan dari …") and invoices.

### 5.2 Criteria

- 2 syllables (max 3), and easy to pronounce in both English and Indonesian.
- Suggests **time, rhythm or class**.
- No gambling, crypto or finance connotation.
- `.com` / `.app` / `.id` plausibly available or affordable, and the GitHub org free.
- No conflicting registered mark in Nice classes 9, 41 and 42 (WIPO Global Brand DB, PDKI Indonesia, USPTO, EUIPO).
- Works as a CLI and crate name (`<name>`, `<name>-server`), and isn't a common English word that's impossible to search for.

### 5.3 Shortlist (the maintainer runs the checks; I can't verify availability)

| Name | Meaning / story | Fit | Risks to check |
|---|---|---|---|
| **Kenong** ⭐ | The gamelan gong that **marks time**: it divides the musical cycle into equal parts, the way the product divides the week into fair, non-clashing slots. An Indonesian origin story that's also distinctive globally | Time + music + fairness (*equal* parts, a nod to the codename "Equalizer"). Short and unique | Pronunciation outside SEA ("keh-nong"); music-centric for tutoring schools (mitigation: the tagline is generic) |
| **Ritma** | From *ritme* (ID) / rhythm. "The rhythm of your school" | Short, bilingual-friendly | Likely existing small brands; generic in ID |
| **Irama** | "Rhythm, tempo" in Indonesian/Malay | Beautiful, positive | Generic word in ID (weak trademark), a common business name |
| **Studiora** | *studio* + *ora* (hour) | Says "studio hours" | 3–4 syllables; check EU marks |
| **Openbell** | The school bell, open | Clear for every school type, English-friendly | Two words merged; weaker in Indonesian |
| **Kelasa** | From *kelas* (class) | ID-friendly, generic for all course types | Means nothing in English |
| **Sanggar** | An Indonesian word for an arts/dance/music studio-school | Culturally exact for the ICP | Generic word, not trademarkable alone (e.g. "Sanggar OS" would be needed) |
| **Metrona** | From metronome | Time-keeping, music | Check existing SaaS uses |
| **Slotwise** | Smart slots | Very clear in English | Generic-sounding, English-only |
| **Classly** | Class + -ly | Clear | Very likely taken; a common pattern |

My original recommendation was Kenong, with Ritma as the fallback.

**Decision (2026-10-25): Ritma.** The maintainer chose it for being short, bilingual and easy to say.
- **Tagline:** *"The rhythm of your school."* / *"Irama sekolah Anda."*
- **Story line:** "Every school has a rhythm: lessons, fees, attendance. Ritma keeps it steady."
- **Logo:** the logo brief in §7 still works. The circle with one highlighted segment reads as a beat in a bar.
- **Searchability:** "ritma" is a common word in Indonesian, so always pair it with a category word in copy and SEO ("Ritma school software", "Ritma aplikasi kursus").

### 5.4 Rename procedure (applied 2026-10-25 in the docs; code follows via the tasks)

1. `crates/eq-core/src/brand.rs` and `web/src/brand.ts` hold `NAME = "Ritma"` and the tagline (T-005).
2. The binary is `ritma`. That covers the `[[bin]]` name in eq-server, the Dockerfile, deploy templates and the install script.
3. User-facing env vars use `RITMA_*`. Paths are `/var/lib/ritma` and `/opt/ritma`. The mDNS name is `ritma.local`.
4. Internal identifiers **stay** because they're stable and invisible to users:
   - crate prefix `eq-`, Rust paths `eq_*`;
   - the cookie `eq_rt` and the header `X-EQ-CSRF`;
   - DB roles `eq_app` / `eq_platform`;
   - key-derivation labels `eq/…/v1`. Changing these would break encryption.
5. Register the GitHub org, domains and social handles (§5.5).

### 5.5 Handles, domains, trademark (maintainer to-do)

| Asset | Plan | Status (2026-10-25) |
|---|---|---|
| GitHub org | `ritmahq`, repo `ritmahq/ritma` (see `brand/GITHUB.md`) | `ritmahq` appeared free. `ritma` is taken by a user account |
| Spare orgs to squat | `ritma-app`, `getritma` (redirect to `ritmahq`) | Appeared free |
| Developer domain | `ritma.dev` (docs and problem-type URIs `https://ritma.dev/problems/<code>`, in `PROBLEM_BASE`) | **Unchecked.** If it's unavailable, change `PROBLEM_BASE` in T-002 before M1 ships, because problem URIs are stable identifiers |
| Product domain `<domain>` | Candidates: `ritma.app`, `ritma.id`, `getritma.com`. It's used for `app.`, `get.`, `demo.`, `releases.` and `status.` (spec 11/14/15) | **Unchecked** |
| Tunnel domain `<tunnel-domain>` | A separate domain for `<slug>.<tunnel-domain>` (DST-22), so that abuse on tunnels can't hurt the main domain's reputation | **Unchecked** |
| Trademark | Search WIPO Global Brand DB, PDKI (Indonesia) and USPTO in classes 9/41/42 for "RITMA". Then file in Indonesia (DJKI) first | **Unchecked.** "Ritma" is likely already used by small businesses, so check for software and education uses specifically |
| Social | `@ritmahq` on X, Instagram, YouTube, Threads and LinkedIn | Unchecked |

Once a domain is chosen, replace `<domain>` / `<tunnel-domain>` in `spec/` and `tasks/` in one commit.

## 6. Voice & tone

| Audience | Tone | Example (EN) | Contoh (ID) |
|---|---|---|---|
| Owner (setup, settings) | Calm, confident, plain | "Late fees are added the day after the due date. You can change this anytime." | "Denda ditambahkan sehari setelah jatuh tempo. Bisa diubah kapan saja." |
| Admin (daily work) | Efficient, specific | "Room 2 is taken at 15:00. Try 15:50 or Room 3." | "Ruang 2 sudah terisi jam 15:00. Coba 15:50 atau Ruang 3." |
| Parent (portal, WhatsApp) | Warm, polite, short | "Hi Mrs. Sari, Ayu's piano lesson is today at 15:00 in Room 2." | "Halo Ibu Sari, les piano Ayu hari ini pukul 15:00 di Ruang 2." |
| Developers (README, docs) | Precise, friendly, no hype | "One binary. Postgres. That's it." | — |

Rules: second person, active voice, no blame, no exclamation marks in errors. Indonesian uses polite *Anda* in the app, and *Bapak/Ibu* in parent messages.

## 7. Visual identity brief

- **Logo concept** (built: files and usage rules in [GUIDELINES.md](GUIDELINES.md) §2):
  - The mark is a circle divided like a bar of music with **one segment highlighted**: one beat, one time slot taken, the others free. It reads as a clock, a rhythm and a calendar slot at the same time.
  - The wordmark is set in Plus Jakarta Sans 700, lowercase, tight tracking (−1%).
  - The mark must work at 16 px (favicon) and in one colour.
- **Colour:** brand blue (OKLCH 0.49 0.20 255 = `#005EB9` in sRGB) as primary, with warm amber accent (0.75 0.16 70) used sparingly for "moments" (success confetti, highlights). Neutrals are cool grey. The source of truth is `spec/09-design-system.md` §2.
- **Typography:** Plus Jakarta Sans (headings, designed by Tokotype in Jakarta, another nod to the origin) and Inter (body).
- **Imagery:** real small-school moments (hands on piano keys, a parent's phone with the schedule, a front desk). Warm natural light. No generic stock "business people", no 3D blobs, no neon. Screenshots are always shown in a device frame, light and dark.
- **Illustration:** simple line icons (lucide style, 1.5 px). Empty states use a single-colour line illustration in brand-200/brand-600.

## 8. Asset shot list (for launch)

| Asset | Spec | Notes |
|---|---|---|
| **Hero GIF/video: "3-step setup"** | 1280×720, ≤ 8 MB GIF + MP4/WebM, 45–60 s | Template → features → rules with the live summary → dashboard. The single most important asset |
| README banner | 1280×320, light + dark (`<picture>`) | **Done:** `assets/banner-{light,dark}.svg` (logo, tagline, slot motif). Add 3 fanned screenshots once the UI exists |
| GitHub social preview | 1280×640 | **Done:** `assets/social-preview.{svg,png}`. Upload the PNG in repo Settings → Social preview |
| Screenshots (8) | 2880×1800 desktop, 1290×2796 phone | Schedule grid, rules + summary, invoice, parent portal (phone), check-in (phone), dashboard, features toggle, conflict + alternatives |
| Concurrency demo clip | 20 s terminal recording | "50 requests, 1 booking" with `oha` + psql, for HN and dev audiences |
| App icons | 1024 master → PWA 192/512 maskable, favicon.svg, tray icons | `logo/favicon.svg` and `logo/ritma-mark.svg` are the masters (GUIDELINES.md §2.4) |
| WhatsApp message samples | Images of real message templates | For the landing page "what parents see" |

## 9. Do & Don't

- **Do** show the product solving a real school's week. Use actual numbers ("saved 6 hours a week").
- **Do** credit the origin: "Born as a university capstone for a real Indonesian music school". Use it as a story. Don't name the school or its city without written permission.
- **Don't** use the words "revolutionary", "AI-powered" or "enterprise-grade".
- **Don't** show children's faces in marketing without written parental consent.
