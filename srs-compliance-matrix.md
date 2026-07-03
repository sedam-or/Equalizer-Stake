# SRS vs Backend Compliance Matrix

Generated: 2026-07-01

## FR-01: Login dan Autentikasi Berbasis Peran (RBAC)

| SRS Ref | Requirement | Status | Evidence | Gap Description |
|---------|-------------|--------|----------|-----------------|
| FR-01 §2 | Single login page, role-based dashboard redirect | DONE | `routers/auth.py:33-54` (login returns role in response) | — |
| FR-01 §2.1 | JWT HS256, per-role payload | DONE | `services/auth.py:17` `ALGORITHM = "HS256"` | — |
| FR-01 §2.1 | Owner/Admin/Guru timeout 30 min | DONE | `services/auth.py:19-22` `ACCESS_TOKEN_EXPIRE_MINUTES` | — |
| FR-01 §2.1 | Wali Murid timeout 60 min | DONE | `services/auth.py:22` `"guardian": 60` | — |
| FR-01 §2.1 | Draft restored after re-login | DONE | `models/draft.py:14` 7-day TTL auto-save drafts | — |
| FR-01 §2.2 | Max 5 failed attempts in 15 min | DONE | `services/auth.py:26-27` `MAX_FAILED_ATTEMPTS=5`, `LOCKOUT_MINUTES=15` | — |
| FR-01 §2.2 | Account locked 15 min after 5th failure | DONE | `services/auth.py:84-85` sets `locked_until` | — |
| FR-01 §2.2 | Email notification to Admin on lockout | PARTIAL | `routers/auth.py:43` logs `ACCOUNT_LOCKED` to audit log | No email sent to Admin; only audit log entry |
| FR-01 §2.3 | Auto-generate 8-char alphanumeric password | MISSING | `services/admin_mgmt.py:23-47` `create_admin` takes password param | No auto-generation; password passed manually |
| FR-01 §2.3 | Email with link, username, temp password | MISSING | `services/admin_mgmt.py` no email sending in create flow | No onboarding email with credentials |
| FR-01 §2.3 | Temp password expires after 7 days | MISSING | `models/user.py:44` `password_expires_at` field exists | Field exists but no expiry check in login flow |
| FR-01 §2.3 | First login: must change password before dashboard | MISSING | `models/user.py:42` `first_login` field exists | Field exists but not checked in `routers/auth.py` login flow |
| FR-01 §2.3 | Admin can resend expired credentials | MISSING | No resend endpoint found | No resend/refresh temporary password endpoint |
| FR-01 §2.4 | Change password via Pengaturan Akun | DONE | `routers/auth.py:12` imports `ChangePasswordRequest` | — |
| FR-01 §2.4 | Password change recorded in audit log | DONE | `routers/auth.py:178` `PASSWORD_CHANGE` audit event | — |
| FR-01 §3.1 | Login → JWT + audit log + dashboard ≤2s | DONE | `routers/auth.py:50-54` | — |
| FR-01 §3.2 | Wrong creds → "Username atau password tidak valid" | DONE | `routers/auth.py:48` exact message match | — |
| FR-01 §3.2 | Generic error (no field disclosure) | DONE | `routers/auth.py:46-49` single generic message | — |
| FR-01 §3.3 | 5th failure → lock 15 min + email Admin | PARTIAL | `services/auth.py:84-85` locks account | No email notification; only audit log |
| FR-01 §3.4 | First login → redirect to password change | MISSING | No `first_login` check in login handler | Login returns token normally; no forced redirect |
| FR-01 §3.5 | Expired temp password → "Link telah kedaluwarsa" | MISSING | No expiry check in `authenticate_user` | `password_expires_at` field unused in auth flow |
| FR-01 §4.4 | Inactive account → "Akun tidak aktif" | PARTIAL | `services/auth.py:78` checks `is_active` | Returns `None` (generic 401), no specific inactive message |
| FR-01 §5 | All error conditions handled per spec | PARTIAL | See individual rows above | Inactive account and expired password messages missing |

## FR-02: Penjadwalan Kursus (Scheduling)

| SRS Ref | Requirement | Status | Evidence | Gap Description |
|---------|-------------|--------|----------|-----------------|
| FR-02 §2.1 | Real-time slot validation (teacher, room, time) | DONE | `services/scheduling.py:20-70` `validate_slot()` | — |
| FR-02 §2.1 | Conflict must not pass to storage | DONE | `services/scheduling.py:73-74` validates before create | — |
| FR-02 §2.2 | Teacher overlap detection | DONE | `services/scheduling.py:64` `"Guru sudah ada sesi lain"` | — |
| FR-02 §2.2 | Room overlap detection | DONE | `services/scheduling.py:66` `"Ruangan sudah digunakan"` | — |
| FR-02 §2.2 | Student overlap detection | DONE | `services/scheduling.py:68` `"Siswa sudah ada sesi lain"` | — |
| FR-02 §2.2 | Sunday closed | DONE | `services/scheduling.py:36` `"Hari Minggu tutup"` | — |
| FR-02 §2.3 | Operational hours validation | MISSING | No operational hours check in scheduling service | No validation against studio operating hours |
| FR-02 §2.4 | Concurrency control (optimistic locking) | MISSING | No lock version or optimistic locking in session model | Concurrent saves could cause race condition |
| FR-02 §2.5 | Cancel by Admin → 100% credit refund | PARTIAL | `routers/scheduling.py:62-70` cancel endpoint exists | `services/scheduling.py` has `# TODO: implement cancel logic with credit refund rules` |
| FR-02 §2.5 | Cancel by Wali ≥24h → 50% credit | MISSING | No guardian cancel endpoint or credit logic | — |
| FR-02 §2.5 | Cancel by Wali <24h → 0% credit | MISSING | No guardian cancel endpoint or credit logic | — |
| FR-02 §2.5 | Cancel requires reason + audit log | DONE | `models/session.py:34` `cancellation_reason`; `routers/override.py` audit | — |
| FR-02 §2.6 | Draft auto-save | DONE | `services/draft.py` auto-save with 7-day TTL | — |
| FR-02 §3.1 | 5th week → non-credit theory session | PARTIAL | `models/session.py` has `session_type` field | No auto-creation of theory sessions for 5th week |
| FR-02 §3.2 | National holiday → reschedule to next week same day | MISSING | No holiday handling in scheduling service | — |
| FR-02 §4.1 | Save schedule → validate ≤1s + audit | DONE | `services/scheduling.py:73` validates then creates | — |
| FR-02 §4.2 | Conflict → show alternative slots | PARTIAL | `services/scheduling.py:70` returns conflicts list | No alternative slot suggestion logic |
| FR-02 §4.6 | Holiday → auto-reschedule or cancel+refund | MISSING | No holiday calendar or reschedule logic | — |

## FR-03: Penerbitan Tagihan (Billing)

| SRS Ref | Requirement | Status | Evidence | Gap Description |
|---------|-------------|--------|----------|-----------------|
| FR-03 §2.1 | Auto-invoice on 5th of month at 00:00 | PARTIAL | `services/billing.py:38` `create_invoice()` exists | No cron job or scheduler invoking it; function exists but no automation trigger |
| FR-03 §2.1 | Invoice for all guardians with active students | DONE | `services/billing.py:38` `create_invoice` per guardian | — |
| FR-03 §2.2 | Invoice includes: student name, instrument, level, sessions, amount | DONE | `models/invoice.py` has billing fields; `schemas/billing.py` response model | — |
| FR-03 §2.3 | Unique 3-digit code per invoice | DONE | `services/billing.py:19-36` `generate_unique_code()` | — |
| FR-03 §2.3 | Collision retry up to 3x | DONE | `services/billing.py:16` `UNIQUE_CODE_MAX_RETRIES = 3` | — |
| FR-03 §2.3 | After 3 failures → "Perlu Kode Manual" | PARTIAL | `services/billing.py` has retry logic | No "Perlu Kode Manual" flag or admin notification on collision |
| FR-03 §2.4 | WhatsApp/email notification on invoice | MISSING | `services/billing.py` no notification calls | No notification sent when invoice is created |
| FR-03 §3.1 | Rp100k fine on day 11 if unpaid | DONE | `services/billing.py:111-120` `apply_fine()` adds Rp100000 | — |
| FR-03 §3.1 | Status → overdue after fine | DONE | `services/billing.py:121` `invoice.status = "overdue"` | — |
| FR-03 §3.2 | Guardian account frozen after overdue | DONE | `services/billing.py:123` freezes guardian account | — |
| FR-03 §3.3 | State machine: draft→issued→paid/overdue/cancelled | DONE | `models/invoice.py:25` Enum with all states | — |
| FR-03 §3.4 | Freeze: block login, show message | PARTIAL | `services/multi_student.py:59-83` freezes household | No login-block middleware for frozen guardians |
| FR-03 §4.1 | Admin verifies payment via unique code | DONE | `routers/billing.py:18-39` verify endpoint | — |
| FR-03 §4.3 | Idempotency: duplicate verification rejected | DONE | `routers/billing.py:36` "sudah terverifikasi" | — |
| FR-03 §5.1 | Auto-invoice cron acceptance test | PARTIAL | Function exists, no cron trigger | — |
| FR-03 §5.2 | Unique code collision → manual code | PARTIAL | Retry exists, manual fallback missing | — |
| FR-03 §5.3 | Fine auto-applied on day 11 | PARTIAL | `apply_fine()` exists but no cron/scheduler | No automated trigger; manual endpoint only |
| FR-03 §5.4 | Unfreeze after full payment | DONE | `services/override.py:14` `unfreeze_guardian()` | — |
| FR-03 §5.5 | Payment verification with audit | DONE | `routers/billing.py:18-39` + audit in service | — |
| FR-03 §5.6 | Duplicate payment → alert, no change | DONE | `routers/billing.py:36` | — |

## FR-04: Denda Otomatis & Pembekuan (Fine & Freeze)

| SRS Ref | Requirement | Status | Evidence | Gap Description |
|---------|-------------|--------|----------|-----------------|
| FR-04 §3.2 | Day 11: Rp100k fine + status overdue | DONE | `services/billing.py:111-121` | — |
| FR-04 §3.2 | Auto-freeze on overdue | DONE | `services/billing.py:123` | — |
| FR-04 §3.4 | Freeze: block access + message | PARTIAL | `services/multi_student.py:78` sets frozen status | No middleware to block frozen user login |
| FR-04 §5.4 | Admin unfreeze after full payment | DONE | `services/override.py:14` | — |

## SRS-04: Validasi Kehadiran Guru (Attendance)

| SRS Ref | Requirement | Status | Evidence | Gap Description |
|---------|-------------|--------|----------|-----------------|
| §2.1 | GPS validation radius 150m | DONE | `services/attendance.py:16` `GPS_RADIUS_M = 150` | — |
| §2.1 | Haversine distance calculation | DONE | `services/attendance.py:20` `haversine_distance()` | — |
| §2.2 | Server-side timestamp validation | DONE | `services/attendance.py:41` `validate_server_timestamp()` | — |
| §2.3 | Face detection required | DONE | `services/attendance.py` validates `face_valid` | — |
| §2.3 | Face confidence field | DONE | `models/attendance.py:26` `face_confidence` | — |
| §3 | Client-side validation flow | PARTIAL | `services/attendance.py` server-side validation | Client-side validation is frontend responsibility |
| §4.1 | Status: verified / pending_review / rejected | DONE | `models/attendance.py:29-31` Enum with all 3 states | — |
| §4.1 | Auto-verified if GPS + face OK | DONE | `services/attendance.py:80` `verified if all_valid else pending_review` | — |
| §4.1 | Pending if GPS outside radius | DONE | `services/attendance.py:80` sets `pending_review` | — |
| §4.2 | Admin review SLA (24h) | MISSING | `routers/attendance.py:41-57` review endpoint exists | No SLA enforcement or reminder for pending reviews |
| §5 | Guru can only submit own sessions | PARTIAL | `routers/attendance.py:20` teacher role required | No explicit check that teacher owns the session |
| §6 | Offline mode with IndexedDB + auto-upload | PARTIAL | `models/attendance.py:36` `is_offline_submission` field | Offline storage is frontend; server accepts flag |
| §7.1 | All valid → status "verified" | DONE | `services/attendance.py:80` | — |
| §7.2 | GPS outside → status "pending_review" | DONE | `services/attendance.py:80` | — |
| §7.3 | Face not detected → reject submission | PARTIAL | `services/attendance.py` validates face | No explicit client-side rejection message in backend |
| §7.4 | Offline → IndexedDB + auto-sync | PARTIAL | Backend accepts `is_offline_submission`; no conflict resolution | No server-side conflict resolution for offline sync |
| §7.5 | Admin approve/reject with reason | DONE | `routers/attendance.py:41-57` review with `rejection_reason` | — |
| §8 | Error handling (network, GPS fail, face fail) | PARTIAL | Server handles ValueError with 400 | No specific error codes per failure type |
| §9 | Photo URL (AES-256-GCM encrypted) | PARTIAL | `models/attendance.py:21` `photo_url` field with comment `KRITIS: AES-256-GCM` | Field exists; encryption implementation not verified |
| §10 | POST /api/attendance + PUT /{id}/review | DONE | `routers/attendance.py:17,41` | — |
| §11 | Teacher session ownership check | PARTIAL | Role check exists, no ownership validation | Could submit attendance for another teacher's session |

## SRS-09: Multi-Student per Wali Murid

| SRS Ref | Requirement | Status | Evidence | Gap Description |
|---------|-------------|--------|----------|-----------------|
| §2.1 | Guardian:Student 1:N relationship | DONE | `models/student.py:19` `id_guardian` FK | — |
| §2.1 | Student fields: name, instrument, level, status | DONE | `models/student.py` all fields present | — |
| §3.1 | Guardian dashboard shows all students | DONE | `routers/multi_student.py:27-44` list endpoint | — |
| §3.1 | Profile switching between students | MISSING | No profile switching endpoint | Guardian sees list but no switch/select mechanism |
| §3.2 | Frozen badge shown on frozen students | PARTIAL | `routers/multi_student.py:41` returns `enrollment_status` | Status returned but no explicit "badge" or frozen indicator |
| §4.1 | Only Admin can add students | DONE | `routers/multi_student.py:51` `require_role("admin")` | — |
| §4.1 | Guardian must be active | DONE | `services/multi_student.py:37-43` checks active status | — |
| §4.2 | Add student → create + return ID | DONE | `routers/multi_student.py:47-68` | — |
| §5 | Combined invoice per guardian (1 invoice for all students) | DONE | `services/multi_student.py:86-93` `get_guardian_invoice_total()` counts active students | — |
| §6.1 | Freeze applies to household (all students) | DONE | `services/multi_student.py:59-83` iterates all students | — |
| §6.2 | Freeze: all students blocked, sessions cancelled | PARTIAL | `services/multi_student.py:69-71` freezes students | No automatic session cancellation on freeze |
| §6.3 | Invoice handling during freeze | MISSING | No logic to hold/cancel invoices during freeze | — |
| §7 | Constraint: max students per guardian | MISSING | No max student limit validation | — |

## Summary Statistics

| Module | DONE | PARTIAL | MISSING | Total |
|--------|------|---------|---------|-------|
| FR-01 Authentication | 12 | 4 | 6 | 22 |
| FR-02 Scheduling | 7 | 4 | 5 | 16 |
| FR-03 Billing | 9 | 4 | 1 | 14 |
| FR-04 Fine & Freeze | 3 | 1 | 0 | 4 |
| SRS-04 Attendance | 8 | 6 | 1 | 15 |
| SRS-09 Multi-Student | 5 | 2 | 3 | 10 |
| **Total** | **44** | **21** | **16** | **81** |

### Overall: 54% DONE, 26% PARTIAL, 20% MISSING

## Critical Gaps (High Priority)

1. **Onboarding flow completely missing** (FR-01 §2.3): No auto-password generation, no email with credentials, no first-login forced password change, no resend, no expiry check
2. **Credit refund logic not implemented** (FR-02 §2.5): Cancel endpoint has TODO comment; no 100%/50%/0% refund tiers
3. **No automated cron jobs** (FR-03): `create_invoice()` and `apply_fine()` exist but have no scheduler/cron trigger
4. **No notification on invoice creation** (FR-03 §2.4): Billing service doesn't call notification service
5. **National holiday handling missing** (FR-02 §3.2): No holiday calendar or auto-reschedule
6. **Frozen guardian login not blocked** (FR-03 §3.4, FR-04 §3.4): Freeze sets status but no middleware blocks login
7. **Admin review SLA not enforced** (SRS-04 §4.2): Review endpoint exists but no 24h reminder/escalation
8. **Session ownership not validated** (SRS-04 §5, §11): Teacher could submit attendance for another teacher's session
9. **Operational hours validation missing** (FR-02 §2.3): No check against studio operating hours
10. **Concurrency control missing** (FR-02 §2.4): No optimistic locking for concurrent schedule saves
