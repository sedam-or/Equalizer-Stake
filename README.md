# Equalizer-Stake

Deterministic Course & Studio Reservation Engine

---

## Ringkasan

Equalizer-Stake menggantikan operasional manual (catatan kertas, spreadsheet, WhatsApp) dengan web app terotomasi berbasis PWA.

**Masalah yang dipecahkan:**
- Double-booking 3-5x/bulan → 0
- No-show 20-30% → ≤8%
- Admin kerja manual 2-3 jam/hari → ≤45 menit
- Invoice manual penuh error → otomatis + kode unik
- Tidak ada audit trail → immutable log 7 tahun

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Python 3.12+ / FastAPI / SQLAlchemy async / Pydantic v2 |
| Frontend | Next.js 15 / React 18 / Tailwind CSS (PWA) |
| Database | PostgreSQL 16 (asyncpg) |
| Cache | Redis 7 (sessions, locking, rate limit, queue) |
| Auth | JWT HS256, role-based (Owner/Admin/Teacher/Guardian) |
| Notification | WhatsApp Gateway + SMTP email fallback |
| Storage | Object storage (attendance photos) |
| Deploy | Docker, cloud Jakarta region, auto-scaling |

---

## Fitur Utama (7 Modul)

1. **Penjadwalan** — Real-time conflict prevention, concurrency control, libur nasional, minggu ke-5
2. **Penagihan** — Auto-invoice tanggal 5, kode unik 3 digit, denda Rp100k tanggal 11, freeze akun
3. **Kehadiran** — Foto + GPS 150m + face-api.js client-side ≥30%, server verify
4. **Notifikasi** — WhatsApp + email, retry 3x, circuit breaker, fallback otomatis
5. **Pelaporan** — Dashboard Owner read-only, laporan bulanan, ekspor cetak
6. **Audit** — Immutable log, 7 tahun retensi, PII anonimisasi setelah 2 tahun
7. **Dokumentasi** — Admin manual, instructor quick guide, Guardian FAQ, tooltip kontekstual

---

## Compliance

- **UU PDP No. 27/2022** — Minimasi data, hak penghapusan, keamanan data, transfer data
- **OWASP Top 10 (2021)** — Mitigasi A02..A07
- **ISO 27001:2022** — Kontrol keamanan informasi
- **WCAG 2.1 Level A** — Aksesibilitas web

---

## Quick Start

```bash
# Backend
cd backend
pip install -r requirements.txt
python -m uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload

# Frontend
cd frontend
npm install --legacy-peer-deps
npm run dev

# Database (PostgreSQL 16 — langsung dari awal)
cd backend
alembic upgrade head

# Docker (full stack — PostgreSQL + Redis + Backend + Frontend)
docker-compose up -d
```

---

## Dokumen

| Dokumen | Lokasi |
|---------|--------|
| SRS Modular (per fitur) | `srs/` |
| SRS Asli (BAB utuh) | `docs/SRS_Equalizer-Stake_*.md` |
| System Architecture | `architecture/` |
| ADRs | `architecture/adr/` |
| Task List | `tasks/TASKS.md` |
| Task Reports | `tasks/report_tasks.md` |
| Workflow | `TASK_WORKFLOW.md` |
| Design System | `design/` |

---

## Tim

Kelompok 7 — Program Studi Rekayasa Keamanan Siber, 2026
