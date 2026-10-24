# SRS Modules — Equalizer-Stake

SRS dipecah menjadi modul terpisah berdasarkan fitur. Setiap modul ter-trace ke FR, UC, dan architecture doc.

---

## Modul

| # | File | FR | UC | Architecture |
|---|------|----|----|-------------|
| 00 | [00-overview.md](00-overview.md) | — | — | 00-system-overview.md |
| 01 | [01-authentication.md](01-authentication.md) | FR-01 | UC-01, UC-02 | 04-security.md |
| 02 | [02-scheduling.md](02-scheduling.md) | FR-02, FR-12 | UC-02..UC-05 | 01-backend.md |
| 03 | [03-billing.md](03-billing.md) | FR-03, FR-04, FR-15 | UC-05, UC-07, UC-08 | 01-backend.md |
| 04 | [04-attendance.md](04-attendance.md) | FR-05 | UC-03, UC-11, UC-12 | 02-frontend.md |
| 05 | [05-notifications.md](05-notifications.md) | FR-06 | UC-04, UC-14, UC-15 | 05-integrations.md |
| 06 | [06-reporting.md](06-reporting.md) | FR-07, FR-13 | UC-09, UC-10 | 01-backend.md |
| 07 | [07-audit-governance.md](07-audit-governance.md) | FR-09, FR-11 | UC-11, UC-12 | 03-database.md |
| 08 | [08-studio-reservation.md](08-studio-reservation.md) | FR-14 | UC-06 | 01-backend.md |
| 09 | [09-multi-student.md](09-multi-student.md) | FR-10 | UC-08 | 01-backend.md |
| 10 | [10-draft-autosave.md](10-draft-autosave.md) | FR-08 | UC-13 | 02-frontend.md |
| 11 | [11-user-interaction-flows.md](11-user-interaction-flows.md) | Semua FR/UC | — | 00-system-overview.md |
| 99 | [99-nfr.md](99-nfr.md) | — | — | 06-deployment.md |
| 99 | [99-entities.md](99-entities.md) | — | — | 03-database.md |
| 99 | [99-uc-diagrams.md](99-uc-diagrams.md) | — | UC-01..UC-20 | — |

## Konvensi Penamaan

- `00-99` → urut berdasarkan modul fungsional
- `99-xxx` → cross-cutting concerns (NFR, entities, UC diagrams)
- Setiap file punya YAML frontmatter dengan `fr`, `uc`, `status` fields

## Cara Menggunakan

1. Cari task di `tasks/TASKS.md`
2. Lihat FR/UC terkait di kolom referensi
3. Buka SRS module yang sesuai dari tabel di atas
4. Cross-reference ke architecture doc untuk technical implementation
