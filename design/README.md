# Design — Equalizer-Stake

Direktori ini berisi dokumentasi design system dan UI/UX untuk Equalizer-BMD Musik.

## Status: Belum Diisi

Design files akan diisi setelah Phase 0 (Infrastruktur & Setup) selesai.

## Rencana Design Files

| File | Deskripsi |
|------|-----------|
| 01-design-system.md | Color palette, spacing, typography |
| 02-components.md | Reusable UI components |
| 03-layout-grid.md | Responsive grid system |
| 04-page-login.md | Login page design |
| 05-page-dashboard.md | Dashboard designs (4 roles) |
| 06-page-scheduling.md | Scheduling calendar UI |
| 07-page-attendance.md | Attendance form + camera |
| 08-page-billing.md | Invoice & payment UI |
| 09-page-studio.md | Studio reservation UI |
| 10-accessibility.md | WCAG 2.1 Level A compliance |

## Design Principles (dari SRS)

- **Mobile-first**: 320px → 480px → 768px → 1024px+
- **WCAG 2.1 Level A**: Contrast, focus indicators, keyboard navigation
- **Bahasa Indonesia**: Semua label, tombol, pesan error
- **High contrast mode**: Untuk ruangan dengan pencahayaan bervariasi
- **Touch targets**: Minimal 44×44px untuk tombol aksi
- **Visual indicators**: Tidak bergantung pada suara (studio bising)
- **Actionable errors**: Setiap pesan error sertakan langkah selanjutnya
