# Tasks — the implementation queue

- **Who writes tasks:** the architect (spec owner). **Who implements:** Codex or human contributors.
- **Format:** [`_TEMPLATE.md`](_TEMPLATE.md). Every task is self-contained: it inlines the spec context, interfaces, rules and tests.
- **Order:** [`INDEX.md`](INDEX.md). Pick the lowest-ID `todo` task whose dependencies are all `done`. The full loop is in `/AGENTS.md` §1.
- **Statuses:** `todo` → `in_progress` → `review` → `done`. `blocked` (see the task's Questions). `draft` = written, but may be revised after earlier milestones; don't start it until it is promoted to `todo`.
- **Size:** S < 150 LOC, M < 400 LOC (excluding generated code and tests). L tasks are split before starting.
- **Security sections:** a task that hits a security trigger (spec/18 SBD-30) has a `## Security` section (threats, controls, abuse-case tests). A `draft` task with triggers is promoted to `todo` only once that section is written.
- **Each milestone ends with an integration task** (the last ID in the milestone, titled "Mx integration") that proves the milestone end to end.

| Milestone | Folder | Status of task files |
|---|---|---|
| M0 Foundation | `M0-foundation/` | ready (`todo`) |
| M1 Core | `M1-core/` | ready |
| M2 Scheduling | `M2-scheduling/` | ready |
| M3 Setup & web app | `M3-setup-web/` | ready |
| M4 Enrollment & billing | `M4-enrollment-billing/` | `draft` (promote after M3 review) |
| M5 Attendance & notify | `M5-attendance-notify/` | `draft` |
| M6 Rental, reports, feedback, privacy | `M6-rental-reports-privacy/` | `draft` |
| M7 Release & distribution | `M7-release-distribution/` | `draft` |
| M8 Desktop | `M8-desktop/` | `draft` |
| M9 Cloud | `M9-cloud/` | `draft` |
