# DESIGN.md — BMD Music School (Equalizer-Stake)

**Version:** 2.0
**Date:** 2026-07-05
**Source:** SRS ISO/IEC/IEEE 29148:2018 + SRS Addendum (Data Asli BMD MS)
**Register:** Product UI — design SERVES the product

---

## 1. PRODUCT OVERVIEW

### 1.1 Apa ini?
Sistem Penjadwalan Kursus & Reservasi Studio Berbasis Web untuk BMD Music School, Yogyakarta.
Web app untuk manajemen kursus musik: pendaftaran siswa, penjadwalan, penagihan, kehadiran, notifikasi.

### 1.2 Siapa penggunanya?

| Peran | Fungsi | Session Timeout |
|-------|--------|-----------------|
| **Owner / Superadmin** | Dashboard read-only, manajemen Admin, override operasional, audit full, pengaturan sistem | 30 menit |
| **Admin Operasional** | Manajemen harian (jadwal, tagihan, siswa, guru) | 30 menit |
| **Guru Musik** | Absensi, lihat jadwal mengajar, feedback ke wali murid | 30 menit |
| **Wali Murid** | Portal 1:N siswa, pembayaran, jadwal, rating guru | 60 menit |

### 1.3 Stack
- **Backend**: FastAPI + SQLAlchemy async + PostgreSQL + Redis
- **Frontend**: Next.js 15 (App Router) + Tailwind CSS + TypeScript
- **Deploy**: Docker + single server

---

## 2. DESIGN SYSTEM

### 2.1 Theme
- **Light default**: #F8FAFC bg, #FFFFFF cards
- **No dark mode** (removed — too many inconsistencies)

### 2.2 Color Palette

| Role | Hex | Usage |
|------|-----|-------|
| Primary | #2563EB (blue-600) | Buttons, links, active states |
| Primary Hover | #1D4ED8 (blue-700) | Button hover |
| Accent | #F59E0B (amber-500) | Highlights, badges, warnings |
| Accent Hover | #D97706 (amber-600) | Accent hover |
| Background | #F8FAFC (slate-50) | Page background |
| Surface | #FFFFFF | Cards, modals, inputs |
| Text Primary | #0F172A (slate-900) | Headings, body text |
| Text Secondary | #64748B (slate-500) | Labels, descriptions |
| Text Muted | #94A3B8 (slate-400) | Placeholders, hints |
| Border | #E2E8F0 (slate-200) | Input borders, dividers |
| Success | #10B981 (emerald-500) | Success states |
| Error | #EF4444 (red-500) | Error states |
| Warning | #F59E0B (amber-500) | Warning states |

### 2.3 Typography

| Element | Font | Weight | Size |
|---------|------|--------|------|
| H1 | Plus Jakarta Sans | Bold (700) | 2rem (32px) |
| H2 | Plus Jakarta Sans | Bold (700) | 1.5rem (24px) |
| H3 | Plus Jakarta Sans | SemiBold (600) | 1.25rem (20px) |
| Body | Inter | Regular (400) | 0.875rem (14px) |
| Small | Inter | Regular (400) | 0.75rem (12px) |
| Mono | JetBrains Mono | Regular (400) | 0.875rem (14px) |

**Line length**: max 65-75ch for prose
**Line height**: 1.5 for body, 1.2 for headings

### 2.4 Spacing

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Tight gaps |
| sm | 8px | Input padding, icon gaps |
| md | 16px | Card padding, section gaps |
| lg | 24px | Section padding |
| xl | 32px | Page padding |
| 2xl | 48px | Section spacing |
| 3xl | 64px | Hero spacing |

### 2.5 Border Radius

| Element | Radius |
|---------|--------|
| Cards | rounded-xl (12px) |
| Buttons | rounded-lg (8px) |
| Inputs | rounded-lg (8px) |
| Badges | rounded-full (9999px) |
| Modals | rounded-xl (12px) |

### 2.6 Shadows

| Level | Value | Usage |
|-------|-------|-------|
| sm | 0 1px 2px rgba(0,0,0,0.05) | Cards |
| md | 0 4px 6px -1px rgba(0,0,0,0.1) | Dropdowns |
| lg | 0 10px 15px -3px rgba(0,0,0,0.1) | Modals |

---

## 3. COMPONENTS

### 3.1 Buttons

```
Primary:    bg-[#2563EB] text-white hover:bg-[#1D4ED8] rounded-lg px-4 py-2.5
Secondary:  border border-gray-300 text-gray-700 hover:bg-gray-50 rounded-lg px-4 py-2.5
Danger:     bg-red-500 text-white hover:bg-red-600 rounded-lg px-4 py-2.5
Ghost:      text-gray-700 hover:bg-gray-100 rounded-lg px-4 py-2.5
```

### 3.2 Inputs

```
Default:    border border-gray-300 rounded-lg px-3 py-2.5 text-sm
Focus:      border-[#2563EB] ring-2 ring-[#2563EB]/20
Error:      border-red-500 ring-2 ring-red-500/20
Disabled:   bg-gray-50 text-gray-400 cursor-not-allowed
```

### 3.3 Cards

```
Default:    bg-white rounded-xl border border-gray-200 shadow-sm p-5
Hover:      hover:border-gray-300 hover:shadow-md transition
```

### 3.4 Tables

```
Desktop:    Standard table with header row
Mobile:     MobileTable component (cards instead of rows)
Pagination: "Menampilkan X-Y dari Z data"
```

### 3.5 Modals

```
Overlay:    bg-black/50 fixed inset-0 z-50
Content:    bg-white rounded-xl shadow-lg max-w-md mx-auto
Header:     font-heading text-lg font-bold
Footer:     flex justify-end gap-3
```

### 3.6 Toast Notifications

```
Success:    bg-emerald-50 border-emerald-200 text-emerald-800
Error:      bg-red-50 border-red-200 text-red-800
Warning:    bg-amber-50 border-amber-200 text-amber-800
Info:       bg-blue-50 border-blue-200 text-blue-800
```

---

## 4. LAYOUT

### 4.1 Dashboard Layout

```
┌─────────────────────────────────────────┐
│ Sidebar (hidden mobile)  │   Content    │
│ ┌─────────────────────┐  │ ┌─────────┐ │
│ │ Logo                │  │ │ Breadcrumb│ │
│ │ Navigation          │  │ │         │ │
│ │ - Menu items        │  │ │ Page    │ │
│ │ - Role-based        │  │ │ Content │ │
│ │                     │  │ │         │ │
│ │ Theme Toggle        │  │ │         │ │
│ │ Logout              │  │ └─────────┘ │
│ └─────────────────────┘  │             │
│ BottomNav (mobile only)  │             │
└─────────────────────────────────────────┘
```

### 4.2 Responsive Breakpoints

| Breakpoint | Width | Sidebar | Table |
|------------|-------|---------|-------|
| Mobile | <768px | Hidden, BottomNav | Cards |
| Tablet | 768-1024px | Collapsed | Table |
| Desktop | >1024px | Expanded | Table |

### 4.3 Page Structure

```
Dashboard page:
├── Breadcrumbs
├── Page title + description
├── Action buttons (Add, Export, etc.)
├── Filters/Search (if applicable)
├── Content (Table/Card grid)
└── Pagination
```

---

## 5. PAGES & ROUTES

### 5.1 Public Pages

| Route | Page | Purpose |
|-------|------|---------|
| `/` | Landing Page | Marketing, CTA ke /register-course |
| `/register-course` | Register Course | Self-service registration (5 steps) |
| `/login` | Login | Authentication |
| `/forgot-password` | Forgot Password | Password reset |
| `/help` | Help | FAQ page |
| `/instruments` | Instruments | Detail instrumen |

### 5.2 Dashboard Pages (Role-Based)

| Route | Role | Purpose |
|-------|------|---------|
| `/dashboard` | All | Main dashboard (role-specific content) |
| `/dashboard/students` | Admin | Manage siswa |
| `/dashboard/teachers` | Admin | Manage guru |
| `/dashboard/schedule` | Admin | Manage jadwal |
| `/dashboard/billing` | Admin | Manage tagihan |
| `/dashboard/payments` | Admin | Manage pembayaran |
| `/dashboard/registrations` | Admin | Verifikasi pendaftaran |
| `/dashboard/assign-session` | Admin | Assign siswa ke sesi |
| `/dashboard/holidays` | Admin | Manage hari libur |
| `/dashboard/studio` | Admin | Manage ruangan |
| `/dashboard/pricing` | Owner | Manage harga |
| `/dashboard/settings` | Owner | Pengaturan sistem |
| `/dashboard/admins` | Owner | Manage admin |
| `/dashboard/reports` | Owner | Laporan & KPI |
| `/dashboard/audit-log` | Owner | Audit trail |
| `/dashboard/override` | Owner | Override operasional |
| `/dashboard/my-students` | Guardian | Anak saya |
| `/dashboard/my-schedule` | Guardian | Jadwal anak |
| `/dashboard/my-payments` | Guardian | Pembayaran |
| `/dashboard/ratings` | Guardian | Rating guru |
| `/dashboard/my-schedule` | Teacher | Jadwal mengajar |
| `/dashboard/attendance` | Teacher | Absensi |
| `/dashboard/feedback` | Teacher | Feedback ke wali murid |
| `/dashboard/profile` | All | Profil pengguna |
| `/dashboard/notifications` | All | Notifikasi |

---

## 6. BUSINESS RULES (SRS Addendum)

### 6.1 Billing

- **Tanggal 5**: Auto-invoice generated per guardian (aggregat semua siswa aktif)
- **Tanggal 10, 23:59 WIB**: Batas pembayaran
- **Tanggal 11**: Denda Rp 100.000 + freeze akun guardian
- **Unique code**: 3 digit per invoice untuk verifikasi transfer

### 6.2 Attendance

- **GPS**: 150m radius dari lokasi BMD
- **Photo**: Upload foto kehadiran (wajib)
- **Review**: Guru verifikasi/reject dengan alasan
- **Siswa absen**: Jam hangus, tidak bisa diganti

### 6.3 Cuti

- **Max**: 2 bulan (60 hari) per tahun kalender
- **Tracking**: cuti_start_date, cuti_end_date, cuti_days_used
- **Auto-reactivate**: Ketika periode cuti berakhir

### 6.4 Durasi Sesi

- **Valid**: 30 menit atau 45 menit saja
- **Validation**: Saat create session

### 6.5 Jam Operasional

- **Buka**: Senin-Sabtu, 08:00-20:00 WIB
- **Tutup**: Minggu
- **Validation**: Saat create session

### 6.6 Instrumen & Level

**Instrumen (10):**
biola, piano, gitar, drum, keyboard, flute, saxophone, vokal, bass, ensemble

**Level per Instrumen:**

| Instrumen | Level |
|-----------|-------|
| Piano Klasik | pra_elementary, elementary, grade_1..6 |
| Piano Pop | beginner, intermediate, intermediate_2 |
| Vokal | basic, intermediate, advance |
| Biola | basic, intermediate, advance |
| Gitar | basic, intermediate, advance |
| Keyboard | basic, intermediate, advance |
| Drum | basic, intermediate, advance |
| Flute | basic, intermediate, advance |
| Saxophone | basic, intermediate, advance |

### 6.7 Tipe Kelas

| Tipe | Siswa | Keterangan |
|------|-------|------------|
| private | 1 | 1-on-1 |
| intensif_2 | 2 | Intensif 2 siswa |
| intensif_3 | 3 | Intensif 3 siswa |
| semi_intensif | 4-6 | Semi intensif |
| expatriate_private | 1 | English class |
| expatriate_2 | 2 | English 2 siswa |
| expatriate_3 | 3 | English 3 siswa |
| tunas | 1 | Min usia 3.5 tahun |
| paket_liburan | 1-4 | Paket liburan |

### 6.8 Pricing (Registration Fee)

| Instrumen | Biaya Daftar |
|-----------|-------------|
| Piano Klasik | Rp 420.000 |
| Piano Pop | Rp 420.000 |
| Vokal | Rp 420.000 |
| Biola | Rp 540.000 |
| Gitar | Rp 420.000 |
| Keyboard | Rp 480.000 |
| Drum | Rp 420.000 |
| Flute | Rp 420.000 |
| Saxophone | Rp 420.000 |
| Paket Liburan | Rp 60.000 |

---

## 7. COPYWRITING

### 7.1 Landing Page

**Hero:**
- Headline: "Anak Anda Punya Bakat Musik? Kami Bantu Mengembangkannya."
- Subheadline: "Kursus musik profesional di Yogyakarta. Guru berpengalaman, jadwal fleksibel, harga terjangkau."
- CTA: "Daftar Kursus" → /register-course

**Instrumen:**
- Title: "Pilih Instrumen Favorit Anak Anda"
- Desc per instrumen: benefit-focused, bukan fitur

**Tentang:**
- Title: "Tentang BMD"
- Desc: "Bukan hanya kursus musik. Kami membangun karakter anak melalui musik."

**Testimoni:**
- Title: "Apa Kata Mereka?"
- Format: transformation stories, bukan feature quotes

**Harga:**
- Title: "Investasi untuk Masa Depan Anak"
- Format: per instrumen + tipe kelas

**CTA:**
- Title: "Siap Memberikan yang Terbaik untuk Anak?"
- Button: "Daftar Sekarang" → /register-course

### 7.2 Bahasa Indonesia

Semua UI text dalam Bahasa Indonesia:
- Buttons: Simpan, Hapus, Batal, Kembali, Kirim
- Labels: Nama, Email, Telepon, Alamat
- Messages: Berhasil, Gagal, Memuat...
- Placeholders: Cari..., Pilih...

---

## 8. ACCESSIBILITY

### 8.1 WCAG 2.1 Level A

- Color contrast: ≥4.5:1 body text, ≥3:1 large text
- Focus states: focus:ring-2 on all interactive elements
- aria-label: on all icons and image elements
- Keyboard navigation: Tab, Enter, Escape support

### 8.2 Semantic HTML

- Use `<button>` not `<div onClick>`
- Use `<nav>` for navigation
- Use `<main>` for content
- Use `<aside>` for sidebar

---

## 9. MOTION

### 9.1 Transitions

- Duration: 150-250ms
- Easing: ease-out (exponential curves)
- Properties: transform, opacity only (not layout)

### 9.2 Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  * { animation-duration: 0.01ms !important; }
}
```

---

## 10. ANTI-PATTERNS (BANNED)

### 10.1 From impeccable skill

- Side-stripe borders (border-left > 1px)
- Gradient text (background-clip: text)
- Glassmorphism as default
- Hero-metric template (big number + small label)
- Identical card grids
- Tiny uppercase tracked eyebrow on every section
- Numbered section markers (01/02/03) as default

### 10.2 From project experience

- Dark mode remnants (removed)
- Dummy data in dashboards (use real APIs)
- Inconsistent API paths (/api/ vs /api/v1/)
- Wrong token key ("token" vs "access_token")
- Missing error handling (try-catch + error state)
- Missing loading states (Skeleton component)
- Missing empty states (EmptyState component)
