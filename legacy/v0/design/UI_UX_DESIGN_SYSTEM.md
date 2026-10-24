# UI/UX Design System — Equalizer-Stake

**Status:** Primary Reference (menggantikan design-tokens.md lama)
**Last Updated:** 2026-07-03
**Scope:** Landing page, Dashboard, Semua halaman

---

## 1. Color Palette

### Primary Colors

| Token | Hex | Kegunaan |
|-------|-----|----------|
| PRIMARY | #2563EB | Biru — trust, profesional, navigasi |
| SECONDARY | #7C3AED | Ungu — kreativitas, music |
| ACCENT | #F59E0B | Emas — premium, CTA, "mewah" brand EQS |

### Semantic Colors

| Token | Hex | Kegunaan |
|-------|-----|----------|
| SUCCESS | #10B981 | Hijau soft — sukses, hadir, aktif |
| DANGER | #EF4444 | Merah — error, denda, ditolak |
| WARNING | #F59E0B | Kuning — peringatan, offline |
| INFO | #3B82F6 | Biru — info, notifikasi |

### Light Theme (DEFAULT)

| Token | Hex | Kegunaan |
|-------|-----|----------|
| bg | #F8FAFC | Background utama |
| card | #FFFFFF | Card, modal, sidebar |
| border | #E2E8F0 | Border semua komponen |
| text | #1E293B | Text utama |
| text-muted | #64748B | Text sekunder, label |
| text-light | #94A3B8 | Text placeholder |
| sidebar-bg | #FFFFFF | Background sidebar |
| sidebar-border | #E2E8F0 | Border sidebar |

### Dark Theme (optional toggle)

| Token | Hex | Kegunaan |
|-------|-----|----------|
| bg | #0F172A | Background utama |
| card | #1E293B | Card, modal, sidebar |
| border | #334155 | Border semua komponen |
| text | #F1F5F9 | Text utama |
| text-muted | #94A3B8 | Text sekunder |
| sidebar-bg | #1E293B | Background sidebar |
| sidebar-border | #334155 | Border sidebar |

### Landing Page Hero Gradient

```
From: #2563EB (biru) → To: #7C3AED (ungu)
Opacity: 90%
Overlay: white text di atas
```

---

## 2. Typography

### Font Stack

| Elemen | Font | Source |
|--------|------|--------|
| Heading | Plus Jakarta Sans | Google Fonts |
| Body | Inter | Google Fonts (sudah ada) |
| Mono | JetBrains Mono | Untuk kode, unique code |

### Font Scale (4px grid)

| Token | Size | Kegunaan |
|-------|------|----------|
| xs | 12px | Caption, badge |
| sm | 14px | Label, secondary text |
| base | 16px | Body text — MINIMUM untuk mobile |
| lg | 18px | Card titles |
| xl | 20px | Page titles |
| 2xl | 24px | Dashboard headings |
| 3xl | 30px | KPI numbers, hero subhead |
| 4xl | 36px | Hero headline |

### Line Height

| Token | Value | Kegunaan |
|-------|-------|----------|
| tight | 1.25 | Headings |
| normal | 1.5 | Body text |
| loose | 1.75 | Long text, FAQ |

### Font Weight

| Token | Value | Kegunaan |
|-------|-------|----------|
| normal | 400 | Body text |
| medium | 500 | Labels, secondary |
| semibold | 600 | Card titles, buttons |
| bold | 700 | Headings, KPI |

---

## 3. Spacing System (4px grid)

| Token | Value | Kegunaan |
|-------|-------|----------|
| xs | 4px | Icon padding |
| sm | 8px | Input padding |
| md | 12px | Card padding mobile |
| lg | 16px | Card padding desktop |
| xl | 24px | Section gap |
| 2xl | 32px | Page margin |
| 3xl | 48px | Major sections |
| 4xl | 64px | Hero spacing |

---

## 4. Border Radius

| Token | Value | Kegunaan |
|-------|-------|----------|
| sm | 4px | Badge, small elements |
| md | 8px | Buttons, inputs |
| lg | 12px | Cards |
| xl | 16px | Modals, large cards |
| full | 9999px | Avatars, pills |

---

## 5. Shadows

| Token | Value | Kegunaan |
|-------|-------|----------|
| none | — | Clean flat style |
| sm | 0 1px 2px rgba(0,0,0,0.05) | Subtle elevation |
| md | 0 4px 6px rgba(0,0,0,0.07) | Card hover |
| lg | 0 10px 15px rgba(0,0,0,0.1) | Dropdown, popover |
| xl | 0 20px 25px rgba(0,0,0,0.15) | Modal |

---

## 6. Layout System

### Breakpoints

| Name | Min Width | Layout |
|------|-----------|--------|
| Mobile | 0px | 1 column, bottom nav |
| Tablet | 768px | 2 columns, collapsible sidebar |
| Desktop | 1024px | Fixed sidebar + content |
| Wide | 1280px | Max container width |

### Container

```
Max width: 1280px
Margin: mx-auto
Padding: 16px mobile, 24px tablet, 32px desktop
```

### Sidebar (Desktop)

```
Width: 256px (fixed)
Background: theme card color
Border-right: 1px solid theme border
Padding: 12px
```

### Main Content

```
Flex: 1
Padding: 24px desktop, 16px mobile
Max-width: 1280px
```

---

## 7. Component Patterns

### Card

```
Background: theme card
Border: 1px solid theme border
Border-radius: 12px
Padding: 16px mobile, 24px desktop
Shadow: none (flat) or sm (elevated)
Hover: shadow-md, translateY(-2px)
```

### Button

| Variant | Background | Text | Border | Hover |
|---------|-----------|------|--------|-------|
| Primary | #2563EB | white | none | darker blue |
| Secondary | transparent | #2563EB | 1px #2563EB | blue bg 5% |
| Ghost | transparent | text-muted | none | bg 5% |
| Danger | #EF4444 | white | none | darker red |
| Accent | #F59E0B | white | none | darker gold |

Sizes:
- sm: h-8, text-sm, px-3
- md: h-10, text-sm, px-4 (default)
- lg: h-12, text-base, px-6

Border-radius: 8px

### Form Input

```
Height: 40px (md)
Border: 1px solid theme border
Border-radius: 8px
Padding: 0 12px
Background: theme card
Text: theme text
Focus: 2px ring #2563EB
Error: border #EF4444 + red text below
```

### Table

```
Header: bg theme bg, text uppercase, text-xs, font-medium
Row: border-bottom 1px theme border
Row hover: bg 5%
Mobile: Convert to card layout
Action column: right-aligned, icon buttons
```

---

## 8. Dashboard Architecture

### Shared Layout

```
┌──────────┬────────────────────────────────────────┐
│ Sidebar │ Header: [Greeting] [Notif] [Profile] │
│ (desktop)│────────────────────────────────────────│
│ │ Content area (max-w: 1280px, mx-auto) │
├──────────┴────────────────────────────────────────┤
│ Bottom Nav (mobile) │
└──────────────────────────────────────────────────┘
```

### Admin Dashboard

```
Greeting: "Selamat pagi, [Nama]"
Date: "Kamis, 3 Juli 2026"

KPI Cards (4):
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│ 12 │ │ 3 │ │ 5 │ │ 2 │
│ Sesi │ │Pending│ │Overdue│ │Notif │
│ Hari │ │Absen │ │Bayar │ │Gagal │
└──────┘ └──────┘ └──────┘ └──────┘

Content:
┌─────────────────────┐ ┌─────────────────────┐
│ Jadwal Hari Ini │ │ Perlu Review │
│ [table: jam, student, │ │ • Absen pending (3) │
│ instructor, status] │ │ • Tagihan overdue │
└─────────────────────┘ └─────────────────────┘

┌─────────────────────────────────────────────┐
│ Revenue Bulan Ini │
│ Rp 15.200.000 │ ▲ 12% dari bulan lalu │
└─────────────────────────────────────────────┘
```

### Guardian Dashboard

```
Greeting: "Selamat pagi, Pak/Bu [Nama]"
Child: "Anak: Andi Pratama" (auto-select if single)

┌─────────────────────┐ ┌─────────────────────┐
│ Jadwal Minggu Ini │ │ Tagihan │
│ • Sen: Piano 10:00 │ │ Rp 350.000 │
│ • Rab: Gitar 14:00 │ │ Jatuh tempo: 10 Jul │
│ • Jum: Drum 09:00 │ │ [Bayar Sekarang] │
└─────────────────────┘ └─────────────────────┘

┌─────────────────────┐ ┌─────────────────────┐
│ Absensi Terakhir │ │ Kelayakan Ujian │
│ ✅ 1 Jul - Hadir │ │ ████████░░ 80% │
│ ✅ 29 Jun - Hadir │ │ 8/10 sesi hadir │
│ ❌ 28 Jun - Absen │ │ │
└─────────────────────┘ └─────────────────────┘
```

### Teacher Dashboard

```
Greeting: "Selamat pagi, [Nama]"
Summary: "Hari ini: 4 sesi"

┌─────────────────────────────────────────────┐
│ Jadwal Hari Ini [Absen] │
│ 09:00-10:00 Andi (Gitar) [📷] │
│ 10:00-11:00 Rina (Piano) [📷] │
│ 13:00-14:00 Budi (Drum) [📷] │
│ 14:30-15:30 Dewi (Vokal) [📷] │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ Ringkasan Minggu Ini │
│ 12 sesi │ 10 hadir │ 2 pending verifikasi │
└─────────────────────────────────────────────┘
```

### Owner Dashboard

```
Greeting: "Selamat pagi, [Nama]"

┌─────────────────────────────────────────────┐
│ Revenue Trend (chart) │ Occupancy Rate │
│ ████████████ 75% │ 85% (28/33 slots) │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ Student Retention │ Pending Issues │
│ 92% (bulan ini) │ 3 absen, 2 tagihan │
└─────────────────────────────────────────────┘
```

---

## 9. Landing Page

### Section Order (9 sections)

1. **Navbar** (sticky) — navigasi
2. **Hero** — emotional hook + CTA
3. **Trust Bar** — social proof angka
4. **Program Course** — card grid instrumen
5. **Kenapa EQS** — benefit > fitur
6. **Studio** — full-width image
7. **Testimonial** — quote style
8. **Harga** — pricing table
9. **CTA** — urgency + WhatsApp

### Design Style

- **FLAT + SOFT SHADOW** (bukan glassmorphism)
- Shadow: 0 1px 3px rgba(0,0,0,0.1)
- Rounded: 12px cards, 8px buttons
- Hero: gradient biru-ungu (#2563EB → #7C3AED)
- NO snap scroll, NO parallax

### Copywriting Rules

1. **Emosional > Deskriptif**
 - ✅ "Music Builds the Future"
 - ❌ "Learn Music dengan Nyaman"

2. **Benefit > Fitur**
 - ✅ "Studio yang Bikin Anak Betah"
 - ❌ "Studio Premium"

3. **Specific > General**
 - ✅ "150+ pengguna", "Rp 350.000/bulan"
 - ❌ "banyak student", "murah"

4. **Social Proof: Angka + nama real + foto**

5. **CTA: WhatsApp langsung (bukan form panjang)**

6. **Urgency: "Kuota Terbatas", "5 slot trial"**

### Hero Section

```
Headline: "Music Beyond Hobbies.
 Music Builds the Future."

Subhead: "Course music premium untuk anak usia 5-18 tahun
 di . Instructor berpengalaman, sistem digital,
 hasil nyata."

CTA: [Mulai Gratis Trial] ← white button
 Sudah punya akun? Masuk →
```

### Trust Bar

```
"150+ Student Aktif · 9 Instrumen · 10+ Tahun · 4.9★ Google"
(single line, centered, emas #F59E0B)
```

### Section Headers

| Section | Header |
|---------|--------|
| Program | "Pilih Instrumen yang Mencerminkan Bakat Anak" |
| Kenapa | "Yang Membedakan EQS dari Course Lainnya" |
| Studio | "Studio yang Bikin Anak Betah Belajar" |
| Testimonial | "150+ Keluarga Sudah Membuktikan" |
| Harga | "Investasi untuk Masa Depan Anak" |
| CTA | "Mulai Gratis — Kuota Trial Terbatas" |

### Testimonial

- Quote style, large text
- Nama + peran + rating bintang
- Specific + emotional
- Contoh: "3 bulan", "minta les tambahan"

### Pricing

- 3 columns: Trial / Reguler (populer) / Intensif
- Middle = highlighted (border emas)
- Harga: emas accent
- CTA: gold button

---

## 10. Error Handling UX

### Pattern: Inline Form Errors + Toast API Errors

| Error Type | Display | Duration |
|------------|---------|----------|
| Form validation | Inline di bawah field | Persistent |
| API error | Toast (bottom-right) | 5s auto-dismiss |
| Fatal error (401/403/404/500) | Dedicated error page | — |
| Network offline | Toast warning | Manual dismiss |

### Error Messages (Bahasa Indonesia)

| Code | Message |
|------|---------|
| 401 | "Sesi expired. Silakan login kembali." |
| 403 | "Anda tidak memiliki akses." |
| 404 | "Data tidak ditemukan." |
| 409 | "Data sudah diperbarui oleh pengguna lain." |
| 422 | "Data yang diisi tidak valid." |
| 429 | "Terlalu banyak percobaan. Tunggu sebentar." |
| 500 | "Terjadi kesalahan server. Coba lagi atau hubungi admin." |
| OFFLINE | "Tidak ada koneksi internet." |

### Toast Component

```
Position: bottom-right (mobile: top)
Style: rounded-lg, shadow-lg, border-left colored
Animation: slide in from right (0.2s)

SUCCESS: [✓] + green border + 3s auto-dismiss
ERROR: [✗] + red border + 5s + "Coba lagi" button
WARNING: [!] + yellow border + manual dismiss
INFO: [i] + blue border + 3s auto-dismiss
```

---

## 11. Form Validation UX

### Pattern: Hybrid (onChange + onBlur)

- **onChange:** update status visual ✓/✗ (real-time, debounced 300ms)
- **onBlur:** tampilkan error message (saat pindah field)
- **onSubmit:** revalidate semua + fokus ke error pertama

### Validation Rules

| Field | Rule | Error Message |
|-------|------|---------------|
| Required | Empty | "Field ini wajib diisi" |
| Email | Format | "Format email tidak valid" |
| Phone | 12-13 digit, awali 0 | "Nomor harus 12-13 digit, awali 0" |
| Password | Min 8, huruf + angka | "Minimal 8 karakter, ada huruf + angka" |
| Number | Numeric | "Harus berupa angka" |

### Visual Indicators

```
✓ Valid: green border + check icon (✓)
✗ Invalid: red border + X icon (✗) + message below
Empty: gray border (no indicator)
Focused: 2px ring primary color
```

---

## 12. Confirmation Dialogs

### Pattern: Smart Confirm (4 severity levels)

| Level | Trigger | Content | Actions |
|-------|---------|---------|---------|
| Simple | Logout | "Yakin ingin keluar?" | [Batal] [Keluar] |
| Destructive | Delete | Warning text + item detail | [Batal] [Hapus] |
| Financial | Bayar | Amount display | [Batal] [Bayar Rp 350.000] |
| Complex | Cancel | Input alasan required | [Batal] [Batalkan] |

### Modal Design

```
Max width: 400px
Centered overlay
Backdrop: black 50% opacity
Animation: slide up + fade in (0.2s)
Close: X button + Escape key + backdrop click
```

---

## 13. Date/Time Format

### Library: dayjs (2KB gzipped)

```javascript
import dayjs from 'dayjs'
import 'dayjs/locale/id'
import relativeTime from 'dayjs/plugin/relativeTime'
dayjs.locale('id')
dayjs.extend(relativeTime)
```

### Formats

| Type | Format | Example |
|------|--------|---------|
| Full date | D MMMM YYYY | "3 Juli 2026" |
| Short date | D MMM YYYY | "3 Jul 2026" |
| Time | HH:mm WIB | "09:00 WIB" |
| Relative | fromNow() | "2 jam yang lalu" |
| DateTime | D MMMM YYYY, HH:mm | "3 Juli 2026, 09:00" |

---

## 14. Currency Format

### Pattern: Custom Rupiah Formatter

```javascript
formatRupiah(350000) // "Rp 350.000"
formatRupiah(1500000) // "Rp 1.500.000"
formatRupiahShort(350000) // "Rp 350rb"
formatRupiahShort(1500000) // "Rp 1,5jt"
```

### Rules

- Prefix: "Rp " (ada spasi)
- Separator: titik (.) untuk ribuan
- Tidak ada desimal (bulat)
- Simpan sebagai number, display sebagai string
- Full format: tagihan, invoice, detail
- Short format: dashboard cards, mobile

---

## 15. Accessibility (WCAG 2.1 Level A)

- `lang="id"` on HTML
- `aria-label` on icon-only buttons
- `<label>` on all form inputs
- Min font: 16px body
- Contrast: ≥4.5:1 for text
- Keyboard navigation: all interactive elements
- Focus visible: 2px ring on all focusable elements

---

## 16. Stakeholder Profiles

| Role | Usia | Kebutuhan | UI Priority |
|------|------|-----------|-------------|
| Owner | 40-55 | Overview cepat, revenue | Dashboard ringkas |
| Admin | 25-40 | Efisiensi operasional | Banyak data, filter |
| Instructor | 25-45 | Mobile-first, cepat | Large buttons, minimal |
| Guardian | 35-55 | Simpel, jelas, tenang | Large text, sedikit menu |

---

## 17. Animation & Transitions

### Micro-interactions

| Action | Animation | Duration |
|--------|-----------|----------|
| Page load | Fade in | 0.2s |
| Card hover | Lift shadow + translateY(-2px) | 0.15s |
| Button click | Scale 0.98 | 0.1s |
| Modal open | Slide up + fade in | 0.2s |
| Toast appear | Slide in from right | 0.2s |
| Theme toggle | Color crossfade | 0.3s |

### Loading States

| Type | Animation |
|------|-----------|
| Skeleton | Pulse animation (opacity 0.5 → 1) |
| Spinner | Rotate 360deg, 1s linear infinite |
| Button | Spinner inside button + disabled |

---

## 18. Notification Badge

### Pattern

```
Position: top-right of icon
Background: #EF4444 (danger)
Text: white, font-xs
Size: 16px circle
Count: "3" (max 99, then "99+")
Animation: bounce on new notification
Auto-clear: when notification page opened
```

---

## 19. Search & Filter

### Pattern

```
Search: input with search icon, debounced 300ms
Filter: dropdown chips, multi-select
Sort: dropdown (Terbaru, Terlama, A-Z, Z-A)
Active filters: chips with X to remove
Result count: "Menampilkan 1-20 dari 45 data"
```

---

## 20. Pagination UX

### Pattern

```
[← Prev] [1] [2] [3] ... [8] [Next →]
"Menampilkan 1-20 dari 45 data"

Current page: bg primary, text white
Other pages: bg transparent, text muted
Disabled: opacity 50%, no click
```

---

## 21. Print Styles

### Pattern

```
@media print {
 - Hide: sidebar, bottom nav, buttons, toast
 - Show: content only
 - Font: black on white
 - No shadows, no gradients
 - Page break: avoid inside tables
}
```

---

## 22. Theme Toggle

### Pattern

```
Location: sidebar header (desktop), profile page (mobile)
Icon: 🌙 (dark) / ☀️ (light)
Persist: localStorage('theme')
Default: light
Transition: color crossfade 0.3s
Implementation: CSS variables + Tailwind dark: prefix
```
