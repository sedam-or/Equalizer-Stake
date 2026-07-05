# Backend Service Audit — T-180

**Date:** 2026-07-05
**Scope:** All 24 service files in backend/app/services/

---

## SERVICE SUMMARY

| Service | Functions | Purpose | Status |
|---------|-----------|---------|--------|
| admin_mgmt | 3 | Admin CRUD operations | ✅ |
| attendance | 5 | GPS validation, submit, review | ✅ |
| audit | 3 | Create audit log entries | ✅ |
| audit_retention | 2 | 7-year retention enforcement | ✅ |
| auth | 5 | Password hash, verify, JWT | ✅ |
| background_queue | 3 | Async notification queue | ✅ |
| billing | 4 | Invoice, unique_code, fine, verify | ✅ |
| draft | 3 | Auto-save form drafts | ✅ |
| eligibility | 1 | Exam eligibility check | ✅ |
| email | 3 | Email sending (stub) | ⚠️ |
| encryption | 4 | AES-256-GCM, blind index | ✅ |
| import_service | 3 | CSV data import | ✅ |
| jwt_blacklist | 2 | Token blacklist (Redis) | ✅ |
| multi_student | 3 | Guardian freeze/unfreeze | ✅ |
| notification | 8 | Create, send, retry, circuit breaker | ✅ |
| override | 2 | Owner override operations | ✅ |
| photo_storage | 4 | File upload, delete | ✅ |
| pricing | 4 | Pricing CRUD | ✅ |
| privacy | 3 | Data deletion requests | ✅ |
| reports | 4 | Dashboard stats | ✅ |
| scheduler | 6 | Cron jobs (5 jobs) | ✅ |
| scheduling | 10 | Session management | ✅ |
| settings | 3 | System settings CRUD | ✅ |
| whatsapp | 3 | WhatsApp sending (stub) | ⚠️ |

**TOTALS:** 24 services, 91 functions

---

## CRON JOBS (scheduler.py)

| Job | Schedule | Purpose | Status |
|-----|----------|---------|--------|
| job_auto_invoice | 5th of month | Create invoices per guardian | ✅ |
| job_fine_freeze | 11th of month | Apply fine Rp100k, freeze account | ✅ |
| job_draft_cleanup | Daily 3 AM | Delete old drafts | ✅ |
| job_session_reminder | Every hour | Remind upcoming sessions | ✅ |
| job_database_backup | Daily 2 AM | PostgreSQL dump | ✅ |

---

## BUSINESS LOGIC AUDIT

### 1. Billing Logic — ✅ CORRECT

**Auto-invoice (5th of month):**
- Get all active guardians
- For each guardian, get active students
- Sum pricing per student (instrument + level + class_type)
- Create invoice with unique_code (3 digit)
- Send notification

**Fine + Freeze (11th of month):**
- Get overdue invoices (status=issued, due_date < today)
- Apply fine Rp100.000
- Freeze guardian account
- Send notification

**Payment verification:**
- Match unique_code
- Update invoice status to "paid"
- Create payment record
- Send notification

### 2. Scheduling Logic — ✅ CORRECT

**Slot validation:**
- Check room availability (no conflict)
- Check teacher availability (no conflict)
- Check student availability (no conflict)
- Return conflicts + alternative slots

**Session types:**
- reguler, teori, rehearsal, recording, kompensasi

### 3. Attendance Logic — ✅ CORRECT

**GPS validation:**
- Haversine distance calculation
- 150m radius from BMD location
- Server timestamp validation

**Submit flow:**
- Validate GPS within radius
- Validate session ownership
- Upload photo
- Create attendance record
- Send notification to guardian

**Review flow:**
- Teacher reviews attendance
- Verify/reject with reason
- Update verification_status

### 4. Notification Logic — ✅ CORRECT

**Circuit breaker:**
- 3 failures → open circuit
- 10 minute cooldown
- Per channel (WhatsApp, email)

**Retry logic:**
- 3 retries with exponential backoff (10/20/40s)
- Group by guardian

**Channels:**
- WhatsApp (primary)
- Email (secondary)
- SMS (tertiary)

### 5. Encryption Logic — ✅ CORRECT

**AES-256-GCM:**
- Encrypt PII fields
- Decrypt on read
- Key from environment variable

**Blind index:**
- SHA-256 hash for searchable fields
- email_hash, phone_hash

---

## CRITICAL FINDINGS

### 1. WhatsApp Service — ⚠️ STUB

**Status:** Code exists but no real API integration
**Impact:** Notifications won't be sent via WhatsApp
**Fix:** Integrate with WhatsApp Business API

### 2. Email Service — ⚠️ STUB

**Status:** Code exists but no real SMTP configuration
**Impact:** Email notifications won't be sent
**Fix:** Configure SMTP settings

### 3. Missing Input Validation — ⚠️

**Services without input validation:**
- pricing.py — no validation on price values
- settings.py — no validation on setting values
- draft.py — no validation on form data

**Impact:** Invalid data can be inserted
**Fix:** Add Pydantic validation in routers

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Integrate WhatsApp API:** Connect to real WhatsApp Business API
2. **Configure SMTP:** Set up email sending
3. **Add input validation to:** pricing, settings, draft

### P2 — MEDIUM
4. **Add retry logic to:** billing cron jobs (in case of DB errors)
5. **Add monitoring to:** scheduler jobs (success/failure logging)
6. **Add rate limiting to:** notification service (prevent spam)
