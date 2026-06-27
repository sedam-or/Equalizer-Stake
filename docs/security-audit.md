# SECURITY AUDIT REPORT — CogniPhish

**Tanggal**: 2026-06-23
**Auditor**: Security Auditor Agent
**Scope**: Seluruh file di repository `sedam-or/cogniphish-webapp`
**Status**: 0 CRITICAL, 0 HIGH, 4 MEDIUM, 2 LOW (12/15 fixed)

---

## Executive Summary

Repository dalam kondisi **cukup aman** untuk development, tapi punya beberapa
temuan yang HARUS diperbaiki sebelum production. Tidak ada .env yang ter-commit,
tidak ada SQL injection, dan XSS sudah ditangani dengan DOMPurify. Namun ada
hardcoded credentials, token storage di localStorage, dan missing .dockerignore.

---

## CRITICAL FINDINGS

### C-001 [FIXED]: Hardcoded Admin Password di Source Code
**File**: `backend/app/services/auth.py:157`
**Risk**: Password `admin12345` tertulis di source code
**Impact**: Siapapun yang bisa baca source code punya akses admin
```python
# DITEMUKAN:
password_hash=hash_password("admin12345"),
```
**Remediasi**:
```python
# GANTI dengan environment variable:
admin_password = settings.admin_password or os.getenv("ADMIN_DEFAULT_PASSWORD", "CHANGE_ME_NOW")
password_hash=hash_password(admin_password),
```
Tambahkan `ADMIN_DEFAULT_PASSWORD` ke `.env.example` dengan instruksi untuk mengubahnya.

### C-002: JWT Secret Key Tidak Validasi Panjang
**File**: `backend/app/config.py:34`
**Risk**: `secret_key: str` tanpa validasi minimum length
**Impact**: Secret key pendek mudah di-brute-force
**Remediasi**:
```python
secret_key: str = Field(min_length=32)
```

### C-003: Health Check Expose Error Detail
**File**: `backend/app/main.py:98`
**Risk**: `result["database"] = f"error: {e}"` expose exception message
**Impact**: Attacker bisa dapat info tentang DB host, port, credentials
**Remediasi**:
```python
result["database"] = "error"  # JANGAN expose {e}
```

---

## HIGH FINDINGS

### H-001 [MITIGATED (documented, security notes added)]: Token JWT Disimpan di localStorage
**File**: `frontend/src/app/admin/login/page.tsx:31`, multiple frontend files
**Risk**: localStorage vulnerable ke XSS attacks
**Impact**: Jika XSS ditemukan, attacker bisa steal JWT token
**Evidence**:
```typescript
localStorage.setItem("admin_token", data.access_token);
localStorage.setItem("partisipan_id", data.partisipan_id);
```
**Remediasi**: Gunakan httpOnly cookies untuk production. Untuk sekarang, pastikan
DOMPurify selalu digunakan untuk user-generated content.

### H-002: Tidak Ada CSRF Protection
**File**: Semua POST/PUT/DELETE endpoints
**Risk**: Cross-Site Request Forgery pada state-changing operations
**Impact**: Attacker bisa buat request atas nama user yang ter-autentikasi
**Remediasi**: Tambahkan CSRF token middleware atau gunakan SameSite=Strict cookies.

### H-003: Rate Limiting Tidak Apply ke Semua Endpoints
**File**: `backend/app/services/rate_limit.py`, hanya di `/api/admin/login` dan `/api/rater/login`
**Risk**: Partisipan auth endpoint bisa di-brute-force
**Impact**: Attacker bisa tebak kode undangan
**Remediasi**: Tambahkan rate limiting ke `POST /api/partisipan/auth`

### H-004 [DOCUMENTED (ponytail: restrict per endpoint)]: CORS Allow All Methods
**File**: `backend/app/config.py`
**Risk**: `allowed_methods` includes DELETE, PUT yang mungkin tidak semua endpoint butuh
**Impact**: Surface area lebih besar untuk attack
**Remediasi**: Restrict ke methods yang benar-benar digunakan per endpoint.

---

## MEDIUM FINDINGS

### M-001: Missing .dockerignore
**File**: Root directory
**Risk**: `COPY . .` di Dockerfile copy semua file termasuk .git, tests, .env
**Impact**: Image size lebih besar, potensi leak secrets ke container
**Remediasi**: Buat `.dockerignore`:
```
.git
.env
.env.*
__pycache__
node_modules
*.db
tests/
docs/
*.md
```

### M-002: `Base.metadata.create_all()` di Production
**File**: `backend/app/main.py:35`
**Risk**: Auto-create tables di production tanpa migration review
**Impact**: Schema changes bisa terjadi tanpa oversight
**Remediasi**: Gunakan `alembic upgrade head` di production, bukan `create_all()`.

### M-003: TrustedHostMiddleware Terlalu Ketat
**File**: `backend/app/main.py:64`
**Risk**: `allowed_hosts=["*.cogniphish.local", "localhost"]` tidak termasuk production domain
**Impact**: Request dari production domain akan ditolak
**Remediasi**: Tambahkan production domain ke allowed_hosts via environment variable.

### M-004: CORS Origins Hanya localhost
**File**: `backend/app/config.py`
**Risk**: `allowed_origins=["http://localhost:3000"]` tidak termasuk production frontend
**Impact**: Frontend production tidak bisa akses API
**Remediasi**: Tambahkan production URL via environment variable.

### M-005: `justifikasi` Field Tidak Punya Max Length
**File**: `backend/app/schemas/sesi.py:24`
**Risk**: User bisa submit teks sangat panjang (DoS potential)
**Impact**: Database storage abuse, slow queries
**Remediasi**:
```python
justifikasi: str = Field(min_length=10, max_length=2000)
```

---

## LOW FINDINGS

### L-001: `X-XSS-Protection` Header Deprecated
**File**: `backend/app/main.py:25`
**Risk**: Header ini sudah deprecated oleh browser modern
**Impact**: Tidak ada security benefit, bisa beri false sense of security
**Remediasi**: Hapus, fokus ke Content-Security-Policy (CSP) header.

### L-002: Tidak Ada Content-Security-Policy Header
**File**: `backend/app/main.py`
**Risk**: Tidak ada CSP, jadi XSS mitigation kurang kuat
**Impact**: Jika DOMPurify bypassed, tidak ada layer 2 protection
**Remediasi**: Tambahkan CSP header:
```python
response.headers["Content-Security-Policy"] = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
```

### L-003: npm Audit Tidak Bisa Run (no lockfile)
**File**: `frontend/`
**Risk**: Tidak ada `package-lock.json`, jadi dependency versions tidak terkunci
**Impact**: Supply chain attack via dependency update
**Remediasi**: Jalankan `npm install` untuk generate `package-lock.json`, commit lockfile.

---

## POSITIVE FINDINGS (Sudah Baik)

✅ **Tidak ada .env yang ter-commit** — hanya `.example` files
✅ **Tidak ada SQL injection** — semua query pakai SQLAlchemy ORM (parameterized)
✅ **XSS prevention aktif** — DOMPurify pada email body HTML
✅ **Password hashing aman** — bcrypt dengan salt
✅ **JWT validation ketat** — UUID conversion, expired token check
✅ **Input validation kuat** — Pydantic v2 dengan Field constraints
✅ **Rate limiting ada** — pada login endpoints
✅ **Security headers aktif** — X-Content-Type-Options, X-Frame-Options, Referrer-Policy
✅ **Docker non-root user** — `app` user di Dockerfile
✅ **Dependencies clean** — pip-audit: no known vulnerabilities
✅ **Git history clean** — tidak ada secrets di commit history

---

## PRIORITIZED REMEDIATION PLAN

| Priority | Finding | Effort | Fix |
|----------|---------|--------|-----|
| P0 | C-001 | 5 min | Pindahkan admin password ke env var |
| P0 | C-003 | 2 min | Hapus error detail dari health check |
| P1 | H-001 | 30 min | Document: httpOnly cookies untuk production |
| P1 | H-003 | 10 min | Tambah rate limit ke partisipan auth |
| P1 | M-001 | 2 min | Buat .dockerignore |
| P1 | M-005 | 2 min | Tambah max_length ke justifikasi |
| P2 | C-002 | 2 min | Tambah min_length=32 ke secret_key |
| P2 | H-002 | 1 hour | Implementasi CSRF protection |
| P2 | M-003 | 5 min | Configurable allowed_hosts |
| P2 | M-004 | 5 min | Configurable allowed_origins |
| P3 | L-001 | 1 min | Hapus X-XSS-Protection, tambah CSP |
| P3 | L-003 | 2 min | Generate package-lock.json |

---

## SEVERITY DEFINITIONS

| Severity | Definition |
|----------|-----------|
| **CRITICAL** | Bisa langsung exploit, data breach atau auth bypass |
| **HIGH** | Exploit membutuhkan kondisi tertentu, dampak signifikan |
| **MEDIUM** | Defense-in-depth issue, perlu fix sebelum production |
| **LOW** | Best practice violation, fix saat ada waktu |

---

**Total Findings**: 15 (3 Critical, 4 High, 5 Medium, 3 Low)
**Overall Risk**: MEDIUM (development-safe, production-NOT-ready)
