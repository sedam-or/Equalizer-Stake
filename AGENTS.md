# Equalizer-Stake — BMD Musik Web App

## Ringkasan Proyek
Sistem Penjadwalan Kursus & Reservasi Studio Berbasis Web untuk BMD Music School, Yogyakarta.
Stack: FastAPI + Next.js (PWA) + PostgreSQL + Redis + Docker.
SRS: ISO/IEC/IEEE 29148:2018 | UU PDP No. 27/2022

## Dokumen Lengkap
- **SRS Modular** (ter-trace per fitur): `srs/` — 00-overview sampai 99-uc-diagrams
- **SRS Asli** (BAB utuh): `docs/SRS_Equalizer-Stake_*.md`
- **System Architecture**: `architecture/` — 00-system-overview sampai 06-deployment
- **ADRs**: `architecture/adr/` — 5 keputusan arsitektur
- Role Definitions: `ROLES.md`
- Task List: `tasks/TASKS.md`
- Task Reports: `tasks/report_tasks.md`
- Task Workflow: `TASK_WORKFLOW.md`

## Aturan Utama
1. Selalu baca `srs/` modular sebelum mengerjakan task (bukan BAB utuh di docs/).
2. Ikuti arsitektur di `architecture/` untuk implementasi teknis.
3. Ikuti role yang ditentukan di `ROLES.md`.
4. Sebelum rollback, cek `tasks/report_tasks.md` untuk task dependencies.
5. **SETIAP task selesai → tulis laporan ke `tasks/report_tasks.md`**.
6. **SETIAP task → ikuti TASK_WORKFLOW.md** (12-step).
7. **Keamanan**: AES-256-GCM PII, TLS 1.2+, JWT HS256, UU PDP compliance.
8. **Audit Log**: Immutable, 7 tahun. Jika log gagal → rollback.
9. **Traceability**: Setiap kode harus trace ke FR-xx/UC-xx dari SRS.

## SRS Context (Quick Reference)
- **4 Roles**: Owner (read-only dashboard), Admin Operasional (harian), Guru Musik (absensi), Wali Murid (1:N siswa)
- **7 Modules**: Penjadwalan, Penagihan, Kehadiran, Notifikasi, Pelaporan, Audit/Governance, Dokumentasi
- **15 FRs**: FR-01..FR-15 (13 Approved, 2 Proposed)
- **20 UCs**: UC1..UC20 across 7 modules
- **11 Entities**: User, Guardian, Student, Teacher, Room, Session, Invoice, Payment, Attendance, Notification, AuditLog
- **Billing**: Auto-invoice tanggal 5, denda Rp100k tanggal 11, freeze akun
- **Attendance**: GPS 150m, face-api.js client-side ≥30%, server verify
- **Notif**: WhatsApp+email, retry 3x (10/20/40s), circuit breaker
- **MVP**: 10 minggu, ≤50 concurrent users, PWA, single location (Seturan Yogya)
