# ROLES — Equalizer-Stake Agent Roles

## System Roles (SRS-defined)

### 1. Owner / Superadmin (Pemilik Equalizer-Stake)
- **Akses**: Dashboard monitoring, audit log (semua kategori), manajemen Admin, override operasional, pengaturan sistem
- **Session timeout**: 30 menit
- **Otoritas**: Buat/hapus/nonaktifkan Admin, override freeze/tagihan/jadwal, kelola tarif, maintenance window
- **Product Champion**: Founder/Owner Equalizer-Stake
- **Catatan**: Superadmin — bisa akses semua fitur termasuk yang biasanya hanya Admin

### 2. Admin Operasional
- **Akses**: Input/ubah jadwal, verifikasi pembayaran, manajemen akun, monitoring notifikasi
- **Session timeout**: 30 menit
- **Otoritas**: Penjadwalan harian, validasi absensi, eskalasi bentrok jadwal
- **Product Champion**: Kepala Admin Equalizer-Stake

### 3. Instructor
- **Akses**: Kalender pribadi, input absensi + foto, konfirmasi kehadiran, terima reminder
- **Session timeout**: 30 menit
- **Otoritas**: Standar penilaian kehadiran, persetujuan reschedule, kelayakan level student

### 4. Guardian
- **Akses**: Jadwal multi-anak, booking studio, terima tagihan & reminder, riwayat kehadiran
- **Session timeout**: 60 menit
- **Relasi**: 1 Wali : N Student (kakak-adik dalam satu pengguna)

---

## Development Agent Roles

### 1. System Architect
- **Bertanggung jawab**: ERD, arsitektur sistem, keputusan teknis tingkat tinggi
- **Scope**: BAB 4 (UML), BAB 5 (ERD), infrastruktur, security architecture

### 2. PRD Decomposer (SRS Decomposer)
- **Bertanggung jawab**: Memecah SRS menjadi TASKS.md, mengurutkan prioritas berdasarkan Wiegers scoring
- **Scope**: Semua BAB SRS → task breakdown

### 3. Orchestrator
- **Bertanggung jawab**: Mengkoordinasi antar role (dipegang oleh user + assistant)
- **Scope**: Task assignment, dependency management, progress tracking

### 4. Backend Developer
- **Bertanggung jawab**: FastAPI, SQLAlchemy, endpoint API, logika bisnis (billing, scheduling, attendance)
- **Scope**: FR-01..FR-15 backend implementation, cron jobs, audit log

### 5. Frontend Developer
- **Bertanggung jawab**: Next.js PWA, komponen UI (kalender, form absensi, dashboard), face-api.js integration
- **Scope**: Mobile-first responsive, offline capability, WCAG 2.1 Level A

### 6. QA Engineer
- **Bertanggung jawab**: Unit test, integration test, UAT, validasi data
- **Scope**: Test cases TC-01..TC-15, acceptance criteria Given-When-Then

### 7. Security Auditor
- **Bertanggung jawab**: Review kode dari sisi keamanan (OWASP Top 10, UU PDP compliance)
- **Scope**: SEC-01..SEC-08, AES-256-GCM, JWT, rate limiting, input validation

### 8. Performance Optimizer
- **Bertanggung jawab**: Optimasi query DB, caching Redis, load time, auto-scaling config
- **Scope**: NFR Performance targets (≤2s page, ≤1s validation, ≤15s notif)

### 9. DevOps Engineer
- **Bertanggung jawab**: Dockerfile, docker-compose, deployment config, monitoring, backup
- **Scope**: Health check endpoints, alert thresholds, rollback procedures

### 10. Data Analyst
- **Bertanggung jawab**: Skema ekspor data (CSV/JSON), logika perhitungan statistik, laporan Owner
- **Scope**: FR-13 dashboard, FR-07 exam eligibility, reporting module
