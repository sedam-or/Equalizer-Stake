# Task hygiene audit

Read-only baseline: `milestone/m3-setup-web` at `66177dd` (2026-10-28). The scan read all `tasks/**/T-*.md` files and cited candidate lines before changes. `scripts/tasks-index.sh --check` and `python3 -I scripts/erd.py --check` passed. Inventory: 2 `blocked`, 72 `draft`, 14 `review`, 40 `todo`; total 128.

## Required template sections

Required headings come from `tasks/_TEMPLATE.md:13-65`. Each row below is a real missing-section finding; line 1 identifies the task file and the absent headings were checked across the full file.

| Task | Status | Evidence | Missing headings | Verdict |
|---|---|---|---|---|
| T-108 | todo | `tasks/M1-core/T-108-jobs-and-cron.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-109 | todo | `tasks/M1-core/T-109-audit.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-110 | todo | `tasks/M1-core/T-110-identity-login-refresh.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-111 | todo | `tasks/M1-core/T-111-invites-reset-me-totp.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-112 | todo | `tasks/M1-core/T-112-people-and-rbac.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-113 | todo | `tasks/M1-core/T-113-core-policy-settings-meta-api.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-114 | todo | `tasks/M1-core/T-114-openapi-contract.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-115 | todo | `tasks/M1-core/T-115-http-hardening.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-116 | todo | `tasks/M1-core/T-116-isolation-and-log-hygiene.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-117 | todo | `tasks/M1-core/T-117-m1-integration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-201 | todo | `tasks/M2-scheduling/T-201-scheduling-schema.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-202 | todo | `tasks/M2-scheduling/T-202-scheduling-policy.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-203 | todo | `tasks/M2-scheduling/T-203-calendar-holds.md:1` | Files, Business rules, Security, Implementation notes, Review | real |
| T-204 | todo | `tasks/M2-scheduling/T-204-catalog-api.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-205 | todo | `tasks/M2-scheduling/T-205-slot-rules-domain.md:1` | Files, Business rules, Security, Implementation notes, Review | real |
| T-206 | todo | `tasks/M2-scheduling/T-206-session-create-and-check.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-207 | todo | `tasks/M2-scheduling/T-207-alternatives.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-208 | todo | `tasks/M2-scheduling/T-208-reschedule-cancel.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-209 | todo | `tasks/M2-scheduling/T-209-series.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-210 | todo | `tasks/M2-scheduling/T-210-materialization-jobs.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-211 | todo | `tasks/M2-scheduling/T-211-reads-availability-ics.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-212 | todo | `tasks/M2-scheduling/T-212-cross-module-consumers.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-213 | todo | `tasks/M2-scheduling/T-213-m2-concurrency-proof.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-301 | todo | `tasks/M3-setup-web/T-301-presets.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-302 | todo | `tasks/M3-setup-web/T-302-setup-api.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-303 | todo | `tasks/M3-setup-web/T-303-files-and-drafts.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-304 | todo | `tasks/M3-setup-web/T-304-design-tokens-and-theming.md:1` | Context, Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-305 | todo | `tasks/M3-setup-web/T-305-ui-primitives-and-ds-page.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-306 | todo | `tasks/M3-setup-web/T-306-web-client-auth-i18n.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-307 | todo | `tasks/M3-setup-web/T-307-app-shell-nav-pwa.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-308 | todo | `tasks/M3-setup-web/T-308-auth-pages-and-profile.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-309 | todo | `tasks/M3-setup-web/T-309-policy-form.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-310 | todo | `tasks/M3-setup-web/T-310-setup-wizard-ui.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-311 | todo | `tasks/M3-setup-web/T-311-settings-pages.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-312 | todo | `tasks/M3-setup-web/T-312-schedule-grid.md:1` | Context, Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-313 | todo | `tasks/M3-setup-web/T-313-admin-pages.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-314 | todo | `tasks/M3-setup-web/T-314-dashboard.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-315 | todo | `tasks/M3-setup-web/T-315-m3-integration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-401 | draft | `tasks/M4-enrollment-billing/T-401-enrollment-schema-and-policy.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-402 | draft | `tasks/M4-enrollment-billing/T-402-enrollment-lifecycle.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-403 | draft | `tasks/M4-enrollment-billing/T-403-public-registration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-404 | draft | `tasks/M4-enrollment-billing/T-404-registration-approval.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-405 | draft | `tasks/M4-enrollment-billing/T-405-enrollment-changes.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-406 | draft | `tasks/M4-enrollment-billing/T-406-leave.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-407 | draft | `tasks/M4-enrollment-billing/T-407-makeup-credits.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-408 | draft | `tasks/M4-enrollment-billing/T-408-eligibility-and-ports.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-409 | draft | `tasks/M4-enrollment-billing/T-409-import.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-410 | draft | `tasks/M4-enrollment-billing/T-410-billing-schema-and-policy.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-411 | draft | `tasks/M4-enrollment-billing/T-411-billing-domain.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-412 | draft | `tasks/M4-enrollment-billing/T-412-cycle-issuance.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-413 | draft | `tasks/M4-enrollment-billing/T-413-payments.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-414 | draft | `tasks/M4-enrollment-billing/T-414-overdue-and-payer-state.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-415 | draft | `tasks/M4-enrollment-billing/T-415-adjustments-and-documents.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-416 | draft | `tasks/M4-enrollment-billing/T-416-billing-reminders-and-checklist.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-417 | draft | `tasks/M4-enrollment-billing/T-417-web-enrollment.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-418 | draft | `tasks/M4-enrollment-billing/T-418-web-billing.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-419 | draft | `tasks/M4-enrollment-billing/T-419-m4-integration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-501 | draft | `tasks/M5-attendance-notify/T-501-attendance-schema-and-policy.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-502 | draft | `tasks/M5-attendance-notify/T-502-checkin-evaluation.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-503 | draft | `tasks/M5-attendance-notify/T-503-checkin-api-and-review.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-504 | draft | `tasks/M5-attendance-notify/T-504-student-marks.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-505 | draft | `tasks/M5-attendance-notify/T-505-attendance-jobs.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-506 | draft | `tasks/M5-attendance-notify/T-506-notify-core.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-507 | draft | `tasks/M5-attendance-notify/T-507-notify-channels.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-508 | draft | `tasks/M5-attendance-notify/T-508-notify-routing.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-509 | draft | `tasks/M5-attendance-notify/T-509-notify-templates-and-subscriptions.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-510 | draft | `tasks/M5-attendance-notify/T-510-session-reminders.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-511 | draft | `tasks/M5-attendance-notify/T-511-web-checkin.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-512 | draft | `tasks/M5-attendance-notify/T-512-web-attendance-notify.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-513 | draft | `tasks/M5-attendance-notify/T-513-m5-integration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-601 | draft | `tasks/M6-rental-reports-privacy/T-601-rental-schema-and-policy.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-602 | draft | `tasks/M6-rental-reports-privacy/T-602-booking-domain.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-603 | draft | `tasks/M6-rental-reports-privacy/T-603-reservations-api.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-604 | draft | `tasks/M6-rental-reports-privacy/T-604-rental-jobs.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-605 | draft | `tasks/M6-rental-reports-privacy/T-605-public-booking.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-606 | draft | `tasks/M6-rental-reports-privacy/T-606-reports-widgets.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-607 | draft | `tasks/M6-rental-reports-privacy/T-607-printables-and-csv.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-608 | draft | `tasks/M6-rental-reports-privacy/T-608-feedback.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-609 | draft | `tasks/M6-rental-reports-privacy/T-609-privacy-export.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-610 | draft | `tasks/M6-rental-reports-privacy/T-610-deletion-requests.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-611 | draft | `tasks/M6-rental-reports-privacy/T-611-web-rental.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-612 | draft | `tasks/M6-rental-reports-privacy/T-612-web-reports-feedback-audit.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-613 | draft | `tasks/M6-rental-reports-privacy/T-613-m6-integration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-701 | draft | `tasks/M7-release-distribution/T-701-backup-restore.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-702 | draft | `tasks/M7-release-distribution/T-702-export-everything.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-703 | draft | `tasks/M7-release-distribution/T-703-telemetry.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-704 | draft | `tasks/M7-release-distribution/T-704-release-pipeline.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-705 | draft | `tasks/M7-release-distribution/T-705-key-rotation.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-706 | draft | `tasks/M7-release-distribution/T-706-installer-and-compose.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-707 | draft | `tasks/M7-release-distribution/T-707-one-click-templates.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-708 | draft | `tasks/M7-release-distribution/T-708-demo-instance.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-709 | draft | `tasks/M7-release-distribution/T-709-observability.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-710 | draft | `tasks/M7-release-distribution/T-710-docs-site.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-711 | draft | `tasks/M7-release-distribution/T-711-v1-readiness.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-801 | draft | `tasks/M8-desktop/T-801-tauri-shell.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-802 | draft | `tasks/M8-desktop/T-802-embedded-postgres-and-keychain.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-803 | draft | `tasks/M8-desktop/T-803-tray-and-lan.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-804 | draft | `tasks/M8-desktop/T-804-secure-tunnel.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-805 | draft | `tasks/M8-desktop/T-805-local-ca.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-806 | draft | `tasks/M8-desktop/T-806-desktop-backups.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-807 | draft | `tasks/M8-desktop/T-807-updater-and-signing.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-808 | draft | `tasks/M8-desktop/T-808-desktop-integration.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-901 | draft | `tasks/M9-cloud/T-901-platform-crate.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-902 | draft | `tasks/M9-cloud/T-902-multi-tenant-routing.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-903 | draft | `tasks/M9-cloud/T-903-signup.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-904 | draft | `tasks/M9-cloud/T-904-plans-and-metering.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-905 | draft | `tasks/M9-cloud/T-905-subscription-billing.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-906 | draft | `tasks/M9-cloud/T-906-tenant-backup-and-import.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-907 | draft | `tasks/M9-cloud/T-907-abuse-and-agpl.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |
| T-908 | draft | `tasks/M9-cloud/T-908-tunnel-broker-and-infra.md:1` | Files, Interfaces, Business rules, Security, Implementation notes, Review | real |

## Size candidates

Counts use individual rows in the `Files` section, top-level `ACn` entries, and named tests in `Tests to write` (or AC targets when the test table says “AC names above”). Limits: >8 file entries, >8 AC, or >12 named tests (CQ-101). A missing `Files` heading is reported under required sections, not counted as zero work.

| Task | Status | Files | AC | Tests | Verdict |
|---|---|---:|---:|---:|---|
| T-110 | todo | n/a | 9 | 6 | real; split the login/refresh scope before implementation |
| T-112 | todo | n/a | 9 | 11 | real; split the people/RBAC scope before implementation |
| T-113 | todo | n/a | 10 | 4 | real; split the settings/meta API scope before implementation |
| T-302 | todo | n/a | 9 | 9 | real; split setup API work before implementation |
| T-002 | review | 9 | 10 | 15 | real; record as follow-up, do not rewrite completed task scope |
| T-102 | review | n/a | 8 | 15 | real; record as follow-up, do not rewrite completed task scope |
| T-102a | review | 8 | 9 | 27 | real; record as follow-up, do not rewrite completed task scope |
| T-104 | review | 9 | 5 | 6 | real; record as follow-up, do not rewrite completed task scope |
| T-105 | review | 13 | 6 | 11 | real; record as follow-up, do not rewrite completed task scope |

T-117's AC1 is one end-to-end journey with 12 ordered actions, not 12 independent acceptance criteria; it remains one integration test and is not counted as an oversize AC list.

## Replaced dependency candidates

| Task / source | Candidate | Verdict |
|---|---|---|
| `tasks/M0-foundation/T-001-cargo-workspace.md:59` | `utoipa-axum = "0.2"` | real stale task pin; update to 0.3 |
| `tasks/M0-foundation/T-001-cargo-workspace.md:98` | `phonenumber = "0.3"` | real stale task pin; update to `rlibphonenumber = "2"` |
| `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:267-272` | `phonenumber`, `atomic-polyfill` | false positive: resolved historical Q1 and context, not active dependency instructions |
| `tasks/M1-core/T-103-core-crypto.md:175-178` | `phonenumber`, `atomic-polyfill` | false positive: resolved historical Q1 and context, not active dependency instructions |
| `tasks/M1-core/T-105-module-system.md:206` | `pastey` / `paste` | false positive: resolved note explicitly says `pastey` is maintained and `paste` is not a dependency |
| `tasks/M0-foundation/T-004-server-cli-http-skeleton.md:77` | “paste the output” | false positive: ordinary prose |
| `tasks/M3-setup-web/T-305-ui-primitives-and-ds-page.md:21,53` | “paste-tolerant” | false positive: input behavior, not the Rust crate |
| `tasks/M3-setup-web/T-306a-generated-validation-and-form-harness.md:17,28,78` | `libphonenumber-js` | false positive: frontend library named by spec 24, not Rust `phonenumber` |
| `Cargo.toml:27-28,67` | `utoipa=6`, `utoipa-axum=0.3`, `rlibphonenumber=2` | correct current workspace pins; no direct `paste` or `atomic-polyfill` use found in manifests |

## Security trigger candidates

The scanner looked for the SBD-30 trigger families in operative task sections, then checked the cited line in context. The real candidates require a task-specific Security section; the false positives below are domain vocabulary or static UI/configuration work.

| Task | Status | Evidence | Trigger | Verdict |
|---|---|---|---|---|
| T-108 | todo | `tasks/M1-core/T-108-jobs-and-cron.md:15` | `tenant_id` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-109 | todo | `tasks/M1-core/T-109-audit.md:19` | `tenant_id` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-110 | todo | `tasks/M1-core/T-110-identity-login-refresh.md:12` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-111 | todo | `tasks/M1-core/T-111-invites-reset-me-totp.md:12` | `password` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-112 | todo | `tasks/M1-core/T-112-people-and-rbac.md:12` | `PII` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-113 | todo | `tasks/M1-core/T-113-core-policy-settings-meta-api.md:28` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-114 | todo | `tasks/M1-core/T-114-openapi-contract.md:27` | `Email` | false positive: OpenAPI field metadata only; no runtime auth or PII handling. |
| T-115 | todo | `tasks/M1-core/T-115-http-hardening.md:16` | `tenant_id` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-116 | todo | `tasks/M1-core/T-116-isolation-and-log-hygiene.md:12` | `PII` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-117 | todo | `tasks/M1-core/T-117-m1-integration.md:16` | `Token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-201 | todo | `tasks/M2-scheduling/T-201-scheduling-schema.md:22` | `tenant_id` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-206 | todo | `tasks/M2-scheduling/T-206-session-create-and-check.md:15` | `Idempotency-Key` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-209 | todo | `tasks/M2-scheduling/T-209-series.md:18` | `Idempotency-Key` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-210 | todo | `tasks/M2-scheduling/T-210-materialization-jobs.md:36` | `tenant_id` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-211 | todo | `tasks/M2-scheduling/T-211-reads-availability-ics.md:12` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-212 | todo | `tasks/M2-scheduling/T-212-cross-module-consumers.md:16` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-213 | todo | `tasks/M2-scheduling/T-213-m2-concurrency-proof.md:15` | `Idempotency-Key` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-301 | todo | `tasks/M3-setup-web/T-301-presets.md:25` | `payment` | false positive: preset asset validation only; no secret, PII or tenant authorization flow. |
| T-302 | todo | `tasks/M3-setup-web/T-302-setup-api.md:12` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-303 | todo | `tasks/M3-setup-web/T-303-files-and-drafts.md:12` | `upload` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-304 | todo | `tasks/M3-setup-web/T-304-design-tokens-and-theming.md:12` | `token` | false positive: visual token/theming work only; no private data or authorization surface. |
| T-305 | todo | `tasks/M3-setup-web/T-305-ui-primitives-and-ds-page.md:24` | `token` | false positive: design-system primitives/demo only; no sensitive-data handling or auth surface. |
| T-306 | todo | `tasks/M3-setup-web/T-306-web-client-auth-i18n.md:12` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-307 | todo | `tasks/M3-setup-web/T-307-app-shell-nav-pwa.md:12` | `login` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-308 | todo | `tasks/M3-setup-web/T-308-auth-pages-and-profile.md:12` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-309 | todo | `tasks/M3-setup-web/T-309-policy-form.md:22` | `payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-310 | todo | `tasks/M3-setup-web/T-310-setup-wizard-ui.md:15` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-311 | todo | `tasks/M3-setup-web/T-311-settings-pages.md:26` | `upload` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-312 | todo | `tasks/M3-setup-web/T-312-schedule-grid.md:17` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-313 | todo | `tasks/M3-setup-web/T-313-admin-pages.md:24` | `PII` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-315 | todo | `tasks/M3-setup-web/T-315-m3-integration.md:12` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-401 | draft | `tasks/M4-enrollment-billing/T-401-enrollment-schema-and-policy.md:17` | `RLS` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-403 | draft | `tasks/M4-enrollment-billing/T-403-public-registration.md:19` | `Payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-404 | draft | `tasks/M4-enrollment-billing/T-404-registration-approval.md:17` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-409 | draft | `tasks/M4-enrollment-billing/T-409-import.md:18` | `Upload` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-410 | draft | `tasks/M4-enrollment-billing/T-410-billing-schema-and-policy.md:14` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-411 | draft | `tasks/M4-enrollment-billing/T-411-billing-domain.md:17` | `payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-412 | draft | `tasks/M4-enrollment-billing/T-412-cycle-issuance.md:14` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-413 | draft | `tasks/M4-enrollment-billing/T-413-payments.md:14` | `Payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-414 | draft | `tasks/M4-enrollment-billing/T-414-overdue-and-payer-state.md:14` | `payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-415 | draft | `tasks/M4-enrollment-billing/T-415-adjustments-and-documents.md:14` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-416 | draft | `tasks/M4-enrollment-billing/T-416-billing-reminders-and-checklist.md:17` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-417 | draft | `tasks/M4-enrollment-billing/T-417-web-enrollment.md:14` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-418 | draft | `tasks/M4-enrollment-billing/T-418-web-billing.md:17` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-419 | draft | `tasks/M4-enrollment-billing/T-419-m4-integration.md:17` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-503 | draft | `tasks/M5-attendance-notify/T-503-checkin-api-and-review.md:14` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-506 | draft | `tasks/M5-attendance-notify/T-506-notify-core.md:17` | `RLS` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-507 | draft | `tasks/M5-attendance-notify/T-507-notify-channels.md:14` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-509 | draft | `tasks/M5-attendance-notify/T-509-notify-templates-and-subscriptions.md:24` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-513 | draft | `tasks/M5-attendance-notify/T-513-m5-integration.md:17` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-601 | draft | `tasks/M6-rental-reports-privacy/T-601-rental-schema-and-policy.md:14` | `payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-603 | draft | `tasks/M6-rental-reports-privacy/T-603-reservations-api.md:14` | `payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-604 | draft | `tasks/M6-rental-reports-privacy/T-604-rental-jobs.md:17` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-605 | draft | `tasks/M6-rental-reports-privacy/T-605-public-booking.md:17` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-607 | draft | `tasks/M6-rental-reports-privacy/T-607-printables-and-csv.md:17` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-610 | draft | `tasks/M6-rental-reports-privacy/T-610-deletion-requests.md:17` | `RLS` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-612 | draft | `tasks/M6-rental-reports-privacy/T-612-web-reports-feedback-audit.md:17` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-613 | draft | `tasks/M6-rental-reports-privacy/T-613-m6-integration.md:17` | `invoice` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-703 | draft | `tasks/M7-release-distribution/T-703-telemetry.md:28` | `PII` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-706 | draft | `tasks/M7-release-distribution/T-706-installer-and-compose.md:18` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-707 | draft | `tasks/M7-release-distribution/T-707-one-click-templates.md:17` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-708 | draft | `tasks/M7-release-distribution/T-708-demo-instance.md:17` | `password` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-709 | draft | `tasks/M7-release-distribution/T-709-observability.md:17` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-801 | draft | `tasks/M8-desktop/T-801-tauri-shell.md:17` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-802 | draft | `tasks/M8-desktop/T-802-embedded-postgres-and-keychain.md:17` | `password` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-803 | draft | `tasks/M8-desktop/T-803-tray-and-lan.md:14` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-804 | draft | `tasks/M8-desktop/T-804-secure-tunnel.md:14` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-808 | draft | `tasks/M8-desktop/T-808-desktop-integration.md:18` | `phone` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-901 | draft | `tasks/M9-cloud/T-901-platform-crate.md:14` | `RLS` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-903 | draft | `tasks/M9-cloud/T-903-signup.md:14` | `token` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-904 | draft | `tasks/M9-cloud/T-904-plans-and-metering.md:24` | `Payment` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-905 | draft | `tasks/M9-cloud/T-905-subscription-billing.md:17` | `webhook` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |
| T-908 | draft | `tasks/M9-cloud/T-908-tunnel-broker-and-infra.md:17` | `email` | real SBD-30 trigger; add a task-specific Security section from spec 18/25/22. |

## Dependency candidates

A dependency is real only where the cited task provides the named symbol/table/contract. Existing indirect ordering is not a substitute when the implementation names a direct contract. Forward references that would create a cycle are listed separately for self-resolution.

| Task | Status | Evidence | Missing dependency | Contract | Verdict |
|---|---|---|---|---|---|
| T-109 | todo | `tasks/M1-core/T-109-audit.md:30` | T-003 | eq_app audit-schema grants | real |
| T-109 | todo | `tasks/M1-core/T-109-audit.md:56` | T-105 | AuditSink | real |
| T-111 | todo | `tasks/M1-core/T-111-invites-reset-me-totp.md:26` | T-103 | phone normalisation and blind index | real |
| T-113 | todo | `tasks/M1-core/T-113-core-policy-settings-meta-api.md:31` | T-104 | tenant resolution denormalisation | real |
| T-113 | todo | `tasks/M1-core/T-113-core-policy-settings-meta-api.md:49` | T-105 | Module::has_data | real |
| T-114 | todo | `tasks/M1-core/T-114-openapi-contract.md:22` | T-004 | ritma openapi CLI | real |
| T-114 | todo | `tasks/M1-core/T-114-openapi-contract.md:24` | T-005 | gen:api script | real |
| T-114 | todo | `tasks/M1-core/T-114-openapi-contract.md:23` | T-006 | contract CI job | real |
| T-114 | todo | `tasks/M1-core/T-114-openapi-contract.md:27` | T-102a | field-kind OpenAPI components | real |
| T-115 | todo | `tasks/M1-core/T-115-http-hardening.md:16` | T-101 | idempotency_keys schema | real |
| T-115 | todo | `tasks/M1-core/T-115-http-hardening.md:33` | T-104 | client_ip trusted-proxy contract | real |
| T-115 | todo | `tasks/M1-core/T-115-http-hardening.md:47` | T-111 | If-Match on /me | real |
| T-117 | todo | `tasks/M1-core/T-117-m1-integration.md:15` | T-105 | AuditSink/EventSink contracts | real |
| T-117 | todo | `tasks/M1-core/T-117-m1-integration.md:15` | T-106 | PolicyInit wiring | real |
| T-117 | todo | `tasks/M1-core/T-117-m1-integration.md:17` | T-107 | dispatcher/EventSink | real |
| T-202 | todo | `tasks/M2-scheduling/T-202-scheduling-policy.md:57` | T-105 | SchedulingModule trait | real |
| T-202 | todo | `tasks/M2-scheduling/T-202-scheduling-policy.md:57` | T-113 | policy PUT hook | real |
| T-206 | todo | `tasks/M2-scheduling/T-206-session-create-and-check.md:33` | T-112 | ScopeResolver port | real |
| T-211 | todo | `tasks/M2-scheduling/T-211-reads-availability-ics.md:28` | T-207 | availability free_starts | real |
| T-212 | todo | `tasks/M2-scheduling/T-212-cross-module-consumers.md:28` | T-209 | SeriesMembership port | real |
| T-303 | todo | `tasks/M3-setup-web/T-303-files-and-drafts.md:33` | T-002 | config fields | real |
| T-303 | todo | `tasks/M3-setup-web/T-303-files-and-drafts.md:15` | T-101 | files and drafts tables | real |
| T-303 | todo | `tasks/M3-setup-web/T-303-files-and-drafts.md:32` | T-103 | encrypted file storage | real |
| T-303 | todo | `tasks/M3-setup-web/T-303-files-and-drafts.md:40` | T-115 | upload rate-limit layer | real |
| T-308 | todo | `tasks/M3-setup-web/T-308-auth-pages-and-profile.md:32` | T-211 | calendar-feed endpoints | real |
| T-309 | todo | `tasks/M3-setup-web/T-309-policy-form.md:42` | T-106 | policy schema validation | real |
| T-313 | todo | `tasks/M3-setup-web/T-313-admin-pages.md:24` | T-112 | people API | real |
| T-313 | todo | `tasks/M3-setup-web/T-313-admin-pages.md:22` | T-204 | catalog API | real |
| T-313 | todo | `tasks/M3-setup-web/T-313-admin-pages.md:20` | T-208 | session actions | real |
| T-402 | draft | `tasks/M4-enrollment-billing/T-402-enrollment-lifecycle.md:20` | T-209 | SeriesMembership port | real |
| T-402 | draft | `tasks/M4-enrollment-billing/T-402-enrollment-lifecycle.md:25` | T-210 | materialisation contract | real |
| T-402 | draft | `tasks/M4-enrollment-billing/T-402-enrollment-lifecycle.md:19` | T-212 | EnrollmentLookup/Status ports | real |
| T-403 | draft | `tasks/M4-enrollment-billing/T-403-public-registration.md:19` | T-303 | FileService | real |
| T-406 | draft | `tasks/M4-enrollment-billing/T-406-leave.md:19` | T-212 | leave consumer | real |
| T-407 | draft | `tasks/M4-enrollment-billing/T-407-makeup-credits.md:20` | T-208 | makeup participant endpoint | real |
| T-409 | draft | `tasks/M4-enrollment-billing/T-409-import.md:18` | T-303 | import FileService | real |
| T-410 | draft | `tasks/M4-enrollment-billing/T-410-billing-schema-and-policy.md:19` | T-113 | CurrencyLockCheck | real |
| T-412 | draft | `tasks/M4-enrollment-billing/T-412-cycle-issuance.md:18` | T-408 | BillableSource | real |
| T-501 | draft | `tasks/M5-attendance-notify/T-501-attendance-schema-and-policy.md:20` | T-408 | AttendanceStats port | real |
| T-503 | draft | `tasks/M5-attendance-notify/T-503-checkin-api-and-review.md:18` | T-303 | check-in FileService | real |
| T-506 | draft | `tasks/M5-attendance-notify/T-506-notify-core.md:19` | T-416 | Notifier request API | real |
| T-507 | draft | `tasks/M5-attendance-notify/T-507-notify-channels.md:18` | T-103 | secret encryption | real |
| T-511 | draft | `tasks/M5-attendance-notify/T-511-web-checkin.md:19` | T-306 | auth client/logout cleanup | real |
| T-603 | draft | `tasks/M6-rental-reports-privacy/T-603-reservations-api.md:18` | T-415 | cancel terms/invoice adjustments | real |
| T-608 | draft | `tasks/M6-rental-reports-privacy/T-608-feedback.md:18` | T-113 | permission registry | real |
| T-610 | draft | `tasks/M6-rental-reports-privacy/T-610-deletion-requests.md:19` | T-109 | SubjectErase/anonymize | real |
| T-611 | draft | `tasks/M6-rental-reports-privacy/T-611-web-rental.md:17` | T-312 | SlotPicker | real |
| T-705 | draft | `tasks/M7-release-distribution/T-705-key-rotation.md:17` | T-103 | key rotation/re-wrap | real |
| T-806 | draft | `tasks/M8-desktop/T-806-desktop-backups.md:17` | T-701 | backup format | real |
| T-902 | draft | `tasks/M9-cloud/T-902-multi-tenant-routing.md:17` | T-104 | tenant host resolution | real |
| T-903 | draft | `tasks/M9-cloud/T-903-signup.md:17` | T-302 | setup school flow | real |

### False-positive or cyclic task references

| Task | Evidence | Verdict |
|---|---|---|
| T-110 | `tasks/M1-core/T-110-identity-login-refresh.md:99` | T-111/T-112 named as Out-of-scope successors; not implemented here. |
| T-112 | `tasks/M1-core/T-112-people-and-rbac.md:59` | T-303 avatar upload is expressly Out of scope. |
| T-113 | `tasks/M1-core/T-113-core-policy-settings-meta-api.md:51` | T-303 file route is optional; task returns logo_url null until it exists. |
| T-114 | `tasks/M1-core/T-114-openapi-contract.md:28` | T-306a generated client files are a later drift-check extension; T-306a depends on T-114, so reverse dependency would cycle. |
| T-201 | `tasks/M2-scheduling/T-201-scheduling-schema.md:26` | T-203 owns the shared calendar type and depends on T-201; schema task does not consume that type. |
| T-204 | `tasks/M2-scheduling/T-204-catalog-api.md:43` | T-206 is explicitly future API work. |
| T-207 | `tasks/M2-scheduling/T-207-alternatives.md:32` | T-208 is a future consumer, not a dependency of the finder. |
| T-209 | `tasks/M2-scheduling/T-209-series.md:25` | T-210 materialization reference contradicts the task’s own later statement that T-209 owns the reusable core; resolve by making T-209 the provider. |
| T-310 | `tasks/M3-setup-web/T-310-setup-wizard-ui.md:40` | T-314 renders the welcome banner after T-310 records the query contract; no dependency may point backward into the T-314 chain. |
| T-804 | `tasks/M8-desktop/T-804-secure-tunnel.md:17` | T-908 is a later broker; resolve by defining an adapter in T-804 and implementing it in T-908. |
| T-808 | `tasks/M8-desktop/T-808-desktop-integration.md:17` | T-906 is later cloud import; keep M8 export format independent and let M9 import it. |

## Schema candidates

| Task | Evidence | Verdict |
|---|---|---|
| T-106 | `tasks/M1-core/T-106-policy-store.md` Q1 and TD-001 | `resolve_tenant_preset(uuid)` is the architect-authorized SECURITY DEFINER function; no table/column is added. Use `0011_core_tenant_preset_resolver.sql`, the next unused number after mapped `0010`, and test ERD drift. |
| T-107 | `tasks/M1-core/T-107-outbox-dispatcher.md:17-23,133` | `outbox`, `event_receipts`, and `jobs` match spec 06 §4.1 and ERD-06; `next_attempt_at`/`dead_letter_at` occur only in a recorded self-resolution, not the DDL. |
| T-109 | `tasks/M1-core/T-109-audit.md:18-28` | audit tables match spec 06 §4.9/4.11 and ERD-06. |
| T-110 | `tasks/M1-core/T-110-identity-login-refresh.md:76` | real mismatch: `revoked_reason` is requested by the task but absent from spec 06 §4.2 and ERD-06. Resolve by deriving replay from existing `family_id`, revocation, and newer-row state; add no column. |
| T-101 | `tasks/M1-core/T-101-core-schema.md:22` | tenant resolver functions are named by the task; no undeclared table or column. |
| T-211 | `tasks/M2-scheduling/T-211-reads-availability-ics.md` | `persons.ics_secret_version` exists in spec 06 §4.12 and maps to `0012_ics.sql`; real mapped schema. |
| Other SQL matches | `rg` scan of all task files | no other table or column outside spec 06 §3-4 / ERD-06 found. |

## Previously recorded conflicts and self-resolutions

Line numbers below are the `Qn (blocking)` and `Self-resolved` records found during the scan. Existing historical notes are retained; each self-resolution still needs a TD reference in `tasks/DECISIONS.md` and both the task Questions and Report Deviations.

| Task | Status | Evidence lines | Finding |
|---|---|---|---|
| T-002 | review | `tasks/M0-foundation/T-002-core-config-clock-errors.md: 265 Q1 (blocking), 275 Self-resolved, 281 Self-resolved, 282 Self-resolved, 284 Q4 (blocking)` | real recorded conflict/decision; map to TD before completion |
| T-003 | review | `tasks/M0-foundation/T-003-core-db-and-roles.md: 201 Self-resolved, 202 Self-resolved, 203 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-004 | review | `tasks/M0-foundation/T-004-server-cli-http-skeleton.md: 81 Self-resolved, 82 Self-resolved, 83 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-005 | review | `tasks/M0-foundation/T-005-web-scaffold-and-embed.md: 156 Self-resolved, 157 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-006 | review | `tasks/M0-foundation/T-006-ci-makefile-devdb.md: 172 Self-resolved, 173 Self-resolved, 174 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-007 | review | `tasks/M0-foundation/T-007-testkit.md: 168 Self-resolved, 169 Self-resolved, 170 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-008 | review | `tasks/M0-foundation/T-008-m0-integration.md: 142 Self-resolved, 143 Self-resolved, 144 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-101 | review | `tasks/M1-core/T-101-core-schema.md: 156 Self-resolved, 157 Self-resolved, 158 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-102 | review | `tasks/M1-core/T-102-money-and-policy-types.md: 228 Self-resolved, 229 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-102a | review | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md: 264 Self-resolved, 265 Self-resolved, 266 Self-resolved, 268 Self-resolved, 269 Self-resolved, 270 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-103 | review | `tasks/M1-core/T-103-core-crypto.md: 172 Self-resolved, 173 Self-resolved, 174 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-104 | review | `tasks/M1-core/T-104-tenant-context-and-db-begin.md: 184 Self-resolved, 185 Self-resolved, 186 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-105 | review | `tasks/M1-core/T-105-module-system.md: 203 Self-resolved, 204 Self-resolved, 205 Self-resolved` | real recorded conflict/decision; map to TD before completion |
| T-106 | blocked | `tasks/M1-core/T-106-policy-store.md: 161 Self-resolved, 162 Self-resolved, 163 Q1 (blocking)` | real recorded conflict/decision; map to TD before completion |
| T-107 | blocked | `tasks/M1-core/T-107-outbox-dispatcher.md: 133 Self-resolved, 134 Self-resolved, 135 Self-resolved, 136 Q1 (blocking)` | real recorded conflict/decision; map to TD before completion |

## Audit result

The audit found stale task pins, missing required task sections, oversized active tasks, missing direct dependency edges, security sections to add, one schema/interface contradiction (T-110), and cyclic forward references (T-209/T-210, T-804/T-908, T-808/T-906). These findings are not silently accepted: decisions and each applied resolution are recorded in `tasks/DECISIONS.md` and the affected task Questions/Reports. No repository changes had been made when this baseline was read.
