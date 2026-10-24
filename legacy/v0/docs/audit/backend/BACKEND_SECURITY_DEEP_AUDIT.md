# Backend Security Deep Audit — T-181

**Date:** 2026-07-05
**Scope:** Auth flow, JWT, encryption, rate limiting, CORS

---

## 1. AUTHENTICATION FLOW

### 1.1 Login Flow — ✅ SECURE

```
1. User submits username + password
2. Check account lock (locked_until)
3. Verify password (bcrypt)
4. If failed:
 - Increment failed_login_attempts
 - Lock after 5 failures (30 min)
 - Log LOGIN_FAILED
5. If success:
 - Reset failed_login_attempts
 - Create access_token (JWT HS256)
 - Create refresh_token (JWT HS256)
 - Store refresh_token in DB
 - Log LOGIN_SUCCESS
6. Check first_login / temp_password
 - If true → force password change
```

**Status:** ✅ Secure

### 1.2 Token Refresh Flow — ✅ SECURE

```
1. Client sends refresh_token
2. Verify refresh_token (JWT decode)
3. Check token in DB (not blacklisted)
4. Create new access_token
5. Create new refresh_token (rotation)
6. Update DB with new refresh_token
7. Return new tokens
```

**Status:** ✅ Secure (token rotation prevents replay)

### 1.3 Logout Flow — ✅ SECURE

```
1. Client sends logout request
2. Blacklist access_token (Redis)
3. Blacklist refresh_token (Redis)
4. Clear refresh_token in DB
```

**Status:** ✅ Secure

---

## 2. JWT CONFIGURATION

### 2.1 Token Expiry — ✅ CORRECT

| Role | Access Token | Refresh Token |
|------|-------------|---------------|
| owner | 30 min | 7 days |
| admin | 30 min | 7 days |
| teacher | 30 min | 7 days |
| guardian | 60 min | 14 days |

**Status:** ✅ Matches SRS (owner/admin shorter than guardian)

### 2.2 Algorithm — ✅ SECURE

- Algorithm: HS256
- Secret: From environment variable (SECRET_KEY)
- Validation: ValueError if DEBUG=False and default key

**Status:** ✅ Secure

---

## 3. PASSWORD SECURITY

### 3.1 Hashing — ✅ SECURE

- Algorithm: bcrypt
- Salt: Auto-generated per password
- Cost: Default (12 rounds)

**Status:** ✅ Secure

### 3.2 Brute Force Protection — ✅ IMPLEMENTED

- Max attempts: 5
- Lock duration: 30 minutes
- Reset on success: Yes
- Log on lock: ACCOUNT_LOCKED

**Status:** ✅ Secure

---

## 4. RATE LIMITING

### 4.1 Auth Endpoints — ✅ IMPLEMENTED

| Endpoint | Limit | Status |
|----------|-------|--------|
| POST /login | 5/minute | ✅ |
| POST /register | 3/minute | ✅ |
| POST /forgot-password | 5/minute | ✅ |
| POST /change-password | 10/minute | ✅ |
| POST /refresh | 30/minute | ✅ |
| GET /me | 100/minute | ✅ |
| PATCH /me/profile | 20/minute | ✅ |
| POST /first-login-change-password | 10/minute | ✅ |

### 4.2 Other Endpoints — ❌ NO RATE LIMITING

- Students, Teachers, Sessions, Billing — no rate limiting
- Impact: Potential abuse on CRUD operations

---

## 5. PII ENCRYPTION

### 5.1 Encrypted Fields — ✅ IMPLEMENTED

| Model | Fields | Algorithm |
|-------|--------|-----------|
| User | email | AES-256-GCM |
| Guardian | full_name, phone_whatsapp, email, address | AES-256-GCM |
| Student | full_name | AES-256-GCM |
| Attendance | photo_url | AES-256-GCM |
| Registration | phone_whatsapp, email | AES-256-GCM |

### 5.2 Blind Index — ✅ IMPLEMENTED

- User.email_hash → SHA-256
- Guardian.phone_hash → SHA-256

### 5.3 Key Management — ⚠️ NEED REVIEW

- Key from environment variable
- No key rotation mechanism
- No key backup/recovery

---

## 6. CORS CONFIGURATION

### 6.1 Settings — ✅ CONFIGURED

```python
CORSMiddleware,
allow_origins=settings.CORS_ORIGINS,
allow_methods=["*"],
```

**Status:** ⚠️ Need to verify CORS_ORIGINS in production

---

## 7. INPUT VALIDATION

### 7.1 Pydantic Schemas — ✅ IMPLEMENTED

All request bodies use Pydantic models with:
- Type validation
- Min/max length
- Pattern matching
- Enum validation

### 7.2 SQL Injection — ✅ PROTECTED

- SQLAlchemy ORM with parameterized queries
- No raw SQL string concatenation

---

## 8. AUDIT LOGGING

### 8.1 Auth Events — ✅ LOGGED

| Event | Logged |
|-------|--------|
| LOGIN_SUCCESS | ✅ |
| LOGIN_FAILED | ✅ |
| ACCOUNT_LOCKED | ✅ |
| LOGOUT | ✅ |
| REGISTER | ✅ |
| PASSWORD_CHANGE | ✅ |
| FORGOT_PASSWORD | ✅ |
| PASSWORD_RESET | ✅ |

---

## CRITICAL FINDINGS

### 1. Rate Limiting Missing on CRUD — ⚠️

**Issue:** No rate limiting on students, teachers, sessions, billing endpoints
**Impact:** Potential abuse (spam create/delete)
**Fix:** Add rate limiting to all write endpoints

### 2. Key Rotation Missing — ⚠️

**Issue:** No mechanism to rotate encryption keys
**Impact:** If key compromised, all PII exposed
**Fix:** Implement key rotation with versioned keys

### 3. CORS Origins Not Verified — ⚠️

**Issue:** Need to verify CORS_ORIGINS in production
**Impact:** If wildcard, any origin can access API
**Fix:** Set specific origins in production

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Add rate limiting to write endpoints:** POST/PUT/DELETE on students, teachers, sessions
2. **Verify CORS_ORIGINS:** Set specific origins in production

### P2 — MEDIUM
3. **Implement key rotation:** Versioned encryption keys
4. **Add 2FA option:** For owner/admin accounts
5. **Add session management:** View active sessions, force logout
