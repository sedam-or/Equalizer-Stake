# docs/ — Equalizer-Stake SRS Documents

## SRS Documents (ISO/IEC/IEEE 29148:2018)

| BAB | File | Deskripsi |
|-----|------|-----------|
| BAB 1 | `SRS_Equalizer-Stake_Part1_BAB1.md` | Pendahuluan: tujuan, scope, glosarium, referensi |
| BAB 2 | `SRS_Equalizer-Stake_BAB2.md` | Deskripsi Umum: AS-IS/TO-BE, modul, user, constraints |
| BAB 3 | `part3_bab3.md` | Kebutuhan Spesifik: 15 FRs, NFRs, external interfaces |
| BAB 4 | `SRS_Equalizer-Stake_BAB4.md` | Model Analisis: 20 UCs, UML diagrams, state machines |
| BAB 5-7 | `SRS_Equalizer_BAB5_7_v2.md` | ERD (11 entities), traceability, lampiran |

## Additional

| File | Deskripsi |
|------|-----------|
| `security-audit.md` | Security audit findings & recommendations |

## Quick Reference

- **System**: Equalizer-Stake — BMD Music School, Yogyakarta
- **Stack**: FastAPI + Next.js (PWA) + PostgreSQL + Redis + Docker
- **4 Roles**: Owner, Admin Operasional, Guru Musik, Wali Murid
- **7 Modules**: Penjadwalan, Penagihan, Kehadiran, Notifikasi, Pelaporan, Audit, Dokumentasi
- **15 FRs**: FR-01..FR-15 (13 Approved, 2 Proposed)
- **20 UCs**: UC1..UC20
- **11 Entities**: User, Guardian, Student, Teacher, Room, Session, Invoice, Payment, Attendance, Notification, AuditLog
- **Compliance**: UU PDP No. 27/2022, OWASP Top 10, ISO 27001, WCAG 2.1 Level A
