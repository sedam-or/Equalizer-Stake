# TASKS.md — Equalizer-Stake Development Tasks

> **SRS Decomposer**: Memecah SRS menjadi task terukur berdasarkan 7 modul fungsional,
> 11 tabel database, 4 persona pengguna, dan alur bisnis BMD Musik.
> **SRS Modular**: `srs/` (per fitur, ter-trace ke FR/UC)
> **Architecture**: `architecture/` (technical implementation)
> **NFR**: `srs/99-nfr.md` (Performance, Security, Usability, Reliability, Availability)

---

## NFR Reference (Quick)

| Category | ID | Target | Task |
|----------|-----|--------|------|
| Performance | NFR-PERF-01 | Page load ≤2s | All frontend tasks |
| Performance | NFR-PERF-02 | Form validation ≤1s | T-004, T-005, T-006 |
| Performance | NFR-PERF-03 | Notification ≤15s | T-007 |
| Performance | NFR-PERF-04 | ≤50 concurrent users | T-019 |
| Performance | NFR-PERF-05 | Batch invoice ≤5min | T-005 |
| Security | NFR-SEC-01 | AES-256-GCM at-rest | T-002, T-006 |
| Security | NFR-SEC-02 | TLS 1.2+ in-transit | T-020 |
| Security | NFR-SEC-03 | Brute force 5/15min | T-003 |
| Security | NFR-SEC-04 | Rate limit 100/min | T-020 |
| Security | NFR-SEC-05 | OWASP A02-A07 | All tasks |
| Usability | NFR-USA-01 | WCAG 2.1 Level A | All frontend tasks |
| Usability | NFR-USA-02 | Mobile-first 320px+ | All frontend tasks |
| Usability | NFR-USA-03 | Bahasa Indonesia UI | All frontend tasks |
| Usability | NFR-USA-04 | Admin training ≤2h | T-018 |
| Reliability | NFR-REL-01 | Retry 3x exponential | T-007 |
| Reliability | NFR-REL-02 | Circuit breaker | T-007 |
| Reliability | NFR-REL-03 | Atomic booking | T-004 |
| Reliability | NFR-REL-04 | Daily backup | T-020 |
| Reliability | NFR-REL-05 | RTO ≤2h | T-020 |
| Reliability | NFR-REL-06 | RPO ≤1h | T-020 |
| Availability | NFR-AVL-01 | Uptime ≥99% | T-020 |
| Availability | NFR-AVL-02 | Maintenance Mon 02-04 | T-020 |

---

## Phase 0: Infrastruktur & Setup

### T-001: Project Scaffold ✅
- **SRS Ref**: `srs/00-overview.md`, `architecture/06-deployment.md`
- **NFR Ref**: NFR-SEC-02 (TLS), NFR-AVL-01 (Uptime)
- **Prioritas**: High
- **Dependensi**: —
- **Role**: DevOps Engineer
- **Deskripsi**: Inisialisasi monorepo `backend/` (FastAPI) + `frontend/` (Next.js PWA). Docker Compose (PostgreSQL 16 + Redis 7). `.gitignore`, `.env.example`, README.
- **Output**:
  - `docker-compose.yml` — PostgreSQL 16-alpine, Redis 7-alpine, volume persisten
  - `backend/pyproject.toml` — fastapi, sqlalchemy[asyncio], asyncpg, alembic, pydantic-settings, python-jose, passlib, httpx
  - `backend/Dockerfile` — multi-stage, Python 3.12-slim, non-root user
  - `frontend/package.json` — next, react, tailwindcss, clsx, tailwind-merge
  - `frontend/Dockerfile` — node:22-alpine, standalone output

### T-002: Database Schema ✅
- **SRS Ref**: `srs/99-entities.md`, `architecture/03-database.md`
- **NFR Ref**: NFR-SEC-01 (AES-256-GCM), NFR-REL-03 (Atomic)
- **Prioritas**: High
- **Dependensi**: T-001
- **Role**: Backend Developer
- **Deskripsi**: 11 tabel (User, Guardian, Student, Teacher, Room, Session, Invoice, Payment, Attendance, Notification, AuditLog). UUID PKs, soft delete PII, JSONB audit, state machines.
- **Output**:
  - `backend/app/models/` — 11 SQLAlchemy ORM models
  - `backend/alembic/versions/001_initial.py` — 11 tables + 16 enums + indexes
  - PII fields: encrypted columns (AES-256-GCM)
  - Soft delete: is_deleted/deleted_at
  - State machines: Session, Invoice, Attendance, Student

### T-003: Authentication & RBAC (FR-01) ✅
- **SRS Ref**: `srs/01-authentication.md`, `architecture/04-security.md`
- **NFR Ref**: NFR-SEC-03 (Brute force), NFR-SEC-05 (OWASP A07)
- **Prioritas**: High
- **Dependensi**: T-002
- **Role**: Backend Developer
- **Deskripsi**: JWT HS256 per role. Session timeout: Owner 30m, Admin 30m, Guru 30m, Wali 60m. Brute force 5/15min lockout. Onboarding password (auto-gen 8 char, 7-day expiry).
- **Output**:
  - `POST /api/auth/login` — JWT issuance
  - `POST /api/auth/change-password`
  - `POST /api/auth/forgot-password`
  - Auth middleware: get_current_user, require_role
  - Brute force protection (Redis counter)
  - Audit log setiap login

---

## Phase 1: Core Backend

### T-004: Penjadwalan Kursus API (FR-02, FR-12) ✅
- **SRS Ref**: `srs/02-scheduling.md`, `architecture/01-backend.md`
- **NFR Ref**: NFR-PERF-02 (Form ≤1s), NFR-REL-03 (Atomic booking)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: Validasi konflik real-time ≤1detik (guru, ruangan, waktu+10min break). Jam operasional: Senin-Jumat 14:30-17:00, Sabtu 08:00-12:00. Minggu tutup. Concurrency control. Kredit pembatalan: Admin=100%, Wali≥24h=50%, <24h=0%.
- **Output**:
  - `POST /api/sessions/validate` — conflict check
  - `POST /api/sessions/` — create session
  - `PUT /api/sessions/{id}` — update session
  - `POST /api/sessions/{id}/cancel` — cancel + reason
  - `GET /api/sessions/` — list sessions by date
  - Holiday management endpoints
  - Week-5 theory session logic

### T-005: Billing & Invoice API (FR-03, FR-04, FR-15) ✅
- **SRS Ref**: `srs/03-billing.md`, `architecture/01-backend.md`
- **NFR Ref**: NFR-PERF-05 (Batch ≤5min), NFR-PERF-02 (Validation ≤1s)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: Auto-invoice tanggal 5 00:00. Kode unik 3 digit (000-999), max 3 retry. Denda Rp100k tanggal 11. Freeze/unfreeze. State machine: unpaid→partial_paid→overdue→frozen→paid. Manual verification by Admin.
- **Output**:
  - `POST /api/billing/invoices/generate` — cron trigger
  - `POST /api/billing/payments/verify` — match unique code
  - `POST /api/billing/invoices/{id}/fine` — apply denda
  - `POST /api/billing/invoices/{id}/unfreeze` — unfreeze
  - `GET /api/billing/invoices/` — list invoices
  - Idempotency guard (unique constraint)

### T-006: Attendance API (FR-05) ✅
- **SRS Ref**: `srs/04-attendance.md`, `architecture/05-integrations.md`
- **NFR Ref**: NFR-SEC-01 (Photo encryption), NFR-PERF-02 (Validation ≤1s)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: 3 validasi kumulatif: GPS 150m (Haversine + 15m tolerance), waktu server [-30min, +15min], face ≥30% frame. Status: verified/pending_review. Guru tidak bisa edit setelah submit.
- **Output**:
  - `POST /api/attendance/` — submit attendance
  - `PUT /api/attendance/{id}/review` — admin review
  - `GET /api/attendance/` — list by session
  - GPS validation (Haversine formula)
  - Photo storage (AES-256-GCM)

### T-007: Notification Service (FR-06) ✅
- **SRS Ref**: `srs/05-notifications.md`, `architecture/05-integrations.md`
- **NFR Ref**: NFR-PERF-03 (≤15s), NFR-REL-01 (Retry 3x), NFR-REL-02 (Circuit breaker)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Backend Developer
- **Deskripsi**: Multi-channel: WhatsApp (non-kritis) + Email (kritis). Retry 3x exponential (10/20/40s). Circuit breaker >5 fails → disable 30min. Semua notif ke Wali Murid (bukan per siswa).
- **Output**:
  - `POST /api/notifications/send` — send notification
  - `GET /api/notifications/` — list notifications
  - `POST /api/notifications/circuit-breaker/reset` — manual reset
  - Retry logic with exponential backoff
  - Circuit breaker state machine
  - Channel priority (critical=email, non-critical=whatsapp)

### T-008: Audit Log Service (FR-11) ✅
- **SRS Ref**: `srs/07-audit-governance.md`, `architecture/03-database.md`
- **NFR Ref**: NFR-SEC-05 (OWASP A05), NFR-REL-03 (Atomic)
- **Prioritas**: High
- **Dependensi**: T-002
- **Role**: Backend Developer
- **Deskripsi**: Immutable audit log. Before/after state (JSONB). IP + user-agent capture. Rollback jika log gagal. 7 tahun retensi. PII masking setelah 2 tahun.
- **Output**:
  - `GET /api/audit/` — query audit logs
  - `create_audit_log()` — atomic with data change
  - `model_to_dict()` — convert to JSONB
  - Role-based access: Owner=all, Admin=operational
  - Rollback pattern: if log fails, data change rolls back

---

## Phase 2: Frontend PWA

### T-009: PWA Shell & Navigation ✅
- **SRS Ref**: `srs/00-overview.md`, `architecture/02-frontend.md`
- **NFR Ref**: NFR-USA-01 (WCAG), NFR-USA-02 (Mobile-first), NFR-USA-03 (Indonesian)
- **Prioritas**: High
- **Dependensi**: T-001
- **Role**: Frontend Developer
- **Deskripsi**: Next.js PWA layout. Mobile-first responsive. Role-based navigation (sidebar desktop, bottom nav mobile). Indonesian UI. WCAG 2.1 Level A.
- **Output**:
  - `frontend/src/app/layout.tsx` — Root layout (Poppins + Inter fonts)
  - `frontend/src/components/Sidebar.tsx` — Desktop sidebar (6 items)
  - `frontend/src/components/BottomNav.tsx` — Mobile bottom nav (5 items)
  - `frontend/public/manifest.json` — PWA manifest

### T-010: Login & Dashboard Pages ✅
- **SRS Ref**: `srs/01-authentication.md`, `srs/09-multi-student.md`
- **NFR Ref**: NFR-USA-03 (Indonesian), NFR-SEC-03 (Brute force UI)
- **Prioritas**: High
- **Dependensi**: T-003, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Login form + API call + role-based redirect. 4 role dashboards: Owner (KPI + manage), Admin (KPI + actions), Guru (sessions), Wali Murid (students + invoices).
- **Output**:
  - `frontend/src/app/login/page.tsx` — Login form + error handling
  - `frontend/src/app/dashboard/page.tsx` — 4 role-based dashboards

### T-011: Scheduling UI ✅
- **SRS Ref**: `srs/02-scheduling.md`
- **NFR Ref**: NFR-PERF-02 (Conflict ≤1s), NFR-USA-02 (Mobile-first)
- **Prioritas**: High
- **Dependensi**: T-004, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Date picker, session list (status badges), new session form modal, conflict validation indicator.
- **Output**:
  - `frontend/src/app/dashboard/schedule/page.tsx` — Schedule page + form

### T-012: Attendance UI + Face Detection ✅
- **SRS Ref**: `srs/04-attendance.md`
- **NFR Ref**: NFR-USA-01 (WCAG), NFR-PERF-02 (Face ≤1s)
- **Prioritas**: High
- **Dependensi**: T-006, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Camera preview, photo capture, face detection status, GPS validation, submit flow, status indicators.
- **Output**:
  - `frontend/src/app/dashboard/attendance/page.tsx` — Attendance page

### T-013: Billing & Payment UI ✅
- **SRS Ref**: `srs/03-billing.md`
- **NFR Ref**: NFR-USA-03 (Indonesian), NFR-USA-02 (Mobile-first)
- **Prioritas**: High
- **Dependensi**: T-005, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Summary cards, invoice list (status badges), expandable detail, unique code display, payment instructions, overdue warning.
- **Output**:
  - `frontend/src/app/dashboard/billing/page.tsx` — Billing page

### T-014: Notification UI ✅
- **SRS Ref**: `srs/05-notifications.md`
- **NFR Ref**: NFR-USA-02 (Mobile-first), NFR-USA-03 (Indonesian)
- **Prioritas**: Medium
- **Dependensi**: T-007, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Notification list (icon + timestamp + status), filter by channel/category, mark as read, toggle settings.
- **Output**:
  - `frontend/src/app/dashboard/notifications/page.tsx` — Notification list
  - Notification toggle component (Admin only)

### T-015: Studio Reservation UI (FR-14) ✅
- **SRS Ref**: `srs/08-studio-reservation.md`
- **NFR Ref**: NFR-PERF-02 (Conflict ≤1s), NFR-USA-02 (Mobile-first)
- **Prioritas**: Medium
- **Dependensi**: T-004, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Kalender studio untuk rehearsal (self-booking 09:00-21:00) dan recording (Admin approval). Slot availability, nearest alternative suggestion.
- **Output**:
  - `frontend/src/app/dashboard/studio/page.tsx` — Studio calendar + booking
  - Rehearsal: self-service booking
  - Recording: Admin approval flow

### T-016: Landing Page ✅
- **SRS Ref**: `srs/00-overview.md`, `design/landing-page-design.md`
- **NFR Ref**: NFR-USA-01 (WCAG), NFR-USA-02 (Mobile-first), NFR-PERF-01 (Load ≤2s)
- **Prioritas**: High
- **Dependensi**: T-009
- **Role**: Frontend Developer
- **Deskripsi**: 7 sections: Hero (FloatingLines + SplashCursor), Layanan (MagicBento), Cara Kerja (GlassSurface), Studio (CircularGallery), Testimonial, CTA, Footer. GooeyNav navigation.
- **Output**:
  - `frontend/src/app/page.tsx` — Landing page
  - `frontend/src/components/landing/` — 7 section components
  - `frontend/src/components/ui/` — 11 React Bits components
  - Palette: #000000, #08CB00, #253900, #EEEEEE

---

## Phase 3: Compliance & Reporting

### T-017: Data Deletion Flow (FR-09, UU PDP) ✅
- **SRS Ref**: `srs/07-audit-governance.md`
- **NFR Ref**: NFR-SEC-01 (PII protection), NFR-USA-03 (Indonesian)
- **Prioritas**: High
- **Dependensi**: T-003, T-008
- **Role**: Backend + Frontend
- **Deskripsi**: Form pengajuan hapus data (Wali Murid/Guru). Admin verification. PII anonymization (REDACTED-[hash]). Photo deletion. Audit log "Tindakan Kepatuhan". ≤14 hari kerja.
- **Output**:
  - `POST /api/privacy/delete-request` — submit request
  - `PUT /api/privacy/delete-request/{id}/verify` — admin verify
  - Anonymization logic (REDACTED-[hash])
  - Photo permanent deletion
  - Audit log category "Tindakan Kepatuhan"

### T-018: Multi-Student Guardian (FR-10) ✅
- **SRS Ref**: `srs/09-multi-student.md`
- **NFR Ref**: NFR-USA-02 (Mobile-first), NFR-USA-03 (Indonesian)
- **Prioritas**: Medium
- **Dependensi**: T-005, T-010
- **Role**: Backend + Frontend
- **Deskripsi**: 1 Guardian : N Student. Profile switching. One invoice covers all students. Freeze applies to ALL students. Unified dashboard.
- **Output**:
  - `POST /api/guardian/students` — add student (Admin only)
  - `GET /api/guardian/students` — list students
  - Profile switcher UI
  - Household-level freeze logic

### T-019: Exam Eligibility Tracking (FR-07) ✅
- **SRS Ref**: `srs/06-reporting.md`
- **NFR Ref**: NFR-PERF-01 (Load ≤2s), NFR-USA-03 (Indonesian)
- **Prioritas**: Medium
- **Dependensi**: T-006
- **Role**: Backend + Frontend
- **Deskripsi**: Auto-calculate: kehadiran ≥90%, max 1 no-show/3 bulan, akun aktif. New students (<3mo) = incomplete data. PendingVerification = warning flag.
- **Output**:
  - `GET /api/reports/eligibility/{student_id}` — check eligibility
  - Eligibility dashboard component
  - Warning indicators

### T-020: Owner Dashboard & Reports (FR-13) ✅
- **SRS Ref**: `srs/06-reporting.md`
- **NFR Ref**: NFR-PERF-01 (Load ≤3s), NFR-USA-01 (WCAG)
- **Prioritas**: Medium
- **Dependensi**: T-005, T-006, T-008
- **Role**: Frontend Developer
- **Deskripsi**: Read-only dashboard. KPI: revenue, attendance, frozen accounts, studio utilization, audit summary. Export printable. Load ≤3s.
- **Output**:
  - Owner dashboard page (already in T-010)
  - `GET /api/reports/owner-dashboard` — aggregate KPI
  - Export component (printable format)

### T-021: Auto-Save Draft (FR-08) ✅
- **SRS Ref**: `srs/10-draft-autosave.md`
- **NFR Ref**: NFR-REL-03 (Atomic), NFR-USA-02 (Mobile-first)
- **Prioritas**: Medium
- **Dependensi**: T-004, T-009
- **Role**: Frontend Developer
- **Deskripsi**: Auto-save 30s ke IndexedDB + server sync. Restore on re-login. "Belum Tersinkronisasi" indicator. Data loss ≤1min.
- **Output**:
  - `frontend/src/hooks/useAutoSave.ts` — auto-save hook
  - IndexedDB integration
  - Server sync on re-login
  - Sync status indicator

---

## Phase 4: Integration & External

### T-022: WhatsApp Gateway Integration ✅
- **SRS Ref**: `srs/05-notifications.md`, `architecture/05-integrations.md`
- **NFR Ref**: NFR-REL-01 (Retry 3x), NFR-REL-02 (Circuit breaker)
- **Prioritas**: High
- **Dependensi**: T-007
- **Role**: Backend Developer
- **Deskripsi**: REST API ke WhatsApp gateway. Retry 3x exponential. Circuit breaker. Fallback ke email. Status logging.
- **Output**:
  - `backend/app/services/whatsapp.py` — WhatsApp client
  - Gateway integration (unofficial API)
  - Retry + circuit breaker logic

### T-023: Email SMTP Integration ✅
- **SRS Ref**: `srs/05-notifications.md`, `architecture/05-integrations.md`
- **NFR Ref**: NFR-REL-01 (Retry 3x), NFR-SEC-02 (TLS)
- **Prioritas**: High
- **Dependensi**: T-007
- **Role**: Backend Developer
- **Deskripsi**: SMTP email sending. TLS 1.2+. Retry 3x. Template email (invoice, reminder, freeze).
- **Output**:
  - `backend/app/services/email.py` — SMTP client
  - Email templates (Jinja2)
  - TLS configuration

---

## Phase 5: Testing & Security

### T-024: Unit & Integration Tests ✅
- **SRS Ref**: `srs/99-nfr.md`
- **NFR Ref**: NFR-SEC-05 (OWASP), NFR-REL-03 (Atomic)
- **Prioritas**: High
- **Dependensi**: Phase 1-4 selesai
- **Role**: QA Engineer
- **Deskripsi**: pytest untuk backend. Jest/Vitest untuk frontend. Coverage ≥80%. Edge cases: conflict, concurrent, offline.
- **Output**:
  - `backend/tests/` — unit + integration tests
  - `frontend/__tests__/` — component tests
  - CI pipeline test step

### T-025: Security Audit & Penetration Test ✅
- **SRS Ref**: `srs/99-nfr.md` (NFR-SEC-01..05)
- **NFR Ref**: NFR-SEC-01..05 (All security NFRs)
- **Prioritas**: High
- **Dependensi**: T-024
- **Role**: Security Engineer
- **Deskripsi**: OWASP ZAP scan. SQL injection, XSS, CSRF testing. JWT validation. Rate limiting verification. PII encryption check.
- **Output**:
  - Security scan report
  - Fix all findings
  - OWASP compliance matrix

### T-026: Performance & Load Testing ✅
- **SRS Ref**: `srs/99-nfr.md` (NFR-PERF-01..05)
- **NFR Ref**: NFR-PERF-01..05 (All performance NFRs)
- **Prioritas**: High
- **Dependensi**: T-024
- **Role**: QA Engineer
- **Deskripsi**: k6/Artillery load test. ≤50 concurrent users. Page load ≤2s. Form validation ≤1s. Notification ≤15s.
- **Output**:
  - Load test scripts
  - Performance report
  - Optimization recommendations

---

## Phase 6: Deployment & Operations

### T-027: Docker Production Build ✅
- **SRS Ref**: `architecture/06-deployment.md`
- **NFR Ref**: NFR-AVL-01 (Uptime ≥99%), NFR-REL-04 (Daily backup)
- **Prioritas**: High
- **Dependensi**: Phase 1-5 selesai
- **Role**: DevOps Engineer
- **Deskripsi**: Docker production multi-stage build. docker-compose.prod.yml. Environment variables. Health check endpoints.
- **Output**:
  - `docker-compose.prod.yml` — production config
  - Health check endpoints: /health, /health/db, /health/notif
  - Environment variable management

### T-028: Cloud Deployment (Jakarta Region)
- **SRS Ref**: `architecture/06-deployment.md`
- **NFR Ref**: NFR-AVL-01 (Uptime ≥99%), NFR-AVL-02 (Maintenance)
- **Prioritas**: High
- **Dependensi**: T-027
- **Role**: DevOps Engineer
- **Deskripsi**: Deploy ke cloud Jakarta region. SSL/TLS setup. DNS configuration. CDN for static assets.
- **Output**:
  - Cloud instance (AWS/GCP Jakarta)
  - SSL certificate (Let's Encrypt)
  - DNS configuration
  - CDN setup

### T-029: Monitoring & Alerting
- **SRS Ref**: `architecture/06-deployment.md`
- **NFR Ref**: NFR-AVL-01 (Uptime), NFR-REL-05 (RTO ≤2h)
- **Prioritas**: High
- **Dependensi**: T-028
- **Role**: DevOps Engineer
- **Deskripsi**: Uptime monitoring. Error rate alerting. Database size monitoring. Notification failure alerting.
- **Output**:
  - Monitoring dashboard (Grafana/Prometheus)
  - Alert rules (email + WhatsApp)
  - Log aggregation

### T-030: Backup & Recovery
- **SRS Ref**: `architecture/06-deployment.md`
- **NFR Ref**: NFR-REL-04 (Daily backup), NFR-REL-05 (RTO ≤2h), NFR-REL-06 (RPO ≤1h)
- **Prioritas**: High
- **Dependensi**: T-028
- **Role**: DevOps Engineer
- **Deskripsi**: Daily backup otomatis 03:00 WIB. 30 hari retention. Monthly restore test. Rollback procedure.
- **Output**:
  - Backup script (daily cron)
  - Restore procedure document
  - Monthly test restore

---

## Phase 7: Documentation & Training

### T-031: API Documentation
- **SRS Ref**: `architecture/01-backend.md`
- **NFR Ref**: NFR-USA-04 (Admin training ≤2h)
- **Prioritas**: Medium
- **Dependensi**: Phase 1 selesai
- **Role**: Technical Writer
- **Deskripsi**: OpenAPI/Swagger docs. Endpoint reference. Authentication guide. Error codes reference.
- **Output**:
  - FastAPI auto-generated OpenAPI
  - API reference document
  - Error codes list (AUTH_001..SYS_001)

### T-032: Admin Operation Manual
- **SRS Ref**: `srs/00-overview.md`
- **NFR Ref**: NFR-USA-04 (Admin training ≤2h)
- **Prioritas**: Medium
- **Dependensi**: Phase 2-3 selesai
- **Role**: Technical Writer
- **Deskripsi**: Panduan lengkap Admin: CRUD siswa, jadwal, tagihan, absensi, notifikasi. Screenshots.
- **Output**:
  - Admin manual (PDF/online)
  - Quick reference card

### T-033: User Documentation (Guru + Wali Murid)
- **SRS Ref**: `srs/00-overview.md`
- **NFR Ref**: NFR-USA-03 (Indonesian), NFR-USA-04 (Training)
- **Prioritas**: Low
- **Dependensi**: Phase 2-3 selesai
- **Role**: Technical Writer
- **Deskripsi**: Guru Quick Guide (absensi mobile). Wali Murid FAQ (jadwal, tagihan, booking). Online help tooltips.
- **Output**:
  - Guru quick guide
  - Wali Murid FAQ
  - In-app tooltips

---

## Phase 8: Final Verification

### T-034: User Acceptance Testing (UAT)
- **SRS Ref**: `srs/99-uc-diagrams.md` (20 UCs)
- **NFR Ref**: All NFRs
- **Prioritas**: High
- **Dependensi**: Phase 1-7 selesai
- **Role**: QA + Stakeholder
- **Deskripsi**: UAT dengan stakeholder BMD Musik. Test semua 20 Use Cases. Bug fixing. Sign-off.
- **Output**:
  - UAT test cases (20 UCs)
  - Bug report
  - Sign-off document

### T-035: Production Launch
- **SRS Ref**: `architecture/06-deployment.md`
- **NFR Ref**: NFR-AVL-01 (Uptime ≥99%)
- **Prioritas**: High
- **Dependensi**: T-034
- **Role**: DevOps + PM
- **Deskripsi**: Go-live checklist. DNS cutover. Monitoring verification. Rollback plan ready.
- **Output**:
  - Go-live checklist
  - Rollback procedure
  - Post-launch monitoring (24h)

---

## Phase 3b: Superadmin Features (ADR-006)

### T-036: Admin Account Management (Superadmin) ✅
- **SRS Ref**: `srs/11-user-interaction-flows.md`, ADR-006
- **NFR Ref**: NFR-SEC-01 (AES-256-GCM), NFR-SEC-03 (Brute force)
- **Prioritas**: High
- **Dependensi**: T-003
- **Role**: Fullstack
- **Deskripsi**: Superadmin-only CRUD akun Admin. Buat, nonaktifkan, hapus (PII anonim). Konfirmasi ketik username sebelum hapus. Force logout semua session saat Admin dihapus. Audit log setiap aksi.
- **Output**:
  - `backend/app/routers/admin_mgmt.py` — CRUD admin accounts (owner-only)
  - `backend/app/services/admin_mgmt.py` — create, deactivate, delete admin
  - `frontend/src/app/dashboard/admins/page.tsx` — Admin management UI
  - Konfirmasi hapus: ketik username

### T-037: Operational Override (Superadmin) ✅
- **SRS Ref**: `srs/11-user-interaction-flows.md`, ADR-006
- **NFR Ref**: NFR-SEC-01, NFR-REL-03 (Atomic)
- **Prioritas**: High
- **Dependensi**: T-004, T-005, T-006
- **Role**: Fullstack
- **Deskripsi**: Override operasional: buka freeze akun, batalkan tagihan, revert jadwal. Semua aksi tercatat di audit log kategori "keamanan". Konfirmasi sebelum override.
- **Output**:
  - `backend/app/routers/override.py` — override endpoints (owner-only)
  - `backend/app/services/override.py` — unfreeze, cancel invoice, revert schedule
  - `frontend/src/app/dashboard/override/page.tsx` — Override UI

### T-038: System Settings (Superadmin) ✅
- **SRS Ref**: `srs/11-user-interaction-flows.md`, ADR-006
- **NFR Ref**: NFR-SEC-04 (Rate limit), NFR-REL-01 (Retry)
- **Prioritas**: Medium
- **Dependensi**: T-003
- **Role**: Fullstack
- **Deskripsi**: Pengaturan sistem: maintenance window, alert threshold, WhatsApp gateway config. Tersimpan di DB, bisa diubah runtime.
- **Output**:
  - `backend/app/models/system_setting.py` — SystemSetting model
  - `backend/app/routers/settings.py` — CRUD settings (owner-only)
  - `frontend/src/app/dashboard/settings/page.tsx` — Settings UI

### T-039: Tarif Management ✅
- **SRS Ref**: `srs/11-user-interaction-flows.md`, ADR-006
- **NFR Ref**: NFR-PERF-02 (Validation ≤1s)
- **Prioritas**: Medium
- **Dependensi**: T-005
- **Role**: Fullstack
- **Deskripsi**: Kelola tarif kursus/studio. Owner + Admin bisa ubah. History perubahan di audit log. Tarif berlaku efektif dari tanggal tertentu.
- **Output**:
  - `backend/app/models/pricing.py` — Pricing model (instrument, level, effective_date)
  - `backend/app/routers/pricing.py` — CRUD pricing (owner + admin)
  - `frontend/src/app/dashboard/pricing/page.tsx` — Pricing management UI

## Phase 2b: Remaining Pages

### T-040: Profile Page (Shared)
- **Status**: ✅ Selesai
- **SRS Ref**: UC-02 (Kelola Profil)
- **Role**: Fullstack
- **Deskripsi**: Profil user shared semua role. View/edit info, change password.
- **Output**:
  - `backend/app/routers/auth.py` — Fixed /me, /change-password, added /me/profile + PATCH /me/profile
  - `backend/app/schemas/auth.py` — Added UpdateProfileRequest, ProfileResponse
  - `frontend/src/app/dashboard/profile/page.tsx` — Profile page
  - `frontend/src/components/Sidebar.tsx` — Added profile link
  - `frontend/src/components/BottomNav.tsx` — Added profile icon for mobile

### T-041: Owner Audit Log + Reports ✅
- **Status**: ⏳
- **SRS Ref**: UC-17 (Ekspor Laporan), UC-18 (Audit Log)
- **Role**: Fullstack
- **Deskripsi**: Audit log viewer + laporan/reports dashboard untuk Owner.
- **Output**:
  - `frontend/src/app/dashboard/audit-log/page.tsx`
  - `frontend/src/app/dashboard/reports/page.tsx`

### T-042: Admin Students + Holidays + Payments
- **Status**: ⏳
- **SRS Ref**: UC-05, UC-06, UC-08
- **Role**: Fullstack
- **Deskripsi**: Manajemen siswa, konfigurasi libur, verifikasi pembayaran manual.
- **Output**:
  - `frontend/src/app/dashboard/students/page.tsx` (upgrade)
  - `frontend/src/app/dashboard/holidays/page.tsx`
  - `frontend/src/app/dashboard/payments/page.tsx`

### T-043: Guru + Wali Murid Portal Pages
- **Status**: ⏳
- **SRS Ref**: UC-03, UC-06, UC-11
- **Role**: Fullstack
- **Deskripsi**: My Schedule (guru/WM), My Children, My Payments, My Bookings.
- **Output**:
  - `frontend/src/app/dashboard/my-schedule/page.tsx`
  - `frontend/src/app/dashboard/my-children/page.tsx`
  - `frontend/src/app/dashboard/my-payments/page.tsx`
  - `frontend/src/app/dashboard/my-bookings/page.tsx`

## Summary

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 0: Infrastructure | T-001..T-003 | ✅ 3/3 done |
| Phase 1: Core Backend | T-004..T-008 | ✅ 5/5 done |
| Phase 2: Frontend PWA | T-009..T-016 | ✅ 8/8 |
| Phase 3: Compliance | T-017..T-021 | ✅ 5/5 |
| Phase 3b: Superadmin | T-036..T-039 | ✅ 4/4 |
| Phase 2b: Pages | T-040..T-043 | ✅ 2/4 |
| Phase 4: Integration | T-022..T-023 | ✅ 2/2 |
| Phase 5: Testing | T-024..T-026 | ✅ 3/3 |
| Phase 6: Deployment | T-027..T-030 | ⏳ 0/4 |
| Phase 7: Documentation | T-031..T-033 | ⏳ 0/3 |
| Phase 8: Final | T-034..T-035 | ⏳ 0/2 |
| **Total** | **39 tasks** | **17/39 done** |
