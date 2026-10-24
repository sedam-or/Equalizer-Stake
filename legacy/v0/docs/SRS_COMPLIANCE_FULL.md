# SRS vs Implementation — Full Compliance Matrix
**Tanggal**: 2026-07-01
**Scope**: All SRS modules (01-11) + entities + NFRs
**Total requirements checked**: ~150+

---

## Executive Summary

| Category | DONE | PARTIAL | MISSING | Total |
|----------|------|---------|---------|-------|
| Auth (FR-01) | 6 | 4 | 2 | 12 |
| Scheduling (FR-02) | 5 | 4 | 3 | 12 |
| Billing (FR-03) | 6 | 3 | 4 | 13 |
| Attendance (FR-04) | 7 | 3 | 3 | 13 |
| Multi-Student (FR-09) | 4 | 2 | 1 | 7 |
| Notifications (FR-05) | 11 | 3 | 6 | 20 |
| Reporting (FR-06) | 5 | 6 | 5 | 16 |
| Audit & Governance (FR-07) | 13 | 4 | 4 | 21 |
| Studio Reservation (FR-08) | 2 | 4 | 6 | 12 |
| Draft Auto-Save (FR-10) | 5 | 3 | 1 | 9 |
| Entities (11 models) | 11 | 0 | 0 | 11 |
| NFRs | 4 | 7 | 10 | 21 |
| **TOTAL** | **79** | **43** | **48** | **~170** |

**Overall compliance: ~46% DONE, ~25% PARTIAL, ~29% MISSING**

---

## CRITICAL GAPS (Blocking Production)

### 1. PII Encryption (NFR-SEC-01) — CRITICAL
- `guardian.full_name`, `student.full_name`, `attendance.photo_url` marked KRITIS but stored plaintext
- No AES-256-GCM encrypt/decrypt service exists
- No KMS/Vault integration

### 2. Onboarding Flow (FR-01 §2.3) — CRITICAL
- No auto-password generation
- No email with credentials
- `first_login` field exists but unused in login flow
- No forced password change on first login

### 3. Cron Jobs Missing — CRITICAL
- `create_invoice()` exists but no scheduler triggers it (should run tanggal 5, 00:00)
- `apply_fine()` exists but no scheduler (should run tanggal 11, 00:00)
- No draft cleanup scheduler
- No notification reminder scheduler (1 jam sebelum sesi)

### 4. Studio Reservation (FR-08) — MOST INCOMPLETE
- No guardian self-booking (only admin can create sessions)
- No recording approval workflow
- No studio hourly pricing (only course pricing exists)
- No alternative slot suggestions on conflict
- No DB-level unique constraint for double-booking prevention
- Cancel session is TODO stub

### 5. Default Secret Key (NFR-SEC-05-A02)
- `SECRET_KEY = "change-me-in-production"` as default in config.py
- Warning logged on startup but no enforcement

---

## HIGH PRIORITY GAPS

### 6. Notification Grouping (FR-05 §4)
- No logic to merge N student notifications into 1 message per guardian

### 7. Admin Channel Toggle (FR-05 §5)
- Schema exists but no API endpoint to enable/disable channels per category

### 8. Frozen Guardian Login (FR-03)
- Freeze sets `account_status` but no auth middleware blocks login for frozen accounts

### 9. Credit Refund on Cancel (FR-02)
- Cancel endpoint has `TODO: implement cancel logic with credit refund rules`

### 10. Audit Log Gaps
- No `category` field (operasional/keamanan/kepatuhan)
- No 7-year retention mechanism
- No PII anonymization cron
- Session creation not audit-logged
- Notification sends not audit-logged
- Dashboard access not audit-logged

### 11. Reporting Gaps
- No PDF/HTML export
- No date range filter
- No 6-month revenue trend
- No per-studio breakdown
- No anomaly flag detection

### 12. Security Headers
- Missing `Content-Security-Policy` header
- Missing `Strict-Transport-Security` header
- No MFA implementation

---

## MEDIUM PRIORITY GAPS

### 13. Concurrency Control
- No optimistic locking for concurrent schedule saves
- Race condition possible on slot booking

### 14. GPS Precision (Entity)
- `Attendance.gps_lat/gps_lng` use `Float` instead of `Numeric(10,7)`

### 15. Session Ownership Validation
- Teacher could submit attendance for another teacher's session

### 16. Operational Hours
- No validation against studio operating hours

### 17. Admin Review SLA
- No 24h reminder for pending attendance reviews

### 18. WhatsApp Rate Limiting
- No throttling (max 100 msg/sec per SRS)

### 19. Payment Gateway Circuit Breaker
- WhatsApp/Email have circuit breakers, payment gateway does not

### 20. Draft Conflict Resolution
- No `updated_at` comparison for concurrent edits

---

## What Works Well (DONE)

1. JWT HS256 with per-role timeout
2. Brute force lockout (5 attempts / 15 min)
3. bcrypt password hashing (8 char minimum)
4. SQLAlchemy ORM (SQL injection safe)
5. Rate limiting (100/min global)
6. Notification retry + circuit breaker (WhatsApp + Email)
7. Audit log model with full structure
8. UU PDP data deletion flow (Pasal 19)
9. Eligibility check (90% attendance, 1 no-show max)
10. All 11 SRS entities have matching models
11. Draft auto-save with 7-day TTL
12. Multi-student guardian relationship
13. Session conflict validation

---

## Recommended Priority Order

1. **PII Encryption** — legal compliance (UU PDP)
2. **Cron Jobs** — billing won't work without auto-invoicing
3. **Onboarding Flow** — admin can't create accounts properly
4. **Frozen Login Block** — security hole
5. **Studio Reservation** — major feature gap
6. **Audit Log completeness** — compliance requirement
7. **Notification grouping** — UX for multi-student guardians
8. **Security headers (CSP, HSTS)** — OWASP compliance
9. **Concurrency control** — data integrity
10. **Reporting exports** — owner needs printable reports
