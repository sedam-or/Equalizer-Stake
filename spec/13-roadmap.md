# 13 — Roadmap → tasks

The milestone overview for humans is in `/ROADMAP.md`. The executable plan is `tasks/INDEX.md` (generated from task frontmatter by `scripts/tasks-index.sh`). Tasks are executed one at a time following `/WORKFLOW.md`. A milestone's drafts are promoted only after the previous milestone's integration task is `done` (WORKFLOW §6).

| Milestone | Exit criterion (proved by its integration task) |
|---|---|
| M0 Foundation | Release build embeds the SPA; `/healthz` + `/` served; CI green on all jobs |
| M1 Core | API test: create tenant via service → login → toggle modules (dependency rules) → put policy (versioned, audited) → audit chain verifies; tenant isolation test passes |
| M2 Scheduling | TST-05 concurrency proof (50 → 1) and all scheduling acceptance criteria pass |
| M3 Setup & web app | Playwright: fresh install → 3-step setup (music preset) → create first lesson in the grid, in < 60 s CI time; axe clean |
| M4 Enrollment & billing | E2E: registration → approval → cycle invoice → overdue → late fee + freeze → payment verify → unfreeze (time travel) |
| M5 Attendance & notify | E2E: check-in pass/fail paths + review; notifications delivered with fallback via mock channels |
| M6 Rental/reports/feedback/privacy | E2E per module + DSR erase verified |
| M7 Release | Install script on a clean VM (CI docker-in-docker) reaches `/setup`; release workflow dry-run produces signed artifacts |
| M8 Desktop | Desktop app on CI runners (Win/macOS/Linux) boots, completes setup, backup + restore works |
| M9 Cloud | Signup → tenant → setup; plan limits enforced; webhook-driven subscription states |

Task ID ranges: M0 `T-0xx`, M1 `T-1xx`, …, M9 `T-9xx`.
