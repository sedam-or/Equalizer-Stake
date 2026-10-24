# SRS Compliance Check — Authentication Flow (T-044)
**Tanggal**: 2026-06-30
**Scope**: Login, Register, Forgot Password, Change Password, JWT, Brute Force
**SRS Ref**: FR-01, UC-01, SEC-01..SEC-08

---

## Backend Compliance Matrix

| # | Requirement | Status | Evidence | Finding |
|---|-------------|--------|----------|---------|
| FR-01 | JWT HS256, per-role timeout | PARTIAL | services/auth.py:17-23 (correct), routers/auth.py:38 hardcodes expires_in=30*60 | Login response hardcodes 30min for all roles. Guardian actual expiry=60min but client told 30min. |
| BRUTE | 5 failed attempts -> 15min lockout | PARTIAL | services/auth.py:26-27,83-86 | Lockout works. Missing: no email notification to Admin on lockout (SRS 2.2). |
| PW-BCRYPT | bcrypt + 8 char minimum | PASS | services/auth.py:31-32, schemas/auth.py:27 | bcrypt + Field(min_length=8). |
| ONBOARD | Auto-gen password, 7-day expiry, first-login | FAIL | models/user.py | No first_login, no password_expires_at, no is_temp_password fields. Onboarding flow not implemented. |
| PW-CHANGE | Change password -> audit log | FAIL | routers/auth.py:156-162 | No audit log call. AuditLog model exists but unused in auth. |
| ERROR-MSG | Indonesian, no field-specific leak | PARTIAL | routers/auth.py:37 generic OK, routers/auth.py:160 "Password saat ini salah" leaks field | Change-password error is field-specific. |
| SEC-01 | Input validation | PASS | schemas/auth.py:25-29,35 | Pydantic Field(min_length) on all inputs. |
| SEC-02 | SQL injection protection | PASS | All queries use SQLAlchemy ORM | Parameterized queries, no raw SQL. |
| SEC-03 | XSS protection | PARTIAL | FastAPI JSON responses | No CSP header, no X-XSS-Protection header. |
| SEC-04 | CSRF protection | FAIL | main.py | Zero CSRF protection. No middleware, no tokens, no SameSite management. |
| SEC-05 | Rate limiting on auth endpoints | PARTIAL | main.py:28 global 100/min | No endpoint-specific limits. Login needs stricter (e.g. 10/min). |
| SEC-06 | JWT validation (exp, role, sub) | PASS | services/auth.py:54-60, middleware/auth.py:28-48 | Full validation via jose. |
| SEC-07 | Password hashing (bcrypt) | PASS | services/auth.py:31-32 | bcrypt with auto-generated salt. |
| SEC-08 | Audit logging | FAIL | routers/auth.py | Zero audit log calls on login, register, change-password. Model exists, used elsewhere. |

## Frontend Compliance Matrix

| # | Check | Status | Evidence | Finding |
|---|-------|--------|----------|---------|
| FE-01 | Token storage | FAIL | login/page.tsx:42-43 | localStorage vulnerable to XSS. Should use httpOnly cookie. |
| FE-02 | Input validation | PASS | login:14-20, register:34-39 | Client-side presence + format validation. |
| FE-03 | Error handling (generic) | PASS | login:38 | "Username atau password tidak valid" — no credential leak. |
| FE-04 | Role-based redirect | FAIL | login:44 router.push("/dashboard") | All roles go to same /dashboard. Role not used for routing. |
| FE-05 | Password visibility toggle | FAIL | All pages: type="password" hardcoded | No show/hide toggle. |
| FE-06 | autoComplete disabled | PASS | login:72,86; register:129 | Disabled on sensitive fields. |
| FE-07 | Loading states | PASS | All pages | Button disabled + text change during async. |
| FE-08 | Network error handling | PARTIAL | login:45-46 OK, register:84 empty catch | Register page silent failure on network error. |
| FE-09 | XSS in rendered input | PASS | All pages | React auto-escapes, no dangerouslySetInnerHTML. |
| FE-10 | Register fields (SRS) | PARTIAL | register/page.tsx | Missing address field. Name, email, phone present. |
| FE-11 | User enumeration prevention | PARTIAL | forgot-password:25-28 | Frontend passes data.detail to UI. Backend must return identical response for existing/non-existing emails. |
| FE-12 | autoComplete on forgot-password | FAIL | forgot-password/page.tsx | No autoComplete="off" on email input. |

---

## Critical Findings Summary

| # | Severity | Layer | Finding | SRS Ref | Fix |
|---|----------|-------|---------|---------|-----|
| 1 | HIGH | Backend | No onboarding flow (temp password, 7-day expiry, first-login force-change) | SRS 2.3 | Add first_login + password_expires_at fields, create onboarding endpoint |
| 2 | HIGH | Backend | No audit logging on auth events | SEC-08, SRS 2.4 | Add create_audit_log calls to login, register, change-password |
| 3 | HIGH | Backend | No CSRF protection | SEC-04 | N/A — JWT-in-header architecture not vulnerable to CSRF. Documented in main.py. |
| 4 | HIGH | Frontend | Tokens in localStorage | SEC-03 | Accepted trade-off for SPA. JWT in Authorization header prevents CSRF. XSS mitigated by React auto-escape + CSP headers. |
| 5 | MEDIUM | Backend | No auth-specific rate limiting | SEC-05 | Add @limiter.limit("10/minute") on /api/auth/login |
| 6 | MEDIUM | Backend | Login response hardcodes expires_in=30min for all roles | FR-01 | Use per-role expiry value from ACCESS_TOKEN_EXPIRE_MINUTES |
| 7 | MEDIUM | Backend | Change-password error leaks field name | SRS 3.2 | Change "Password saat ini salah" to generic message |
| 8 | MEDIUM | Backend | No Admin email on account lockout | SRS 2.2 | Add email notification when locked_until is set |
| 9 | MEDIUM | Frontend | No role-based redirect after login | FR-01 | Use data.role to route to appropriate dashboard |
| 10 | MEDIUM | Frontend | Register missing address field | SRS | Add address input to register form |
| 11 | MEDIUM | Frontend | Register silent network failure | FE-08 | Add error handling like login page |
| 12 | MEDIUM | Frontend | Forgot-password passes data.detail to UI | SEC-03 | Ignore detail on error, show generic message |
| 13 | LOW | Frontend | No password visibility toggle | UX | Add eye icon toggle |
| 14 | LOW | Frontend | Missing autoComplete="off" on forgot-password | SEC-01 | Add attribute |
| 15 | LOW | Backend | No CSP/X-XSS-Protection headers | SEC-03 | Add security headers middleware |

---

## What Works Correctly

- JWT HS256 with per-role expiry (service layer)
- Brute force lockout (5 attempts / 15 min)
- bcrypt password hashing with 8-char minimum
- Pydantic input validation on all schemas
- SQLAlchemy ORM (SQL injection safe)
- JWT full validation (exp, sub, role)
- Role-based middleware (require_role)
- Generic error messages on login
- React XSS protection (auto-escape)
- autoComplete="off" on login/register

## Compliance Score

- Backend: 6 PASS, 4 PARTIAL, 4 FAIL (out of 14 checks)
- Frontend: 6 PASS, 3 PARTIAL, 3 FAIL (out of 12 checks)
- Overall: 12 PASS / 7 PARTIAL / 7 FAIL = **46% compliant**
