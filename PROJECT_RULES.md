# Equalizer-Stake — Project Rules & Agent Contract

Dokumen ini wajib dibaca oleh SEMUA agent sebelum mengerjakan task apapun.

---

## 1. Identitas Project

- **Nama**: Equalizer-Stake
- **Tujuan**: Sistem penjadwalan course & reservasi studio berbasis web (PWA)
- **Standar**: ISO/IEC/IEEE 29148:2018, UU PDP No. 27/2022, OWASP Top 10, ISO 27001

## 2. Stack Teknis (Final)

- **Backend**: Python 3.12+ / FastAPI / SQLAlchemy async / Pydantic v2
- **Frontend**: Next.js 15 / React 18 / Tailwind CSS (PWA)
- **Database**: PostgreSQL 16 (asyncpg) — JSONB untuk before_state/after_state di AuditLog
- **Cache**: Redis 7 (sessions, slot locking, rate limiting, cron job queue)
- **Auth**: JWT HS256, bcrypt password hashing
- **Notification**: WhatsApp Gateway REST API + SMTP email fallback
- **Face Detection**: face-api.js / MediaPipe (client-side)
- **Deploy**: Docker, cloud Jakarta region

## 3. Aturan Wajib Development

### 3.1 Keamanan (Non-Negotiable)
- Semua PII wajib AES-256-GCM encryption at-rest
- TLS 1.2+ untuk semua koneksi in-transit
- JWT: HS256, role-based expiry (Owner 30m, Admin 15m, Instructor 30m, Guardian 60m)
- Rate limiting: 100 req/min per user pada endpoint kritis
- Input validation: server-side untuk SEMUA user input
- Password: bcrypt, minimal 8 karakter, alphanumeric

### 3.2 Audit Log (Immutable)
- Setiap perubahan data kritis WAJIB dicatat sebelum dianggap selesai
- Format: timestamp, actor_id, entity_name, entity_id, before_state, after_state, ip_address
- Jika audit log gagal → ROLLBACK perubahan data
- Retensi: 7 tahun, PII dianonimkan setelah 2 tahun
- Hapus permanen hanya setelah ≥7 tahun via scheduled purging

### 3.3 Data Model
- Soft delete (is_deleted/deleted_at) untuk PII entities — TIDAK cascade delete
- RESTRICT DELETE untuk relasional (Session, Attendance, Payment)
- 1 Guardian : N Student — semua notifikasi dan invoice ke Guardian
- Attendance: UNIQUE(id_session + id_student)
- Invoice: unique_code UNIQUE per Guardian per bulan aktif

### 3.4 Business Rules (Hardcoded)
- Invoice: tanggal 5 pukul 00:00 (cron)
- Jatuh tempo: tanggal 10 pukul 23:59
- Denda: Rp100.000 flat tanggal 11 pukul 00:00 + freeze akun
- DP ≥70%: akses sampai tanggal 10, blocked setelah tanggal 11
- Jam operasional: Sen-Jum 14:30-17:00 WIB, Sab 08:00-12:00 WIB
- Jeda antar-sesi: 10 menit wajib
- Minggu ke-5: Non-Credit Theory Class (30 menit)
- Cuti student: maksimal 1 bulan, lebih = re-register

### 3.5 Notifikasi
- Kritis (pembayaran, freeze): email utama, WhatsApp cadangan
- Non-kritis (reminder 1 jam): WhatsApp utama, email cadangan
- Retry 3x: 10s → 20s → 40s sebelum fallback
- Circuit breaker: >5 consecutive failures → nonaktifkan sementara WhatsApp
- Semua ke Guardian, bukan per student

### 3.6 Attendance Validation
- 3 aspek kumulatif: GPS 150m dari (-7.7265, 110.4162), waktu server, wajah ≥30% frame
- Client-side face detection (face-api.js/MediaPipe) ≤1 detik
- Server redundant check untuk keamanan
- Gagal 1 aspek → pending_verification (bukan reject)
- Admin review ≤2 jam

## 4. Out of Scope (MVP)

- Native mobile app (PWA instead)
- Payment gateway otomatis (manual Admin)
- AI/ML atau analitik prediktif
- Multi-cabang
- Streaming/rekaman sesi daring
- Inventaris peralatan music
- Sistem akuntansi eksternal

## 5. Error Handling Standard

- Semua error message dalam Bahasa Indonesia
- Tidak expose technical details ke user
- Selalu sertakan langkah tindakan yang disarankan
- Error codes: AUTH_xxx, ATTEND_xxx, BILL_xxx, SCHED_xxx, PAY_xxx, SYS_xxx, PRIVACY_xxx

## 6. Testing Standard

- Unit test: pytest + httpx untuk backend
- Integration test: endpoint testing dengan test database
- UAT: acceptance criteria Given-When-Then dari SRS
- Security: OWASP ZAP scan, penetration test manual
- Performance: load test ≤50 concurrent users

## 7. Deployment Standard

- Docker multi-stage build
- Health check endpoints: /health, /health/db, /health/notif
- RTO ≤2 jam, RPO ≤1 jam
- Backup harian otomatis
- Maintenance window: Senin 02:00-04:00 WIB
- Data residency: region Jakarta (UU PDP)
