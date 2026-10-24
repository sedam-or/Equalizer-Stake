# Module: reports (optional)

Purpose: the owner/admin dashboard and printable reports. It adapts to whichever modules are enabled.
Policy: `ReportsPolicy` (04 §6.10). No tables of its own in v1. It reads through ports and read-only SQL views.
Legacy reference: `v0-capstone:srs/06-reporting.md`, `backend/app/services/reports.py`.

## 1. Rules

- **RPT-01. Widgets are module-aware.** Each widget declares `requires: ModuleId`. The dashboard shows `policy.widgets ∩ available` widgets. Widgets:

| Key | Requires | Content |
|---|---|---|
| `today` | scheduling | Today's sessions count, in progress, next 3, rooms in use |
| `attention` | core | "Needs attention" list (materialisation conflicts, pending check-ins, payment verifications, needs_manual_reference, unpaid refunds, setup checklist) |
| `revenue` | billing | Collected this period, invoiced, outstanding, 6-month trend (bar), overdue amount |
| `receivables_aging` | billing | 0–7 / 8–30 / 31–60 / >60 days overdue |
| `frozen` | billing | Count of restricted/frozen payers, longest duration, list link |
| `attendance_rate` | attendance | Rate by offering, by instructor; values with < `min_sessions_for_rate` marked `*` |
| `utilization` | scheduling | % of open hours booked, per resource, per weekday heatmap (7×hours) |
| `enrollment_flow` | enrollment | New, ended, on leave this period; pending registrations |
| `rental_occupancy` | rental | Booked hours, revenue, top resources |
| `instructor_ratings` | feedback | Average stars per instructor (≥ 3 ratings), trend |
| `audit_summary` | audit | Today's entries by category, security alerts |

- **RPT-02. Privacy.** Revenue widgets show aggregates only (v0 rule). Per-transaction detail lives in billing screens behind billing permissions.
- **RPT-03. Performance.** Each widget is a separate endpoint: `GET /reports/widgets/{key}?from&to`, cached for 60 s per tenant and range in memory. Each must answer in ≤ 500 ms for 500 students and 2 years of data, which needs indexes on the time columns. The SPA loads widgets in parallel with skeletons.
- **RPT-04. Printable reports.** `GET /reports/print/{report}?from&to` returns HTML with a print stylesheet. Reports: `monthly_summary` (all widgets as tables), `attendance_by_student`, `receivables` and `instructor_hours` (sessions taught and minutes per instructor, for payroll handoff).
- **RPT-05. CSV exports.** Every list screen (students, sessions, invoices, payments, marks) has "Export CSV" with the current filters. The export is a streamed response and is audited.
- **RPT-06. Access.** Owner: all widgets. Admin: all except `revenue` and `audit_summary`, unless granted. Grants use `reports.revenue.read`, which defaults to owner only.

## 2. Acceptance criteria

1. **Module-aware.** Given billing disabled, Then `/reports/widgets` doesn't list revenue widgets, and GET `/reports/widgets/revenue` → 404 module-disabled.
2. **Incomplete data.** Given an instructor with 3 sessions and `min_sessions_for_rate=5`, Then the attendance_rate entry has `insufficient: true`.
3. **Utilisation.** Given opening hours of 2.5 h/day × 5 days in one room and 5 h booked in that week, Then utilisation is 40%.
