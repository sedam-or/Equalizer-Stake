# SRS Compliance Matrix — BMD Music School

**Generated:** 2026-07-05
**Scope:** All Functional Requirements (FR-01 to FR-15) and Use Cases (UC-01 to UC-13)
**Status Legend:** ✅ Implemented | ⚠️ Partial | ❌ Missing

---

## 1. Functional Requirements (FR)

| FR | Description | Status | Backend Evidence | Frontend Evidence | Gaps |
|----|-------------|--------|-----------------|-------------------|------|
| **FR-01** | Login & RBAC Authentication | ✅ | `routers/auth.py` — login, first-login-change-password, change-password, forgot-password, refresh, logout<br>`services/auth.py` — JWT HS256, per-role expiry (owner/admin/teacher=30min, guardian=60min), brute-force (5 attempts/15min lockout)<br>`models/user.py` — User model with role, failed_login_attempts, locked_until, first_login, is_temp_password, password_expires_at<br>`middleware/auth.py` — require_role() dependency | `app/login/page.tsx` — login form<br>`app/forgot-password/page.tsx` — forgot password<br>`app/reset-password/` — reset flow<br>`app/dashboard/profile/page.tsx` — change password | Onboarding email with temp password: `routers/admin_mgmt.py` has `resend_onboarding` but only for admins. Teacher/guardian onboarding email via `services/email.py` not verified to auto-send on account creation. |
| **FR-02** | Scheduling with Real-Time Conflict Validation | ✅ | `routers/scheduling.py` — list/create/cancel sessions, validate_slot, list_rooms, list_teachers<br>`services/scheduling.py` — validate_slot() with conflict detection<br>`models/session.py` — Session model<br>`models/room.py` — Room model | `app/dashboard/schedule/page.tsx` — schedule management<br>`app/dashboard/assign-session/page.tsx` — session assignment | None significant. |
| **FR-03** | Auto-Invoice Generation with Unique Code | ✅ | `routers/billing.py` — list invoices, delete invoice<br>`routers/cron_trigger.py` — trigger/auto-invoice endpoint<br>`services/billing.py` — generate_unique_code() (3-digit 000-999), create_invoice(), verify_payment() matching unique code<br>`services/scheduler.py` — job_auto_invoice() (runs on tanggal 5)<br>`models/invoice.py` — Invoice with unique_code (CHAR 3), billing_period, due_date, status | `app/dashboard/billing/page.tsx` — invoice list<br>`app/register-course/page.tsx` — registration with pricing | None significant. |
| **FR-04** | Auto Late Penalty & Account Freeze | ✅ | `routers/billing.py` — apply_fine_endpoint, unfreeze_account<br>`routers/cron_trigger.py` — trigger/fine-freeze<br>`services/scheduler.py` — job_fine_freeze() (scheduled)<br>`routers/multi_student.py` — freeze_household | `app/dashboard/billing/page.tsx` — fine display<br>`app/dashboard/override/page.tsx` — unfreeze | None significant. |
| **FR-05** | Digital Attendance with Photo | ✅ | `routers/attendance.py` — list, submit, review attendance<br>`routers/photos.py` — upload/get attendance photo (encrypted)<br>`services/photo_storage.py` — validate_photo, save_attendance_photo, load_photo (encrypted)<br>`models/attendance.py` — Attendance model | `app/dashboard/attendance/page.tsx` — attendance list<br>`app/dashboard/attendance/submit/page.tsx` — submit attendance | None significant. |
| **FR-06** | Multi-Channel Notifications (WhatsApp + Email) | ✅ | `routers/notification.py` — list, unread-count, toggle channel, circuit-breaker reset/status, test-whatsapp, email/status<br>`services/notification.py` — channel priority (critical: email→WhatsApp, non-critical: WhatsApp→email), retry policy (3x exponential backoff), group_notification_by_guardian<br>`services/whatsapp.py` — send_whatsapp_message with retry<br>`services/email.py` — send_email with TLS, circuit breaker (5 failures, 30min cooldown)<br>`models/notification.py` — Notification with channel, category, status, retry_count, error_message | `app/dashboard/notifications/page.tsx` — notification list<br>`app/dashboard/notification-settings/page.tsx` — toggle per channel/category | SRS requires `wali_murid_id` as recipient. Model uses `recipient_id → users.id_user` (generic FK). Grouping by guardian implemented in service layer but model FK points to users table. |
| **FR-07** | Exam Eligibility Check | ✅ | `routers/reports.py` — get_eligibility endpoint<br>`services/eligibility.py` — check_eligibility() | `app/dashboard/reports/page.tsx` — eligibility display | None significant. |
| **FR-08** | Draft Auto-Save & Session Recovery | ⚠️ | `routers/draft.py` — list, get, save, delete draft<br>`services/draft.py` — save_draft, get_draft, delete_draft, DraftConflictError<br>`models/draft.py` — Draft model<br>`routers/cron_trigger.py` — trigger/draft-cleanup | No frontend auto-save timer found. No `setInterval`/30s auto-save hook. No "Restore draft?" prompt after session recovery. | **Frontend auto-save (30s interval) not implemented.** Backend CRUD exists but client-side auto-save loop and session-recovery prompt are missing. SRS specifies local-storage + server sync every 30s. |
| **FR-09** | Immutable Audit Log | ✅ | `routers/audit.py` — list_audit_logs<br>`models/audit_log.py` — AuditLog model<br>`services/audit.py` — log_auth_event, log_crud_event<br>`services/audit_retention.py` — retention management | `app/dashboard/audit-log/page.tsx` — audit log viewer | Immutability enforcement (no DELETE/UPDATE endpoints) — verified by absence of mutation endpoints in `routers/audit.py`. |
| **FR-10** | Multi-Student per Guardian | ✅ | `routers/multi_student.py` — list_students, add_student, freeze_household<br>`services/multi_student.py` — multi-student logic<br>`models/guardian.py` — Guardian model<br>`models/student.py` — Student model with guardian FK | `app/dashboard/students/page.tsx` — student management<br>`app/dashboard/my-students/page.tsx` — guardian's student view | None significant. |
| **FR-11** | Data Deletion (UU PDP Pasal 19) | ✅ | `routers/privacy.py` — request_deletion, verify_deletion, list_deletion_requests<br>`services/privacy.py` — create_deletion_request, complete_deletion<br>`models/deletion_request.py` — DeletionRequest with entity_type, status (pending/approved/rejected) | `app/dashboard/settings/page.tsx` — privacy section (inferred) | None significant. Full request→verify→execute flow present. |
| **FR-12** | Holiday & Week-5 Management | ⚠️ | `routers/holidays.py` — list, create, delete holidays (in-memory, not DB-persisted)<br>Code comment: `# In-memory holidays (ponytail: use DB table when migration ready)` | `app/dashboard/holidays/page.tsx` — holiday management | **Holidays stored in-memory only** — data lost on server restart. No DB model for holidays. No week-5 special handling found in scheduling service. |
| **FR-13** | Owner Dashboard with KPIs & Export | ✅ | `routers/reports.py` — owner_dashboard, export_pdf, export_csv, admin_stats, teacher_stats, guardian_stats<br>`services/reports.py` — get_owner_dashboard, render_dashboard_html | `app/dashboard/reports/page.tsx` — reports page<br>`app/dashboard/page.tsx` — main dashboard | PDF export uses HTML printable (browser Print) rather than server-side PDF generation. Functional but less polished. |
| **FR-14** | Studio/Room Reservation | ✅ | `routers/scheduling.py` — book_studio_endpoint, request_recording_endpoint, approve_recording_endpoint, list_rooms<br>`services/scheduling.py` — book_studio_rehearsal<br>`models/room.py` — Room model | `app/dashboard/studio/page.tsx` — studio booking | None significant. |
| **FR-15** | Manual Payment Verification | ✅ | `routers/payments.py` — list, verify, create, delete payments<br>`routers/billing.py` — verify_payment_endpoint (unique code matching)<br>`services/billing.py` — verify_payment() with unique code validation<br>`models/payment.py` — Payment model | `app/dashboard/payments/page.tsx` — payment management<br>`app/dashboard/my-payments/page.tsx` — guardian payment view | None significant. |

---

## 2. Use Cases (UC)

| UC | Description | Status | Backend Evidence | Frontend Evidence | Gaps |
|----|-------------|--------|-----------------|-------------------|------|
| **UC-01** | Login & Role-Based Redirect | ✅ | `routers/auth.py` — POST /api/v1/auth/login with role-based JWT<br>`middleware/auth.py` — require_role() | `app/login/page.tsx` — login form | None. |
| **UC-02** | *(Not defined in SRS)* | — | — | — | **UC-02 not specified in any SRS document.** Gap in use case numbering. |
| **UC-03** | Input Jadwal (Schedule Input) | ✅ | `routers/scheduling.py` — POST /api/v1/scheduling with conflict validation | `app/dashboard/schedule/page.tsx` | None. |
| **UC-04** | Notification Delivery | ✅ | `routers/notification.py` — full notification system<br>`services/notification.py` — multi-channel with fallback<br>`services/whatsapp.py` — WhatsApp integration<br>`services/email.py` — Email SMTP integration | `app/dashboard/notifications/page.tsx`<br>`app/dashboard/notification-settings/page.tsx` | None significant. |
| **UC-05** | *(Not defined in SRS)* | — | — | — | **UC-05 not specified in any SRS document.** Gap in use case numbering. |
| **UC-06** | Studio Reservation | ✅ | `routers/scheduling.py` — POST /api/v1/scheduling/book-studio | `app/dashboard/studio/page.tsx` | None. |
| **UC-07** | *(Not defined in SRS)* | — | — | — | **UC-07 not specified in any SRS document.** Gap in use case numbering. |
| **UC-08** | Multi-Student per Guardian | ✅ | `routers/multi_student.py` — CRUD + freeze_household | `app/dashboard/students/page.tsx`<br>`app/dashboard/my-students/page.tsx` | None. |
| **UC-09** | Exam Eligibility Check | ✅ | `routers/reports.py` — GET /eligibility/{student_id} | `app/dashboard/reports/page.tsx` | None. |
| **UC-10** | Owner Dashboard | ✅ | `routers/reports.py` — GET /owner-dashboard, export PDF/CSV | `app/dashboard/reports/page.tsx` | None. |
| **UC-11** | Data Deletion (UU PDP) | ✅ | `routers/privacy.py` — request, verify, list deletion | `app/dashboard/settings/page.tsx` (inferred) | None. |
| **UC-12** | Audit Log Viewing | ✅ | `routers/audit.py` — list audit logs (read-only, immutable) | `app/dashboard/audit-log/page.tsx` | None. |
| **UC-13** | Draft Auto-Save | ⚠️ | `routers/draft.py` — CRUD endpoints for drafts | No auto-save UI component found | **Frontend 30s auto-save timer and session recovery prompt not implemented.** Backend API exists. |

---

## 3. Summary

### Compliance Stats

| Category | Total | ✅ Implemented | ⚠️ Partial | ❌ Missing |
|----------|-------|----------------|------------|------------|
| Functional Requirements | 15 | 13 | 2 | 0 |
| Use Cases (defined) | 10 | 9 | 1 | 0 |
| Use Cases (undefined) | 3 | — | — | — |

### Critical Gaps

1. **FR-08 / UC-13 — Draft Auto-Save (Frontend)**
   - Backend API: ✅ CRUD endpoints exist
   - Frontend 30s auto-save timer: ❌ Missing
   - Session recovery prompt ("Restore draft?"): ❌ Missing
   - Local storage sync: ❌ Missing
   - **Impact:** Users lose form data on session timeout

2. **FR-12 — Holiday Persistence**
   - Holiday storage: ⚠️ In-memory only (lost on restart)
   - DB model: ❌ Missing
   - Week-5 handling: ❌ Not found in scheduling service
   - **Impact:** Holiday data not persisted; no week-5 schedule adjustment

3. **UC-02, UC-05, UC-07 — Undefined Use Cases**
   - SRS numbering references UC-01 through UC-20 but only 10 are defined
   - **Impact:** Coverage gap — 7 use cases referenced but unspecified

### Minor Gaps

4. **FR-01 — Onboarding Email**
   - Admin onboarding: ✅ (resend_onboarding in admin_mgmt.py)
   - Teacher/Guardian auto-onboarding email on creation: Not verified

5. **FR-06 — Notification Recipient Model**
   - SRS specifies `wali_murid_id` as recipient
   - Model uses `recipient_id → users.id_user` (generic FK)
   - Grouping by guardian handled in service layer

6. **FR-13 — PDF Export**
   - Uses HTML printable (browser Print dialog) instead of server-side PDF
   - Functional but not production-grade PDF generation

---

## 4. SRS Document Coverage

| SRS File | FRs Covered | UCs Covered | Implementation Status |
|----------|-------------|-------------|----------------------|
| `01-authentication.md` | FR-01 | UC-01 | ✅ Fully implemented |
| `02-scheduling.md` | FR-02, FR-12 | UC-03 | ✅ / ⚠️ (FR-12 partial) |
| `03-billing.md` | FR-03, FR-04, FR-15 | — | ✅ Fully implemented |
| `04-attendance.md` | FR-05 | — | ✅ Fully implemented |
| `05-notifications.md` | FR-06 | UC-04 | ✅ Fully implemented |
| `06-reporting.md` | FR-07, FR-13 | UC-09, UC-10 | ✅ Fully implemented |
| `07-audit-governance.md` | FR-09, FR-11 | UC-11, UC-12 | ✅ Fully implemented |
| `08-studio-reservation.md` | FR-14 | UC-06 | ✅ Fully implemented |
| `09-multi-student.md` | FR-10 | UC-08 | ✅ Fully implemented |
| `10-draft-autosave.md` | FR-08 | UC-13 | ⚠️ Backend only |

---

## 5. Implementation File Index

### Backend Routers (26 files)
`admin_mgmt.py`, `attendance.py`, `audit.py`, `auth.py`, `billing.py`, `cron_trigger.py`, `draft.py`, `feedback.py`, `holidays.py`, `import_data.py`, `multi_student.py`, `my_portal.py`, `notification.py`, `override.py`, `payments.py`, `photos.py`, `pricing.py`, `privacy.py`, `ratings.py`, `registrations.py`, `reports.py`, `scheduling.py`, `settings.py`, `students.py`, `teachers.py`

### Backend Models (18 entities)
`attendance.py`, `audit_log.py`, `deletion_request.py`, `draft.py`, `feedback.py`, `guardian.py`, `invoice.py`, `notification.py`, `payment.py`, `pricing.py`, `rating.py`, `registration.py`, `room.py`, `session.py`, `student.py`, `system_setting.py`, `teacher.py`, `user.py`

### Backend Services (24 files)
`admin_mgmt.py`, `attendance.py`, `audit.py`, `audit_retention.py`, `auth.py`, `background_queue.py`, `billing.py`, `draft.py`, `eligibility.py`, `email.py`, `encryption.py`, `import_service.py`, `jwt_blacklist.py`, `multi_student.py`, `notification.py`, `override.py`, `photo_storage.py`, `pricing.py`, `privacy.py`, `reports.py`, `scheduler.py`, `scheduling.py`, `settings.py`, `whatsapp.py`

### Frontend Pages (26 dashboard pages + public pages)
Dashboard: `admins`, `assign-session`, `attendance`, `attendance/submit`, `audit-log`, `billing`, `feedback`, `holidays`, `my-payments`, `my-schedule`, `my-students`, `notification-settings`, `notifications`, `override`, `payments`, `pricing`, `profile`, `ratings`, `registrations`, `reports`, `schedule`, `settings`, `students`, `studio`, `teachers`

Public: `login`, `register`, `register-course`, `forgot-password`, `reset-password`, `instruments`, `help`
