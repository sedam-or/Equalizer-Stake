# Equalizer-Stake — Comprehensive UI/UX Design Specification

> **Version**: 2.0
> **Date**: 2026-07-04
> **Status**: Active
> **SRS Reference**: FR-01..FR-15, UC-01..UC-20

---

## Daftar Isi

1. [Brand Identity](#1-brand-identity)
2. [Color System](#2-color-system)
3. [Typography](#3-typography)
4. [Spacing & Layout](#4-spacing--layout)
5. [Component Library](#5-component-library)
6. [Page Specifications](#6-page-specifications)
7. [User Flows](#7-user-flows)
8. [Responsive Behavior](#8-responsive-behavior)
9. [Accessibility](#9-accessibility)
10. [Animation & Motion](#10-animation--motion)
11. [Error Handling UI](#11-error-handling-ui)
12. [UAT Scenarios](#12-uat-scenarios)
13. [Test Cases](#13-test-cases)
14. [Acceptance Criteria Matrix](#14-acceptance-criteria-matrix)

---

## 1. Brand Identity

### 1.1 Brand Essence
- **Nama**: Equalizer-Stake (EQS)
- **Tagline**: "Music Beyond Hobbies. Music Builds the Future."
- **Lokasi**:
- **Positioning**: Premium music school untuk pengguna kelas menengah-atas
- **Target Users**: Anak usia 5-17 tahun (student), Orang tua 30-55 tahun (guardian)

### 1.2 Brand Values
- Profesional (instructor berpengalaman)
- Terjangkau (harga premium tapi worth it)
- Personal (perhatian per student)
- Modern (teknologi + music)

### 1.3 Tone of Voice
- Formal tapi hangat (Bahasa Indonesia)
- Percaya diri, bukan sombong
- Benefit-over-feature ("Anak Anda tampil percaya diri" bukan "Course 8 pertemuan")
- Emosional headline, fakta body

---

## 2. Color System

### 2.1 Primary Palette

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| primary-500 | #2563EB | 37, 99, 235 | Buttons, links, active states |
| primary-600 | #1D4ED8 | 29, 78, 216 | Hover states |
| primary-50 | #EFF6FF | 239, 246, 255 | Light backgrounds, badges |

### 2.2 Accent Palette

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| accent-500 | #F59E0B | 245, 158, 11 | CTA WhatsApp, pricing, ratings |
| accent-600 | #D97706 | 217, 119, 6 | Accent hover |

### 2.3 Neutral Palette

| Token | Hex | Usage |
|-------|-----|-------|
| bg-primary | #F8FAFC | Main page background |
| surface | #FFFFFF | Cards, modals, forms |
| border-default | #E2E8F0 | Borders, dividers |
| border-light | #F1F5F9 | Table row separators |
| text-primary | #1E293B | Headings, important text |
| text-secondary | #64748B | Descriptions, labels |
| text-muted | #94A3B8 | Placeholders, hints |

### 2.4 Semantic Colors

| Token | Hex | Usage |
|-------|-----|-------|
| success | #10B981 | Active status, success messages |
| warning | #F59E0B | Pending status, warnings |
| error | #EF4444 | Error messages, overdue |
| info | #2563EB | Info messages |

### 2.5 Dark Mode (Optional Toggle)

| Token | Hex | Usage |
|-------|-----|-------|
| dark-bg | #0F172A | Page background |
| dark-surface | #1E293B | Cards |
| dark-text | #F1F5F9 | Text on dark |

### 2.6 Color Rules
- Primary blue (#2563EB) untuk SEMUA interactive elements
- Gold (#F59E0B) HANYA untuk CTA WhatsApp dan pricing highlights
- NEVER use neon green (#08CB00)
- NEVER use pure black (#000) for backgrounds
- NEVER use gradient on cards/buttons (hanya hero + CTA sections)

---

## 3. Typography

### 3.1 Font Families

| Token | Font | Usage |
|-------|------|-------|
| font-heading | Plus Jakarta Sans | All headings (h1-h6) |
| font-body | Inter | Body text, inputs, labels |
| font-mono | JetBrains Mono | Code, IDs, technical data |

### 3.2 Type Scale

| Token | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| hero | 3rem (48px) | 800 | 1.2 | Landing hero headline |
| h1 | 1.875rem (30px) | 700 | 1.3 | Page titles |
| h2 | 1.5rem (24px) | 700 | 1.4 | Section titles |
| h3 | 1.125rem (18px) | 700 | 1.4 | Card titles |
| body-lg | 1rem (16px) | 500 | 1.6 | Important body text |
| body | 1rem (16px) | 400 | 1.6 | Default body text |
| body-sm | 0.875rem (14px) | 400 | 1.5 | Secondary text |
| caption | 0.75rem (12px) | 400 | 1.4 | Timestamps, hints |

### 3.3 Typography Rules
- Headings: Plus Jakarta Sans, NEVER Inter
- Body minimum: 14px (accessibility)
- iOS zoom prevention: inputs minimum 16px
- Text color on white bg: minimum contrast ratio 4.5:1 (WCAG AA)
- Indonesian locale for dates: "3 Juli 2026" (bukan "July 3, 2026")

---

## 4. Spacing & Layout

### 4.1 Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| xs | 0.25rem (4px) | Tight gaps |
| sm | 0.5rem (8px) | Icon gaps, small margins |
| md | 1rem (16px) | Standard padding |
| lg | 1.5rem (24px) | Card padding |
| xl | 2rem (32px) | Section padding |
| 2xl | 3rem (48px) | Major section spacing |
| 3xl | 4rem (64px) | Page section breaks |

### 4.2 Layout Grid

| Breakpoint | Width | Columns | Gutter | Margin |
|------------|-------|---------|--------|--------|
| Mobile | < 640px | 1 | 16px | 16px |
| Tablet | 640-1024px | 2-3 | 16px | 24px |
| Desktop | > 1024px | 4-12 | 24px | auto (max 1280px) |

### 4.3 Component Sizing

| Component | Height | Width |
|-----------|--------|-------|
| Button (default) | 44px min | Auto |
| Button (full-width) | 44px min | 100% |
| Input | 44px min | 100% |
| Card | Auto | 100% or grid |
| Sidebar (desktop) | 100vh | 256px |
| Bottom nav (mobile) | 64px | 100% |
| Table row | 48px | 100% |

### 4.4 Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| sm | 0.375rem (6px) | Badges, small elements |
| md | 0.5rem (8px) | Buttons, inputs |
| lg | 0.75rem (12px) | Cards |
| xl | 1rem (16px) | Large cards, modals |
| full | 9999px | Circular (avatars, pills) |

### 4.5 Shadows

| Token | Value | Usage |
|-------|-------|-------|
| sm | 0 1px 2px rgba(0,0,0,0.05) | Subtle depth |
| md | 0 4px 20px rgba(0,0,0,0.06) | Cards |
| lg | 0 8px 25px rgba(0,0,0,0.1) | Cards on hover |
| xl | 0 20px 50px rgba(0,0,0,0.15) | Modals, dropdowns |

---

## 5. Component Library

### 5.1 Buttons

#### Primary Button
```
Classes: bg-[#2563EB] text-white font-semibold rounded-lg py-2.5 px-5
Hover: bg-[#1D4ED8]
Active: scale(0.98)
Disabled: opacity-50 cursor-not-allowed
Min height: 44px
```

#### Secondary Button
```
Classes: border border-gray-300 bg-white text-gray-900 font-medium rounded-lg py-2.5 px-5
Hover: bg-gray-50
Border on focus: ring-2 ring-[#2563EB]/20
```

#### Gold CTA Button (WhatsApp)
```
Classes: bg-[#F59E0B] text-black font-bold rounded-full py-3 px-8
Hover: bg-[#D97706]
Shadow: 0 4px 15px rgba(245, 158, 11, 0.3)
Only for: WhatsApp links, pricing CTAs
```

#### Icon Button
```
Classes: p-2 rounded-lg hover:bg-gray-100
Min size: 44px × 44px (touch target)
```

### 5.2 Cards

#### Default Card
```
Classes: bg-white border border-gray-200 rounded-xl p-6
Shadow: none (flat design)
Hover: translateY(-2px) shadow-lg (0.3s transition)
```

#### KPI Card (Dashboard)
```
Layout: flex items-center justify-between
Left: text-sm text-gray-500 (label)
Right: text-2xl font-bold text-gray-900 (value)
Optional: icon in primary-50 circle
```

#### Profile Card
```
Layout: flex items-center gap-4
Avatar: 48px circle, bg-gray-200, initials
Info: name (font-bold) + role (text-sm text-gray-500)
```

### 5.3 Forms

#### Text Input
```
Classes: w-full rounded-lg border border-gray-300 bg-white py-2.5 px-4 text-sm
Focus: border-[#2563EB] ring-2 ring-[#2563EB]/20
Placeholder: text-gray-400
Error: border-red-500 ring-2 ring-red-500/20
Font size: 16px minimum (iOS zoom prevention)
```

#### Select Dropdown
```
Same as text input + appearance-none + custom chevron icon
Options: text-sm, bg-white
```

#### Textarea
```
Same as text input + min-height 100px + resize-vertical
```

#### Checkbox
```
Size: 18px × 18px
Checked: bg-[#2563EB] border-[#2563EB]
Unchecked: border-gray-300
Label: ml-3 text-sm text-gray-700
```

#### Radio Button
```
Same as checkbox but rounded-full
Selected inner dot: bg-[#2563EB]
```

#### Form Validation Pattern
```
1. On page load: fields empty, no errors shown
2. On blur: validate field, show error if invalid
3. On change (after first blur): validate on every change
4. On submit: validate all fields, show all errors
5. Error position: below input, text-xs text-red-600
6. Success indicator: green checkmark icon right of input
```

### 5.4 Tables

#### Desktop Table
```
Container: overflow-x-auto rounded-xl border border-gray-200 bg-white
Header: bg-gray-50 text-sm font-medium text-gray-500
Cells: px-4 py-3 text-sm
Row hover: bg-gray-50
Row border: border-b border-gray-100
Status badges: rounded-full px-2.5 py-0.5 text-xs font-medium
```

#### Mobile Card View
```
Each row becomes a card:
- Rounded-xl border border-gray-200 bg-white p-4
- Key-value pairs stacked vertically
- Label: text-xs text-gray-500
- Value: text-sm font-medium text-gray-900
- First row has border-b border-gray-100
```

### 5.5 Badges & Status

| Status | Background | Text | Icon |
|--------|-----------|------|------|
| Active / Terverifikasi | bg-green-100 | text-green-700 | ✓ |
| Pending / Menunggu | bg-yellow-100 | text-yellow-700 | ⏳ |
| Overdue / Terlambat | bg-red-100 | text-red-700 | ⚠️ |
| Completed / Selesai | bg-blue-100 | text-blue-700 | ✅ |
| Cancelled / Dibatalkan | bg-gray-100 | text-gray-600 | ✕ |

### 5.6 Toast Notifications

```
Position: fixed top-4 right-4 z-50
Width: max-w-sm
Animation: slide-in from right (0.3s)
Auto-dismiss: success 3s, error 5s, warning 5s, info 3s
Close button: ✕ on top-right

Types:
- Success: bg-white border-l-4 border-green-500, icon ✓
- Error: bg-white border-l-4 border-red-500, icon ✕
- Warning: bg-white border-l-4 border-yellow-500, icon ⚠️
- Info: bg-white border-l-4 border-blue-500, icon ℹ️
```

### 5.7 Confirmation Dialog

```
Backdrop: bg-black/50 backdrop-blur-sm
Card: bg-white rounded-xl max-w-md mx-auto p-6
Animation: fade-in backdrop + slide-up card

Severity levels:
1. Simple (info): Blue icon, "Konfirmasi" title, blue confirm button
2. Destructive (danger): Red icon, "Hapus?" title, red confirm button
3. Financial: Gold icon, "Pembayaran" title, gold confirm button
4. Complex: Purple icon, "Perubahan" title, blue confirm button

Buttons: Cancel (secondary) + Confirm (primary/danger)
Loading state: spinner on confirm button
```

### 5.8 Search Input

```
Icon: 🔍 on left (text-gray-400)
Input: same as text input with pl-10
Clear button: ✕ on right (visible only when value not empty)
Debounce: 300ms
```

### 5.9 Filter Chips

```
Container: flex flex-wrap gap-2
Chip: rounded-full px-3 py-1.5 text-xs font-medium
Inactive: border border-gray-300 bg-white text-gray-600
Active: bg-[#2563EB] text-white
Hover (inactive): border-[#2563EB] text-[#2563EB]
```

### 5.10 Pagination

```
Layout: flex items-center justify-between
Left: "Menampilkan 1-20 dari 45 data" (text-sm text-gray-500)
Right: Prev button | "1 / 3" | Next button
Buttons: rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-sm
Disabled: opacity-40
```

### 5.11 Breadcrumbs

```
Nav: aria-label="Breadcrumb" mb-4 text-sm text-gray-500
Items: separated by /
First: "Dashboard" (always linked)
Last: text-gray-900 font-medium (not linked)
Middle: hover:text-[#2563EB]
```

### 5.12 Empty State

```
Container: border border-dashed border-gray-300 bg-white py-16 px-6 text-center
Icon: 5xl emoji (mb-4)
Title: font-heading text-lg font-bold text-gray-900
Description: mt-2 max-w-sm text-sm text-gray-500
CTA: primary button (mt-6)
```

### 5.13 Loading Skeleton

```
Animation: animate-pulse (1.5s infinite)
Color: bg-gray-200
Border radius: rounded-lg
Variants:
- SkeletonCard: h-32 rounded-xl
- SkeletonTable: 5 rows of h-12 rounded-lg
- SkeletonText: h-4 w-2/3
```

### 5.14 Sidebar (Desktop)

```
Width: 256px
Height: 100vh (fixed)
Background: bg-white
Border: border-r border-gray-200
Sections:
 - Logo area: py-6 px-4
 - Nav items: px-3, each with icon + label
 - Active: bg-[#2563EB] text-black (NOTE: dark text on blue)
 - Inactive: text-gray-500 hover:bg-gray-100
 - Theme toggle at bottom
```

### 5.15 Bottom Nav (Mobile)

```
Position: fixed bottom-0, full width
Height: 64px
Background: bg-white
Border: border-t border-gray-200
Items: 4-5 max
Each: flex-col items-center, icon + label (text-[10px])
Active: text-[#2563EB]
Inactive: text-gray-500
Only visible for: guardian, teacher roles
```

### 5.16 Theme Toggle

```
Component: sun/moon icon button
Position: bottom of sidebar
Behavior: toggles .dark class on <html>
Storage: localStorage key "theme"
Default: light
Animation: icon swap (no transition needed)
```

---

## 6. Page Specifications

### 6.1 Landing Page (/)

#### 6.1.1 Navbar
- Position: sticky top-0
- Background: bg-white, border-b border-gray-100
- Left: Logo text "EQS" (font-heading font-bold text-xl)
- Center: Nav links (Beranda, Instrumen, Tentang, Kontak)
- Right: "Masuk" (text link) + "Daftar Course" (primary button)
- Mobile: hamburger menu

#### 6.1.2 Hero Section
- Background: bg-gradient-to-br from-[#2563EB] to-[#7C3AED]
- Padding: py-24
- Headline: "Music Beyond Hobbies." (text-white, hero size)
- Subheadline: "Music Builds the Future." (text-white/80)
- Description: "Bersama Equalizer-Stake, anak Anda learn music dari instructor profesional..." (text-white/70)
- CTA: Gold WhatsApp button + secondary white outline button
- Trust bar: 4 items (150+ Student, 9 Instrumen, 10+ Tahun, 4.9★)

#### 6.1.3 Instruments Section
- Background: bg-[#F8FAFC]
- 3-column grid of instrument cards
- Each: emoji + name + price + "Mulai dari Rp XXrb/bulan"

#### 6.1.4 Benefits Section
- Background: bg-white
- 3-column cards: "Instructor Profesional", "Jadwal Fleksibel", "Studio Lengkap"

#### 6.1.5 Testimonials Section
- Background: bg-[#F8FAFC]
- 3 parent testimonial cards
- Each: avatar (initials) + name + child's instrument + quote + star rating

#### 6.1.6 Pricing Section
- Background: bg-white
- 3 tier cards: Trial (Rp 150k), Reguler (Rp 300k), Premium (Rp 500k)
- Features list per tier
- Gold CTA buttons

#### 6.1.7 FAQ Section
- Background: bg-[#F8FAFC]
- Accordion style (click to expand)
- 5-7 questions

#### 6.1.8 CTA Section
- Background: bg-gradient-to-br from-[#2563EB] to-[#7C3AED]
- Headline + gold WhatsApp button

#### 6.1.9 Footer
- Background: bg-gray-900
- 4 columns: Brand, Link, Kontak, Sosial
- Copyright bar

### 6.2 Login Page (/login)

- Centered layout with AuthLayout (light gradient bg)
- White card (max-w-md, p-8)
- Title: "Masuk" (blue, font-heading)
- Subtitle: "Kelola jadwal music Anda dalam satu tempat"
- Fields: Username, Password (with eye toggle)
- Button: "Masuk" (primary, full-width)
- Links: "Belum punya akun? Daftar" + "Lupa Password?"
- Error display: inline below fields (text-red-600)
- Loading: button shows spinner, disabled state

### 6.3 Register Page (/register)

- Same layout as login
- Fields: Username, Password, Confirm Password, Role (select), Full Name
- Phone number field for guardian/teacher
- Button: "Daftar"
- Link: "Sudah punya akun? Masuk"

### 6.4 Dashboard (/dashboard)

#### 6.4.1 Admin/Owner Dashboard
- Greeting: "Selamat pagi, [Nama]"
- Date: "Senin, 7 Juli 2026"
- KPI cards (4-column grid):
 - Total Student Aktif (number, icon 👨‍🎓)
 - Sesi Bulan Ini (number, icon 📅)
 - Tagihan Pending (number + Rp format, icon 💰)
 - Tingkat Kehadiran (percentage, icon 📊)
- Recent Sessions table (5 rows)
- Recent Audit Log (5 rows)
- Quick links: Students, Schedule, Billing, Reports

#### 6.4.2 Teacher Dashboard
- Greeting: "Selamat pagi, [Nama Instructor]"
- KPI cards (4-column):
 - Sesi Hari Ini
 - Sesi Minggu Ini
 - Terverifikasi
 - Pending Verifikasi
- Today's Sessions list (cards, not table)
- Quick links: Lihat Jadwal, Absensi

#### 6.4.3 Guardian Dashboard
- Greeting: "Selamat pagi, [Nama Wali]"
- Children cards (1 per child):
 - Name + instrument + level
 - Upcoming sessions (max 2)
 - Pending invoice (if any)
- Quick links: Anak Saya, Jadwal, Tagihan, Profil

### 6.5 My Schedule (/dashboard/my-schedule)

- Breadcrumbs: Dashboard / Jadwal Saya
- Title: "Jadwal Saya"
- Teacher mode: instrument tabs filter
- Guardian mode: student selector dropdown
- Schedule list: date, time, teacher/student, instrument, room, status badge
- Status colors: completed (green), upcoming (blue), cancelled (red)

### 6.6 Students List (/dashboard/students)

- Breadcrumbs: Dashboard / Student
- Search input + filter chips (status: active, inactive, all)
- Table (desktop) / Cards (mobile)
- Columns: Name, Guardian, Instrument, Level, Status, Actions
- Actions: View, Edit, Delete (with confirmation dialog)
- Pagination: "Menampilkan 1-20 dari X data"
- Empty state: "Belum ada student terdaftar"

### 6.7 Attendance (/dashboard/attendance)

- Breadcrumbs: Dashboard / Absensi
- Date picker at top
- Session cards with:
 - Student name, time, instrument
 - GPS status (within/outside radius)
 - Photo upload area
 - Submit button (disabled until photo taken)
- Offline indicator: sync count badge

### 6.8 Billing (/dashboard/billing)

- Breadcrumbs: Dashboard / Tagihan
- Filter: status (all, pending, paid, overdue)
- Table/cards: Invoice #, Student, Amount (Rp format), Due Date, Status, Actions
- Actions: View, Mark Paid (with confirmation)
- Overdue highlighting: red text + warning icon

### 6.9 Notifications (/dashboard/notifications)

- Breadcrumbs: Dashboard / Notifikasi
- List of notification cards
- Each: icon + title + description + timestamp
- Unread: blue left border + bold text
- Click to mark as read
- Mark all as read button

### 6.10 Profile (/dashboard/profile)

- Breadcrumbs: Dashboard / Profil
- Photo upload section (avatar with initials fallback)
- Edit profile form: Full Name, Email, Phone, Address
- Change password section
- Save button + success toast

### 6.11 Settings (/dashboard/settings)

- Breadcrumbs: Dashboard / Pengaturan
- Notification preferences (toggles)
- Theme toggle (light/dark)
- Language (Indonesian only for now)

### 6.12 Feedback (/dashboard/feedback) — Instructor only

- Breadcrumbs: Dashboard / Feedback
- Form: Session ID, Rating (1-5 slider), Category (dropdown), Comment (textarea)
- Submit button
- Success toast on submit

### 6.13 Ratings (/dashboard/ratings) — Guardian only

- Breadcrumbs: Dashboard / Rating Instructor
- Form: Teacher ID, Rating (1-5 slider), Comment (textarea)
- Submit button (gold theme)
- Success toast on submit

### 6.14 Help Page (/help)

- FAQ accordion (5-7 questions)
- Contact info card: Address, WhatsApp, Hours
- Back to dashboard link

### 6.15 Instruments Page (/instruments)

- Public page (no auth required)
- 9 instrument cards in grid
- Each: emoji + name + description + price
- CTA: "Daftar via WhatsApp" (gold button)
- Navbar + footer (no sidebar)

---

## 7. User Flows

### 7.1 New Student Registration Flow
```
1. Landing page → Click "Daftar Course"
2. Register page → Fill form (name, phone, email, instrument, schedule preference)
3. Submit → Success message → Redirect to login
4. Admin reviews → Creates student + guardian records
5. Guardian receives WhatsApp notification → Account activated
6. Guardian logs in → Dashboard shows children
```

### 7.2 Login Flow
```
1. Click "Masuk" on landing page
2. Enter username + password
3. Validate → If error: show inline error below field
4. If success → Store token + role in localStorage
5. Redirect based on role:
 - admin/owner → /dashboard (admin view)
 - teacher → /dashboard (teacher view)
 - guardian → /dashboard (guardian view)
```

### 7.3 Attendance Flow (Teacher)
```
1. Teacher opens Attendance page
2. Sees today's sessions list
3. Selects a session → GPS check (150m radius)
4. If within radius → Camera opens for photo
5. Photo taken → Submit attendance
6. If outside radius → Error: "Anda harus berada di dalam area studio"
7. Offline → Queue for sync when online
```

### 7.4 Payment Flow (Guardian)
```
1. Guardian sees pending invoice on dashboard
2. Clicks "Bayar" → Confirmation dialog
3. Shows: amount, due date, payment instructions
4. Confirms → Redirect to WhatsApp with payment proof
5. Admin verifies payment → Invoice marked as "paid"
6. Guardian sees updated status
```

### 7.5 Session Booking Flow
```
1. Admin opens Schedule page
2. Clicks "Buat Sesi Baru"
3. Fills: teacher, student, room, date, time
4. System checks: room availability, teacher schedule
5. If conflict → Error: "Ruangan/instructor sudah terjadwal"
6. If OK → Create session → Notification to teacher + guardian
```

### 7.6 Feedback Flow
```
1. Teacher finishes session
2. Opens Feedback page
3. Enters session ID → Selects rating (1-5) + category + comment
4. Submits → Feedback stored
5. Guardian can view feedback on their dashboard
```

---

## 8. Responsive Behavior

### 8.1 Breakpoints

| Breakpoint | Width | Behavior |
|------------|-------|----------|
| Mobile S | 320px | Single column, stacked cards, bottom nav |
| Mobile L | 425px | Single column, slightly more space |
| Tablet | 768px | 2-column grids, sidebar hidden |
| Laptop | 1024px | Sidebar visible, 3-column grids |
| Desktop | 1280px | Full layout, 4-column grids |

### 8.2 Component Behavior

| Component | Mobile | Tablet | Desktop |
|-----------|--------|--------|---------|
| Sidebar | Hidden (bottom nav) | Hidden | Visible (256px) |
| Tables | Card view | Card view | Table view |
| KPI cards | 1-column | 2-column | 4-column |
| Forms | Full-width inputs | Full-width | Max-w-lg |
| Modals | Full-width | Max-w-md | Max-w-md |
| Breadcrumbs | Hidden on very small | Visible | Visible |
| Page title | text-xl | text-2xl | text-2xl |

### 8.3 Touch Targets
- Minimum: 44px × 44px for ALL interactive elements
- Buttons: min-h-[44px]
- Links in nav: py-2.5 minimum
- Checkboxes/radios: 18px minimum + surrounding padding

---

## 9. Accessibility

### 9.1 WCAG 2.1 Level A Requirements

| Criterion | Implementation |
|-----------|---------------|
| 1.1.1 Non-text Content | Alt text on images, aria-label on icon-only buttons |
| 1.3.1 Info & Relationships | Semantic HTML: nav, main, aside, h1-h6 hierarchy |
| 1.4.1 Use of Color | Status uses icon + color (not color alone) |
| 1.4.3 Contrast (Minimum) | 4.5:1 text contrast, 3:1 large text |
| 2.1.1 Keyboard | All interactive elements keyboard accessible |
| 2.4.1 Bypass Blocks | Skip to main content link |
| 2.4.2 Page Titled | Descriptive page titles |
| 2.4.6 Headings and Labels | Descriptive headings, visible labels |
| 3.3.1 Error Identification | Inline error messages below fields |
| 3.3.2 Labels or Instructions | Visible labels on all form fields |
| 4.1.2 Name, Role, Value | ARIA attributes on custom components |

### 9.2 Keyboard Navigation
- Tab order: logical left-to-right, top-to-bottom
- Focus visible: ring-2 ring-[#2563EB]/20 on all focusable elements
- Escape key: closes modals, dropdowns
- Enter/Space: activates buttons, checkboxes
- Arrow keys: navigates within radio groups, tabs

### 9.3 Screen Reader Support
- All images: descriptive alt text or alt="" for decorative
- All buttons: aria-label when using icon only
- All forms: associated labels (htmlFor)
- All status: aria-live="polite" for dynamic content
- All modals: role="dialog", aria-modal="true"

---

## 10. Animation & Motion

### 10.1 Timing Functions

| Type | Duration | Easing | Usage |
|------|----------|--------|-------|
| Instant | 100ms | ease | Button press |
| Fast | 200ms | ease-out | Fade in, color change |
| Normal | 300ms | ease-in-out | Slide, expand |
| Slow | 500ms | ease-in-out | Page transition |

### 10.2 Animations

| Animation | Trigger | Properties |
|-----------|---------|------------|
| Fade in | Page load, modal open | opacity 0→1, 200ms |
| Slide up | Modal, toast | translateY(20px)→0, opacity 0→1, 300ms |
| Slide in right | Toast | translateX(100%)→0, 300ms |
| Scale | Button active | scale(0.98), 100ms |
| Lift | Card hover | translateY(-2px) + shadow, 200ms |
| Pulse | Loading skeleton | opacity animation, 1.5s infinite |
| Bounce | Notification badge | translateY oscillation, 1s infinite |

### 10.3 Reduced Motion
```css
@media (prefers-reduced-motion: reduce) {
 *, *::before, *::after {
 animation-duration: 0.01ms !important;
 transition-duration: 0.01ms !important;
 }
}
```

---

## 11. Error Handling UI

### 11.1 Inline Form Errors
- Position: below input field
- Style: text-xs text-red-600
- Icon: ⚠️ before text
- Appears: on blur (first time) or on change (after first blur)
- Disappears: when field becomes valid

### 11.2 API Error Toast
- Type: error (red border-left)
- Title: "Gagal" (bold)
- Message: specific error from API (e.g., "Username atau password salah")
- Auto-dismiss: 5 seconds

### 11.3 Page Error State
- 404: centered, 🔍 emoji, "404", "Halaman tidak ditemukan", blue CTA button
- 500: centered, ⚠️ emoji, "Terjadi Kesalahan", "Silakan muat ulang", reload button
- Network error: "Tidak ada koneksi internet", retry button

### 11.4 Empty States
- Icon: relevant emoji (📭 for no data, 🔍 for no search results)
- Title: descriptive (e.g., "Belum ada student terdaftar")
- Description: helpful context
- CTA: action to fill data (e.g., "Tambah Student")

### 11.5 Confirmation Dialogs

| Action | Severity | Title | Confirm Color |
|--------|----------|-------|---------------|
| Delete item | Destructive | "Hapus [item]?" | Red |
| Mark as paid | Financial | "Konfirmasi Pembayaran?" | Gold |
| Save changes | Simple | "Simpan perubahan?" | Blue |
| Status change | Complex | "Ubah status?" | Blue |

### 11.6 Offline Indicator
- Banner at top: "Anda sedang offline. Data akan disinkronkan saat online."
- Background: bg-yellow-100, text-yellow-800
- Sync queue count shown

---

## 12. UAT Scenarios

### UC-01: Login (FR-01)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-01.1 | User on login page | Enter valid username + password + click Masuk | Redirect to dashboard with correct role view | |
| UAT-01.2 | User on login page | Enter wrong password | Show error "Username atau password salah" below password field | |
| UAT-01.3 | User on login page | Leave username empty + submit | Show error "Username wajib diisi" below username field | |
| UAT-01.4 | User on login page | Leave password empty + submit | Show error "Password wajib diisi" below password field | |
| UAT-01.5 | User on login page | Click eye icon on password | Password becomes visible text | |
| UAT-01.6 | User on login page | Click eye icon again | Password becomes dots again | |
| UAT-01.7 | User on login page | Click "Daftar" link | Redirect to register page | |
| UAT-01.8 | User on login page | Click "Lupa Password?" link | Redirect to forgot password page | |
| UAT-01.9 | Logged in user | Navigate to /login directly | Redirect to dashboard (already authenticated) | |

### UC-02: Register (FR-02)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-02.1 | User on register page | Fill all fields correctly + submit | Success message + redirect to login | |
| UAT-02.2 | User on register page | Enter duplicate username | Show error "Username sudah digunakan" | |
| UAT-02.3 | User on register page | Password < 8 chars | Show error "Password minimal 8 karakter" | |
| UAT-02.4 | User on register page | Password confirm doesn't match | Show error "Password tidak cocok" | |
| UAT-02.5 | User on register page | Invalid email format | Show error "Email tidak valid" | |

### UC-03: View Dashboard (FR-16)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-03.1 | Admin logged in | View dashboard | Shows admin KPI cards with real data | |
| UAT-03.2 | Teacher logged in | View dashboard | Shows teacher KPI cards + today's sessions | |
| UAT-03.3 | Guardian logged in | View dashboard | Shows children cards + upcoming sessions | |
| UAT-03.4 | Any role | View dashboard | Greeting shows "Selamat [time], [Name]" | |
| UAT-03.5 | Any role | View dashboard | KPI values match database counts | |
| UAT-03.6 | Any role | View dashboard | No dummy/hardcoded data shown | |

### UC-04: Manage Students (FR-03)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-04.1 | Admin on students page | Search by name | Table filters to matching students | |
| UAT-04.2 | Admin on students page | Click filter chip "Active" | Only active students shown | |
| UAT-04.3 | Admin on students page | Click "Tambah Student" | Student form opens | |
| UAT-04.4 | Admin filling student form | Leave required field empty | Inline error shown | |
| UAT-04.5 | Admin on students page | Click delete on student | Confirmation dialog appears | |
| UAT-04.6 | Admin on delete dialog | Click "Hapus" | Student deleted, toast "Student berhasil dihapus" | |
| UAT-04.7 | Admin on students page | Click delete, then "Batal" | Dialog closes, nothing deleted | |
| UAT-04.8 | Admin on students page | View pagination | Shows "Menampilkan 1-20 dari X data" | |
| UAT-04.9 | Admin on students page | No students exist | Empty state shown with CTA | |

### UC-05: Schedule Management (FR-04)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-05.1 | Admin on schedule page | Create session (teacher + student + room + date + time) | Session created, notification sent | |
| UAT-05.2 | Admin on schedule page | Create session with conflicting room | Error "Ruangan sudah terjadwal" | |
| UAT-05.3 | Admin on schedule page | Create session with conflicting teacher | Error "Instructor sudah terjadwal" | |
| UAT-05.4 | Teacher on schedule page | View schedule | Only own sessions shown | |
| UAT-05.5 | Teacher on schedule page | Filter by instrument | Sessions filtered correctly | |
| UAT-05.6 | Guardian on schedule page | Select child from dropdown | Sessions for that child shown | |
| UAT-05.7 | Guardian on schedule page | Select "Semua Anak" | All children's sessions shown | |

### UC-06: Attendance (FR-05)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-06.1 | Teacher at studio (within 150m) | Submit attendance with photo | Attendance submitted successfully | |
| UAT-06.2 | Teacher outside studio (>150m) | Try to submit attendance | Error "Anda harus berada di dalam area studio" | |
| UAT-06.3 | Teacher with no internet | Submit attendance | Queued for sync, offline indicator shown | |
| UAT-06.4 | Teacher comes online | Previous attendance queued | Auto-sync, sync count decreases | |
| UAT-06.5 | Admin views attendance | Check verification count | Matches submitted attendance records | |

### UC-07: Billing (FR-06)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-07.1 | System on 5th of month | Auto-generate invoices | Invoices created for all active students | |
| UAT-07.2 | Guardian views dashboard | Check pending invoices | Correct amount + due date shown | |
| UAT-07.3 | Admin marks invoice as paid | Confirm payment | Invoice status → "paid", audit logged | |
| UAT-07.4 | System on 11th of month | Invoice still unpaid | Fine Rp 100k added, status → "overdue" | |
| UAT-07.5 | Guardian on billing page | View invoice details | Amount, due date, status, payment history | |
| UAT-07.6 | Admin on billing page | Filter by status "overdue" | Only overdue invoices shown | |

### UC-08: Notifications (FR-07)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-08.1 | User with new notification | View sidebar | Red badge shows unread count | |
| UAT-08.2 | User with new notification | Click notification icon | Badge disappears, notifications page opens | |
| UAT-08.3 | User on notifications page | Mark notification as read | Read status updated, badge count decreases | |
| UAT-08.4 | User on notifications page | Click "Semua dibaca" | All notifications marked as read | |
| UAT-08.5 | Admin creates session | System generates notification | Teacher + guardian receive notification | |

### UC-09: Reports (FR-11)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-09.1 | Admin on reports page | Export revenue CSV | File downloads with correct data | |
| UAT-09.2 | Admin on reports page | Export attendance CSV | File downloads with attendance data | |
| UAT-09.3 | Admin on reports page | Export with date filter | Only data within date range | |
| UAT-09.4 | Owner views dashboard | Check revenue summary | Matches billing records | |

### UC-10: Profile Management (FR-14)

| Scenario | Given | When | Then | Pass/Fail |
|----------|-------|------|------|-----------|
| UAT-10.1 | User on profile page | Click "Edit Profil" | Form fields become editable | |
| UAT-10.2 | User editing profile | Change name + save | Success toast, name updated | |
| UAT-10.3 | User on profile page | Change password (valid) | Success toast, redirected to login | |
| UAT-10.4 | User on profile page | Change password (wrong current) | Error "Password saat ini salah" | |
| UAT-10.5 | User on profile page | Upload photo | Photo preview updates | |

---

## 13. Test Cases

### 13.1 Authentication Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-01 | Login with valid credentials | 1. Navigate to /login<br>2. Enter username: admin<br>3. Enter password: admin123<br>4. Click "Masuk" | Redirect to /dashboard, greeting "Selamat pagi, Admin" | P0 |
| TC-02 | Login with invalid password | 1. Navigate to /login<br>2. Enter username: admin<br>3. Enter password: wrong<br>4. Click "Masuk" | Error "Username atau password salah" below password field | P0 |
| TC-03 | Login with empty fields | 1. Navigate to /login<br>2. Leave both fields empty<br>3. Click "Masuk" | Errors "Username wajib diisi" and "Password wajib diisi" | P0 |
| TC-04 | Password visibility toggle | 1. Navigate to /login<br>2. Enter password "test123"<br>3. Click eye icon | Password shows as "test123"<br>4. Click eye again → shows as dots | P1 |
| TC-05 | Register new user | 1. Navigate to /register<br>2. Fill all fields correctly<br>3. Click "Daftar" | Success message, redirect to /login | P0 |
| TC-06 | Register duplicate username | 1. Navigate to /register<br>2. Enter existing username<br>3. Submit | Error "Username sudah digunakan" | P1 |
| TC-07 | JWT token expiry | 1. Login<br>2. Wait for token to expire<br>3. Navigate to /dashboard | Redirect to /login | P0 |
| TC-08 | Logout | 1. Login<br>2. Click logout<br>3. Try to access /dashboard | Redirect to /login | P0 |

### 13.2 Dashboard Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-09 | Admin dashboard loads real data | 1. Login as admin<br>2. View dashboard | KPI cards show real DB counts | P0 |
| TC-10 | Teacher dashboard shows today's sessions | 1. Login as teacher<br>2. View dashboard | Today's sessions listed with correct data | P0 |
| TC-11 | Guardian dashboard shows children | 1. Login as guardian<br>2. View dashboard | Children cards with names + instruments | P0 |
| TC-12 | Dashboard personal greeting | 1. Login as any role<br>2. Check greeting | "Selamat [pagi/siang/sore], [Name]" | P1 |
| TC-13 | Dashboard date display | 1. Login<br>2. Check date | Shows today's date in Indonesian format | P2 |
| TC-14 | Dashboard loading state | 1. Login<br>2. Observe initial load | Skeleton placeholders shown during fetch | P2 |

### 13.3 Student Management Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-15 | List students | 1. Login as admin<br>2. Navigate to /dashboard/students | Table shows all students | P0 |
| TC-16 | Search students | 1. Navigate to students<br>2. Type name in search | Table filters in real-time (300ms debounce) | P1 |
| TC-17 | Filter students by status | 1. Navigate to students<br>2. Click "Active" filter chip | Only active students shown | P1 |
| TC-18 | Add new student | 1. Click "Tambah Student"<br>2. Fill form<br>3. Submit | Student created, appears in list | P0 |
| TC-19 | Edit student | 1. Click edit on student<br>2. Change name<br>3. Save | Student updated, toast "Berhasil" | P1 |
| TC-20 | Delete student | 1. Click delete<br>2. Confirm in dialog | Student removed, toast "Berhasil dihapus" | P0 |
| TC-21 | Pagination | 1. Have 45+ students<br>2. View list | "Menampilkan 1-20 dari 45 data"<br>Next button enabled | P1 |
| TC-22 | Empty students list | 1. No students in DB<br>2. View list | Empty state with CTA "Tambah Student" | P2 |

### 13.4 Schedule Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-23 | View schedule (admin) | 1. Login as admin<br>2. View schedule | All sessions shown | P0 |
| TC-24 | Create session | 1. Click "Buat Sesi Baru"<br>2. Fill teacher, student, room, date, time<br>3. Submit | Session created, appears in list | P0 |
| TC-25 | Room conflict detection | 1. Create session<br>2. Try same room + same time | Error "Ruangan sudah terjadwal" | P0 |
| TC-26 | Teacher schedule filter | 1. Login as teacher<br>2. View schedule | Only own sessions shown | P0 |
| TC-27 | Guardian student filter | 1. Login as guardian<br>2. Select child from dropdown<br>3. View schedule | Only selected child's sessions shown | P1 |

### 13.5 Attendance Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-28 | Submit attendance (in range) | 1. Teacher at studio<br>2. Select session<br>3. Take photo<br>4. Submit | Attendance submitted, status "Terverifikasi" | P0 |
| TC-29 | Submit attendance (out of range) | 1. Teacher >150m from studio<br>2. Try to submit | Error: GPS radius exceeded | P0 |
| TC-30 | Offline attendance | 1. Teacher loses internet<br>2. Submit attendance | Queued locally, sync count shown | P1 |
| TC-31 | Auto-sync when online | 1. Teacher was offline<br>2. Internet restored | Queued attendance auto-syncs | P1 |

### 13.6 Billing Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-32 | View invoices | 1. Login as guardian<br>2. View billing | Invoices shown with correct amounts | P0 |
| TC-33 | Invoice status badges | 1. View invoices<br>2. Check statuses | Pending=yellow, Paid=green, Overdue=red | P1 |
| TC-34 | Mark invoice paid | 1. Admin selects invoice<br>2. Confirms payment | Status → "paid", audit logged | P0 |
| TC-35 | Auto-fine (11th of month) | 1. Invoice unpaid past 11th | Fine Rp 100k added, status → "overdue" | P1 |
| TC-36 | Export revenue CSV | 1. Admin clicks export<br>2. Selects date range | CSV downloads with correct data | P1 |

### 13.7 Responsive Design Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-37 | Mobile layout (375px) | 1. Resize browser to 375px<br>2. View dashboard | Bottom nav visible, sidebar hidden, cards stacked | P0 |
| TC-38 | Tablet layout (768px) | 1. Resize to 768px<br>2. View dashboard | 2-column grids, no sidebar | P1 |
| TC-39 | Desktop layout (1280px) | 1. Resize to 1280px<br>2. View dashboard | Sidebar visible, 4-column KPI grid | P0 |
| TC-40 | Table → Card (mobile) | 1. View students on 375px | Students shown as cards, not table | P1 |
| TC-41 | Touch targets (mobile) | 1. View on 375px<br>2. Tap buttons | All buttons minimum 44px height | P0 |
| TC-42 | iOS zoom prevention | 1. View on iPhone<br>2. Tap input field | No zoom on focus (font-size 16px) | P0 |

### 13.8 Theme Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-43 | Default theme is light | 1. Open app (new browser) | Light theme (#F8FAFC bg) | P0 |
| TC-44 | Toggle to dark | 1. Click theme toggle in sidebar | Dark theme applied, icon changes to sun | P1 |
| TC-45 | Theme persistence | 1. Toggle to dark<br>2. Refresh page | Dark theme persists (localStorage) | P1 |
| TC-46 | Toggle back to light | 1. In dark mode<br>2. Click toggle | Light theme restored | P1 |

### 13.9 Error Handling Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-47 | 404 page | 1. Navigate to /nonexistent | 404 page with "Halaman tidak ditemukan" | P1 |
| TC-48 | API error toast | 1. Trigger API error | Red error toast with message | P1 |
| TC-49 | Network error | 1. Disconnect internet<br>2. Try to load data | Error state with retry button | P1 |
| TC-50 | Form validation errors | 1. Submit empty form | All required field errors shown | P0 |

### 13.10 Accessibility Test Cases

| ID | Test Case | Steps | Expected Result | Priority |
|----|-----------|-------|-----------------|----------|
| TC-51 | Keyboard navigation | 1. Tab through login page | All elements focusable in logical order | P0 |
| TC-52 | Focus visible | 1. Tab to any button | Blue focus ring visible | P1 |
| TC-53 | Skip to content | 1. Tab from top | "Lewati ke konten utama" link appears | P1 |
| TC-54 | Screen reader | 1. Use NVDA/VoiceOver | All labels, headings, status read correctly | P1 |
| TC-55 | Color contrast | 1. Check text on white bg | Minimum 4.5:1 contrast ratio | P0 |

---

## 14. Acceptance Criteria Matrix

| FR | Feature | Acceptance Criteria | UAT | Test Cases |
|----|---------|-------------------|-----|------------|
| FR-01 | Login | Valid creds → dashboard; invalid → error; JWT stored | UAT-01.1..01.9 | TC-01..TC-08 |
| FR-02 | Register | Valid form → success; duplicate → error | UAT-02.1..02.5 | TC-05..TC-06 |
| FR-03 | Student CRUD | Create/read/update/delete with validation | UAT-04.1..04.9 | TC-15..TC-22 |
| FR-04 | Schedule | CRUD sessions with conflict detection | UAT-05.1..05.7 | TC-23..TC-27 |
| FR-05 | Attendance | GPS validation + photo + offline sync | UAT-06.1..06.5 | TC-28..TC-31 |
| FR-06 | Billing | Auto-invoice, payment tracking, fine | UAT-07.1..07.6 | TC-32..TC-36 |
| FR-07 | Notifications | Real-time + badge + mark read | UAT-08.1..08.5 | TC-08 |
| FR-08 | Reports | CSV export with date filter | UAT-09.1..09.4 | TC-36 |
| FR-09 | Audit Log | Immutable, 7-year retention | — | — |
| FR-10 | Theme | Light default + dark toggle + persist | — | TC-43..TC-46 |
| FR-11 | Responsive | 375px → 1280px+ | — | TC-37..TC-42 |
| FR-12 | Accessibility | WCAG 2.1 Level A | — | TC-51..TC-55 |
| FR-13 | Error Handling | 404, 500, validation, offline | — | TC-47..TC-50 |
| FR-14 | Profile | Edit info, change password, upload photo | UAT-10.1..10.5 | TC-14 |
| FR-15 | Feedback/Rating | Teacher→Guardian feedback, Guardian→Teacher rating | — | — |

---

## Appendix A: Indonesian Locale Reference

### Date Format
| Format | Example |
|--------|---------|
| Full date | "Senin, 7 Juli 2026" |
| Short date | "7 Jul 2026" |
| Time | "09:00 WIB" |
| DateTime | "7 Juli 2026, 09:00 WIB" |
| Relative | "2 jam yang lalu", "3 hari yang lalu" |

### Currency Format
| Format | Example |
|--------|---------|
| Full | "Rp 350.000" |
| Short (mobile) | "Rp 350rb", "Rp 1,5jt" |
| Range | "Rp 250.000 - Rp 500.000" |

### Status Labels
| English | Indonesian |
|---------|-----------|
| Active | Aktif |
| Inactive | Tidak Aktif |
| Pending | Menunggu |
| Completed | Selesai |
| Cancelled | Dibatalkan |
| Overdue | Terlambat |
| Verified | Terverifikasi |
| Draft | Draf |

### Day Names
| Short | Full |
|-------|------|
| Sn | Senin |
| Sl | Selasa |
| Rb | Rabu |
| Km | Kamis |
| Jm | Jumat |
| Sb | Sabtu |
| Mg | Minggu |

### Month Names
| Short | Full |
|-------|------|
| Jan | Januari |
| Feb | Februari |
| Mar | Maret |
| Apr | April |
| Mei | Mei |
| Jun | Juni |
| Jul | Juli |
| Ags | Agustus |
| Sep | September |
| Okt | Oktober |
| Nov | November |
| Des | Desember |
