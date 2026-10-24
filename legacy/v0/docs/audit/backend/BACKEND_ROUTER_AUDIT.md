# Backend Router Audit — T-178

**Date:** 2026-07-05
**Scope:** All 25 router files in backend/app/routers/

---

## SUMMARY

| Router | Prefix | Endpoints | Auth | Audit Log | Issues |
|--------|--------|-----------|------|-----------|--------|
| admin_mgmt | /api/v1/admin-mgmt | 6 | 7 ✅ | 6 ✅ | — |
| attendance | /api/v1/attendance | 3 | 4 ✅ | 3 ✅ | — |
| audit | /api/v1/audit | 1 | 2 ✅ | 0 | Read-only OK |
| auth | /api/v1/auth | 10 | 6 ⚠️ | 0 | Public endpoints expected |
| billing | /api/v1/billing | 5 | 6 ✅ | 5 ✅ | — |
| cron_trigger | MISSING | 4 | 4 ✅ | 0 | ⚠️ No prefix in router |
| draft | /api/v1/drafts | 4 | 5 ✅ | 0 | ⚠️ No audit log |
| feedback | /api/v1/feedback | 2 | 3 ✅ | 2 ✅ | — |
| holidays | /api/v1/holidays | 3 | 4 ✅ | 0 | ⚠️ No audit log |
| import_data | /api/v1/admin/import | 2 | 3 ✅ | 2 ✅ | — |
| multi_student | /api/v1/guardian | 3 | 4 ✅ | 0 | ⚠️ No audit log |
| my_portal | /api/v1/my | 6 | 6 ✅ | 0 | Read-only OK |
| notification | /api/v1/notifications | 7 | 8 ✅ | 4 ✅ | — |
| override | /api/v1/override | 3 | 4 ✅ | 4 ✅ | — |
| payments | /api/v1/payments | 4 | 5 ✅ | 4 ✅ | — |
| photos | MAIN.PY PREFIX | 5 | 5 ✅ | 0 | ⚠️ Prefix in main.py |
| pricing | /api/v1/pricing | 4 | 5 ✅ | 3 ✅ | — |
| privacy | /api/v1/privacy | 3 | 4 ✅ | 0 | ⚠️ No audit log |
| ratings | /api/v1/ratings | 2 | 3 ✅ | 2 ✅ | — |
| registrations | /api/v1/registrations | 6 | 5 ⚠️ | 3 ⚠️ | 2 public endpoints OK |
| reports | /api/v1/reports | 7 | 8 ✅ | 4 ✅ | — |
| scheduling | /api/v1/sessions | 9 | 10 ✅ | 6 ✅ | — |
| settings | /api/v1/settings | 3 | 4 ✅ | 2 ✅ | — |
| students | /api/v1/students | 5 | 6 ✅ | 5 ✅ | — |
| teachers | /api/v1/teachers | 5 | 6 ✅ | 0 | ⚠️ No audit log |

**TOTALS:** 25 routers, 109 endpoints, 107 with auth (98%), 60 with audit log (55%)

---

## CRITICAL FINDINGS

### 1. Missing Audit Log on Mutations — ⚠️

Routers with mutations but NO audit log:
- **draft.py** — POST/DELETE endpoints, no audit
- **holidays.py** — POST/DELETE endpoints, no audit
- **multi_student.py** — POST freeze endpoint, no audit
- **privacy.py** — POST/PUT endpoints, no audit
- **teachers.py** — POST/PUT/DELETE endpoints, no audit

**Impact:** Can't track who created/deleted records
**Fix:** Add create_audit_log() to all mutation endpoints

### 2. Cron Trigger Missing Prefix — ⚠️

**cron_trigger.py** has no prefix in router definition.
Prefix is set in main.py: `app.include_router(cron_trigger.router, prefix="/api/admin/cron")`

**Impact:** Inconsistent with other routers
**Fix:** Move prefix to router definition

### 3. Photos Prefix in main.py — ⚠️

**photos.py** has no prefix in router definition.
Prefix is set in main.py: `app.include_router(photos.router, prefix="/api/v1/photos")`

**Impact:** Inconsistent with other routers
**Fix:** Move prefix to router definition

---

## AUTH GUARD ANALYSIS

### Public Endpoints (No Auth) — ✅ EXPECTED

| Router | Endpoint | Reason |
|--------|----------|--------|
| auth | POST /login | Public |
| auth | POST /register | Public |
| auth | POST /forgot-password | Public |
| auth | POST /refresh | Uses token |
| registrations | POST / | Public registration |
| registrations | GET /available-schedules | Public |

### Role Distribution

| Role | Endpoints |
|------|-----------|
| admin | 45 |
| owner | 35 |
| teacher | 12 |
| guardian | 15 |
| Public | 6 |

---

## RESPONSE FORMAT ANALYSIS

### Consistent Format (data/total/limit/offset):
- students ✅
- teachers ✅
- sessions/rooms ✅
- sessions/teachers ✅
- billing/invoices ✅
- pricing ✅
- holidays ✅
- notifications ✅
- admin-mgmt ✅

### Inconsistent Format:
- auth/me — returns single object
- auth/me/profile — returns single object
- reports/* — returns custom objects
- override/* — returns custom objects

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Add audit log to:** draft, holidays, multi_student, privacy, teachers
2. **Move prefix to router:** cron_trigger, photos

### P2 — MEDIUM
3. **Standardize response format:** All list endpoints should use {data, total, limit, offset}
4. **Add pagination to:** my_portal endpoints (schedule, children, payments, students, attendance, invoices)
