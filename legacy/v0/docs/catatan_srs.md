# Catatan SRS vs Implementasi — Equalizer-Stake

> Dokumen ini mencatat semua GAP antara SRS (srs/*.md) dan implementasi aktual.
> Dibuat: 2026-07-03 | Terakhir diperbarui: 2026-07-03

---

## 1. GAP KRITIS (Harus fix sebelum go-live)

### 1.1 FR-01: Audit Log on Login/Logout
- **SRS**: "mencatat waktu login di audit log" (FR-01, §4.1 step 6)
- **Implementasi**: 0 audit log calls di `auth.py` (8 endpoints)
- **Impact**: Login/logout tidak tercatat, audit trail tidak lengkap
- **Task**: T-097

### 1.2 FR-01: Refresh Token Endpoint
- **SRS Entity**: User punya field `refresh_token` dan `token_expires_at`
- **Implementasi**: Field ada di DB, TAPI tidak ada endpoint refresh/logout
- **Impact**: Token expiry = harus login ulang, tidak ada invalidation
- **Task**: T-092

### 1.3 FR-01: Email Notification Saat Akun Terkunci
- **SRS §2.2**: "Notifikasi email dikirim ke Admin saat akun terkunci"
- **Implementasi**: 0 email sends di auth service saat lockout
- **Impact**: Admin tidak tahu ada brute force attempt
- **Task**: T-097

### 1.4 NFR-SEC-04: Rate Limiting Hanya 2/20 Routers
- **SRS**: "100 request/menit per IP/user. Endpoint sensitif limit lebih ketat"
- **Implementasi**: auth.py (1 endpoint), payments.py (1 endpoint) saja
- **Impact**: Semua endpoint lain tanpa rate limit → abuse potential
- **Task**: T-089

### 1.5 NFR-SEC-01: PII Encryption Hanya 3/7 Fields
- **SRS**: "Semua kolom PII dienkripsi AES-256-GCM"
- **Implementasi**: Hanya `guardian.full_name`, `student.full_name`, `attendance.photo_url`
- **Tidak ter-encrypt**: `guardian.email`, `guardian.phone_whatsapp`, `registration.email`, `registration.phone_whatsapp`, `user.email`
- **Impact**: UU PDP compliance violation
- **Task**: T-090

### 1.6 NFR-SEC-05: JWT Secret Default in Production
- **SRS A02**: "Tidak menyimpan secret di source code"
- **Implementasi**: `SECRET_KEY = "change-me-in-production"` di config.py
- **Impact**: Jika env var tidak diset, production pakai default secret → JWT bisa di-decode
- **Task**: T-091

### 1.7 NFR-REL-04: Tidak Ada Database Backup Cron
- **SRS**: "Database di-backup otomatis setiap hari pukul 03:00 WIB"
- **Implementasi**: 0 backup references di docker-compose atau scheduler
- **Impact**: Data loss jika database corrupt
- **Task**: T-098

### 1.8 NFR-REL-05/06: Tidak Ada DR Plan
- **SRS**: "RTO ≤2 jam, RPO ≤1 jam"
- **Implementasi**: Tidak ada dokumentasi restore procedure
- **Impact**: Recovery time tidak terukur
- **Task**: T-099

---

## 2. GAP PENTING (Fix segera setelah MVP)

### 2.1 FR-05: Guardian Notification Grouping
- **SRS §4**: "Jika 1 Guardian memiliki N student, notifikasi digabungkan menjadi 1 pesan"
- **Implementasi**: `group_notification_by_guardian` ada di service, TAPI router tidak panggil
- **Impact**: Guardian dapat N notifikasi terpisah (spammy)

### 2.2 FR-10: Draft Debounced Auto-Save
- **SRS FR-10**: Draft auto-save (debounced)
- **Implementasi**: Hanya manual save (klik tombol), tidak ada debounce/auto-save
- **Impact**: User harus ingat save manual → data bisa hilang

### 2.3 FR-04: Offline Attendance Sync
- **SRS FR-04**: Field `is_offline_submission` ada
- **Implementasi**: Field ada di model, TAPI tidak ada sync logic (IndexedDB → server)
- **Impact**: Attendance offline tidak bisa di-sync

### 2.4 Audit Log Coverage 50%
- **SRS FR-07**: Semua data mutation harus ada audit log
- **Implementasi**: 10/20 routers punya audit log
- **Missing**: students (0/4), billing (0/3), attendance (0/3), payments (0/3), auth (0/8), draft (0/4), photos (0/5)
- **Tasks**: T-093..T-097

### 2.5 Soft Delete Coverage
- **SRS Entities**: "Semua entitas menggunakan soft-delete (deleted_at)"
- **Implementasi**: Hanya user, guardian, student, teacher yang ada `deleted_at`
- **Missing**: registration, invoice, payment, notification
- **Task**: T-103

### 2.6 Background Job Queue
- **SRS NFR-REL-01**: "retry 3x dengan exponential backoff"
- **Implementasi**: Email/WhatsApp dikirim inline/synchronous
- **Impact**: Request block sampai email terkirim, timeout risk

### 2.7 Audit Log Category Enum
- **SRS FR-07**: Category: SECURITY, DATA, SYSTEM, BILLING
- **Implementasi**: Tidak ada field `category` di AuditLog model
- **Impact**: Tidak bisa filter by category

### 2.8 Rate Limit In-Memory (Redis Tidak Dipakai)
- **SRS NFR-SEC-04**: Rate limit 100/minute
- **Implementasi**: Redis sudah di-config tapi tidak dipakai, rate limit in-memory
- **Impact**: Rate limit reset saat restart, tidak scalable

### 2.9 N+1 Query di my_payments
- **SRS NFR-PERF-01**: Page load ≤2 detik
- **Implementasi**: loop invoices → 1 query payment per invoice
- **Impact**: 50 invoices = 50 queries, performance degradation

---

## 3. GAP NICE TO HAVE

### 3.1 WCAG 2.1 Level A
- **SRS NFR-USA-01**: "Semua elemen interaktif memiliki label, kontras ≥4.5:1, keyboard nav"
- **Implementasi**: Partial compliance, belum di-audit formal

### 3.2 API Documentation
- **SRS Phase 7**: API docs untuk integrasi
- **Implementasi**: Belum ada OpenAPI/Swagger export

### 3.3 User Documentation
- **SRS Phase 7**: Manual untuk Admin, Instructor, Guardian
- **Implementasi**: Belum ada

---

## 4. IMPLEMENTASI YANG SUDAH SESUAI SRS

| SRS Requirement | Status | Catatan |
|---|---|---|
| FR-01: JWT HS256 per-role | ✅ | Token timeout per role |
| FR-01: Brute force 5 attempts | ✅ | Lockout 15 menit |
| FR-01: First-login password change | ✅ | Flag first_login |
| FR-01: Password expiry 7 hari | ✅ | password_expires_at |
| FR-02: Conflict detection | ✅ | FOR UPDATE |
| FR-02: Atomic booking | ✅ | Database transaction |
| FR-03: Auto-invoice tanggal 5 | ✅ | Scheduler job |
| FR-03: Fine Rp100k + freeze | ✅ | apply_fine |
| FR-03: Unique code | ✅ | generate_unique_code |
| FR-04: GPS 150m validation | ✅ | haversine_distance |
| FR-04: Face validation ≥30% | ✅ | face_valid, face_confidence |
| FR-04: Time window validation | ✅ | [-30min, +15min] |
| FR-05: WhatsApp + Email | ✅ | Dual channel |
| FR-05: Retry 3x exponential | ✅ | 10/20/40s |
| FR-05: Circuit breaker | ✅ | Per channel |
| FR-05: Notification toggle | ✅ | Admin dashboard |
| FR-05: Delivery status logging | ✅ | notification model |
| FR-07: Immutable audit log | ✅ | AuditLog model |
| FR-07: Standard fields | ✅ | table, record, action, before/after |
| FR-09: 1 Guardian : N Student | ✅ | FK relationship |
| FR-09: Freeze household | ✅ | freeze_guardian_household |
| FR-10: Draft server-side save | ✅ | Draft model |
| FR-10: TTL 7 hari cleanup | ✅ | Scheduler job |
| FR-10: Conflict detection | ✅ | Version-based |
| NFR-SEC-01: AES-256-GCM | ✅ | 3 fields (partial) |
| NFR-SEC-05: CSP, HSTS, XSS | ✅ | Security headers |
| NFR-SEC-05: SQL injection safe | ✅ | SQLAlchemy ORM |
| NFR-USA-02: Mobile-first | ✅ | 320px breakpoint |
| NFR-USA-03: Bahasa Indonesia | ✅ | All UI text |

---

## 5. RINGKASAN GAP PER PRIORITY

| Priority | Gap Count | Tasks |
|---|---|---|
| 🔴 KRITIS | 8 gaps | T-089..T-100 (12 tasks) |
| 🟡 PENTING | 9 gaps | T-101..T-114 (14 tasks) |
| 🟢 NICE TO HAVE | 3 gaps | T-115 (1 task) |
| **Total** | **20 gaps** | **27 tasks** |
