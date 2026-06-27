# TASKS.md — Equalizer-Stake Development Tasks

> SRS Decomposer: Memecah SRS menjadi task terukur berdasarkan 7 modul fungsional,
> 11 tabel database, 4 persona pengguna, dan alur bisnis BMD Musik.
> SRS Modular: `srs/` (per fitur, ter-trace ke FR/UC)
> Architecture: `architecture/` (technical implementation)

---

## Phase 0: Infrastruktur & Setup

### T-001: Project Scaffold ✅
- **SRS Ref**: `srs/00-overview.md`, `architecture/06-deployment.md` & Monorepo Setup
- **Prioritas**: High
- **Dependensi**: —
- **Role**: DevOps Engineer
- **Deskripsi**: Inisialisasi monorepo dengan struktur `backend/` (FastAPI) dan `frontend/` (Next.js PWA). Setup `.gitignore`, `docker-compose.yml` (PostgreSQL 16 + Redis 7), `.env.example`, dan README dasar.
- **Output**:
  - `docker-compose.yml` — PostgreSQL 16-alpine, Redis 7-alpine, volume persisten
  - `backend/pyproject.toml` — dependencies: fastapi, sqlalchemy[asyncio], asyncpg, alembic, pydantic-settings, python-jose, bcrypt, httpx
  - `backend/Dockerfile` — multi-stage, Python 3.12-slim, non-root user
  - `frontend/package.json` — next, react, tailwindcss, clsx, tailwind-merge
  - `frontend/Dockerfile` — node:22-alpine
- **SRS Ref**: BAB 2.4 (Technical Constraints), BAB 3.3 (Hardware Spec)

### T-002: Database Schema ✅
- **SRS Ref**: `srs/99-entities.md`, `architecture/03-database.md` & Migrations
- **Prioritas**: High
- **Dependensi**: T-001
- **Role**: Backend Developer
- **Deskripsi**: Implementasi 11 tabel database (User, Guardian, Student, Teacher, Room, Session, Invoice, Payment, Attendance, Notification, AuditLog) sesuai ERD BAB 5. Setup Alembic migrations. Soft delete untuk PII entities.
- **Output**:
  - `backend/app/models/` — SQLAlchemy ORM models (11 entities)
  - `backend/alembic/` — initial migration
  - PII fields: AES-256-GCM encrypted
  - Soft delete: is_deleted/deleted_at columns
  - Indexes sesuai ERD constraints
- **SRS Ref**: BAB 5 (ERD, Data Dictionary), BAB 3.2 (Security NFR)

### T-003: Authentication & RBAC (FR-01) ✅
- **SRS Ref**: `srs/01-authentication.md`, `architecture/04-security.md` & RBAC (FR-01)
- **Prioritas**: High
- **Dependensi**: T-002
- **Role**: Backend Developer
- **Deskripsi**: Implementasi login, JWT token, role-based access control. 4 roles (Owner/Admin/Teacher/Guardian) dengan session timeout berbeda. Brute force protection (5 attempts/15min). Onboarding password awal via email.
- **Output**:
  - `POST /api/auth/login` — JWT issuance
  - `POST /api/auth/change-password` — password change
  - JWT middleware dengan role checking
  - Brute force lockout mechanism
  - Audit log untuk setiap login
- **SRS Ref**: FR-01, UC-01, BAB 4.2 (Login Scenario)

---

## Phase 1: Core Backend

### T-004: Penjadwalan ✅ Kursus API (FR-02, FR-12)
- **SRS Ref**: `srs/02-scheduling.md`, `architecture/01-backend.md` Kursus API (FR-02, FR-12)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: API penjadwalan dengan validasi konflik real-time (≤1 detik). Concurrency control. Jam operasional. Jeda 10 menit. Libur nasional. Minggu ke-5 non-kredit.
- **SRS Ref**: FR-02, FR-12, UC-02, UC-05, Business Rules A, E

### T-005: Billing ✅ & Invoice API (FR-03, FR-04, FR-15)
- **SRS Ref**: `srs/03-billing.md`, `architecture/01-backend.md` & Invoice API (FR-03, FR-04, FR-15)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: Auto-invoice tanggal 5 (cron), kode unik 3 digit, denda Rp100k tanggal 11, freeze/unfreeze, verifikasi pembayaran manual Admin. State machine: unpaid→partial_paid→overdue→frozen→paid.
- **SRS Ref**: FR-03, FR-04, FR-15, UC-05, UC-07, UC-08, Business Rules C

### T-006: Attendance ✅ API (FR-05)
- **SRS Ref**: `srs/04-attendance.md`, `architecture/05-integrations.md` API (FR-05)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: API absensi dengan validasi GPS 150m, waktu server, foto upload. Pending verification flow. Photo storage dengan AES-256-GCM.
- **SRS Ref**: FR-05, UC-03, UC-11, UC-12, Business Rules B

### T-007: Notification ✅ Service (FR-06)
- **SRS Ref**: `srs/05-notifications.md`, `architecture/05-integrations.md` Service (FR-06)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: Multi-channel notification (WhatsApp + email). Retry 3x (10/20/40s). Circuit breaker. Fallback mechanism. Notification log.
- **SRS Ref**: FR-06, UC-04, UC-14, UC-15, BAB 7.2 (Integration Contracts)

### T-008: Audit Log Service (FR-11)
- **SRS Ref**: `srs/07-audit-governance.md`, `architecture/03-database.md` Service (FR-11)
- **Prioritas**: High
- **Dependensi**: T-002
- **Role**: Backend Developer
- **Deskripsi**: Immutable audit log untuk semua perubahan data kritis. Before/after state (JSONB). Rollback jika log gagal. 7 tahun retensi. PII masking setelah 2 tahun.
- **SRS Ref**: FR-11, UC-11, UC-18, SEC-06

---

## Phase 2: Frontend PWA

### T-009: PWA Shell & Navigation
- **Prioritas**: High
- **Dependensi**: T-001
- **Role**: Frontend Developer
- **Deskripsi**: Setup Next.js PWA dengan service worker. Mobile-first responsive layout. Role-based navigation. Indonesian language UI. WCAG 2.1 Level A.
- **SRS Ref**: BAB 3.3 (UI), BAB 2.7 (Operating Environment), NFR Usability

### T-010: Login & Dashboard Pages
- **Prioritas**: High
- **Dependensi**: T-003, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Halaman login, role-based dashboard (Owner monitoring, Admin operasional, Guru jadwal, Wali Murid multi-anak). Auto-save draft (IndexedDB).
- **SRS Ref**: FR-01, FR-08, FR-10, FR-13, UC-01, UC-02, UC-10

### T-011: Scheduling UI
- **Prioritas**: High
- **Dependensi**: T-004, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Form input jadwal dengan real-time conflict validation. Kalender view. Slot locking indicator. Reschedule flow.
- **SRS Ref**: FR-02, FR-12, UC-02, UC-03, UC-05

### T-012: Attendance UI + Face Detection
- **Prioritas**: High
- **Dependensi**: T-006, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Form absensi dengan kamera, face-api.js/MediaPipe client-side detection (≥30% frame), GPS validation, offline draft.
- **SRS Ref**: FR-05, UC-03, UC-11, UC-12

### T-013: Billing & Payment UI
- **Prioritas**: High
- **Dependensi**: T-005, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Invoice view untuk Wali Murid. Payment verification dashboard untuk Admin. Unique code display. Freeze/unfreeze status.
- **SRS Ref**: FR-03, FR-04, FR-15, UC-05, UC-07, UC-08, UC-10

### T-014: Studio Reservation UI (FR-14)
- **Prioritas**: Medium
- **Dependensi**: T-004, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Kalender studio untuk rehearsal (self-booking) dan recording (Admin approval). Real-time availability.
- **SRS Ref**: FR-14, UC-06

---

## Phase 3: Compliance & Reporting

### T-015: Data Deletion Flow (FR-09)
- **Prioritas**: High
- **Dependensi**: T-003, T-008
- **Role**: Backend + Frontend
- **Deskripsi**: Form pengajuan hapus data (Wali Murid/Guru). Admin verification. PII anonymization (REDACTED-[hash]). Photo deletion. Audit log "Tindakan Kepatuhan". ≤14 hari kerja.
- **SRS Ref**: FR-09, UC-19, UC-20, SEC-05, UU PDP Pasal 19

### T-016: Exam Eligibility Tracking (FR-07)
- **Prioritas**: Medium
- **Dependensi**: T-006
- **Role**: Backend + Frontend
- **Deskripsi**: Auto-calculate kelayakan ujian: kehadiran ≥90%, max 1 no-show/3 bulan, akun aktif. Dashboard view.
- **SRS Ref**: FR-07, UC-09, UC-13

### T-017: Owner Dashboard & Reports (FR-13)
- **Prioritas**: Medium
- **Dependensi**: T-005, T-006, T-008
- **Role**: Frontend Developer
- **Deskripsi**: Dashboard monitoring read-only untuk Owner. KPI: pendapatan, kehadiran, akun beku, utilisasi studio. Ekspor laporan.
- **SRS Ref**: FR-13, UC-10, UC-16, UC-17

### T-018: User Documentation
- **Prioritas**: Low
- **Dependensi**: Semua task selesai
- **Role**: QA Engineer
- **Deskripsi**: Admin Operation Manual, Guru Quick Guide, Wali Murid FAQ. Online help tooltip pada form kritis.
- **SRS Ref**: BAB 2.2 Module 7

---

## Phase 4: Testing & Deployment

### T-019: Integration Testing & UAT
- **Prioritas**: High
- **Dependensi**: Phase 1-3 selesai
- **Role**: QA Engineer
- **Deskripsi**: End-to-end testing semua FR. UAT dengan stakeholder BMD Musik. Performance testing (≤50 concurrent). Security scan (OWASP ZAP).
- **SRS Ref**: BAB 6 (Traceability), BAB 7.5 (Launch Checklist)

### T-020: Deployment & Monitoring Setup
- **Prioritas**: High
- **Dependensi**: T-019
- **Role**: DevOps Engineer
- **Deskripsi**: Docker production build. Cloud deployment (Jakarta region). Health check endpoints. Alert thresholds. Backup configuration. SSL/TLS setup.
- **SRS Ref**: BAB 7.4 (Monitoring), BAB 7.5 (Rollback), NFR Availability
```
