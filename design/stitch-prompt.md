# Stitch Prototype Prompt — Equalizer-Stake

Gunakan prompt di bawah ini untuk membuat prototype di Stitch/Penzu/figma/mana saja.

---

## PROMPT UNTUK STITCH

```
Build a complete web-app prototype for "Equalizer-Stake" — a music school scheduling, billing, and attendance management system for BMD Music School, Yogyakarta, Indonesia.

## Design Direction
- Style: Modern, clean, professional. White/light background with subtle blue accents (#1E40AF primary, #3B82F6 secondary)
- Mobile-first responsive (320px → 480px → 768px → 1024px+)
- Indonesian language for all UI text, buttons, labels, and error messages
- High contrast mode support (WCAG 2.1 Level AA)
- Touch targets minimum 44×44px for mobile
- Cards with subtle shadows, rounded corners (12px), clean typography
- No dark mode

## System Overview
4 user roles: Owner/Superadmin, Admin, Guru (Teacher), Wali Murid (Guardian/Parent)
1 Guardian account manages N Students (siblings)
PWA — installable from browser

## SCREENS TO BUILD

### 1. Login Page
- Clean centered card layout
- Logo "BMD Music School" at top
- Username + password fields
- "Masuk" (Login) button — primary blue
- "Lupa Password?" link below
- Language: Indonesian

### 2. Owner/Superadmin Dashboard
- Sidebar navigation: Dashboard, Manajemen Admin, Audit Log, Eskalasi, Pengaturan
- Main area: KPI cards (Total Pendapatan, Tingkat Kehadiran, Akun Frozen, Utilisasi Studio)
- Recent activity feed
- "Ekspor Laporan" button
- Read-only feel (no edit buttons on data)

### 3. Admin Dashboard
- Sidebar navigation: Dashboard, Jadwal, Absensi Pending, Tagihan, Siswa, Guru, Notifikasi, Pengaturan
- Main area: Today's overview cards (Sesi Hari Ini, Pending Verifikasi, Tagihan Belum Bayar, Notifikasi Gagal)
- Quick action buttons: "Input Jadwal Baru", "Verifikasi Pembayaran"

### 4. Admin — Schedule Management (CRITICAL SCREEN)
- Calendar view (week view) with color-coded slots:
  - Green = available, Red = booked/locked, Yellow = pending, Gray = holiday
- "Tambah Jadwal Baru" button → opens form modal
- Schedule form:
  - Dropdown: Pilih Siswa
  - Dropdown: Pilih Guru
  - Dropdown: Pilih Ruangan
  - Date picker
  - Time picker (start + end)
  - Duration auto-calculated (30min or 45-50min)
  - "Simpan Jadwal" button
- Conflict warning banner (red) if slot taken
- "3 slot alternatif" suggestion panel when conflict
- Slot status badges: Draft, Pending, Locked (green lock icon), Completed, Cancelled

### 5. Admin — Attendance Review
- List of pending verifications with student photo thumbnails
- Each row: Student name, session time, Guru name, photo preview, GPS status (✓/✗), Face status (✓/✗)
- "Setuju" (green) and "Tolak" (red) buttons per row
- Filter: All / Pending / Verified / Rejected
- Photo lightbox on click (larger view)

### 6. Admin — Billing & Payments
- Table of invoices with columns: Wali Murid, Bulan, Nominal, Kode Unik, Status, Aksi
- Status badges: Lunas (green), Belum Bayar (red), Denda (orange), Beku (gray)
- "Verifikasi Pembayaran" modal:
  - Shows transfer details
  - Match unique code input
  - Confirm/reject buttons
- Invoice detail view: breakdown per student, total amount, unique code displayed prominently

### 7. Guru (Teacher) — Attendance Form (MOBILE VIEW)
- Mobile portrait layout (this is the primary mobile screen)
- Current session card: Siswa name, Jam, Ruangan
- Large camera area (tap to capture)
- Face detection overlay: green border when face detected ≥30%
- GPS status indicator: ✓ (dalam radius) or ✗ (luar radius)
- "Konfirmasi Absensi" button (large, blue, 44×44px minimum)
- Status feedback: "Hadir ✓" (green) or "Menunggu Verifikasi ⏳" (yellow)

### 8. Guru — My Schedule (MOBILE VIEW)
- Weekly calendar view (compact)
- Today highlighted
- Session cards: Time, Student, Room, Status
- "Absensi" button only for active/ongoing sessions

### 9. Wali Murid — Multi-Student Dashboard (MOBILE VIEW)
- Top: Student switcher tabs/pills
  - "Andi (Piano)" [Active — green badge]
  - "Sari (Gitar)" [Active — green badge]
  - "Budi (Violin)" [Frozen — red badge]
- Below: Selected student's schedule for this week
- Status cards: Next Session, Tagihan Aktif, Kehadiran Bulan Ini
- Notification bell with unread count

### 10. Wali Murid — Invoice View
- Invoice card: Bulan, Total, Kode Unik (3 digits, large, prominent)
- Kode unik displayed as: "Rp 350.XXX" where XXX is the unique code
- Transfer instruction text
- Status: Lunas / Belum Bayar / Denda
- If frozen: Red banner "Akun Dibekukan — Hubungi Admin"

### 11. Wali Murid — Studio Booking
- Calendar view for studio (time slots grid)
- Available slots: Green
- Booked slots: Red
- Tap to select → Confirmation modal
- "Konfirmasi Reservasi" button

### 12. Notification Center (All Roles)
- List of notifications with icons:
  - 🔔 Reminder jadwal (blue)
  - 💰 Tagihan (orange)
  - ✅ Pembayaran dikonfirmasi (green)
  - ⚠️ Akun dibekukan (red)
  - 📋 Absensi pending (yellow)
- Each: timestamp, message, status (sent/failed)

### 13. Admin — Bulk Operations
- Tab layout: Bulk Jadwal | Bulk Notifikasi | Bulk Verifikasi
- Bulk Jadwal: CSV upload area with drag-drop zone, preview table, import button
- Bulk Notifikasi: Recipient selector (all Wali Murid / filter), message textarea, send button
- Bulk Verifikasi: Checkbox list of invoices, "Konfirmasi Semua" button

### 14. Owner — Superadmin Panel
- Tab: Manajemen Admin | Deputasi | Pengaturan Sistem
- Manajemen Admin: Table of Admins with status, last login, actions (Nonaktifkan/Hapus)
- Deputasi: Form to set Admin B covering for Admin A with date range
- Pengaturan: WhatsApp gateway config, SMTP config, maintenance window

### 15. Escalation Form (Guru/Wali Murid)
- Modal/popup form
- Subjek dropdown: Jadwal, Pembayaran, Teknis, Keluhan
- Pesan textarea
- Lampiran upload (optional)
- "Kirim ke Owner" button

### 16. Lupa Password Page
- Email input field
- "Kirim Link Reset" button
- Success message: "Link reset telah dikirim ke email Anda"

## COMPONENT LIBRARY NEEDED

### Buttons
- Primary (blue): "Simpan", "Konfirmasi", "Masuk"
- Secondary (outline): "Batal", "Kembali"
- Danger (red): "Hapus", "Tolak", "Nonaktifkan"
- Success (green): "Setujui", "Verifikasi"
- Ghost: "Lihat Detail", "Ekspor"

### Badges/Tags
- Active (green): "Aktif", "Lunas", "Hadir"
- Pending (yellow): "Menunggu Verifikasi", "Pending"
- Error (red): "Beku", "Gagal", "Tolak"
- Info (blue): "Draft", "Locked"

### Form Elements
- Input fields with labels in Indonesian
- Dropdowns with search
- Date/time pickers
- Toggle switches (for notification channels)
- Radio buttons
- Checkbox lists

### Cards
- KPI cards (icon + number + label + trend arrow)
- Session cards (time + student + room + status)
- Invoice card (amount + unique code + status)
- Notification cards (icon + message + timestamp)

### Navigation
- Sidebar (desktop): Icons + labels, collapsible
- Bottom nav (mobile): 4-5 icons
- Breadcrumbs

### Tables
- Sortable headers
- Row actions (view/edit/delete icons)
- Pagination
- Status badges in cells

### Modals
- Confirmation dialogs
- Form modals (schedule, payment verify)
- Photo lightbox

### Alerts/Banners
- Success (green): "Jadwal tersimpan", "Pembayaran dikonfirmasi"
- Warning (yellow): "Menunggu verifikasi", "Session akan timeout"
- Error (red): "Konflik slot", "Akun dibekukan"
- Info (blue): "Draft tersimpan", "Notifikasi dikirim"

## KEY INTERACTIONS TO PROTOTYPE

1. Login → Dashboard (role-based redirect)
2. Admin: Tap "Tambah Jadwal" → Fill form → Save → Success toast
3. Admin: See conflict warning → View 3 alternatives → Pick one
4. Guru (mobile): Open absensi → Take photo → See face detection → Submit → Status feedback
5. Wali Murid (mobile): Switch between students → See different data
6. Wali Murid: View invoice → See unique code prominently → Copy code
7. Admin: Review pending attendance → Approve/Reject
8. Admin: Verify payment → Match code → Confirm
9. Owner: Manage Admin → Deactivate → Confirmation dialog
10. Wali Murid: Submit escalation → Form → Success
11. Admin: Bulk import → Upload CSV → Preview → Import
12. Any user: "Lupa Password" → Email → Reset flow

## TEXT/LANGUAGE
All UI text in Indonesian:
- Buttons: Masuk, Simpan, Batal, Konfirmasi, Hapus, Setuju, Tolak, Kirim, Ekspor
- Labels: Nama, Email, Password, Tanggal, Waktu, Ruangan, Guru, Siswa, Status, Keterangan
- Navigation: Dashboard, Jadwal, Absensi, Tagihan, Siswa, Guru, Notifikasi, Pengaturan
- Status: Aktif, Nonaktif, Beku, Lunas, Belum Bayar, Denda, Pending, Draft, Locked
- Messages: "Jadwal berhasil disimpan", "Konflik slot terdeteksi", "Pembayaran dikonfirmasi"
- Errors: "Kode unik tidak ditemukan", "Wajah tidak terdeteksi", "GPS di luar radius"
```

---

## CATATAN PENTING

1. **Mobile-first**: Guru dan Wali Murid PRIMARY di mobile. Admin di desktop.
2. **Unique Code**: Ini fitur kunci — kode 3 digit harus BESAR dan PROMINEN di invoice.
3. **Face Detection UI**: Overlay hijau saat wajah terdeteksi, panduan retake saat gagal.
4. **Status Badges**: Konsisten — hijau (aktif/lunas/hadir), kuning (pending), merah (beku/gagal), biru (draft/locked).
5. **Indonesian Language**: SEMUA teks dalam Bahasa Indonesia.
6. **Touch Targets**: Minimal 44×44px untuk mobile (guru di studio bising, harus mudah diketuk).
