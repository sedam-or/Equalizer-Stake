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

### T-043: Guru + Wali Murid Portal Pages ✅
- **Status**: ✅ Selesai (3/4 pages, my-bookings deferred)
- **SRS Ref**: UC-03, UC-06, UC-11
- **Role**: Fullstack
- **Deskripsi**: My Schedule (guru/WM), My Children, My Payments, My Bookings.
- **Output**:
  - `frontend/src/app/dashboard/my-schedule/page.tsx`
  - `frontend/src/app/dashboard/my-children/page.tsx`
  - `frontend/src/app/dashboard/my-payments/page.tsx`
  - `frontend/src/app/dashboard/my-bookings/page.tsx`

---

## 15. Security Audit — Authentication Flow

### T-044: Audit Autentikasi (Login, Register, Forgot Password, Verification)
- **Status**: ✅ Selesai
- **SRS Ref**: FR-01, UC-01, SEC-01..SEC-08
- **Role**: Security Auditor
- **Deskripsi**: Full audit alur autentikasi — login, register, forgot-password, JWT validation, brute force protection, password policy. Bandingkan implementasi vs SRS. Output: compliance matrix + findings.
- **Dependencies**: T-003 (Authentication & RBAC) ✅
- **Tasks**:
  1. Audit backend auth endpoints (login, register, forgot-password, change-password)
  2. Audit auth middleware (JWT validation, role extraction, expiry check)
  3. Audit brute force protection (failed_attempts, lockout logic)
  4. Audit frontend auth pages (error handling, token storage, redirect logic)
  5. Audit password policy (hashing, onboarding, first-login, expiry)
  6. Compare vs SRS acceptance criteria (Given-When-Then)
  7. Output compliance matrix ke docs/SRS_COMPLIANCE_AUTH.md

### T-045: Fix Auth Audit Findings ✅
- **Status**: ✅ Selesai
- **SRS Ref**: FR-01, UC-01, SEC-01..SEC-08
- **Role**: Fullstack
- **Deskripsi**: Fix all 15 findings from T-044 auth audit.
- **Dependencies**: T-044 (Auth Audit) ✅
- **Findings fixed**:
  1. FINDING 1 (HIGH): Onboarding model fields added (first_login, is_temp_password, password_expires_at) + migration SQL
  2. FINDING 2 (HIGH): Audit logging on all auth events (6 calls: LOGIN_SUCCESS, LOGIN_FAILED, REGISTER, PASSWORD_CHANGE, ACCOUNT_LOCKED)
  3. FINDING 3 (HIGH): CSRF — documented as N/A for JWT-in-header architecture
  4. FINDING 4 (HIGH): localStorage — accepted trade-off, documented
  5. FINDING 5 (MEDIUM): Rate limiting @limiter.limit('10/minute') on login endpoint
  6. FINDING 6 (MEDIUM): Per-role expires_in using ACCESS_TOKEN_EXPIRE_MINUTES dict
  7. FINDING 7 (MEDIUM): Change-password error → generic 'Password lama tidak valid'
  8. FINDING 8 (MEDIUM): Admin lockout notification — logged as ACCOUNT_LOCKED (email requires email service setup)
  9. FINDING 9 (MEDIUM): Role-based redirect (teacher/guardian → /dashboard/my-schedule)
  10. FINDING 10 (MEDIUM): Address field added to register form + backend schema
  11. FINDING 11 (MEDIUM): Register network error handling added
  12. FINDING 12 (MEDIUM): Forgot-password user enumeration fixed
  13. FINDING 13 (LOW): Password visibility toggle (Eye/EyeOff) on login + register
  14. FINDING 14 (LOW): autoComplete='off' on forgot-password email
  15. FINDING 15 (LOW): Security headers middleware (X-Content-Type-Options, X-Frame-Options, X-XSS-Protection, Referrer-Policy)

### T-046: Sidebar RBAC + Seed Script ✅
- **Status**: ✅ Selesai
- **SRS Ref**: FR-01, ADR-006 (Owner=Superadmin)
- **Role**: Fullstack
- **Deskripsi**: Fix sidebar role filtering + create seed script for 4 test accounts.
- **Dependencies**: T-044 ✅
- **Changes**:
  1. Sidebar.tsx — added `roles` array per nav section, filter by current role
  2. seed_accounts.py — creates owner/admin/teacher/guardian accounts
- **Role mapping**:
  - owner: Operasional + Manajemen + Notifications + Profile
  - admin: Operasional + Notifications + Profile
  - teacher: Portal (my-schedule only) + Notifications + Profile
  - guardian: Portal (my-schedule, my-children, my-payments) + Notifications + Profile

### T-047: Dashboard Dark Theme + Dummy Data ✅
- **Status**: ✅ Selesai
- **SRS Ref**: All dashboard FRs
- **Role**: Fullstack
- **Deskripsi**: Apply dark theme to all 18 dashboard pages + layout/sidebar/bottomnav. Add 5 dummy data per page.
- **Dependencies**: T-046 ✅

---

## 12. SRS Compliance Fixes (from Full Gap Analysis)

### T-048: PII Encryption Service (AES-256-GCM)
- **Status**: ✅ Selesai
- **SRS Ref**: NFR-SEC-01, UU PDP
- **Deskripsi**: Implement AES-256-GCM encryption for PII fields (guardian.full_name, student.full_name, attendance.photo_url). Create encrypt/decrypt service, integrate with models.
- **Dependencies**: T-045 ✅

### T-049: Cron Jobs (Auto-Invoice, Fine, Cleanup, Reminders)
- **Status**: ✅ Selesai
- **SRS Ref**: FR-03 §2, FR-02 §5, SRS-10 §4
- **Deskripsi**: Implement scheduler for: auto-invoice tanggal 5, fine+freeze tanggal 11, draft cleanup, notification reminders. Use APScheduler or background tasks.
- **Dependencies**: T-048 ⏳

### T-050: Onboarding Flow (Temp Password + Forced Change)
- **Status**: ✅ Selesai
- **SRS Ref**: FR-01 §2.3, UC-01 §3.4
- **Deskripsi**: Auto-generate temp password, email credentials, force password change on first login. Use first_login + password_expires_at fields (already added in T-045).
- **Dependencies**: T-048 ⏳

### T-051: Frozen Login Block + Credit Refund + Cancel Logic
- **Status**: ✅ Selesai
- **SRS Ref**: FR-03 §3, FR-02 §5
- **Deskripsi**: Block login for frozen guardians. Implement cancel session with credit refund tiers. Complete TODO stub in scheduling router.
- **Dependencies**: T-048 ⏳

### T-052: Audit Log Completeness
- **Status**: ✅ Selesai
- **SRS Ref**: FR-07 §2, NFR-SEC-08
- **Deskripsi**: Add category field to audit_log model. Add audit logging to: session creation, notification sends, dashboard access, pricing changes. Add 7-year retention + PII anonymization cron.
- **Dependencies**: T-048 ⏳

### T-053: Notification Enhancements
- **Status**: ✅ Selesai
- **SRS Ref**: FR-05 §4, §5, §7
- **Deskripsi**: Group notifications per guardian (N students → 1 msg). Add admin channel toggle endpoint. Add CRITICAL_FAILURE status + admin alert. Add notification audit logging. Add WhatsApp rate limiting.
- **Dependencies**: T-052 ⏳

### T-054: Security Headers + Secret Key + Concurrency
- **Status**: ✅ Selesai
- **SRS Ref**: NFR-SEC-05, NFR-REL-03
- **Deskripsi**: Add CSP + HSTS headers. Enforce non-default SECRET_KEY. Add optimistic locking for concurrent schedule saves. Fix GPS precision (Float → Numeric(10,7)). Add session ownership validation.
- **Dependencies**: T-048 ⏳

### T-055: Studio Reservation (Guardian Self-Booking)
- **Status**: ❌ CANCELLED — Studio booking = WhatsApp admin (di luar sistem). Room hanya lokasi kursus.
- **SRS Ref**: FR-08, UC-06
- **Deskripsi**: Guardian self-booking for rehearsal. Recording approval flow. Studio hourly pricing model. Alternative slot suggestions. DB unique constraint for double-booking. Cancel with H-1 rule.
- **Dependencies**: T-054 ⏳

### T-056: Reporting Enhancements
- **Status**: ✅ Selesai
- **SRS Ref**: FR-06 §3
- **Deskripsi**: PDF/HTML export endpoint. Date range filter on dashboard. 6-month revenue trend. Per-studio breakdown. Anomaly flag detection. Attendance <5 sessions asterisk.
- **Dependencies**: T-052 ⏳

### T-057: Admin Channel Toggle + Notification Grouping UI
- **Status**: ✅ Selesai
- **SRS Ref**: FR-05 §5
- **Deskripsi**: Frontend: admin settings page for notification channel toggle per category. Guardian: grouped notification display.
- **Dependencies**: T-053 ⏳

### T-058: Payment Gateway Circuit Breaker + Rate Limiting
- **Status**: ✅ Selesai
- **SRS Ref**: NFR-REL-02, NFR-SEC-04
- **Deskripsi**: Add circuit breaker for payment gateway. Add WhatsApp rate limiting (100 msg/sec). Add payment-specific rate limiting on verify endpoint.
- **Dependencies**: T-053 ⏳

### T-059: Draft Conflict Resolution + Operational Hours
- **Status**: ⏳
- **SRS Ref**: FR-10 §7, FR-02 §2
### T-059: Draft Conflict Resolution + Composite Unique Constraint
- **Status**: ✅ Selesai

---


## 13. UAT-Driven End-to-End Flow
**Patokan:** docs/UAT_BMD.md — setiap task = 1 alur UAT

### T-060: UAT-3 — Admin Buat Jadwal → Guru/Guardian Lihat
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 3 (3.1-3.5)
- **Deskripsi**: 
  - Admin: form buat sesi (guru, siswa, ruangan, hari, jam) + conflict detection
  - Guru: my-schedule fetch real data dari API
  - Guardian: my-schedule fetch real data anak dari API
- **Test**: Login admin → buat 1 jadwal → login guru → lihat jadwal → login guardian → lihat jadwal anak

### T-061: UAT-4 — Guru Submit Absensi (Foto + GPS)
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 4 (4.1-4.7)
- **Deskripsi**: 
  - Guru: page submit absensi (camera upload + GPS auto-detect)
  - Validasi: jarak ≤150m, foto ≤2MB, face ≥30%
  - Submit → status pending_verification
- **Test**: Login guru → buka jadwal hari ini → klik absen → foto → GPS → submit

### T-062: UAT-5 — Admin Verifikasi Absensi
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 5 (5.1-5.5)
- **Deskripsi**: 
  - Admin: fetch pending absensi dari API
  - Detail modal: foto, GPS, jarak, face confidence
  - Verifikasi/Tolak → PUT /api/attendance/{id}/review
- **Test**: Login admin → buka absensi pending → review foto → verifikasi

### T-063: UAT-7 — Guardian Bayar + Upload Bukti Transfer
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 7 (7.1-7.5)
- **Deskripsi**: 
  - Guardian: lihat invoice real dari API (bukan dummy)
  - Upload foto bukti transfer + input jumlah
  - Submit → status pending_verification
- **Test**: Login guardian → buka tagihan → klik invoice → upload bukti → submit

### T-064: UAT-8 — Admin Verifikasi Pembayaran (Kode Unik)
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 8 (8.1-8.6)
- **Deskripsi**: 
  - Admin: fetch pending payments dari API
  - Review bukti transfer + input kode unik 3 digit
  - Verifikasi → invoice lunas, duplikat detection
- **Test**: Login admin → buka billing pending → review bukti → input kode → verifikasi

### T-065: UAT-2 — Guardian Portal (Anak, Jadwal, Tagihan)
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 2 (2.1-2.4)
- **Deskripsi**: 
  - Connect my-schedule, my-children, my-payments ke real API
  - Hapus semua dummy data, fetch dari database
- **Test**: Login guardian → lihat anak → lihat jadwal → lihat tagihan (semua data real)

### T-066: UAT-1 — Visitor Daftar Kursus → Admin Approve
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 1 (1.1-1.6)
- **Deskripsi**: 
  - Landing page: tombol "Daftar Kursus" → form (nama, WA, email, alamat, instrumen, level, tipe kelas)
  - Admin: menu Pendaftaran → approve → generate akun guardian
  - Notifikasi WA ke guardian
- **Test**: Buka landing → daftar → login admin → approve → login guardian

### T-067: Master Data — Instrumen, Level, Tipe Kelas, Harga
- **Status**: ✅ Selesai
- **UAT Ref**: Supporting data untuk semua alur
- **Deskripsi**: 
  - Seed 9 instrumen, level per instrumen, 9 tipe kelas
  - Harga asli BMD MS +20% sesuai addendum SRS
  - Update Pricing model + seed script
- **Test**: Query pricing → semua kombinasi instrumen×level×tipe ada

### T-068: UAT-6+9+10 — Test Cron Jobs (Invoice, Denda, Notifikasi)
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 6 (6.1-6.5), Alur 9 (9.1-9.5), Alur 10 (10.1-10.6)
- **Deskripsi**: 
  - Manual trigger cron jobs untuk test
  - Verify: invoice generated, kode unik benar, reminder terkirim
  - Verify: denda Rp 100k, freeze akun, guardian blocked
  - Verify: notifikasi di semua event
- **Test**: Trigger manual → cek DB → cek notifikasi

### T-069: UAT-11 — Data Deletion (UU PDP)
- **Status**: ✅ Selesai
- **UAT Ref**: Alur 11 (11.1-11.5)
- **Deskripsi**: 
  - Guardian: tombol "Hapus Data Saya" di profile
  - Admin: menu Privacy → review → proses
  - PII dianonimkan, audit log tercatat
- **Test**: Login guardian → minta hapus → login admin → proses → verify data anonim

### T-070: End-to-End Integration Test (Semua 11 Alur)
- **Status**: ✅ Selesai
- **UAT Ref**: docs/UAT_BMD.md — semua alur
- **Deskripsi**: 
  - Jalankan SEMUA 11 alur secara berurutan
  - Setiap step di-verify sesuai tabel UAT
  - Semua harus PASS = sistem siap produksi
- **Test**: Ikuti UAT_BMD.md dari Alur 1 sampai 11, centang setiap step

---

## 14. Fitur Baru (dari diskusi + Stitch prompt)

### T-071: Foto Profil Semua Role
- **Status**: ⏳
- **SRS Ref**: Addendum §12
- **Deskripsi**: 
  - Backend: POST /api/users/me/photo (upload foto, max 2MB, JPEG/PNG)
  - Backend: GET /api/users/me/photo (ambil foto terenkripsi)
  - Frontend: upload foto di halaman Profile (semua role)
  - Default: avatar inisial nama (warna random)
  - Enkripsi: AES-256-GCM sebelum simpan
- **Test**: Login setiap role → buka Profile → upload foto → refresh → foto tampil

### T-072: Feedback Guru ke Wali Murid
- **Status**: ⏳
- **SRS Ref**: Addendum §13
- **Deskripsi**: 
  - Backend: POST /api/feedback (guru kirim feedback per siswa)
  - Backend: GET /api/feedback?student_id=X (wali lihat feedback anak)
  - Frontend: guru page /dashboard/feedback (form + riwayat)
  - Frontend: wali page /dashboard/my-children (tampilkan feedback guru)
  - Kategori: Kemajuan, Perilaku, Kehadiran, Rekomendasi
  - Rating: 1-5 per kategori
- **Test**: Login guru → kirim feedback → login wali → lihat feedback di Anak Saya

### T-073: Penilaian Wali Murid ke Guru
- **Status**: ⏳
- **SRS Ref**: Addendum §14
- **Deskripsi**: 
  - Backend: POST /api/ratings (wali beri rating ke guru)
  - Backend: GET /api/ratings?teacher_id=X (admin lihat rating guru)
  - Frontend: wali page /dashboard/rating (form + riwayat)
  - Frontend: admin page /dashboard/teachers (tampilkan rating guru)
  - Rating: 1-5 bintang + komentar
  - Validasi: hanya bisa menilai guru yang mengajar anaknya
- **Test**: Login wali → beri rating guru → login admin → lihat rating

### T-074: Page Detail Instrumen dan Kelas
- **Status**: ⏳
- **SRS Ref**: Addendum §15
- **Deskripsi**: 
  - Frontend: /instruments (public page)
  - Per instrumen: deskripsi, level, tipe kelas, harga, durasi, usia minimal
  - Data dari GET /api/pricing/
  - CTA "Daftar Sekarang" per instrumen
  - Tabel ringkasan harga semua instrumen
- **Test**: Buka /instruments → semua 9 instrumen tampil → klik CTA → redirect ke /register-course

### T-075: Page Bantuan
- **Status**: ⏳
- **SRS Ref**: Addendum §16
- **Deskripsi**: 
  - Frontend: /help (public, FAQ accordion + search + kontak WhatsApp)
  - Frontend: /dashboard/help (per role, panduan penggunaan)
  - Kategori: Umum, Akun, Pembayaran, Jadwal, Absensi, Privasi
  - Kontak: WhatsApp admin, telepon, alamat, jam operasional
- **Test**: Buka /help → FAQ tampil → search berfungsi → kontak WhatsApp terlihat

### T-076: Email Reset Password Pages
- **Status**: ⏳
- **SRS Ref**: Addendum §17
- **Deskripsi**: 
  - Frontend: /reset-password/success (ikon centang hijau + tombol Login)
  - Frontend: /reset-password/error (ikon silang merah + tombol Kirim Ulang)
  - Backend: token validasi (expired 1 jam, rate limit 3x/jam/email)
- **Test**: Request reset → klik link → success page. Tunggu 1 jam → klik link → error page

### T-077: Mobile-First Wali Murid Dashboard
- **Status**: ⏳
- **SRS Ref**: Addendum §18, NFR-USA-02
- **Deskripsi**: 
  - Refactor semua halaman wali murid ke mobile-first
  - Bottom navigation di mobile (<768px)
  - Sidebar di desktop (>=768px)
  - Kartu besar, tombol besar, font besar
  - Test di viewport 320px, 375px, 768px, 1024px
- **Test**: Buka dashboard wali di mobile → bottom nav tampil → semua page responsive

## Summary

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 0: Infrastructure | T-001..T-003 | ✅ 3/3 done |
| Phase 1: Core Backend | T-004..T-008 | ✅ 5/5 done |
| Phase 2: Frontend PWA | T-009..T-016 | ✅ 8/8 |
| Phase 3: Compliance | T-017..T-021 | ✅ 5/5 |
| Phase 3b: Superadmin | T-036..T-039 | ✅ 4/4 |
| Phase 2b: Pages | T-040..T-043 | ✅ 3/4 (my-bookings deferred) |
| Phase 4: Integration | T-022..T-023 | ✅ 2/2 |
| Phase 5: Testing | T-024..T-026 | ✅ 3/3 |
| Phase 6: Deployment | T-027..T-030 | ⏳ 0/4 |
| Phase 7: Documentation | T-031..T-033 | ⏳ 0/3 |
| Phase 8: Final | T-034..T-035 | ⏳ 0/2 |
| Phase 9: Security Audit | T-044..T-045 | ✅ 2/2 |
| Phase 10: RBAC | T-046 | ✅ 1/1 |
| Phase 11: Dashboard UI | T-047 | ✅ 1/1 |
| Phase 12: SRS Compliance | T-048..T-059 | ✅ 11/12 (T-055 cancelled) |
| Phase 13: UAT-Driven Flow | T-060..T-070 | ✅ 11/11 |
| Phase 14: Fitur Baru | T-071..T-077 | ⏳ 0/7 |
| Phase 15: Critical Fixes | T-078..T-085 | ✅ 8/8 |
| Phase 16: Pagination | T-086..T-088 | ✅ 3/3 |
| Phase 17: KRITIS — Security & SRS Compliance | T-089..T-100 | ✅ 12/12 |
| Phase 18: IMPORTANT — Features & Performance | T-101..T-114 | ✅ 14/14 |
| Phase 19: NICE TO HAVE | T-115 | ✅ 1/1 |
| Phase 20: INFRASTRUCTURE — External Services | T-116..T-118 | ⏳ 0/3 |
| Phase 21: BACKEND CRITICAL FIXES | T-119..T-127 | ✅ 9/9 |
| Phase 22: FRONTEND REDESIGN — LIGHT THEME + UX | T-128..T-155 | ⏳ 27/28 |
| Phase 23: LANDING PAGE + REGISTRATION REBUILD | T-156..T-162 | ⏳ 0/7 |
| Phase 24: GUARDIAN + TEACHER FLOW FIX | T-163..T-168 | ⏳ 0/6 |
| Phase 25: COMPREHENSIVE SYSTEM AUDIT | T-169..T-177 | ⏳ 0/9 |
| Phase 26: DEEP AUDIT — FRONTEND vs BACKEND | T-178..T-185 | ⏳ 0/8 |
| Phase 27: AUDIT FIXES — BACKEND | T-186..T-192 | ⏳ 0/7 |
| Phase 28: AUDIT FIXES — FRONTEND | T-193..T-197 | ⏳ 0/5 |
| **Total** | **162 tasks** | **120/162 done** |

---

## 15. Critical Fixes (sistem tidak jalan tanpa ini)

### T-078: Seed Data Terhubung Antar Role
- **Status**: ✅ Selesai
- **Priority**: 🔴 CRITICAL
- **Deskripsi**: 
  - Buat 1 guardian + 1 guru + 3 siswa + 3 sesi + 3 invoice yang saling terhubung
  - Guardian punya siswa → siswa punya jadwal → jadwal punya guru
  - Invoice terhubung ke guardian → ada kode unik
  - Login sebagai guardian → lihat anak, jadwal, tagihan (semua data real)
  - Login sebagai guru → lihat jadwal (data real)
  - Login sebagai admin → lihat jadwal, absensi pending, tagihan pending
- **Test**: Login 3 role → semua lihat data yang sama → data mengalir

### T-079: GPS Validation Server-Side
- **Status**: ✅ Selesai
- **Priority**: 🔴 CRITICAL
- **Deskripsi**: 
  - Backend: hitung jarak dari koordinat yang dikirim ke studio (-7.737428, 110.429669)
  - Jika jarak > 150m → tolak absensi, return error "Anda di luar jangkauan studio"
  - Simpan gps_distance_m di tabel attendance
  - Tidak hanya terima koordinat, tapi VALIDASI
- **Test**: Kirim GPS 100m → accepted. Kirim GPS 500m → rejected

### T-080: Foto Storage Service
- **Status**: ✅ Selesai
- **Priority**: 🔴 CRITICAL
- **Deskripsi**: 
  - Backend: simpan foto absensi ke file system (bukan DB langsung)
  - Path: /uploads/attendance/{session_id}_{timestamp}.jpg
  - Encrypt sebelum simpan (AES-256-GCM)
  - Simpan path di kolom photo_url (bukan base64)
  - Endpoint GET /api/attendance/{id}/photo → return decrypted photo
  - Foto profil: simpan ke /uploads/profiles/{user_id}.jpg
- **Test**: Submit absensi dengan foto → file tersimpan → bisa diambil kembali

### T-081: Mobile Navigation (BottomNav) untuk Wali Murid
- **Status**: ✅ Selesai
- **Priority**: 🔴 CRITICAL
- **Deskripsi**: 
  - Refactor BottomNav: tampilkan HANYA untuk role guardian di mobile
  - Menu: Dashboard, Anak Saya, Jadwal, Bayar, Profil
  - Sidebar: sembunyikan di mobile (<768px), tampil di desktop
  - Layout guardian: tanpa sidebar di mobile, dengan sidebar di desktop
  - Test di viewport 320px, 375px, 768px, 1024px
- **Test**: Login guardian di mobile → bottom nav tampil → navigasi berfungsi

### T-082: Email Service Configuration
- **Status**: ✅ Selesai
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Konfigurasi SMTP di .env (host, port, user, password)
  - Buat email template: reset password, welcome, invoice reminder
  - Test kirim email ke alamat test
  - Rate limit: max 3 reset per jam per email
- **Test**: Request reset password → email terkirim → klik link → reset berhasil

### T-083: WhatsApp Gateway Configuration
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Konfigurasi WhatsApp gateway di .env (API key, phone number)
  - Template pesan: invoice reminder, denda, freeze, welcome
  - Test kirim WA ke nomor test
  - Retry 3x dengan exponential backoff
- **Test**: Trigger invoice reminder → WA terkirim

### T-084: Cron Job Testing
- **Status**: ✅ Selesai
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Manual trigger setiap cron job untuk test
  - Verify: auto-invoice → invoice created di DB
  - Verify: denda + freeze → guardian frozen di DB
  - Verify: draft cleanup → expired drafts deleted
  - Verify: session reminder → notifikasi terkirim
  - Log setiap job execution
- **Test**: Trigger manual → cek DB → cek log

### T-085: Registration Notification Flow
- **Status**: ✅ Selesai
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Setelah admin approve registrasi → kirim WA ke guardian
  - Template: "Akun BMD MS Anda sudah aktif. Username: {phone}, Password: {temp_password}. Login di {url}"
  - Simpan notifikasi di tabel notifications
  - Audit log: "Registrasi disetujui, notifikasi terkirim"
- **Test**: Admin approve → guardian terima WA dengan credentials

---

## 16. Pagination (Data Tidak Lagi Fetch Semuanya)

### T-086: Pagination Backend — Semua List Endpoint
- **Status**: ✅ Selesai (re-eksekusi 2026-07-03)
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Tambah `limit` + `offset` parameter ke SEMUA list endpoint
  - Return format: `{ data: [...], total: N, limit: N, offset: N }`
  - Default: limit=20, offset=0
  - Max: limit=100
  - Endpoint yang perlu ditambah:
    - /api/admin-mgmt/admins
    - /api/notifications
    - /api/pricing
    - /api/settings
    - /api/holidays
    - /api/registrations
    - /api/billing
    - /api/attendance
    - /api/my/children
    - /api/my/schedule
    - /api/my/payments
- **Test**: GET /api/students?limit=10&offset=0 → return 10 data + total

### T-087: Pagination Frontend — Semua Dashboard Page
- **Status**: ✅ Selesai (re-eksekusi 2026-07-03)
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Tambah pagination UI ke SEMUA dashboard page yang list data
  - Component: `<Pagination total={N} limit={20} offset={0} onChange={fn} />`
  - UI: "Menampilkan 1-20 dari 150" + tombol Prev/Next + nomor halaman
  - Mobile: tombol besar, mudah di-tap
  - Page yang perlu ditambah:
    - students, payments, billing, attendance, schedule
    - notifications, pricing, holidays, registrations
    - admins, audit-log, override, settings
    - my-children, my-schedule, my-payments
- **Test**: Buka students → tampil 20 data → klik Next → tampil data berikutnya

### T-088: Pagination Testing & Optimization
- **Status**: ✅ Selesai (2026-07-03)
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**: 
  - Test semua endpoint dengan data besar (500+ records)
  - Verify: tidak ada N+1 query
  - Verify: response time ≤2s untuk 1000 records
  - Verify: frontend tidak fetch semua data
  - Add database index pada kolom yang sering di-sort/filter
- **Test**: Seed 1000 students → GET /api/students?limit=20 → response ≤2s

## 17. KRITIS — Security & SRS Compliance

### T-089: Rate Limiting — Semua Auth Endpoints
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-01, NFR-SEC-04
- **Deskripsi**:
  - Tambah `@limiter.limit` ke semua auth endpoints: login, register, forgot-password, reset-password, change-password
  - Target: 5 attempts/minute per IP untuk login, 3/minute untuk register
  - Saat ini hanya auth.py:1 endpoint punya rate limit
  - SRS NFR-SEC-04: "Rate limit 100 request/menit per IP/user. Endpoint sensitif limit lebih ketat"
- **Test**: 6x login brute force → 429 Too Many Requests
- **Deps**: T-086 ✅

### T-090: PII Encryption — Email + Phone Fields
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-SEC-01, UU PDP No. 27/2022
- **Deskripsi**:
  - Encrypt: guardian.email, guardian.phone_whatsapp
  - Encrypt: registration.email, registration.phone_whatsapp
  - Encrypt: user.email (dengan blind index untuk lookup)
  - SRS NFR-SEC-01: "Semua kolom PII dienkripsi AES-256-GCM"
  - Blind index pattern: hash(email) → simpan hash + encrypted value → lookup by hash
  - Saat ini: guardian.full_name, student.full_name, attendance.photo_url saja yang ter-encrypt
- **Test**: Direct DB query → email/phone ter-encrypt, tidak terbaca plain
- **Deps**: none

### T-091: JWT Secret Validation — Reject Default in Production
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-01, NFR-SEC-05
- **Deskripsi**:
  - Di config.py: jika DEBUG=False dan SECRET_KEY == "change-me-in-production" → raise error saat startup
  - Prevent production deployment dengan default secret
  - Log warning di startup jika secret masih default
  - SRS A02: "Tidak menyimpan secret di source code"
- **Test**: Set DEBUG=false + default secret → app refuses to start
- **Deps**: none

### T-092: Refresh Token Endpoint — JWT Refresh Flow
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-01, SRS Entity User (field refresh_token, token_expires_at)
- **Deskripsi**:
  - Entity User sudah punya field refresh_token dan token_expires_at
  - TAPI tidak ada endpoint refresh token di code
  - Implement POST /api/auth/refresh → validate refresh token → issue new access token
  - Implement POST /api/auth/logout → invalidate refresh token
  - Tanpa ini: user harus login ulang setiap 30-60 menit
- **Test**: Login → gunakan refresh token → dapat access token baru tanpa login
- **Deps**: none

### T-093: Audit Log — Students CRUD
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-07, SRS Audit Governance
- **Deskripsi**:
  - Tambah `create_audit_log` ke students router: create, update, delete, restore
  - SRS wajibkan audit log immutable 7 tahun
  - Saat ini 0/4 endpoints students punya audit log
- **Test**: Create student → audit_logs terisi dengan action INSERT + record_id
- **Deps**: none

### T-094: Audit Log — Billing Endpoints
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-07, FR-03
- **Deskripsi**:
  - Tambah audit log ke billing router: verify payment, apply fine
  - Log siapa yang verify, kapan, hasilnya
- **Test**: Verify payment → audit log berisi changed_by + after_state
- **Deps**: none

### T-095: Audit Log — Attendance Endpoints
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-07, FR-04
- **Deskripsi**:
  - Tambah audit log ke attendance router: submit, review
  - Log submission dengan GPS coords + face_valid
  - Log review dengan verification_status + rejection_reason
- **Test**: Submit attendance → audit log dengan gps_lat, gps_lng, face_valid
- **Deps**: none

### T-096: Audit Log — Payments Endpoints
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-07, FR-03
- **Deskripsi**:
  - Tambah audit log ke payments router: create, verify
  - Log payment amount, method, unique_code verification result
- **Test**: Verify payment → audit log dengan amount + method + status
- **Deps**: none

### T-097: Audit Log — Auth Endpoints
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-07, FR-01
- **Deskripsi**:
  - Tambah audit log ke auth router: login (success/failure), logout, register, change-password
  - SRS 3.1: "mencatat waktu login di audit log"
  - Log IP address untuk security monitoring
- **Test**: Login成功 → audit log dengan action=LOGIN + ip_address
- **Deps**: none

### T-098: Database Backup — Daily 03:00 WIB
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-REL-04
- **Deskripsi**:
  - SRS NFR-REL-04: "Database di-backup otomatis setiap hari pukul 03:00 WIB"
  - Backup disimpan 30 hari
  - Implement via cron job di docker-compose atau pg_dump scheduled task
  - Test restore dilakukan bulanan
- **Test**: Jalankan backup manual → verify file ada di /backups/
- **Deps**: none

### T-099: DR Plan — RTO ≤2h, RPO ≤1h
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-REL-05, NFR-REL-06
- **Deskripsi**:
  - SRS NFR-REL-05: "target recovery ≤2 jam dari waktu kejadian"
  - SRS NFR-REL-06: "Data yang hilang ≤1 jam terakhir"
  - Dokumentasi DR plan: restore procedure, contact list, escalation
  - Test restore dari backup → verify data integrity
- **Test**: Simulasi database corrupt → restore dari backup → data kembali
- **Deps**: T-098

### T-100: N+1 Fix — my_payments Eager Load
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-PERF-01
- **Deskripsi**:
  - Fix N+1 query di my_portal.py:my_payments
  - Saat ini: loop invoices → 1 query payment per invoice (N+1!)
  - Fix: batch load `select(Payment).where(Payment.id_invoice.in_([inv.id for inv in invoices]))`
  - Impact: 50 invoices = 50 queries → 2 queries
- **Test**: GET /api/my/payments?limit=20 → server log max 3 queries
- **Deps**: none

---

## 18. IMPORTANT — Features & Performance

### T-101: Background Job Queue — Email/WhatsApp
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-05, NFR-REL-01
- **Deskripsi**:
  - Implement background job queue untuk email + WhatsApp sending
  - SRS NFR-REL-01: retry 3x dengan exponential backoff
  - Saat ini: inline/synchronous → request block sampai email terkirim
  - Fix: pakai FastAPI BackgroundTasks + asyncio.Queue
  - Email gagal → queue retry, bukan block request
- **Test**: Send notification → request returns 200 immediately, email sent async
- **Deps**: none

### T-102: Guardian Notification Grouping
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-05
- **Deskripsi**:
  - SRS FR-05: "Semua notifikasi dikirim ke Wali Murid, bukan per siswa"
  - SRS: "Jika 1 Wali Murid memiliki N siswa, notifikasi digabungkan menjadi 1 pesan"
  - Service group_notification_by_guardian sudah ada, tapi router tidak panggil
  - Implement: schedule event → group by guardian → send combined message
- **Test**: Guardian dengan 3 siswa → dapat 1 notifikasi gabungan, bukan 3
- **Deps**: T-101

### T-103: Soft Delete — Registration, Invoice, Payment
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-07, SRS Entities
- **Deskripsi**:
  - SRS Entities: "Semua entitas menggunakan soft-delete (deleted_at TIMESTAMP NULL)"
  - Saat ini: registration, invoice, payment tidak ada deleted_at
  - Tambah `deleted_at` column ke models + migration
  - Update semua queries untuk filter `deleted_at IS NULL`
  - Audit trail preserved (SRS compliance)
- **Test**: Delete registration → deleted_at terisi, data masih ada di DB
- **Deps**: Alembic migration

### T-104: Redis Actual Usage — Rate Limit + Session Cache
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-SEC-04, NFR-REL-03
- **Deskripsi**:
  - Redis sudah di-config (REDIS_URL) tapi tidak dipakai
  - Implement: rate limit storage backend = Redis (bukan in-memory)
  - Implement: session cache untuk JWT blacklist (anti concurrent session)
  - Implement: slot availability cache (prevent double booking)
  - Saat ini in-memory = lost on restart, tidak scalable
- **Test**: Restart backend → rate limit counter tidak reset
- **Deps**: Redis harus running di docker-compose

### T-105: Teacher Schedule Filter by Instrument
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-02
- **Deskripsi**:
  - Tambah param `?instrument=piano` ke GET /api/my/schedule (teacher mode)
  - JOIN session → student → filter by student.instrument
  - Guru piano cuma lihat jadwal piano, guru biola cuma lihat biola
  - Backward compat: jika ?instrument tidak dikirim → semua instrumen
- **Test**: Guru dengan 2 instrumen → GET /api/my/schedule?instrument=piano → hanya jadwal piano
- **Deps**: T-086 ✅

### T-106: Teacher Students List (GET /api/my/students)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-02
- **Deskripsi**:
  - NEW endpoint: list murid yang diajar guru ini
  - Filter by `?instrument=piano` → hanya murid piano
  - Ambil via session join: teacher → sessions → students
  - Pagination {data, total, limit, offset}
- **Test**: GET /api/my/students?instrument=piano → return murid piano yang diajar
- **Deps**: T-086 ✅

### T-107: Teacher Attendance List (GET /api/my/attendance)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-04
- **Deskripsi**:
  - NEW endpoint atau extend: teacher lihat absensi
  - Filter by `?instrument=piano`
  - Ambil via attendance → session → student → filter instrument
  - Pagination {data, total, limit, offset}
- **Test**: GET /api/my/attendance?instrument=piano → absensi guru untuk murid piano
- **Deps**: T-086 ✅

### T-108: Guardian Schedule Per Student
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-02
- **Deskripsi**:
  - Tambah param `?student_id=xxx` ke GET /api/my/schedule (guardian mode)
  - Guardian lihat jadwal 1 anak spesifik
  - Backward compat: jika student_id tidak dikirim → semua anak
- **Test**: Guardian dengan 2 anak → GET /api/my/schedule?student_id=xxx → jadwal 1 anak
- **Deps**: T-086 ✅

### T-109: Guardian Attendance Per Student (GET /api/my/attendance)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-04
- **Deskripsi**:
  - NEW endpoint: guardian lihat absensi anak
  - Filter by `?student_id=xxx`
  - Pagination {data, total, limit, offset}
- **Test**: GET /api/my/attendance?student_id=xxx → absensi 1 anak
- **Deps**: T-086 ✅

### T-110: Guardian Invoices Per Student (GET /api/my/invoices)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-03
- **Deskripsi**:
  - NEW endpoint: guardian lihat tagihan per anak
  - Filter by `?student_id=xxx`
  - Pagination {data, total, limit, offset}
- **Test**: GET /api/my/invoices?student_id=xxx → tagihan 1 anak
- **Deps**: T-086 ✅

### T-111: Frontend — Teacher Dashboard Instrument Tabs
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-02, NFR-USA-02
- **Deskripsi**:
  - Tab "Semua | Piano | Biola" di halaman Jadwal/Murid/Absensi guru
  - Tab list = ambil dari teacher.instruments_specialization
  - Klik tab → fetch ?instrument=xxx
  - Tab aktif = green accent (#08CB00)
- **Test**: Login guru 2 instrumen → lihat tab → klik Piano → data piano only
- **Deps**: T-105, T-106, T-107

### T-112: Frontend — Guardian Per-Student Sections
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-09, NFR-USA-02
- **Deskripsi**:
  - Dropdown/section per anak di dashboard guardian (dari my/children)
  - Pilih anak → fetch ?student_id=xxx ke schedule/attendance/invoices
  - Default: tampilkan semua anak (gabungan)
- **Test**: Login guardian 2 anak → pilih Anak A → jadwal/absensi/tagihan Anak A only
- **Deps**: T-108, T-109, T-110

### T-113: Draft Debounced Auto-Save
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-10
- **Deskripsi**:
  - SRS FR-10: draft auto-save (debounced, bukan manual)
  - Saat ini: hanya manual save (klik tombol)
  - Implement: debounce 30 detik → auto-save ke server
  - Client-side: detect form change → wait 30s → POST /api/drafts/{form_key}
  - Server: sudah support save_draft, tinggal client trigger
- **Test**: Isi form → tunggu 30 detik → refresh → draft masih ada
- **Deps**: none

### T-114: Offline Attendance Sync
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-04
- **Deskripsi**:
  - Field is_offline_submission sudah ada di model
  - TAPI tidak ada sync logic untuk offline submissions
  - Implement: client simpan ke IndexedDB saat offline → sync saat online
  - Server: terima offline submission → validate timestamp → process
- **Test**: Submit attendance offline → reconnect → data ter-sync
- **Deps**: none

---

## 19. NICE TO HAVE

### T-115: WCAG 2.1 Level A Compliance
- **Status**: ✅
- **Priority**: 🟢 NICE TO HAVE
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - SRS NFR-USA-01: "Semua elemen interaktif memiliki label, kontras warna ≥4.5:1, navigasi keyboard, alt text"
  - Audit: check semua form inputs punya label
  - Audit: check kontras warna (terutama dark theme)
  - Audit: check keyboard navigation
  - Audit: check alt text pada gambar
  - Saat ini: partial compliance
- **Test**: Lighthouse accessibility score ≥ 80
- **Deps**: none

---

## 20. INFRASTRUCTURE — External Services Integration

### T-116: Object Storage — MinIO/S3 for File Uploads
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-SEC-01, NFR-REL-01
- **Deskripsi**:
  - Saat ini: photo_storage.py pakai local filesystem (`/backend/uploads/`)
  - Ganti dengan MinIO (S3-compatible) untuk production-ready object storage
  - Config: MINIO_ENDPOINT, MINIO_ACCESS_KEY, MINIO_SECRET_KEY, MINIO_BUCKET
  - Update photo_storage.py: upload/download/delete via MinIO client
  - Update privacy.py: hapus dari object storage (saat ini ada `# TODO`)
  - Backup: local filesystem tetap sebagai fallback jika MinIO down
  - Migration: upload existing local files ke MinIO
- **Test**: Upload foto → tersimpan di MinIO → bisa di-download → bisa dihapus
- **Deps**: none

### T-117: WhatsApp Gateway — Real API Integration
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-05, FR-06, NFR-REL-01
- **Deskripsi**:
  - Service wrapper sudah ada (whatsapp.py: retry, circuit breaker, rate limiting)
  - TAPI: WHATSAPP_API_URL dan WHATSAPP_API_KEY masih kosong
  - Pilih provider: Wablas, Fonnte, atau official WhatsApp Business API
  - Configure .env: WHATSAPP_API_URL, WHATSAPP_API_KEY, WHATSAPP_PHONE_ID
  - Test kirim pesan actual ke nomor HP real
  - Handle webhook untuk incoming messages (delivery status, reply)
  - SRS: retry 3x (10/20/40s), circuit breaker 5 failures → 30min cooldown
- **Test**: Kirim pesan WhatsApp actual → diterima → status delivered
- **Deps**: none

### T-118: SMTP Email Service — Real Email Delivery
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-05, NFR-REL-01
- **Deskripsi**:
  - Notification service sudah punya circuit breaker untuk email channel
  - TAPI: tidak ada actual SMTP sending implementation
  - Pilih provider: Gmail SMTP, Mailgun, SendGrid, atau AWS SES
  - Configure .env: SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASSWORD, SMTP_FROM
  - Implement email sending: welcome, forgot-password, invoice, reminder
  - HTML email templates untuk setiap tipe notifikasi
  - Test deliverability: pastikan tidak masuk spam
  - SRS: retry 3x (10/20/40s), circuit breaker
- **Test**: Kirim email actual → diterima → isi benar → tidak spam
- **Deps**: T-101 (Background Job Queue)

---

## 21. BACKEND CRITICAL FIXES

### T-119: GPS Radius Validation (UC-12)
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-04, UC-12
- **Deskripsi**:
  - Server-side GPS radius check ≤150m dari studio
  - Studio coords: -7.737428, 110.429669 (Seturan)
  - Jika di luar radius → status "pending_verification"
  - Haversine formula untuk hitung jarak
- **Test**: Submit GPS 200m dari studio → status pending, bukan hadir
- **Files**: `backend/app/services/attendance.py`, `backend/app/routers/attendance.py`

### T-120: Unfreeze Account Endpoint (UC-10)
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-03, UC-10
- **Deskripsi**:
  - POST /api/billing/unfreeze/{guardian_id}
  - Admin-only: unfreeze akun yang terkena denda
  - Audit log: UNFREEZE action
  - Validasi: akun harus dalam status frozen
- **Test**: Freeze akun → unfreeze → status aktif kembali
- **Files**: `backend/app/routers/billing.py`, `backend/app/services/billing.py`

### T-121: Dashboard Stats API
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: UC-16
- **Deskripsi**:
  - GET /api/dashboard/stats (admin/owner)
  - GET /api/dashboard/teacher (guru)
  - GET /api/dashboard/guardian (wali murid)
  - Return: real data, bukan dummy
  - Include: greeting, KPI, recent items, pending actions
- **Test**: Login admin → dashboard show real numbers
- **Files**: `backend/app/routers/dashboard.py` (new), `backend/app/services/reports.py`

### T-122: API Versioning (v1)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-SEC-01
- **Deskripsi**:
  - Semua router prefix: /api/v1/...
  - Backward compatible: redirect /api/ → /api/v1/
  - Version header: X-API-Version: 1
  - Deprecation header untuk sunset endpoints
- **Test**: GET /api/v1/auth/login → 200
- **Files**: Semua router, `backend/app/main.py`

### T-123: Self-Service Data Deletion Request (UC-19)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-07, UC-19
- **Deskripsi**:
  - POST /api/privacy/delete-request (guardian/teacher)
  - Admin approve/reject: PUT /api/admin/deletion-requests/{id}
  - Audit log: DELETE_REQUEST, DELETE_APPROVED, DELETE_REJECTED
  - SRS: Wali murid/guru bisa request hapus data
- **Test**: Guardian request delete → admin approve → soft delete + anonymize
- **Files**: `backend/app/routers/privacy.py`, `backend/app/models/deletion_request.py` (new)

### T-124: Export Laporan (UC-17)
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: UC-17
- **Deskripsi**:
  - GET /api/reports/export?format=csv&type=revenue
  - GET /api/reports/export?format=csv&type=attendance
  - GET /api/reports/export?format=csv&type=students
  - CSV generation + download
- **Test**: Owner → Laporan → Export → file download
- **Files**: `backend/app/routers/reports.py`

### T-125: Feedback Endpoint (T-072)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-02
- **Deskripsi**:
  - POST /api/feedback (guru → wali murid)
  - GET /api/my/feedback (wali murid lihat feedback)
  - Fields: session_id, rating (1-5), comment, category
  - Audit log: INSERT
- **Test**: Guru kirim feedback → wali murid lihat di portal
- **Files**: `backend/app/routers/feedback.py` (new), `backend/app/models/feedback.py` (new)

### T-126: Rating Endpoint (T-073)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-02
- **Deskripsi**:
  - POST /api/ratings (wali murid → guru)
  - GET /api/teachers/{id}/ratings (admin lihat)
  - Fields: teacher_id, rating (1-5), comment
  - Audit log: INSERT
- **Test**: Wali murid rate guru → admin lihat rata-rata
- **Files**: `backend/app/routers/ratings.py` (new), `backend/app/models/rating.py` (new)

### T-127: Data Migration Import
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR
- **Deskripsi**:
  - POST /api/admin/import/students (CSV upload)
  - POST /api/admin/import/guardians (CSV upload)
  - POST /api/admin/import/sessions (CSV upload)
  - Validation + error report
  - Preview sebelum import
- **Test**: Upload CSV → preview → confirm → data masuk
- **Files**: `backend/app/routers/import.py` (new), `backend/app/services/import_service.py` (new)

---

## 22. FRONTEND REDESIGN — LIGHT THEME + UX

### T-128: Theme System — Light Default + Toggle
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - CSS variables untuk light/dark theme
  - Tailwind dark: prefix
  - Theme toggle component (🌙/☀️)
  - Persist: localStorage
  - Default: light
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §1
- **Test**: Buka dashboard → light theme → toggle → dark → refresh → persist
- **Files**: `frontend/src/lib/theme.ts` (new), `frontend/src/app/globals.css`, `frontend/tailwind.config.ts`

### T-129: Typography Update — Plus Jakarta Sans
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - Install Plus Jakarta Sans (Google Fonts)
  - Heading: Plus Jakarta Sans
  - Body: Inter (existing)
  - Mono: JetBrains Mono
  - Font scale: xs(12) sm(14) base(16) lg(18) xl(20) 2xl(24) 3xl(30) 4xl(36)
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §2
- **Test**: Semua heading pakai Plus Jakarta Sans
- **Files**: `frontend/src/app/layout.tsx`, `frontend/tailwind.config.ts`

### T-130: Color Palette Update — Blue + Gold
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - Replace #08CB00 → #2563EB (primary blue)
  - Add #F59E0B (accent gold)
  - Add #7C3AED (secondary purple)
  - Update: cards, buttons, sidebar, forms
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §1
- **Test**: Semua halaman pakai palette baru
- **Files**: Semua halaman frontend, `tailwind.config.ts`

### T-131: Toast Component
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - components/Toast.tsx (success/error/warning/info)
  - Position: bottom-right (mobile: top)
  - Auto-dismiss: success 3s, error 5s
  - Animation: slide in from right
  - Replace semua alert() → toast
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §10
- **Test**: Submit form → success toast muncul → auto-dismiss
- **Files**: `frontend/src/components/Toast.tsx` (new), semua halaman

### T-132: Form Validation System
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - lib/validation.ts (validateEmail, validatePhone, etc.)
  - onChange: update status ✓/✗
  - onBlur: tampilkan error message
  - Visual: green ✓ / red ✗ + message
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §11
- **Test**: Isi form → blur → error muncul → fix → error hilang
- **Files**: `frontend/src/lib/validation.ts` (new), semua form

### T-133: Confirmation Dialog Component
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - components/ConfirmDialog.tsx
  - 4 severity: simple, destructive, financial, complex
  - Replace window.confirm() → modal
  - Animation: slide up + fade in
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §12
- **Test**: Klik delete → modal muncul → konfirmasi → data terhapus
- **Files**: `frontend/src/components/ConfirmDialog.tsx` (new)

### T-134: Date/Time Formatter
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR
- **Deskripsi**:
  - Install dayjs + locale/id
  - lib/date.ts: formatDate, formatRelative, formatDuration
  - Replace semua hardcoded date format
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §13
- **Test**: Tanggal tampil "3 Juli 2026" bukan "2026-07-03"
- **Files**: `frontend/src/lib/date.ts` (new)

### T-135: Currency Formatter
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR
- **Deskripsi**:
  - lib/currency.ts: formatRupiah, formatRupiahShort
  - Full: "Rp 350.000" (tagihan, invoice)
  - Short: "Rp 350rb" (dashboard cards)
  - Replace semua hardcoded currency format
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §14
- **Test**: Harga tampil "Rp 350.000" bukan "350000"
- **Files**: `frontend/src/lib/currency.ts` (new)

### T-136: Landing Page Redesign
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - Background: light (#F8FAFC)
  - Hero: gradient biru-ungu + emotional copy
  - Trust bar: angka social proof
  - 9 sections sesuai design system
  - Copywriting: emosional, benefit > fitur
  - CTA: WhatsApp langsung
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §9
- **Test**: Buka landing page → light theme → copy menjual → CTA WhatsApp
- **Files**: `frontend/src/app/page.tsx`, semua landing components

### T-137: Dashboard Real Data — Admin
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: UC-16
- **Deskripsi**:
  - Fetch dari GET /api/dashboard/stats (T-121)
  - Greeting personal: "Selamat pagi, [Nama]"
  - KPI cards: real data
  - Jadwal hari ini: real data
  - Perlu review: pending actions
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §8
- **Test**: Login admin → dashboard → semua angka real
- **Files**: `frontend/src/app/dashboard/page.tsx`

### T-138: Dashboard Real Data — Guardian
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: UC-16
- **Deskripsi**:
  - Fetch dari GET /api/dashboard/guardian (T-121)
  - Greeting: "Selamat pagi, Pak/Bu [Nama]"
  - Jadwal minggu ini: real data
  - Tagihan: real data + CTA bayar
  - Absensi terakhir: real data
  - Kelayakan ujian: progress bar
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §8
- **Test**: Login wali murid → dashboard → semua data real
- **Files**: `frontend/src/app/dashboard/page.tsx`

### T-139: Dashboard Real Data — Teacher
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-16
- **Deskripsi**:
  - Fetch dari GET /api/dashboard/teacher (T-121)
  - Greeting: "Selamat pagi, [Nama]"
  - Jadwal hari ini: real data + tombol absen
  - Ringkasan minggu: real data
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §8
- **Test**: Login guru → dashboard → jadwal real
- **Files**: `frontend/src/app/dashboard/page.tsx`

### T-140: Empty States + Loading Skeletons
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - components/EmptyState.tsx (icon + text + CTA)
  - components/Skeleton.tsx (pulse animation)
  - components/ErrorState.tsx (icon + message + retry)
  - Apply ke semua halaman list
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §7
- **Test**: Belum ada data → empty state → klik CTA → form
- **Files**: `frontend/src/components/EmptyState.tsx`, `Skeleton.tsx`, `ErrorState.tsx`

### T-141: Notification Badge
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-05
- **Deskripsi**:
  - Badge merah di sidebar icon
  - Count: "3" (max 99+)
  - Bounce animation on new
  - Auto-clear saat notification page dibuka
  - Fetch: GET /api/notifications/unread-count
- **Test**: Ada notif baru → badge muncul → buka notif → badge hilang
- **Files**: `frontend/src/components/Sidebar.tsx`, `frontend/src/components/BottomNav.tsx`

### T-142: Search & Filter Components
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - components/SearchInput.tsx (debounced 300ms)
  - components/FilterChips.tsx (multi-select)
  - components/SortDropdown.tsx
  - Apply ke: students, sessions, invoices, audit logs
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §19
- **Test**: Ketik nama → filter muncul → sort → hasil berubah
- **Files**: `frontend/src/components/SearchInput.tsx`, `FilterChips.tsx`, `SortDropdown.tsx`

### T-143: Breadcrumbs Component
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - components/Breadcrumbs.tsx
  - Pattern: Dashboard > Jadwal > Detail Sesi
  - Click navigates back
  - Apply ke: detail pages
- **Test**: Buka detail → breadcrumb tampil → klik "Jadwal" → back
- **Files**: `frontend/src/components/Breadcrumbs.tsx`

### T-144: Error Boundary + Error Pages
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - ErrorBoundary component (catch render errors)
  - 401 page → redirect login
  - 403 page → "Anda tidak memiliki akses"
  - 404 page → "Halaman tidak ditemukan"
  - 500 page → "Terjadi kesalahan"
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §10
- **Test**: Akses /nonexistent → 404 page
- **Files**: `frontend/src/app/not-found.tsx`, `error.tsx`

### T-145: Frontend Upload Foto Profil (T-071)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-02
- **Deskripsi**:
  - Upload form di halaman Profile
  - Crop image sebelum upload
  - Preview sebelum submit
  - Backend sudah ada (POST /api/photos)
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §7
- **Test**: Klik foto → upload → crop → preview → simpan
- **Files**: `frontend/src/app/dashboard/profile/page.tsx`

### T-146: Guru Feedback Page (T-072)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-02
- **Deskripsi**:
  - Guru: form feedback ke wali murid
  - Wali murid: lihat feedback dari guru
  - Backend: T-125
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §7
- **Test**: Guru kirim feedback → wali murid lihat
- **Files**: `frontend/src/app/dashboard/feedback/page.tsx` (new)

### T-147: Wali Murid Rating Page (T-073)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-02
- **Deskripsi**:
  - Wali murid: rate guru (1-5 bintang)
  - Admin: lihat rating rata-rata per guru
  - Backend: T-126
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §7
- **Test**: Wali murid rate → admin lihat average
- **Files**: `frontend/src/app/dashboard/ratings/page.tsx` (new)

### T-148: Instruments Public Page (T-074)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - /instruments — public page (tanpa login)
  - List semua instrumen + harga + deskripsi
  - CTA: daftar via WhatsApp
  - Backend: GET /api/pricing (sudah ada)
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §9
- **Test**: Buka /instruments → lihat daftar → klik daftar
- **Files**: `frontend/src/app/instruments/page.tsx` (new)

### T-149: Help Page (T-075)
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - /help — public page
  - FAQ accordion
  - Contact info (WA, email, alamat)
  - Cara daftar, cara bayar, jadwal
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §9
- **Test**: Buka /help → lihat FAQ → expand → baca jawaban
- **Files**: `frontend/src/app/help/page.tsx` (new)

### T-155: About Page (/about)
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-01
- **Deskripsi**:
  - /about — public page
  - Hero: "Tentang BMD Music School"
  - Visi & Misi
  - Cerita Kami (founding story)
  - Guru-Guru Kami (foto + nama + spesialisasi)
  - Studio Kami (foto + deskripsi)
  - Prestasi (angka social proof)
  - CTA: Daftar via WhatsApp
  - Background: light, warm, professional
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §9
- **Test**: Buka /about → baca cerita → lihat guru → klik daftar
- **Files**: `frontend/src/app/about/page.tsx` (new)

### T-150: Reset Password Pages (T-076)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-01
- **Deskripsi**:
  - /reset-password/success → "Link berhasil dikirim"
  - /reset-password/error → "Link tidak valid/expired"
  - /reset-password/[token] → form password baru
  - Backend: sudah ada POST /api/auth/forgot-password
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §9
- **Test**: Lupa password → email → klik link → reset sukses
- **Files**: `frontend/src/app/reset-password/` (new pages)

### T-151: Mobile-First Refactor (T-077)
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - Audit semua halaman di 375px width
  - Fix overflow, text truncation, touch targets
  - Bottom nav: tambah badge notif
  - Forms: full-width inputs
  - Tables: convert ke card di mobile
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §4
- **Test**: Buka di HP 375px → semua halaman usable
- **Files**: Semua halaman frontend

### T-152: Animation & Micro-interactions
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - Page transition: fade (0.2s)
  - Card hover: lift shadow
  - Button click: scale 0.98
  - Modal: slide up
  - Toast: slide in
  - Theme toggle: crossfade
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §17
- **Test**: Hover card → shadow muncul → smooth
- **Files**: `frontend/src/app/globals.css`, components

### T-153: Pagination "Showing X of Y"
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - Tambah text "Menampilkan 1-20 dari 45 data"
  - Update Pagination component
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §20
- **Test**: List page → pagination → text info muncul
- **Files**: `frontend/src/components/Pagination.tsx`

### T-154: Print Styles
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - @media print CSS
  - Hide: sidebar, nav, buttons
  - Show: content only
  - Black on white
  - Referensi: design/UI_UX_DESIGN_SYSTEM.md §21
- **Test**: Ctrl+P → print preview → content only
- **Files**: `frontend/src/app/globals.css`

---


---

## Phase 23: LANDING PAGE + REGISTRATION REBUILD

### T-156: Landing Page — Final Rebuild (Design Synthesis)
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-16, UC-03
- **Deskripsi**:
  - Rebuild landing page sesuai design synthesis (Yousician + Fender + Masterclass)
  - Split hero (text kiri, studio photo kanan)
  - Warm gradient (gold → orange)
  - Typography: bold (bukan extrabold uppercase)
  - 7 sections: Navbar, Hero, Instrumen, Tentang, Testimoni, Harga, CTA, Footer
  - Hapus: FaqSection, KenapaBMDSection, StudioTourSection
  - Tambah: TentangSection (teacher profiles + studio gallery)
  - WhatsApp HANYA untuk Booking Studio (footer)
  - Semua CTA utama → /register-course
- **Test**: Landing page match design synthesis, no WhatsApp CTA except footer
- **Files**: `frontend/src/app/page.tsx`, `frontend/src/components/landing/*.tsx`

### T-157: Pricing Page — Per Instrumen + Tipe Kelas
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-06, FR-16
- **Deskripsi**:
  - Hapus paket Trial/Reguler/Premium dari landing page
  - Ganti dengan tabel harga per instrumen + tipe kelas
  - Data dari SRS: Registration Fee + SPP per bulan
  - Tampilkan: Private, Intensif (2 siswa), Semi Intensif (4 siswa), Expatriate
  - Instrumen: Piano Klasik, Piano Pop, Vokal, Biola, Gitar, Keyboard, Drum, Flute, Saxophone
  - Registration Fee terpisah dari SPP
  - Contoh: "Piano Klasik — Private: Rp 420.000/bulan | Registration Fee: Rp 420.000"
- **Test**: Harga match SRS, per instrumen + tipe kelas, bukan paket Trial/Reguler/Premium
- **Files**: `frontend/src/components/landing/PaketHargaSection.tsx` (rename to HargaSection.tsx)

### T-158: Registration Flow — Self-Service Wali Murid
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-02, UC-02
- **Deskripsi**:
  - Alur: Data diri → Pilih instrumen + tipe kelas → Pilih jadwal → Upload bukti transfer → Submit
  - Step 1: Form data diri Wali Murid (nama, email, HP, alamat)
  - Step 2: Pilih instrumen + tipe kelas (Private, Intensif, dll)
  - Step 3: Tampilkan jadwal yang tersedia (fetch dari API)
  - Step 4: Tampilkan rekening bank + total biaya (Registration Fee + SPP)
  - Step 5: Upload bukti transfer (image)
  - Step 6: Submit → Status "Menunggu Verifikasi"
  - Step 7: Admin verifikasi → Status "Active" → Notifikasi
  - Frontend: Multi-step form dengan progress indicator
  - Backend: POST /api/v1/registrations/ (create registration request)
  - Backend: GET /api/v1/schedules/available (slot kosong)
- **Test**: Wali Murid bisa daftar, pilih instrumen, pilih jadwal, upload bukti, submit
- **Files**: `frontend/src/app/register-course/page.tsx`, `backend/app/routers/registrations.py`

### T-159: Registration API — Backend Endpoint
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-02, UC-02
- **Deskripsi**:
  - POST /api/v1/registrations/ — buat registrasi baru
  - GET /api/v1/registrations/ — list semua registrasi (admin)
  - PUT /api/v1/registrations/{id}/verify — verifikasi pembayaran (admin)
  - GET /api/v1/schedules/available — slot jadwal yang tersedia
  - Model: Registration (id, guardian_data, instrument, class_type, schedule_id, payment_proof, status, created_at)
  - Status flow: pending → verified → active (atau rejected)
  - Simpan bukti transfer di /uploads/registration/
  - Audit log untuk setiap verifikasi
- **Test**: Registrasi berhasil dibuat, admin bisa verifikasi, audit log tercatat
- **Files**: `backend/app/models/registration.py`, `backend/app/routers/registrations.py`

### T-160: Hapus Studio Rental dari System
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-08
- **Deskripsi**:
  - Hapus semua referensi studio rental dari frontend (kecuali footer WhatsApp button)
  - Footer: "Booking Studio" button → WhatsApp (tetap ada)
  - Hapus StudioTourSection (sudah merged ke TentangSection)
  - Hapus referensi "sewa studio" di copywriting
  - Pastikan tidak ada halaman/route untuk studio booking
- **Test**: Tidak ada studio rental di UI, footer WhatsApp button tetap ada
- **Files**: `frontend/src/components/landing/Footer.tsx`

### T-161: Hapus Trial dari Landing Page + Database
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-06
- **Deskripsi**:
  - Hapus paket Trial dari landing page (PaketHargaSection)
  - Tetap ada status `trial` di database (untuk admin manual)
  - Hapus copywriting "Trial Gratis" dari hero, CTA, pricing
  - Ganti dengan "Registration Fee + SPP per bulan"
  - Admin tetap bisa set enrollment_status = "trial" via dashboard
- **Test**: Tidak ada "Trial" di landing page, tetap ada di database enum
- **Files**: `frontend/src/components/landing/PaketHargaSection.tsx`, `frontend/src/components/landing/HeroSection.tsx`

### T-162: Copywriting Rewrite — Benefit > Feature
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-USA-02
- **Deskripsi**:
  - Rewrite semua copywriting sesuai design/copywriting-rewrite.md
  - Hero: "Anak Anda Punya Bakat Musik? Kami Bantu Mengembangkannya."
  - Instrumen: benefit-focused descriptions
  - Tentang: story-driven, bukan deskripsi fitur
  - Testimoni: transformation stories, bukan feature quotes
  - Harga: "Investasi untuk Masa Depan Anak"
  - CTA: "Siap Memberikan yang Terbaik untuk Anak?"
  - Footer: "Booking Studio" (WhatsApp) + "Daftar Kursus" (register)
- **Test**: Copywriting match copywriting-rewrite.md, benefit > feature
- **Files**: Semua file landing page components

---


---

## Phase 24: GUARDIAN + TEACHER FLOW FIX

### T-163: Student CRUD — Backend API
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-02, UC-02
- **Deskripsi**:
  - POST /api/v1/students/ — tambah anak (guardian only)
  - GET /api/v1/students/ — list anak (guardian: milik sendiri, admin: semua)
  - PUT /api/v1/students/{id} — edit anak (guardian/admin)
  - DELETE /api/v1/students/{id} — soft delete
  - Model Student: id_guardian, full_name (encrypted), date_of_birth, instrument (ENUM), level (ENUM), enrollment_status
  - Constraint: 1 guardian bisa punya N students
  - Constraint: 1 student = 1 instrument
  - PII: full_name wajib AES-256-GCM
- **Test**: Guardian bisa CRUD anak, admin bisa lihat semua
- **Files**: `backend/app/models/student.py`, `backend/app/routers/students.py`, `backend/app/schemas/student.py`

### T-164: Anak Saya Page — Guardian Dashboard
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-02, UC-02
- **Deskripsi**:
  - /dashboard/my-students — halaman CRUD anak untuk guardian
  - List semua anak (nama, instrumen, level, status)
  - Form tambah anak: nama, tgl lahir, instrumen (dropdown), level (dropdown)
  - Edit anak: update data
  - Hapus anak: soft delete dengan konfirmasi
  - Fetch dari API: GET /api/v1/students/ (tanpa id, auto-filter by guardian)
- **Test**: Guardian bisa tambah/edit/hapus anak, data muncul di dashboard
- **Files**: `frontend/src/app/dashboard/my-students/page.tsx`

### T-165: Register Course Flow Fix — Multi-Student
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: FR-02, UC-02
- **Deskripsi**:
  - Ubah flow register-course:
    - Step 1: Data Diri Wali Murid
    - Step 2: Buat Akun
    - Step 3: Tambah Anak (bisa lebih dari 1, per anak: nama, instrumen, level)
    - Step 4: Pilih Jadwal (per anak)
    - Step 5: Upload Bukti Transfer
    - Step 6: Selesai
  - Backend: buat User + Guardian + Students (N records) dalam 1 request
  - Setiap Student punya instrument + level sendiri
- **Test**: Wali Murid bisa daftar dengan 3 anak, masing-masing instrumen berbeda
- **Files**: `frontend/src/app/register-course/page.tsx`, `backend/app/routers/registrations.py`

### T-166: Admin Assign Student ke Session
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-03, UC-03
- **Deskripsi**:
  - Admin bisa assign Student → Session (teacher + room + jadwal)
  - GET /api/v1/sessions/available-slots — slot kosong
  - POST /api/v1/sessions/assign — assign student ke slot
  - Admin page: drag-drop atau dropdown untuk assign
  - Constraint: 1 student per session, no overlap
- **Test**: Admin assign student ke session, jadwal muncul di dashboard guru + wali
- **Files**: `backend/app/routers/scheduling.py`, `frontend/src/app/dashboard/schedule/page.tsx`

### T-167: Invoice Per Guardian — Aggregate Billing
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-06, UC-06
- **Deskripsi**:
  - Invoice per Guardian (bukan per Student)
  - Aggregate: Registration Fee + SPP dari semua Student aktif
  - unique_code 3 digit per invoice (untuk verifikasi transfer)
  - Auto-generate tanggal 5 setiap bulan (cron job)
  - Dashboard wali: lihat semua invoice + status
- **Test**: Invoice ter-generate untuk guardian dengan 3 anak, total = sum SPP
- **Files**: `backend/app/services/billing.py`, `frontend/src/app/dashboard/my-payments/page.tsx`

### T-168: Dashboard Wali Murid — Fix Menu & Pages
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-02, UC-02
- **Deskripsi**:
  - Pastikan menu wali murid benar:
    - Jadwal Anak (sessions semua anak)
    - Anak Saya (CRUD student)
    - Pembayaran (invoices + bayar)
    - Beri Nilai (rate teacher)
  - Hapus menu yang tidak relevan untuk wali murid
  - Pastikan redirect setelah login benar
- **Test**: Login sebagai wali_01 → dashboard wali murid → menu lengkap
- **Files**: `frontend/src/components/Sidebar.tsx`, `frontend/src/app/dashboard/`

---


---

## Phase 25: COMPREHENSIVE SYSTEM AUDIT

### T-169: Business Logic Audit — SRS Compliance Check
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: ALL
- **Deskripsi**:
  - Baca semua SRS documents (srs/*.md)
  - Bandingkan setiap FR/UC dengan implementasi aktual
  - Catat: missing features, wrong implementation, incomplete flows
  - Output: SRS_COMPLIANCE_MATRIX.md (FR-xx → status: implemented/partial/missing)
  - Focus: registration flow, billing, attendance, notification, reporting
- **Test**: Setiap FR/UC punya status yang jelas
- **Files**: `docs/audit/SRS_COMPLIANCE_MATRIX.md`

### T-170: Business Logic Audit — Data Model vs Business Rules
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: ALL
- **Deskripsi**:
  - Baca 99-entities.md dan 99-addendum-business-rules.md
  - Bandingkan setiap entity field dengan model SQLAlchemy
  - Catat: missing fields, wrong types, missing constraints, missing enums
  - Focus: Student (instrument/level enum), Invoice (unique_code), Session (status), Guardian (account_status)
  - Output: DATA_MODEL_AUDIT.md
- **Test**: Setiap entity field match SRS
- **Files**: `docs/audit/DATA_MODEL_AUDIT.md`

### T-171: Technical Audit — API Endpoint Completeness
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: ALL
- **Deskripsi**:
  - List semua endpoint di backend (router files)
  - List semua fetch call di frontend
  - Bandingkan: frontend butuh endpoint X → backend punya?
  - Catat: missing endpoints, wrong paths, wrong methods, missing auth
  - Output: API_ENDPOINT_AUDIT.md
- **Test**: Setiap frontend fetch punya backend endpoint
- **Files**: `docs/audit/API_ENDPOINT_AUDIT.md`

### T-172: Technical Audit — Frontend-Backend Alignment
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: ALL
- **Deskripsi**:
  - Bandingkan request/response format antara frontend dan backend
  - Catat: field name mismatch, type mismatch, missing fields, extra fields
  - Focus: students, teachers, sessions, invoices, notifications
  - Output: FRONTEND_BACKEND_ALIGNMENT.md
- **Test**: Setiap API call punya response yang sesuai
- **Files**: `docs/audit/FRONTEND_BACKEND_ALIGNMENT.md`

### T-173: Security Audit — Authentication & Authorization
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **FR/UC terkait**: SEC-01..SEC-08
- **Deskripsi**:
  - Cek semua endpoint: punya auth guard? Role check benar?
  - Cek: JWT validation, token expiry, refresh token flow
  - Cek: PII encryption (AES-256-GCM) untuk field yang wajib
  - Cek: input validation, SQL injection, XSS protection
  - Cek: rate limiting pada endpoint kritis
  - Output: SECURITY_AUDIT.md
- **Test**: Tidak ada endpoint tanpa auth, tidak ada PII tanpa encrypt
- **Files**: `docs/audit/SECURITY_AUDIT.md`

### T-174: Business Logic Audit — User Flow Verification
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: UC-01..UC-20
- **Deskripsi**:
  - Test setiap user flow dari awal sampai akhir:
    - Guardian: register → tambah anak → bayar → lihat jadwal
    - Teacher: login → lihat jadwal → submit absensi → feedback
    - Admin: login → manage siswa → manage guru → verifikasi bayar → assign sesi
    - Owner: login → lihat reports → manage admin → audit log
  - Catat: broken flows, missing steps, wrong redirects
  - Output: USER_FLOW_VERIFICATION.md
- **Test**: Setiap flow bisa dijalankan end-to-end
- **Files**: `docs/audit/USER_FLOW_VERIFICATION.md`

### T-175: Technical Audit — Error Handling & Edge Cases
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: NFR-REL-01, NFR-REL-02
- **Deskripsi**:
  - Cek semua error handling di frontend dan backend
  - Catat: unhandled errors, missing try-catch, wrong error messages
  - Cek edge cases: empty data, network error, timeout, unauthorized
  - Cek: loading states, empty states, error states di UI
  - Output: ERROR_HANDLING_AUDIT.md
- **Test**: Semua error punya handling yang benar
- **Files**: `docs/audit/ERROR_HANDLING_AUDIT.md`

### T-176: Business Logic Audit — Pricing & Billing Accuracy
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **FR/UC terkait**: FR-06, UC-06
- **Deskripsi**:
  - Bandingkan pricing di landing page, register-course, dashboard, database
  - Cek: registration fee, SPP, class type multiplier, instrument pricing
  - Cek: invoice generation logic (cron job), unique_code, due_date, fine
  - Cek: payment verification flow, receipt upload, status update
  - Output: PRICING_BILLING_AUDIT.md
- **Test**: Harga konsisten di semua tempat
- **Files**: `docs/audit/PRICING_BILLING_AUDIT.md`

### T-177: Technical Audit — Performance & Optimization
- **Status**: ✅
- **Priority**: 🟢 NICE
- **FR/UC terkait**: NFR-PERF-01, NFR-PERF-02
- **Deskripsi**:
  - Cek: N+1 queries, missing indexes, eager loading
  - Cek: bundle size, lazy loading, code splitting
  - Cek: API response time (target ≤2s page, ≤1s validation)
  - Cek: image optimization, font loading, CSS efficiency
  - Output: PERFORMANCE_AUDIT.md
- **Test**: Response time sesuai NFR target
- **Files**: `docs/audit/PERFORMANCE_AUDIT.md`

---


---

## Phase 26: DEEP AUDIT — FRONTEND vs BACKEND SEPARATED

### T-178: Backend Audit — Router & Endpoint Integrity
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Deskripsi**:
  - Audit SETIAP router file di backend/app/routers/*.py
  - Per router: list semua endpoint, method, auth guard, request/response schema
  - Cek: prefix consistency (/api/v1/ di semua router)
  - Cek: role guard benar (admin vs owner vs teacher vs guardian)
  - Cek: request validation (Pydantic schema lengkap)
  - Cek: response format konsisten (data/total/limit/offset)
  - Cek: error handling (HTTPException dengan detail message)
  - Cek: audit log pada semua mutation (INSERT/UPDATE/DELETE)
  - Output: BACKEND_ROUTER_AUDIT.md per file
- **Test**: Setiap endpoint punya auth, validation, error handling, audit log
- **Files**: backend/app/routers/*.py, docs/audit/backend/

### T-179: Backend Audit — Model & Migration Integrity
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Deskripsi**:
  - Audit SETIAP model file di backend/app/models/*.py
  - Per model: list semua field, type, constraint, enum, FK, index
  - Cek: enum values match SRS (instrument, level, status, dll)
  - Cek: PII fields encrypted (AES-256-GCM)
  - Cek: soft delete (deleted_at) pada semua entity
  - Cek: timestamp fields (created_at, updated_at)
  - Cek: foreign key relationships benar
  - Cek: unique constraints sesuai SRS
  - Output: BACKEND_MODEL_AUDIT.md per file
- **Test**: Setiap model field match SRS, PII encrypted, constraints benar
- **Files**: backend/app/models/*.py, docs/audit/backend/

### T-180: Backend Audit — Service & Business Logic
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Deskripsi**:
  - Audit SETIAP service file di backend/app/services/*.py
  - Per service: list semua function, input/output, business rules
  - Cek: billing logic (auto-invoice, unique_code, fine, freeze)
  - Cek: scheduling logic (conflict detection, slot validation)
  - Cek: attendance logic (GPS validation, photo upload, review)
  - Cek: notification logic (WhatsApp/email template, retry)
  - Cek: encryption logic (AES-256-GCM, blind index)
  - Cek: audit log logic (before/after state, user_id)
  - Output: BACKEND_SERVICE_AUDIT.md per file
- **Test**: Setiap business rule match SRS, error handling lengkap
- **Files**: backend/app/services/*.py, docs/audit/backend/

### T-181: Backend Audit — Security & Auth Flow
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Deskripsi**:
  - Audit auth.py: login, register, refresh, logout, forgot-password
  - Cek: JWT token flow (access + refresh + expiry)
  - Cek: password hashing (bcrypt)
  - Cek: rate limiting pada semua auth endpoint
  - Cek: brute force protection (failed_login_attempts, locked_until)
  - Cek: PII encryption pada semua sensitive field
  - Cek: CORS configuration
  - Cek: SQL injection protection (SQLAlchemy ORM)
  - Output: BACKEND_SECURITY_DEEP_AUDIT.md
- **Test**: Tidak ada vulnerability, auth flow lengkap
- **Files**: backend/app/routers/auth.py, backend/app/middleware/, docs/audit/backend/

### T-182: Frontend Audit — Page & Component Structure
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Deskripsi**:
  - Audit SETIAP page di frontend/src/app/dashboard/*/page.tsx
  - Per page: list semua fetch calls, state management, error handling
  - Cek: API path benar (/api/v1/ prefix)
  - Cek: token key konsisten (access_token bukan token)
  - Cek: error handling (try-catch + error state + loading state)
  - Cek: empty state (komponen ketika data kosong)
  - Cek: loading state (skeleton/spinner)
  - Cek: role guard (redirect jika role salah)
  - Output: FRONTEND_PAGE_AUDIT.md per page
- **Test**: Setiap page punya error handling, loading, empty state
- **Files**: frontend/src/app/dashboard/*/page.tsx, docs/audit/frontend/

### T-183: Frontend Audit — API Integration & Data Flow
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Deskripsi**:
  - Audit SETIAP fetch call di frontend
  - Per call: URL, method, headers, request body, response handling
  - Cek: response parsing (JSON field names match backend)
  - Cek: error response handling (4xx, 5xx)
  - Cek: pagination handling (total, limit, offset)
  - Cek: filter/search handling (query params)
  - Cek: file upload handling (FormData)
  - Cek: token refresh on 401
  - Output: FRONTEND_API_INTEGRATION_AUDIT.md
- **Test**: Setiap fetch call punya proper error handling dan response parsing
- **Files**: frontend/src/app/**, docs/audit/frontend/

### T-184: Frontend Audit — UI/UX Consistency & Design System
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**:
  - Audit SETIAP page dan component terhadap design/DESIGN.md
  - Per component: colors, typography, spacing, border radius, shadow
  - Cek: warna konsisten (blue #2563EB, gold #F59E0B, gray scale)
  - Cek: font konsisten (Plus Jakarta Sans headings, Inter body)
  - Cek: spacing konsisten (4px grid system)
  - Cek: responsive design (mobile-first, breakpoints)
  - Cek: dark mode remnants (harusnya sudah dihapus)
  - Cek: Bahasa Indonesia pada semua UI text
  - Output: FRONTEND_UI_CONSISTENCY_AUDIT.md
- **Test**: Setiap component match DESIGN.md, responsive, no dark mode
- **Files**: frontend/src/components/*.tsx, frontend/src/app/**/page.tsx, docs/audit/frontend/

### T-185: Frontend Audit — State Management & Data Sync
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **Deskripsi**:
  - Audit state management di semua page
  - Cek: DashboardContext usage (refreshKey, triggerRefresh)
  - Cek: localStorage usage (access_token, role, theme)
  - Cek: useState vs useContext usage
  - Cek: data sync antar page (apakah refresh bekerja?)
  - Cek: stale data handling (kapan refetch?)
  - Cek: optimistic update (jika ada)
  - Output: FRONTEND_STATE_MANAGEMENT_AUDIT.md
- **Test**: Data sinkron antar page, tidak ada stale data
- **Files**: frontend/src/components/DashboardContext.tsx, frontend/src/app/**/page.tsx, docs/audit/frontend/

---


---

## Phase 27: AUDIT FIXES — BACKEND (Per Bagian)

### T-186: Fix Student Level System — Per Instrumen
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Source**: T-179 (BACKEND_MODEL_AUDIT.md)
- **Deskripsi**:
  - SRS Addendum Section 6: level HARUS per instrumen
  - Current: Enum("pemula", "dasar", "menengah", "lanjut") — SALAH
  - Fix: Remove enum constraint, use String(50)
  - Add validation per instrument:
    - Piano Klasik: pra_elementary, elementary, grade_1..6
    - Piano Pop: beginner, intermediate, intermediate_2
    - Vokal/Biola/Gitar/Keyboard/Drum/Flute/Saxophone: basic, intermediate, advance
  - Update registration.py, students.py to accept new levels
  - Update seed scripts
  - DB migration: ALTER TYPE or remove enum constraint
- **Test**: Register student with Piano Klasik grade_3 → success
- **Files**: backend/app/models/student.py, backend/app/routers/students.py, backend/app/routers/registrations.py

### T-187: Add bass to Instrument Enum
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Source**: T-179 (BACKEND_MODEL_AUDIT.md)
- **Deskripsi**:
  - SRS Addendum Section 5: tambah bass
  - Current enum: biola, piano, gitar, drum, keyboard, flute, saxophone, vokal, ensemble
  - Fix: ALTER TYPE instrument_type ADD VALUE 'bass'
  - Update all routers that validate instrument
  - Update seed scripts
- **Test**: Register student with instrument=bass → success
- **Files**: backend/app/models/student.py, backend/app/routers/students.py

### T-188: Add Audit Log to 5 Missing Routers
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Source**: T-178 (BACKEND_ROUTER_AUDIT.md)
- **Deskripsi**:
  - Routers missing audit log on mutations:
    - draft.py — POST/DELETE
    - holidays.py — POST/DELETE
    - multi_student.py — POST freeze
    - privacy.py — POST/PUT
    - teachers.py — POST/PUT/DELETE
  - Fix: Add create_audit_log() to all mutation endpoints
  - Format: create_audit_log(db, user.id_user, "INSERT"/"UPDATE"/"DELETE", entity, entity_id, before_state, after_state)
- **Test**: Create teacher → audit_log entry created
- **Files**: backend/app/routers/draft.py, holidays.py, multi_student.py, privacy.py, teachers.py

### T-189: Add Indexes to Foreign Keys
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **Source**: T-179 (BACKEND_MODEL_AUDIT.md)
- **Deskripsi**:
  - Models missing FK indexes:
    - attendance (id_session, id_student)
    - invoice (id_guardian)
    - payment (id_invoice)
    - session (id_teacher, id_student, id_room)
    - student (id_guardian)
  - Fix: Add index=True to all FK fields
  - DB migration: CREATE INDEX CONCURRENTLY
- **Test**: Query with FK filter uses index (EXPLAIN ANALYZE)
- **Files**: backend/app/models/attendance.py, invoice.py, payment.py, session.py, student.py

### T-190: Add Soft Delete to Missing Models
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **Source**: T-179 (BACKEND_MODEL_AUDIT.md)
- **Deskripsi**:
  - Models missing soft delete:
    - teacher, room, session, feedback, rating
  - Fix: Add deleted_at column, update queries to filter deleted_at IS NULL
  - Add DELETE endpoints that set deleted_at instead of hard delete
- **Test**: Delete teacher → deleted_at set, list excludes deleted
- **Files**: backend/app/models/teacher.py, room.py, session.py, feedback.py, rating.py

### T-191: Move Router Prefix to Router Definition
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **Source**: T-178 (BACKEND_ROUTER_AUDIT.md)
- **Deskripsi**:
  - Routers with prefix in main.py:
    - cron_trigger.py — move prefix to router
    - photos.py — move prefix to router
  - Fix: Add prefix= to APIRouter() in router file, remove from main.py
- **Test**: All endpoints still work after prefix move
- **Files**: backend/app/routers/cron_trigger.py, photos.py, backend/app/main.py

### T-192: Add Rate Limiting to CRUD Endpoints
- **Status**: ✅
- **Priority**: 🟡 IMPORTANT
- **Source**: T-181 (BACKEND_SECURITY_DEEP_AUDIT.md)
- **Deskripsi**:
  - Endpoints missing rate limiting:
    - students POST/PUT/DELETE
    - teachers POST/PUT/DELETE
    - sessions POST/PUT/DELETE
    - billing POST/PUT/DELETE
  - Fix: Add @limiter.limit() to all write endpoints
  - Limits: 20/minute for POST, 10/minute for PUT/DELETE
- **Test**: Spam POST → 429 Too Many Requests after limit
- **Files**: backend/app/routers/students.py, teachers.py, scheduling.py, billing.py

---

## Phase 28: AUDIT FIXES — FRONTEND (Per Bagian)

### T-193: Add Error Handling to 11 Pages
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Source**: T-182 (FRONTEND_PAGE_AUDIT.md)
- **Deskripsi**:
  - Pages without error handling:
    - attendance, billing, my-payments, my-schedule
    - notification-settings, notifications, payments, reports
    - students, studio
  - Fix per page:
    1. Add try-catch to fetch calls
    2. Add error state (setError)
    3. Add loading state (setLoading)
    4. Add error display (ErrorState component)
    5. Use parseApiError() for error messages
- **Test**: Network error → shows error message, not crash
- **Files**: frontend/src/app/dashboard/*/page.tsx (11 files)

### T-194: Add refreshKey to All Data-Fetching Pages
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Source**: T-185 (FRONTEND_STATE_MANAGEMENT_AUDIT.md)
- **Deskripsi**:
  - Pages not using refreshKey: 22 pages
  - Fix per page:
    1. Import useDashboard
    2. Add const { refreshKey } = useDashboard()
    3. Add refreshKey to useEffect dependency
  - Pages: schedule, students, teachers, billing, payments, notifications, my-schedule, my-payments, admin, assign-session, etc.
- **Test**: Add student on my-students → schedule page shows updated data
- **Files**: frontend/src/app/dashboard/*/page.tsx (22 files)

### T-195: Add triggerRefresh to All Data-Mutating Pages
- **Status**: ✅
- **Priority**: 🔴 KRITIS
- **Source**: T-185 (FRONTEND_STATE_MANAGEMENT_AUDIT.md)
- **Deskripsi**:
  - Pages that mutate data but don't triggerRefresh:
    - assign-session (add session)
    - billing (verify payment)
    - registrations (approve/reject)
    - override (override attendance)
    - holidays (add/delete holiday)
    - feedback (add feedback)
    - ratings (add rating)
  - Fix: Add triggerRefresh() after successful mutation
- **Test**: Approve registration → students page shows new student
- **Files**: 7 page files

### T-196: Add Responsive Breakpoints to All Pages
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **Source**: T-184 (FRONTEND_UI_CONSISTENCY_AUDIT.md)
- **Deskripsi**:
  - Only 21 responsive breakpoint classes found
  - Fix: Add md:/lg: classes to all page layouts
  - Focus: sidebar collapse on mobile, table → cards on mobile
  - Use MobileTable component on all list pages
- **Test**: View on mobile (320px) → layout works, no overflow
- **Files**: frontend/src/app/dashboard/*/page.tsx

### T-197: Use Shared Components on All Pages
- **Status**: ⏳
- **Priority**: 🟡 IMPORTANT
- **Source**: T-184 (FRONTEND_UI_CONSISTENCY_AUDIT.md)
- **Deskripsi**:
  - Components exist but underused:
    - EmptyState — use on all list pages when data empty
    - Skeleton — use on all pages during loading
    - ErrorState — use on all pages on error
  - Fix: Add shared components to all pages
- **Test**: Empty data → shows EmptyState, Loading → shows Skeleton, Error → shows ErrorState
- **Files**: frontend/src/app/dashboard/*/page.tsx

---

## ATURAN WAJIB (berlaku untuk SEMUA task)

1. **DILARANG dummy data di frontend** — Semua page HARUS fetch dari API. Tidak ada `const data = [...]` di component.
2. **DILARANG local state untuk data** — Semua data berasal dari `fetch('/api/...')`. State hanya untuk UI (loading, error, modal).
3. **Setiap button HARUS panggil API** — Verifikasi, Tolak, Simpan, Hapus = semua `fetch(method: 'POST/PUT/DELETE')`.
4. **Fallback dummy HANYA jika API error** — `catch(() => setDummy)` hanya sebagai safety net, bukan primary data.
5. **Studio coordinates**: -7.737428, 110.429669 (Seturan, Yogyakarta)
