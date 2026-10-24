# GITHUB — making the repo discoverable, credible and star-worthy

> Companion to `GTM.md` (launch sequence, README funnel) and `BRAND.md` (name, assets).
> Everything here is **honest growth**: no bought stars, no star-for-star rings, no bots. GitHub removes fake stars and can delist repos. One flagged launch costs more than it earns.

## 1. How GitHub actually surfaces a repo (first principles)

A stranger finds a repo through one of six doors. Every tactic below opens one of them.

| Door | What drives it | Our lever |
|---|---|---|
| **Trending** (daily/weekly, overall and per language) | Star *velocity* relative to the repo's own baseline over a short window. A young repo with 300 stars in 24 h beats an old one with 1,000 a week (observed behaviour; the algorithm is not public) | Concentrate launch traffic into **one 24–48 h spike** (§4). Primary language **Rust** (a smaller trending pool than JS/Python, so it's easier to rank) |
| **Search** (GitHub and Google) | Repo name, description, topics, README headings, stars | Description and topics with the words schools search for (§2). README H2s in plain words ("Scheduling", "Monthly fees") |
| **Topics pages** (`github.com/topics/self-hosted`) | Topic tags, sorted by stars | 20 precise topics (§2) |
| **Awesome lists** | Merged PRs into curated lists | `awesome-selfhosted`, `awesome-rust`, `awesome-tauri`, `awesome-indonesia`; each has entry rules (a release, docs and a demo) |
| **Social shares** (HN, Reddit, X, WhatsApp, LinkedIn) | The link card: the social preview image + description | A 1280×640 social preview (BRAND §8) that reads at thumbnail size |
| **Dependents and "Used by"** | Other repos depending on ours | Later: publish small reusable crates (e.g. a `policy-form` JSON-Schema renderer, or a calendar-hold helper) that link back |

**Conversion** (visitor → star) depends on the first screen of the README. A repo with a 10% star rate turns one HN front page (≈ 10–20k visitors) into 1–2k stars. At 1% it's 150. **Fix conversion before buying attention.**

## 2. Repo identity (do once, before any promotion)

- **Owner:** org `ritmahq`, repo `ritmahq/ritma`.
  - An org signals a project rather than a side project, allows several maintainers, and keeps the name if people change.
  - Transferring `sedam-or/Equalizer-Stake` keeps stars, issues and history, and GitHub redirects the old URL. The capstone history is part of the story, and the `v0-capstone` tag stays.
- **Spare orgs:** `ritma-app` and `getritma`, each with only a README pointing to `ritmahq`. This blocks impersonation.
- **Description** (≤ 350 chars, keywords first):
  > Open-source school management for music, dance & course schools: scheduling with zero double bookings, monthly fees, attendance and WhatsApp reminders. Set up in 3 steps. One Rust binary to self-host, or use the cloud. English + Bahasa Indonesia.
- **Website:** the demo URL (`demo.<domain>`) once it exists. Until then, the GitHub Pages docs (§5).
- **Topics (20, the max):** `school-management`, `music-school`, `scheduling`, `booking-system`, `course-management`, `attendance`, `invoicing`, `billing`, `self-hosted`, `open-source`, `rust`, `axum`, `postgresql`, `react`, `tauri`, `multi-tenant`, `education`, `tutoring`, `indonesia`, `saas-alternative`.
- **Social preview:** upload in Settings → General (it can't be set through the API). Logo, "The rhythm of your school", "Open source · Self-host or Cloud", and one screenshot of the schedule grid.
- **Pinned:** pin `ritmahq/ritma` on the org profile and on the maintainer's personal profile. Also create `ritmahq/.github` with `profile/README.md` (org landing page) and the shared community files.

## 3. Repo surface checklist (the "credible at first glance" bar)

Ordered by impact on conversion.

1. **README first screen** (GTM §3):
   - logo, tagline and one-line value statement;
   - the **hero GIF** (3-step setup);
   - three buttons: Demo / Cloud / Self-host.
   - Until the GIF exists, put the 50→1 concurrency clip or a schedule-grid screenshot in its place. **Never** a wall of text.
2. **Badges** (one row, max 6):
   - CI;
   - **double-booking test 50→1: passing** (from the T-213 job);
   - latest release;
   - license AGPL-3.0;
   - translation %;
   - OpenSSF Scorecard.
3. **About sidebar:** description, website, topics, and Releases with real assets.
4. **Community profile 100%** (Insights → Community Standards):
   - already written: README, LICENSE, CODE_OF_CONDUCT, CONTRIBUTING, SECURITY, issue and PR templates;
   - still needed: the description and social preview.
5. **Discussions on**, with categories: Announcements, Q&A, Ideas, Show and tell (schools using it), Presets, Trademark, Bahasa Indonesia. Pin a "👋 Introduce your school" thread.
6. **Labels** (GTM §4): `good first issue`, `help wanted`, `preset`, `i18n`, `module:<id>`, `rfc`, `needs-repro`.
   - Keep **≥ 10 open `good first issue`s** at launch. Each one links its `tasks/**/T-*.md`. That's our edge: issues so well specified a newcomer can finish them.
   - GitHub's "Good first issues" feed and `goodfirstissue.dev` pick them up automatically.
7. **Releases:** from M0 on, cut `v0.x.y` pre-releases with notes and binaries.
   - A repo with zero releases reads as abandoned and fails awesome-list rules.
   - Release notes are generated by release-please (RELEASING.md).
8. **Packages:** `ghcr.io/ritmahq/ritma` appears in the sidebar, which is proof that it runs.
9. **Sponsors:** `.github/FUNDING.yml` (GitHub Sponsors + Open Collective) once the accounts exist (GTM §8).
10. **Repo hygiene that reviewers notice:**
    - branch protection;
    - signed-off commits;
    - Dependabot;
    - CodeQL / cargo-deny;
    - no stale "TODO" branches;
    - issues answered within the SLA.

## 4. Phased plan (tied to `tasks/INDEX.md` milestones)

### Phase A — Build in public (now → M3). Goal: 150 stars, 300 waitlist, first contributors

The repo is interesting *before* the product works. The hook is the process:

- **"Spec-first, agent-implemented open source"**:
  - `spec/` plus 125 self-contained task files that an AI coding agent (or a human newcomer) executes one by one, each reviewed by a human.
  - Post the method (not hype): "How we wrote 125 task files so contributors and agents can build a Rust app without reading the whole spec". This works on HN/Lobsters/r/rust as a *process* story.
- **Weekly progress post** (ID + EN), with each post linking a merged PR and the INDEX burndown.
- **Design-notes series** (GTM §7, posts 2–4): policy-as-data, the one Postgres constraint, the Rust modular monolith.
  - Submit them to *This Week in Rust* (call for participation + blog section) and *Postgres Weekly*.
- **Contributors early:** label M1–M3 tasks that are well-isolated as `good first issue` (e.g. presets, i18n, PolicyForm widgets).
- **Do not** announce on HN/Product Hunt yet. You get one big first impression, and an empty app burns it.

### Phase B — Soft launch (end of M3: wizard + scheduling work). Goal: 500 stars

Prerequisites: the demo URL runs, the hero GIF is in the README, `docker run` works in one line, and there's a `v0.3` release.

- r/selfhosted ("I built an open-source school scheduler with zero double bookings, looking for feedback"), r/rust (the technical angle), and Indonesian dev groups.
- Lobsters (needs an invite), dev.to cross-post of the concurrency article.
- **Not** Show HN yet. Save it for v1.0 when billing and attendance make it a complete product.

### Phase C — Launch v1.0 (M7). Goal: trending in Rust daily + overall weekly, 2,000 stars in 6 months

**Launch-day runbook** (all on one day, so the stars land in one window):

| Time (WIB / UTC) | Action |
|---|---|
| T-7 days | Line up 10–20 people (beta schools, contributors, friends who *genuinely* use or like it) to try the demo and share feedback on launch day. Asking for honest feedback is fine. Asking for stars in bulk is not |
| T-1 day | Freeze: README, GIF, demo reset, docs, `v1.0.0` release with binaries, desktop installers, deploy buttons tested from a clean account |
| 19:00 WIB (08:00 ET, Tue/Wed) | **Show HN:** "Show HN: Ritma, open-source software for music and dance schools (one Rust binary)". The first comment is the maintainer's story: the capstone, the real school, why Rust, what's hard |
| 19:30 WIB | r/selfhosted, r/rust, r/opensource; X/LinkedIn/Threads thread with the GIF; Indonesian communities (Telegram, Facebook music-teacher groups, Kaskus) |
| 20:00–02:00 WIB | Answer **every** comment within 15 minutes. Fix papercuts live and reply "fixed in v1.0.1". This is the highest-leverage hour of the year |
| T+1 day | Product Hunt (Tuesday 00:01 PT launch if HN was Tuesday, else next Tuesday). Awesome-list PRs (`awesome-selfhosted`: Booking and Scheduling; `awesome-rust`: Applications; `awesome-tauri`; `awesome-indonesia`) |
| T+2–7 days | Write-up: "What 20k visitors taught us" with real numbers, which tends to trend a second time |

**Timing rules:** avoid US holidays, big Apple/Google event days, and Indonesian Lebaran week. Tue–Thu morning US time is best for HN.

### Phase D — Sustain (after v1.0)

- One release a month, each with a GIF of the main feature in the notes. Releases notify watchers and bring visitors back.
- Monthly "contributor spotlight" and the all-contributors table in the README.
- Second-wave launches at M8 (desktop: "runs on your front-desk PC", Windows users) and M9 (cloud GA), each with its own story and awesome-list entries.
- Conference talks: RustConf/EuroRust/Rust Indonesia meetups ("A modular monolith from desktop to cloud"), PGConf ("double booking is a constraint problem").

## 5. Docs & SEO (long-tail stars come from Google)

- **GitHub Pages** at `ritmahq.github.io/ritma` (later `docs.<domain>`), Starlight, with docs in EN + ID.
- **Pages that rank for real searches**, one problem each:
  - "aplikasi jadwal les musik", "software sekolah musik gratis", "aplikasi absensi kursus", "cara bikin tagihan SPP otomatis";
  - in English: "open source music school software", "self-hosted lesson scheduling", "My Music Staff alternative open source".
- **Comparison pages** (honest, with a feature table): Ritma vs My Music Staff / Jackrabbit / spreadsheets.
- Every docs page links back to the repo with "Star on GitHub".
- **Searchability:** "ritma" alone is an Indonesian word, so always say "Ritma school software" / "Ritma aplikasi kursus" in titles.

## 6. Metrics (check weekly; GitHub keeps traffic for 14 days only, so export it)

| Metric | Source | Healthy |
|---|---|---|
| Stars/week, star-history curve | `star-history.com/#ritmahq/ritma` | Spikes on launches, slope > 0 between them |
| Visitors → stars conversion | Insights → Traffic (unique visitors) vs stars gained | ≥ 5% (≥ 10% is excellent) |
| Referrers | Traffic → referring sites | Know which channel works, then double down |
| Clones and `ghcr` pulls | Traffic, Packages | Rising: people actually try it |
| Good-first-issue pickup time | Issues | < 7 days |
| First-time contributors / month | Insights → Contributors | ≥ 3 after v1.0 |

Export script (run weekly, store the CSV outside the repo):
```sh
gh api repos/ritmahq/ritma/traffic/views     > traffic-views-$(date +%F).json
gh api repos/ritmahq/ritma/traffic/popular/referrers > referrers-$(date +%F).json
```

## 7. Maintainer checklist (you run these; nothing here was executed for you)

Install the CLI first: `sudo pacman -S github-cli` (or your distro's package), then `gh auth login`.

**Step 1: create the org** `ritmahq` on github.com (web UI: Settings → Organizations → New, free plan). Do the same for `ritma-app` and `getritma`.

**Step 2: transfer and rename the repo.** It keeps stars, issues and redirects.
```sh
gh api -X POST repos/sedam-or/Equalizer-Stake/transfer -f new_owner=ritmahq -f new_name=ritma
git remote set-url origin https://github.com/ritmahq/ritma.git
```

**Step 3: push the local commits and the archive refs.**
```sh
git push origin main
git push origin v0-capstone archive/v0-capstone
```

**Step 4: set the identity.**
```sh
gh repo edit ritmahq/ritma \
  --description "Open-source school management for music, dance & course schools: scheduling with zero double bookings, monthly fees, attendance and WhatsApp reminders. Set up in 3 steps. One Rust binary to self-host, or use the cloud. English + Bahasa Indonesia." \
  --add-topic school-management,music-school,scheduling,booking-system,course-management,attendance,invoicing,billing,self-hosted,open-source,rust,axum,postgresql,react,tauri,multi-tenant,education,tutoring,indonesia,saas-alternative \
  --enable-discussions --enable-issues --enable-wiki=false \
  --enable-squash-merge --enable-merge-commit=false --enable-rebase-merge=false \
  --delete-branch-on-merge
```

**Step 5: create the labels.**
```sh
for l in "good first issue:7057ff" "help wanted:008672" "preset:0e8a16" "i18n:1d76db" "rfc:5319e7" "needs-repro:fbca04" "security-review:b60205"; do
  gh label create "${l%%:*}" --color "${l##*:}" --force -R ritmahq/ritma
done
for m in core identity audit scheduling enrollment billing attendance rental notify reports feedback; do
  gh label create "module:$m" --color c5def5 --force -R ritmahq/ritma
done
```

**Step 6: web-only settings.**
- Social preview image (Settings → General).
- Discussions categories (§3.5).
- Branch ruleset on `main` and `release/**`: apply the settings and the `gh api` call in `BRANCHING.md` §7.
- Private vulnerability reporting: on.
- Org profile README (`ritmahq/.github`).

**Step 7: verify.**
- Insights → Community Standards shows 100%.
- The old URL `github.com/sedam-or/Equalizer-Stake` redirects to the new one.

## 8. Don'ts

- Don't buy stars, join "star exchange" groups, or mass-DM people for stars. GitHub purges them, and HN/Reddit readers check star-history for unnatural cliffs.
- Don't launch on HN twice in a short time. Show HN allows a repost only after significant change.
- Don't post the same text to many subreddits within minutes. Reddit flags it as spam. Adapt each post to its community.
- Don't overclaim in titles ("revolutionary", "AI-powered"; see BRAND §9). Precise and modest titles do better on HN.
- Don't let launch-day issues sit unanswered. A visible unanswered bug report converts worse than no repo at all.
