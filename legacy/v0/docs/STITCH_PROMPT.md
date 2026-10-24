# Prompt untuk Stitch — UI/UX Rombak Total Equalizer-Stake
**App:** Equalizer-Stake — Sistem Penjadwalan Course & Pembayaran
**Stack:** Next.js 15 PWA, dark theme, mobile-first
**Users:** 4 role — Owner (Superadmin), Admin Operasional, Instructor, Guardian

---

## DESIGN SYSTEM

```
Background: #000000 (hitam)
Card: #111111 dengan border #253900 (hijau gelap)
Accent: #08CB00 (hijau terang — EQS brand)
Text Primary: #EEEEEE
Text Secondary: #9CA3AF (gray-400)
Input BG: #1a1a1a
Input Focus: border #08CB00
Button Primary: bg #08CB00, text #000000
Button Danger: bg red-500/20, text red-400
Badge Success: bg #08CB00/10, text #08CB00
Badge Warning: bg yellow-500/10, text yellow-500
Badge Error: bg red-500/10, text red-500
Font: Inter (body), Geist (headings)
Radius: 12px (cards), 8px (buttons), 6px (inputs)
Spacing: 16px (card padding), 24px (section gap)
Shadow: none (dark theme, gunakan border untuk depth)
```

---

## HALAMAN-HALAMAN

### GROUP 1: PUBLIC (tidak perlu login)

#### 1. Landing Page (/)
**Tujuan:** Marketing page, visitor tertarik daftar course
**Sections:**
- Hero: headline " Terpercaya di ", subheadline "Sejak 2010", CTA "Daftar Course"
- Tentang EQS: sejarah singkat, 9 instrumen, MSI standard
- Keunggulan: instructor profesional, kurikulum MSI, sertifikat ABRSM, konser orkestra
- Daftar Instrumen: grid 9 kartu instrumen (Piano, Gitar, Biola, Vokal, Drum, Keyboard, Flute, Saxophone, Bass) dengan ikon
- Harga: tabel perbandingan tipe kelas (Private, Intensif 2-3 org, Semi Intensif, Expatriate) dengan harga per bulan
- Testimoni: 3-5 testimoni placeholder
- CTA: "Daftar Sekarang" → /register-course
- Footer: alamat, telepon, WhatsApp, jam operasional
**Mobile:** Stack vertikal, hero full-width, instrumen grid 2 kolom

#### 2. Daftar Course (/register-course)
**Tujuan:** Calon student mendaftar course baru
**Form:**
- Nama Lengkap (text input)
- WhatsApp (text input, format 08xxx)
- Email (text input)
- Alamat (textarea)
- Instrumen (dropdown: 9 pilihan)
- Level (dropdown dinamis: berubah sesuai instrumen yang dipilih)
 - Piano Klasik: Pra Elementary, Elementary, Grade 1-6
 - Piano Pop: Beginner, Intermediate, Intermediate 2
 - Vokal/Biola/Gitar/Keyboard/Drum/Flute/Saxophone: Basic, Intermediate, Advance
- Tipe Kelas (dropdown: Private, Intensif 2 Student, Intensif 3 Student, Semi Intensif, Expatriate Private, Expatriate 2, Expatriate 3, Tunas/Balita, Paket Liburan)
- Tombol "Daftar"
**Setelah submit:** Success message "Pendaftaran berhasil! Kami akan menghubungi Anda via WhatsApp dalam 1x24 jam."
**Mobile:** Full-width form, single column

#### 3. Login (/login)bagaimana jika profile setiap role setiap user harus ada photonya? bagaimana jika instructor memberikan feedback ke guardian yang diajar? dan bagaimana guardian memberikan penilaian ke instructor
courses? apakah ada di srs? dan untuk tampilan dashboard guardian, haruslah mobile friendly, setuju? dan apakah halaman verifikasi email berhasil dan gagal ketika link lupa password sudah
dikirim ke email? dan bagaimana dengan page bantuan?

**Tujuan:** Semua role login
**Form:**
- Username (text input)
- Password (password input + eye toggle)
- Tombol "Masuk"
- Link "Lupa Password?" → /forgot-password
- Link "Belum punya akun? Daftar Course" → /register-course
**Error:** "Username atau password tidak valid" (generic, tidak leak info)
**First login:** Jika first_login=true → redirect ke /change-password
**Background:** ColorBends animated gradient (hijau EQS)

#### 4. Lupa Password (/forgot-password)
**Tujuan:** User minta reset password via email
**Form:**
- Email (text input)
- Tombol "Kirim Link Reset"
**Setelah submit:** "Jika email terdaftar, link reset telah dikirim. Cek inbox Anda."
**Background:** ColorBends animated gradient

#### 5. Reset Password (/reset-password?token=xxx)
**Tujuan:** User set password baru via link dari email
**Form:**
- Password Baru (password input + eye toggle, min 8 char)
- Konfirmasi Password (password input + eye toggle)
- Tombol "Simpan Password Baru"
**Validasi:** Token expired (>1 jam) → "Link sudah kedaluwarsa. Minta link baru."
**Success:** "Password berhasil diubah! Silakan login."
**Failure (token invalid):** "Link tidak valid. Minta link baru."

#### 6. Ganti Password Pertama (/change-password)
**Tujuan:** Force password change pada first login
**Form:**
- Password Saat Ini (password input)
- Password Baru (password input + eye toggle)
- Konfirmasi Password (password input)
- Tombol "Simpan"
**Validasi:** Password saat ini harus benar, password baru min 8 char, tidak boleh sama dengan lama
**Setelah sukses:** Redirect ke dashboard sesuai role

---

### GROUP 2: WALI MURID (MOBILE-FIRST, 320px minimum)

**PENTING:** Semua halaman Guardian HARUS mobile-first. 60% user akses via HP. Bottom navigation di bawah, sidebar di hide di mobile.

#### 7. Dashboard Guardian (/dashboard)
**Tujuan:** Overview ringkas untuk guardian
**Layout mobile:**
- Header: "Halo, [Nama Wali]" + avatar
- Kartu Anak: scrollable horizontal, setiap kartu = 1 anak (nama, instrumen, level, instructor, status)
- Jadwal Hari Ini: daftar sesi (jam, instructor, ruangan, status)
- Tagihan: kartu ringkas (total belum bayar, jatuh tempo terdekat)
- Quick Actions: "Bayar Sekarang", "Lihat Jadwal", "Lihat Anak"
**Layout desktop:** Grid 2 kolom

#### 8. Anak Saya (/dashboard/my-children)
**Tujuan:** Lihat detail semua anak yang terdaftar
**Content:**
- Kartu per anak: nama, foto placeholder, instrumen, level, instructor, status (aktif/cuti/trial)
- Eligible for exam badge (jika memenuhi syarat)
- Detail: jadwal per anak, riwayat kehadiran, progress
**Mobile:** Stack vertikal, satu kartu per baris

#### 9. Jadwal Saya (/dashboard/my-schedule)
**Tujuan:** Lihat jadwal course anak-anak
**Content:**
- Filter: minggu ini / bulan ini
- Daftar sesi: tanggal, jam, instructor, ruangan, student, status (pending/locked/completed/cancelled)
- Warna hijau = completed, kuning = pending, abu = cancelled
**Mobile:** List view, bukan tabel

#### 10. Pembayaran Saya (/dashboard/my-payments)
**Tujuan:** Lihat tagihan dan bayar
**Content:**
- Tab: Belum Bayar / Riwayat
- Kartu per invoice: periode, jumlah, jatuh tempo, kode unik 3 digit, status
- Tombol "Bayar" → modal upload bukti transfer (foto/screenshot) + input jumlah + pilih metode (Transfer Bank/Tunai/E-Wallet)
- Status: menunggu verifikasi, lunas, overdue
- Kode unik WAJIB terlihat jelas (3 digit, bold, hijau)
**Mobile:** Full-width kartu, tombol bayar sticky di bawah

#### 11. Notifikasi (/dashboard/notifications)
**Tujuan:** Lihat semua notifikasi
**Content:**
- Daftar notifikasi: icon (💰 tagihan, 📅 jadwal, ✅ verifikasi), pesan, waktu
- Badge unread (hijau dot)
- Filter: semua / belum dibaca
**Mobile:** List view

#### 12. Profil Saya (/dashboard/profile)
**Tujuan:** Edit profil dan ganti password
**Content:**
- Foto profil (upload foto baru / placeholder avatar dengan inisial nama)
- Nama, Email, WhatsApp, Alamat (editable)
- Ganti password section
- Tombol "Hapus Data Saya" (UU PDP) → konfirmasi modal
**Mobile:** Full-width form

#### 13. Beri Penilaian Instructor (/dashboard/rating) [BARU — belum di SRS]
**Tujuan:** Wali murid memberikan penilaian ke instructor course anak
**Content:**
- Pilih anak (dropdown jika punya >1 anak)
- Pilih instructor (auto-detect dari jadwal anak)
- Rating bintang (1-5)
- Komentar (textarea, opsional)
- Tombol "Kirim Penilaian"
- Riwayat penilaian sebelumnya
**Mobile:** Full-width form

---

### GROUP 3: GURU MUSIK

#### 14. Dashboard Instructor (/dashboard)
**Tujuan:** Overview jadwal hari ini
**Content:**
- Header: "Halo, [Nama Instructor]"
- Jadwal Hari Ini: daftar sesi (jam, student, ruangan, status)
- Statistik: total sesi bulan ini, tingkat kehadiran
- Quick Actions: "Absen Sekarang"

#### 15. Jadwal Saya (/dashboard/my-schedule)
**Tujuan:** Lihat jadwal mengajar
**Content:**
- Filter: hari ini / minggu ini / bulan ini
- Daftar sesi: tanggal, jam, student, ruangan, status
- Tombol "Absen" pada sesi hari ini yang belum di-absen

#### 16. Absen (/dashboard/attendance/submit)
**Tujuan:** Submit kehadiran dengan bukti foto + GPS
**Steps:**
1. Pilih sesi (radio button dari jadwal hari ini)
2. Ambil foto (camera capture atau upload, max 2MB, JPEG/PNG)
3. GPS auto-detect (browser geolocation)
4. Validasi jarak: ≤150m dari studio (-7.737428, 110.429669)
5. Submit → status "menunggu verifikasi admin"
**Mobile:** Full-width, tombol besar

#### 17. Beri Feedback ke Guardian (/dashboard/feedback) [BARU — belum di SRS]
**Tujuan:** Instructor memberikan feedback progress student ke guardian
**Content:**
- Pilih student (dropdown dari daftar student yang diajar)
- Kategori feedback: Kemajuan, Perilaku, Kehadiran, Rekomendasi
- Rating (1-5) per kategori
- Komentar (textarea)
- Tombol "Kirim Feedback"
- Riwayat feedback yang sudah dikirim
- Feedback ini akan terlihat oleh guardian di halaman "Anak Saya"

---

### GROUP 4: ADMIN OPERASIONAL

#### 18. Dashboard Admin (/dashboard)
**Tujuan:** Overview operasional harian
**Content:**
- 4 KPI cards: Total Student Aktif, Sesi Hari Ini, Tagihan Pending, Absensi Pending
- Jadwal hari ini (tabel)
- Quick actions: Buat Jadwal, Verifikasi Absensi, Verifikasi Pembayaran

#### 19. Pendaftaran (/dashboard/registrations)
**Tujuan:** Review pendaftaran calon student baru
**Content:**
- Tab: Pending / Disetujui / Ditolak
- Tabel: nama, WhatsApp, instrumen, level, tipe kelas, tanggal daftar
- Detail modal: data lengkap + tombol "Setujui" (assign instructor + jadwal) / "Tolak" (alasan)
- Saat setujui: auto-buat akun guardian + kirim notifikasi WA

#### 20. Jadwal (/dashboard/schedule)
**Tujuan:** Kelola jadwal course
**Content:**
- Tabel: tanggal, jam, instructor, student, ruangan, status
- Tombol "+ Buat Jadwal" → form modal (instructor, student, ruangan, tanggal, jam mulai, jam selesai)
- Conflict detection real-time (instructor/student/room overlap)
- Filter: tanggal, instructor, ruangan

#### 21. Absensi (/dashboard/attendance)
**Tujuan:** Verifikasi kehadiran instructor
**Content:**
- Tab: Pending / Verified / Rejected
- Tabel: tanggal, jam, instructor, student, ruangan, status
- Detail modal: foto bukti, GPS koordinat, jarak dari studio, face confidence
- Tombol "Verifikasi" / "Tolak" (alasan)

#### 22. Student (/dashboard/students)
**Tujuan:** Kelola data student
**Content:**
- Tabel: nama, instrumen, level, tipe kelas, instructor, guardian, status
- Search + filter
- Detail modal

#### 23. Tagihan (/dashboard/billing)
**Tujuan:** Kelola tagihan dan verifikasi pembayaran
**Content:**
- Tab: Pending / Lunas / Overdue
- Tabel: invoice, guardian, periode, jumlah, jatuh tempo, kode unik, status
- Detail modal: foto bukti transfer + input kode unik 3 digit
- Verifikasi: kode harus cocok → lunas
- Tolak: alasan

#### 24. Pembayaran (/dashboard/payments)
**Tujuan:** Riwayat semua pembayaran
**Content:**
- Tabel: tanggal, invoice, wali, jumlah, metode, status
- Filter: status, tanggal

#### 25. Libur Nasional (/dashboard/holidays)
**Tujuan:** Kelola hari libur
**Content:**
- Tabel: tanggal, nama, tipe (nasional/operasional)
- Tombol "+ Tambah Libur"

#### 26. Studio (/dashboard/studio)
**Tujuan:** Kelola ruangan studio
**Content:**
- Kartu per ruangan: nama, tipe, kapasitas, peralatan, status

#### 27. Harga (/dashboard/pricing)
**Tujuan:** Kelola harga course
**Content:**
- Tabel: instrumen, tipe kelas, level, SPP/bulan, durasi, biaya daftar
- Filter: instrumen, tipe kelas

#### 28. Notifikasi (/dashboard/notifications)
**Content:** sama seperti Guardian

#### 29. Pengaturan Notifikasi (/dashboard/notification-settings)
**Tujuan:** Toggle channel notifikasi per kategori
**Content:**
- Tabel: kategori (critical/non_critical), deskripsi, WhatsApp on/off, Email on/off

#### 30. Profil (/dashboard/profile)
**Content:** sama seperti Guardian

---

### GROUP 5: OWNER (SUPERADMIN)

#### 31. Dashboard Owner (/dashboard)
**Tujuan:** Executive summary — read-only
**Content:**
- 8 KPI cards: Total Pendapatan, Pendapatan Bulan Ini, Tingkat Kehadiran, Utilitas Studio, Total Student, Tagihan Belum Bayar, Akun Terkunci, Frozen Accounts
- Revenue trend (6 bulan terakhir)
- Per-studio breakdown
- Anomaly flags
- Audit log terbaru

#### 32. Kelola Admin (/dashboard/admins)
**Tujuan:** CRUD admin operasional
**Content:**
- Tabel: nama, email, status, terakhir login
- Tombol "+ Tambah Admin" → form (nama, email, WhatsApp)
- Tombol freeze/unfreeze

#### 33. Override (/dashboard/override)
**Tujuan:** Override keputusan admin
**Content:**
- Form: pilih sesi, pilih aksi (cancel/reschedule), alasan

#### 34. Settings (/dashboard/settings)
**Tujuan:** Pengaturan sistem
**Content:**
- Tabel: key, value, deskripsi, terakhir diubah

#### 35. Laporan (/dashboard/reports)
**Tujuan:** Laporan & analitik
**Content:**
- KPI cards (sama seperti dashboard owner)
- Revenue trend chart
- Per-studio breakdown
- Anomaly flags
- Tombol "Ekspor PDF" (printable HTML)

#### 36. Audit Log (/dashboard/audit-log)
**Tujuan:** Log semua aktivitas
**Content:**
- Tabel: timestamp, user, aksi, tabel, detail, kategori
- Filter: kategori (operasional/keamanan/kepatuhan), tanggal

---

### GROUP 6: PAGES KHUSUS

#### 37. Email Reset Password Berhasil (/reset-password/success)
**Tujuan:** Konfirmasi password berhasil diubah
**Content:**
- Ikon centang hijau besar
- "Password berhasil diubah!"
- "Silakan login dengan password baru Anda."
- Tombol "Login" → /login

#### 38. Email Reset Password Gagal (/reset-password/error)
**Tujuan:** Token invalid atau expired
**Content:**
- Ikon silang merah besar
- "Link tidak valid atau sudah kedaluwarsa."
- "Minta link reset baru."
- Tombol "Kirim Ulang" → /forgot-password

#### 39. Bantuan (/help) [BARU]
**Tujuan:** FAQ dan panduan penggunaan sistem
**Content:**
- Search bar (cari pertanyaan)
- Accordion FAQ per kategori:
 - **Umum**: Apa itu EQS? Bagaimana cara daftar? Berapa biaya course?
 - **Akun**: Bagaimana cara login? Lupa password? Ganti password?
 - **Pembayaran**: Bagaimana cara bayar? Apa itu kode unik? Kapan jatuh tempo? Apa yang terjadi jika terlambat?
 - **Jadwal**: Bagaimana cara lihat jadwal? Bisakah ganti jadwal? Apa yang terjadi jika absen?
 - **Absensi**: Bagaimana instructor absen? Apa itu GPS validation? Mengapa foto wajib?
 - **Privasi**: Bagaimana cara hapus data saya? Apa itu UU PDP?
- Kontak: WhatsApp admin, telepon, alamat
- Jam operasional
**Mobile:** Full-width accordion, search bar sticky di atas

#### 40. Bantuan Admin (/help/admin) [BARU]
**Tujuan:** Panduan khusus admin operasional
**Content:**
- Cara membuat jadwal
- Cara verifikasi absensi
- Cara verifikasi pembayaran (kode unik)
- Cara approve pendaftaran
- Cara freeze/unfreeze akun
- Cara ekspor laporan
- Troubleshooting umum
**Mobile:** Full-width accordion

#### 39. Bantuan (/help) [PUBLIC — tidak perlu login]
**Tujuan:** FAQ dan panduan penggunaan sistem
**Content:**
- Accordion FAQ per kategori:
 - **Umum:** Apa itu EQS? Bagaimana cara daftar? Berapa biaya?
 - **Pembayaran:** Kapan jatuh tempo? Bagaimana cara bayar? Apa itu kode unik?
 - **Jadwal:** Bagaimana jika terlambat? Bagaimana jika instructor absen? Apakah bisa ganti jadwal?
 - **Absensi:** Mengapa jam hangus? Berapa lama cuti? Apa yang terjadi jika lebih dari 1 bulan?
 - **Akun:** Bagaimana cara ganti password? Bagaimana cara hapus akun?
- Kontak: WhatsApp admin, telepon, alamat
- Jam operasional: Senin-Jumat 14:30-17:00, Sabtu 08:00-12:00
**Mobile:** Full-width accordion, tombol WhatsApp besar di bawah

#### 40. Bantuan Dashboard (/dashboard/help) [PRIVATE — perlu login]
**Tujuan:** Panduan penggunaan dashboard per role
**Content:**
- Panduan per role (auto-detect role user):
 - **Guardian:** Cara lihat tagihan, cara bayar, cara lihat jadwal anak
 - **Instructor:** Cara absen, cara lihat jadwal, cara beri feedback
 - **Admin:** Cara buat jadwal, cara verifikasi absensi, cara verifikasi pembayaran
 - **Owner:** Cara lihat laporan, cara kelola admin
- Link ke WhatsApp admin untuk bantuan langsung
**Mobile:** Full-width, step-by-step guide dengan screenshot placeholder

---

## UAT (User Acceptance Test)

### Alur 1: Visitor Daftar Course
```
1.1 Buka / → landing page tampil, tombol "Daftar Course" terlihat
1.2 Klik "Daftar Course" → /register-course
1.3 Isi form lengkap → submit
1.4 Success message tampil
1.5 Login admin → menu Pendaftaran → data baru terlihat
1.6 Admin klik "Setujui" → akun guardian dibuat
1.7 Guardian terima notifikasi WA
```

### Alur 2: Guardian Login & Lihat Anak
```
2.1 Login guardian → dashboard tampil
2.2 Klik "Anak Saya" → daftar anak tampil (data real dari API)
2.3 Klik "Jadwal Saya" → jadwal anak tampil
2.4 Klik "Pembayaran Saya" → invoice tampil dengan kode unik
```

### Alur 3: Admin Buat Jadwal
```
3.1 Login admin → menu Jadwal
3.2 Klik "+ Buat Jadwal" → form tampil
3.3 Isi instructor, student, ruangan, tanggal, jam → submit
3.4 Jadwal muncul di tabel
3.5 Login instructor → Jadwal Saya → jadwal baru terlihat
```

### Alur 4: Instructor Absen
```
4.1 Login instructor → Jadwal Hari Ini
4.2 Klik "Absen" pada sesi
4.3 Upload foto → GPS auto-detect → jarak tampil
4.4 Submit → "Menunggu verifikasi admin"
```

### Alur 5: Admin Verifikasi Absensi
```
5.1 Login admin → menu Absensi → filter Pending
5.2 Klik baris → modal: foto, GPS, jarak, face confidence
5.3 Klik "Verifikasi" → status berubah
```

### Alur 6: Billing Auto-Generate (Tanggal 5)
```
6.1 Tanggal 5, cron job jalan
6.2 Invoice dibuat untuk semua guardian aktif
6.3 Reminder WA + email terkirim
6.4 Guardian login → Pembayaran Saya → invoice baru tampil
```

### Alur 7: Guardian Bayar
```
7.1 Guardian buka Pembayaran Saya → klik invoice
7.2 Upload foto bukti transfer
7.3 Input jumlah + pilih metode
7.4 Submit → "Menunggu verifikasi admin"
```

### Alur 8: Admin Verifikasi Pembayaran
```
8.1 Login admin → menu Tagihan → filter Pending
8.2 Klik baris → modal: foto bukti, detail invoice
8.3 Input kode unik 3 digit → validasi real-time
8.4 Klik "Verifikasi" (hanya aktif jika kode cocok)
8.5 Invoice → lunas
```

### Alur 9: Denda & Freeze (Tanggal 11)
```
9.1 Tanggal 11, cron job jalan
9.2 Invoice overdue → denda Rp 100.000 ditambah
9.3 Guardian frozen → tidak bisa login
9.4 Guardian coba login → "Akun Anda dibekukan"
```

### Alur 10: Forgot Password
```
10.1 Klik "Lupa Password?" di login
10.2 Input email → submit
10.3 "Link reset telah dikirim" message
10.4 Buka email → klik link → /reset-password?token=xxx
10.5 Input password baru → submit
10.6 "Password berhasil diubah!" page
10.7 Login dengan password baru → berhasil
```

### Alur 11: Instructor Feedback → Guardian [BARU]
```
11.1 Login instructor → menu Feedback
11.2 Pilih student → isi rating + komentar
11.3 Submit
11.4 Login guardian → Anak Saya → feedback instructor terlihat
```

### Alur 12: Guardian Rating → Instructor [BARU]
```
12.1 Login guardian → menu Penilaian Instructor
12.2 Pilih instructor → isi rating bintang + komentar
12.3 Submit
12.4 Rating tercatat di sistem
```

### Alur 13: Data Deletion (UU PDP)
```
13.1 Guardian buka Profile → "Hapus Data Saya"
13.2 Konfirmasi → status pending_deletion
13.3 Admin review → proses
13.4 Data PII dianonimkan
```

### Alur 14: Instructor Feedback → Guardian [BARU]
```
14.1 Login instructor → menu Feedback
14.2 Pilih student → isi rating + komentar
14.3 Submit
14.4 Login guardian → Anak Saya → feedback instructor terlihat
```

### Alur 15: Guardian Rating → Instructor [BARU]
```
15.1 Login guardian → menu Penilaian Instructor
15.2 Pilih instructor → isi rating bintang + komentar
15.3 Submit
15.4 Rating tercatat di sistem
```

### Alur 16: Help Page
```
16.1 Buka /help → FAQ tampil per kategori
16.2 Klik pertanyaan → jawaban expand
16.3 Scroll ke bawah → kontak WhatsApp terlihat
16.4 Login → Dashboard → /dashboard/help → panduan per role tampil
```

---

## JUMLAH PAGE TOTAL: 43 halaman

| Group | Jumlah | Role |
|-------|--------|------|
| Public (no auth) | 8 | Visitor |
| Guardian | 8 | Guardian (MOBILE-FIRST) |
| Instructor | 4 | Teacher |
| Admin Operasional | 13 | Admin |
| Owner | 6 | Superadmin |
| Khusus | 4 | Semua |
| **TOTAL** | **43** | |

---

## CATATAN UNTUK STITCH

1. **MOBILE-FIRST untuk Guardian** — 60% user akses via HP. Bottom nav, bukan sidebar. Kartu besar, tombol besar, font besar.
2. **Desktop untuk Admin/Owner** — Sidebar kiri, tabel data dense, banyak kolom.
3. **Instructor di tengah** — Bisa mobile (saat absen di studio) dan desktop (saat di rumah).
4. **ColorBends animated gradient** di background login, register, forgot-password, reset-password.
5. **Dark theme konsisten** — SEMUA page hitam, card #111, accent hijau.
6. **Bahasa Indonesia** — SEMUA teks dalam Bahasa Indonesia.
7. **Kode unik 3 digit** — WAJIB terlihat jelas di invoice guardian (bold, hijau, besar).
8. **Foto bukti** — Placeholder "📷 Foto bukti kehadiran" atau "🧾 Bukti transfer" di modal.
9. **GPS indicator** — Hijau jika ≤150m, merah jika >150m.
10. **Status badges** — Hijau (lunas/verified), kuning (pending), merah (overdue/rejected).
11. **Foto profil** — Setiap role punya foto profil di sidebar/header. Default: avatar lingkaran dengan inisial nama (warna random). Upload: klik foto → modal crop → simpan.
12. **Page Bantuan** — Accordion FAQ, search bar, kontak WhatsApp admin. Ada di sidebar semua role.
