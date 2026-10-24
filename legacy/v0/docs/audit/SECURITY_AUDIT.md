# Security Audit — Authentication & Authorization

**Date:** 2026-07-05
**Scope:** SEC-01..SEC-08 compliance, auth, encryption, rate limiting

---

## 1. AUTHENTICATION

### 1.1 JWT Token — ✅ IMPLEMENTED
- Login returns access_token + refresh_token
- Token stored in localStorage (frontend)
- Auth middleware validates token on protected routes

### 1.2 Role-Based Access Control — ✅ IMPLEMENTED
- `require_role()` dependency on all protected endpoints
- Roles: owner, admin, teacher, guardian
- Each role has specific endpoint access

### 1.3 Public Endpoints (No Auth Required) — ✅ CORRECT
- POST /api/v1/auth/login
- POST /api/v1/auth/register
- POST /api/v1/auth/forgot-password
- POST /api/v1/auth/refresh
- POST /api/v1/registrations/ (public registration)
- GET /api/v1/registrations/available-schedules (public)

### 1.4 Protected Endpoints — ✅ CORRECT
- All /api/v1/students/* → require_role("admin", "owner", "guardian")
- All /api/v1/teachers/* → require_role("admin", "owner")
- All /api/v1/sessions/* → require_role("admin", "owner", "teacher", "guardian")
- All /api/v1/billing/* → require_role("admin", "owner")
- All /api/v1/reports/* → require_role("admin", "owner", "teacher", "guardian")

---

## 2. PII ENCRYPTION (AES-256-GCM)

### 2.1 Encrypted Fields — ✅ IMPLEMENTED

| Model | Field | Encrypted |
|-------|-------|-----------|
| User | email | ✅ AES-256-GCM |
| Guardian | full_name | ✅ AES-256-GCM |
| Guardian | phone_whatsapp | ✅ AES-256-GCM |
| Guardian | email | ✅ AES-256-GCM |
| Guardian | address | ✅ AES-256-GCM |
| Student | full_name | ✅ AES-256-GCM |
| Attendance | photo_url | ✅ AES-256-GCM |
| Registration | phone_whatsapp | ✅ AES-256-GCM |
| Registration | email | ✅ AES-256-GCM |

### 2.2 Blind Index — ✅ IMPLEMENTED
- User.email_hash — SHA-256 for email lookup
- Guardian.phone_hash — SHA-256 for phone lookup

---

## 3. RATE LIMITING

### 3.1 Auth Endpoints — ✅ IMPLEMENTED

| Endpoint | Limit |
|----------|-------|
| POST /login | 5/minute |
| POST /register | 3/minute |
| POST /forgot-password | 5/minute |
| POST /change-password | 10/minute |
| POST /refresh | 30/minute |
| GET /me | 100/minute |
| PATCH /me/profile | 20/minute |

### 3.2 Other Endpoints — ⚠️ NO RATE LIMITING
- Students, Teachers, Sessions, Billing — no rate limiting
- Impact: Potential abuse on CRUD operations
- Recommendation: Add rate limiting on write endpoints

---

## 4. INPUT VALIDATION

### 4.1 Pydantic Schemas — ✅ IMPLEMENTED
- All request bodies use Pydantic models
- Field validation (min_length, max_length, pattern)
- Type validation (UUID, date, time, enum)

### 4.2 SQL Injection — ✅ PROTECTED
- SQLAlchemy ORM with parameterized queries
- No raw SQL string concatenation

### 4.3 XSS — ✅ PROTECTED
- React auto-escapes JSX content
- No dangerouslySetInnerHTML usage found

---

## 5. AUDIT LOGGING

### 5.1 Audit Log — ✅ IMPLEMENTED
- All mutations logged (INSERT, UPDATE, DELETE)
- Before/after state captured
- User ID and timestamp recorded
- Immutable (no update/delete on audit_log table)

---

## 6. FINDINGS SUMMARY

| Category | Status | Issues |
|----------|--------|--------|
| Authentication | ✅ | — |
| Authorization | ✅ | — |
| PII Encryption | ✅ | — |
| Rate Limiting | ⚠️ | Only auth endpoints |
| Input Validation | ✅ | — |
| SQL Injection | ✅ | — |
| XSS | ✅ | — |
| Audit Logging | ✅ | — |

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Rate limiting on write endpoints:** Add to students, teachers, sessions, billing POST/PUT/DELETE

### P2 — MEDIUM
2. **CORS configuration:** Verify allowed origins in production
3. **HTTPS enforcement:** Ensure all production traffic uses HTTPS
4. **Session timeout:** Verify JWT expiry matches SRS (30 min owner/admin, 60 min guardian)
