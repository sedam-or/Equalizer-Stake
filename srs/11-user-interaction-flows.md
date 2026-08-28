---
module: user-guides
version: 1.0
---

# Panduan Alur Interaksi Per Role — Equalizer-Stake

Dokumen ini menjelaskan alur interaksi lengkap untuk setiap role dalam sistem Equalizer-Stake, termasuk flow yang ditambahkan dari analisis gaps SRS.

---

## 1. OWNER / SUPERADMIN

Session: 30 menit | Read-only default + Superadmin privileges

### Superadmin Privileges (di luar scope SRS asli, ditambahkan)

Owner bukan hanya read-only — Owner adalah superadmin yang memiliki hak penuh:

| Aksi | Akses | Catatan |
|------|-------|---------|
| Lihat semua audit log | ✓ | Termasuk log keamanan (Admin hanya lihat log operasional) |
| Buat/hapus akun Admin | ✓ | Superadmin-only: bisa rekrut atau terminate Admin |
| Override keputusan Admin | ✓ | Buka freeze, batalkan tagihan, revert jadwal |
| Ubah tarif course | ✓ | Bersama Admin sebagai data master |
| Pengaturan sistem | ✓ | Maintenance window, alert threshold, WhatsApp gateway config |
| Ekspor semua data | ✓ | Termasuk data audit 7 tahun |

### Alur Login

```
Buka URL → Login (username/password)
 → JWT issued (role=owner, expiry=30menit)
 → Dashboard Monitoring (≤3 detik load)
```

### Alur Dashboard Monitoring

```
Dashboard → Lihat KPI real-time:
 - Total pendapatan berjalan
 - Tingkat kehadiran student
 - Jumlah akun frozen
 - Utilisasi studio
 - Ringkasan audit log
 → Filter/cari data
 → Buka tab audit log (semua kategori: operasional, keamanan, kepatuhan, sistem)
 → Klik "Ekspor Laporan" → unduh format cetak
```

### Alur Manajemen Admin (Superadmin)

```
Buka "Manajemen Admin"
 → Lihat daftar Admin aktif
 → "Tambah Admin":
 Input: nama, email, username
 → Sistem generate password sementara
 → Kirim email undangan ke Admin baru
 → Admin baru login → wajib ubah password
 → "Nonaktifkan Admin":
 Pilih Admin → Konfirmasi → Status = inactive
 → Audit log "Superadmin: nonaktifkan Admin [nama]"
 → "Hapus Admin" (jika dipecat):
 Pilih Admin → Konfirmasi (ketik username untuk verifikasi)
 → PII Admin dianonimkan
 → Semua jadwal yang dikelola Admin dipindah ke Admin lain
 → Audit log "Superadmin: hapus Admin [nama]"
 → Email notifikasi ke Admin yang dihapus
```

### Alur Override Keputusan Admin

```
Buka "Override" → Pilih jenis:
 - Buka Freeze: Pilih akun frozen → Konfirmasi → Akun aktif
 - Batalkan Tagihan: Pilih invoice → Alasan → Status = cancelled
 - Revert Jadwal: Pilih session → Status kembali ke sebelumnya
 → Semua override tercatat di audit log (kategori: keamanan)
```

### Alur Pengaturan Sistem

```
Buka "Pengaturan Sistem":
 - Maintenance window: set jadwal pemeliharaan
 - Alert threshold: set batas peringatan
 - WhatsApp gateway: set API key, test koneksi
 - SMTP: set kredensial email
 - Tarif course: set/ubah tarif per instrumen & format
 → Semua perubahan tercatat di audit log
```

---

## 2. ADMIN OPERASIONAL

Session: 30 menit | Full CRUD operasional

### Alur Login

```
Buka URL → Login (username/password)
 → JWT issued (role=admin, expiry=15menit)
 → Dashboard Admin
 → [Auto-save draft setiap 30 detik via IndexedDB]
```

### Alur Harian

```
LOGIN
 │
 ├── 1. CEK NOTIFIKASI MASUK
 │ ├── Absensi pending verification (≤2 jam review)
 │ ├── Notifikasi gagal kirim (WhatsApp/email)
 │ └── Akun freeze yang perlu diproses unfreeze
 │
 ├── 2. INPUT/UBAH JADWAL
 │ ├── Pilih student, instructor, ruangan, tanggal, waktu
 │ ├── Validasi konflik real-time ≤1 detik
 │ ├── Jika konflik → tampilkan 3 alternatif
 │ ├── Jika available → Simpan → Status "Locked"
 │ ├── Audit log: before/after state
 │ └── Notifikasi ke Instructor & Guardian
 │
 ├── 3. REVIEW ABSENSI PENDING
 │ ├── Buka tab "Absensi Pending"
 │ ├── Lihat foto, GPS, timestamp
 │ ├── Jika valid → Setujui → Status "Present"
 │ ├── Jika tidak valid → Tolak dengan alasan
 │ └── Audit log
 │
 ├── 4. VERIFIKASI PEMBAYARAN
 │ ├── Terima bukti transfer dari Guardian
 │ ├── Cocokkan kode unik 3 digit dengan invoice
 │ ├── Jika cocok → "Konfirmasi Pembayaran"
 │ ├── Status → "Lunas", akun aktif (jika frozen)
 │ ├── Audit log
 │ └── Notifikasi ke Guardian
 │
 └── 5. PROSES UNFREEZE
 ├── Cek akun frozen
 ├── Verifikasi pelunasan (pokok + denda)
 ├── Setujui → Status "Active"
 ├── Audit log
 └── Notifikasi ke Guardian
```

### Alur Buat Akun Baru

```
"Tambah Akun"
 → Pilih role: Instructor atau Guardian
 → Input data:
 Instructor: nama, email, no HP, spesialisasi instrumen, tipe (tetap/freelance)
 Guardian: nama, email, no HP, alamat
 → Sistem generate password sementara (8 char, alphanumeric)
 → Kirim email: link akses + username + password sementara
 → Password sementara berlaku 7 hari
 → Admin dapat "Kirim Ulang" jika expired
```

### Alur Tambah Student ke Guardian

```
"Pilih Guardian" → "Tambah Student"
 → Input: nama, tanggal lahir, instrumen, format kelas, level
 → Submit → Student ditambahkan ke akun Guardian
 → Notifikasi ke Guardian: "Anak [nama] telah terdaftar"
```

### Alur Konfigurasi Libur Nasional

```
"Buka Kalender" → "Kelola Libur Nasional"
 → Input tanggal libur nasional (awal tahun ajaran)
 → Sistem otomatis proses:
 - Jika libur jatuh di hari course → reschedule ke minggu berikutnya
 - Jika libur di minggu ke-5 → batal kelas teori
 → Notifikasi ke Instructor & Guardian yang terdampak
```

### Alur Ubah Tarif Course

```
"Pengaturan" → "Tarif Course"
 → Lihat daftar tarif per instrumen & format
 → "Ubah Tarif" → Input tarif baru → Konfirmasi
 → Berlaku untuk tagihan berikutnya
 → Audit log
```

### Alur Konsultasi Studio Recording

```
Guardian ajukan booking recording
 → Admin terima notifikasi "Booking Recording Menunggu Persetujuan"
 → Admin buka → Lihat detail: tanggal, durasi, kebutuhan
 → Admin konsultasi harga dengan Guardian (WhatsApp/email)
 → Admin input harga → "Setujui" atau "Tolak"
 → Jika setujui → Slot terkunci → Notifikasi ke Guardian
 → Audit log
```

### Alur Toggle Channel Notifikasi

```
"Dashboard" → "Pengaturan Notifikasi"
 → Toggle WhatsApp: ON/OFF
 → Toggle Email: ON/OFF
 → Jika WhatsApp OFF → semua notifikasi via email
 → Jika Email OFF → semua notifikasi via WhatsApp (non-kritis saja)
 → Kritis (pembayaran, freeze) WAJIB ada 1 channel aktif
```

### Alur Reschedule Instructor Berhalangan

```
Instructor laporkan berhalangan (via sistem atau manual)
 → Admin buka jadwal Instructor yang terdampak
 → Pilih sesi → "Reschedule"
 → Sistem cari slot alternatif (instructor pengganti atau jadwal ulang)
 → Konfirmasi → Status "Rescheduled"
 → Kredit student TIDAK dipotong
 → Notifikasi ke Guardian & Instructor pengganti
 → Catat sebagai "hutang jam" Instructor yang berhalangan
```

### Alur Payment Dispute (Klaim Bayar)

```
Guardian klaim sudah bayar tapi tidak terverifikasi:

 Guardian → Dashboard → "Saya Sudah Bayar"
 → Form:
 Pilih tagihan: [dropdown invoice aktif]
 Nominal transfer: [input]
 Tanggal transfer: [input]
 Kode unik: [input]
 Bukti transfer: [upload]
 → Submit

 Sistem → Notifikasi ke Admin: "Klaim Pembayaran dari [Guardian]"

 Admin review:
 → Buka klaim → Lihat bukti transfer
 → Cocokkan dengan bank statement / kode unik
 → Jika cocok → "Verifikasi Klaim" → Status "Paid" → Unfreeze
 → Jika tidak cocok → "Tolak Klaim" → Notifikasi ke Guardian
 dengan alasan → "Silakan hubungi bank atau Admin untuk klarifikasi"

 Semua langkah tercatat di audit log (kategori: keamanan)
```

### Session Timeout Handling

```
30 menit tidak aktif
 → Auto-save draft ke IndexedDB
 → Redirect ke login
 → Login ulang → Deteksi draft
 → "Draft sebelumnya ditemukan. Pulihkan?"
 → Klik "Pulihkan" → Form terisi kembali
```

---

## 3. GURU MUSIK

Session: 30 menit | Mobile-first, simpel

### Alur Login

```
Buka URL di HP → Login (username/password)
 → JWT issued (role=teacher, expiry=30menit)
 → Kalender Pribadi (jadwal hari ini + minggu ini)
```

### Alur Absensi

```
Buka "Absensi Hari Ini"
 │
 ├── 1. PILIH SESI AKTIF
 │ └── Lihat: nama student, jam, ruangan
 │
 ├── 2. AMBIL FOTO SISWA
 │ ├── Kamera HP aktif
 │ ├── Ambil foto
 │ └── Client-side: face-api.js validasi wajah ≥30% frame
 │
 ├── 3. VALIDASI WAJAH (client-side)
 │ ├── Jika GAGAL:
 │ │ └── "Wajah student belum terlihat jelas. Pastikan wajah
 │ │ mengisi minimal sepertiga area foto."
 │ │ → Instructor ambil ulang foto
 │ │ → TIDAK ada request ke server
 │ │
 │ └── Jika LOLOS:
 │ ├── Ambil koordinat GPS perangkat
 │ └── Kirim (foto + GPS + timestamp) ke Backend
 │
 ├── 4. VALIDASI SERVER (backend)
 │ ├── GPS radius 150m dari studio
 │ ├── Timestamp server (bukan device time)
 │ │
 │ ├── Jika SEMUA valid:
 │ │ ├── Status → "Hadir"
 │ │ ├── Kredit student dipotong
 │ │ └── Notifikasi hadir ke Guardian
 │ │
 │ └── Jika ADA yang gagal:
 │ ├── Status → "Menunggu Verifikasi"
 │ ├── Notifikasi ke Admin untuk review ≤2 jam
 │ └── Instructor tidak bisa ubah setelah submit
 │
 └── 5. SELESAI
 └── Kembali ke kalender → Lihat jadwal berikutnya
```

### Alur Offline

```
Internet mati saat ambil absensi
 → Foto tersimpan di perangkat lokal (IndexedDB)
 → GPS timestamp dicatat
 → Saat internet kembali:
 Auto-upload ≤5 menit
 → Backend validasi normal
 → Timestamp server saat upload diterima
```

### Yang TIDAK Bisa Dilakukan Instructor

- ✗ Ubah data absensi setelah submit
- ✗ Lihat jadwal instructor lain
- ✗ Lihat data student selain yang mengajar
- ✗ Input/ubah jadwal course
- ✗ Verifikasi pembayaran
- ✗ Akses audit log

---

## 4. WALI MURID

Session: 60 menit (paling lama) | 1 akun → N student

### Alur Login

```
Buka URL di HP → Login (username/password)
 → JWT issued (role=guardian, expiry=60menit)
 → Dashboard Multi-Student
```

### Alur Dashboard Multi-Student

```
Dashboard
 ├── Lihat daftar semua anak:
 │ Anak A (Piano) [Aktif] — badge hijau
 │ Anak B (Gitar) [Aktif] — badge hijau
 │ Anak C (Violin) [Frozen] — badge merah
 │
 ├── Klik nama anak → Dashboard refresh
 │ ├── Jadwal minggu ini
 │ ├── Status kehadiran
 │ ├── Tagihan aktif
 │ └── Riwayat kehadiran
 │
 └── Switch profil: pilih nama anak lain → data berubah
```

### Alur Booking Studio Rehearsal

```
"Booking Studio" → Pilih "Rehearsal"
 → Buka kalender studio
 → Pilih tanggal & jam (09:00-21:00)
 → Cek ketersediaan real-time
 → Jika available → "Konfirmasi Reservasi"
 → Slot terkunci
 → Notifikasi konfirmasi dikirim
 → Jika slot baru saja dipesan:
 "Slot ini baru saja dipesan. Silakan pilih waktu lain."
 → Tampilkan slot tersedia terdekat
```

### Alur Pembayaran

```
TERIMA INVOICE (tanggal 5):
 WhatsApp: "Tagihan bulan [X] Rp [amount] + kode unik [3 digit]"
 Email: "Tagihan Equalizer-Stake" dengan detail

 → Transfer bank:
 Nominal: Rp [amount] + kode unik [3 digit]
 Contoh: Rp 350.127 (3 digit terakhir = 127)

 → Kirim bukti transfer ke Admin (WhatsApp/email)

 → Admin verifikasi → Status "Lunas"
 Notifikasi: "Pembayaran Anda telah dikonfirmasi."

 → Jika lewat tanggal 10:
 Tanggal 11 00:00: Denda Rp 100.000 + Akun dibekukan
 Notifikasi: "Akun Anda dibekukan karena tagihan belum lunas."
 → Hubungi Admin untuk proses unfreeze
```

### Alur Cancel Sesi

```
Buka jadwal → Pilih sesi → "Batalkan Sesi"
 → Jika ≥24 jam sebelum sesi:
 "Kredit akan dikembalikan 50%. Lanjutkan?"
 → Konfirmasi → Kredit 50% dikembalikan
 → Jika <24 jam sebelum sesi:
 "Kredit akan HANGUS. Lanjutkan?"
 → Konfirmasi → Kredit 0%
 → Alasan pembatalan wajib diisi
 → Audit log
```

### Alur Penghapusan Data (UU PDP Pasal 19)

```
"Pengaturan" → "Hapus Data Saya"
 → Formulir: pilih data yang ingin dihapus
 → Sistem cek: apakah ada tagihan aktif?
 Jika YA → "Permintaan ditolak. Lunasi tagihan terlebih dahulu."
 Jika TIDAK → Submit permintaan

 → Admin review permintaan
 → Admin verifikasi identitas
 → Admin setujui

 → Sistem eksekusi:
 - PII dianonimkan: nama → "REDACTED-[hash]"
 - Foto absensi dihapus permanen
 - Metadata audit tetap 7 tahun (tanpa PII)
 → Notifikasi ke Guardian: "Data Anda telah dihapus."
 (≤14 hari kerja)
```

### Yang TIDAK Bisa Dilakukan Guardian

- ✗ Input/ubah jadwal (hanya Admin)
- ✗ Verifikasi pembayaran (hanya Admin)
- ✗ Ubah data student lain
- ✗ Akses audit log
- ✗ Booking studio recording (perlu approval Admin)

---

## 5. SISTEM (OTOMATIS — CRON JOBS)

### Alur Tagihan Otomatis (Tanggal 5, 00:00)

```
Cron trigger → Ambil daftar Guardian aktif
 → Hitung total biaya per Guardian (semua student)
 → Generate kode unik 3 digit (000-999)
 → Cek duplikasi (maks 3x)
 → Simpan invoice
 → Kirim notifikasi WhatsApp + email
 → Jika WhatsApp gagal → retry 3x → fallback email
```

### Alur Denda & Freeze (Tanggal 11, 00:00)

```
Cron trigger → Cek status tagihan semua Guardian
 → Jika belum lunas:
 Tambahkan denda Rp 100.000
 Ubah status → Frozen
 Audit log
 Kirim notifikasi WhatsApp + email
 → Jika sudah lunas: skip
```

### Alur Notifikasi Reminder (1 Jam Sebelum Sesi)

```
Cron trigger → Ambil sesi yang akan dimulai dalam 1 jam
 → Generate pesan reminder
 → Kirim via WhatsApp
 → Jika delivered → log "Terkirim"
 → Jika gagal → retry 3x (10/20/40 detik) → fallback email
```

### Alur Circuit Breaker

```
WhatsApp gateway gagal >5 kali berturut-turut
 → Sistem nonaktifkan WhatsApp sementara
 → Semua notifikasi non-kritis: blokir
 → Semua notifikasi kritis: via email
 → Admin dapat "Reset" circuit breaker setelah gateway pulih
```

---

## 6. ISSUE YANG SUDAH DIISESI

Semua 6 issues dari analisis sebelumnya sudah didefinisikan flow-nya:

### Issue 1: Admin Session → 30 Menit

Admin session timeout diubah dari 15 menit menjadi **30 menit**. Alasan: Admin sering proses panjang (verifikasi pembayaran, review absensi pending, input jadwal batch). 15 menit terlalu pendek.

```
Session timeout Admin: 30 menit (sama dengan Owner dan Instructor)
Auto-save draft tetap aktif setiap 30 detik via IndexedDB
```

### Issue 2: Flow "Lupa Password"

```
Halaman Login → Klik "Lupa Password?"
 → Input email terdaftar
 → Sistem cek apakah email ada di database
 → Jika ADA:
 Kirim email reset password (link valid 1 jam)
 → Klik link → Form ubah password baru (minimal 8 char)
 → Password tersimpan → Audit log "Password reset via email"
 → Login dengan password baru
 → Jika TIDAK ADA:
 "Email tidak terdaftar. Hubungi Admin."
 → Rate limit: maks 3 reset per jam per email
```

### Issue 3: Force Logout Saat Admin Dihapus

```
Owner hapus Admin:
 → Audit log: "Superadmin: hapus Admin [nama]"
 → Sistem otomatis:
 1. Invalidasi SEMUA JWT token milik Admin tersebut
 2. Force logout dari semua perangkat (browser/HP)
 3. Tampilkan pesan: "Akun Anda telah dinonaktifkan. Hubungi Owner."
 4. Jika Admin sedang login → redirect ke login + pesan di atas
 5. Semua draft milik Admin tetap tersimpan (bisa diakses Admin pengganti)
```

### Issue 4: Bulk Operation

```
Admin → "Bulk Operations":

BULK JADWAL:
 → Upload CSV/Excel: tanggal, student, instructor, ruangan, waktu
 → Sistem validasi setiap baris (konflik, jam operasional, libur)
 → Tampilkan hasil validasi: valid/batal dengan alasan
 → Klik "Import Semua" → batch insert + audit log

BULK NOTIFIKASI:
 → Pilih penerima: semua Guardian aktif, atau filter tertentu
 → Input pesan → Preview
 → Kirim batch → Status per notifikasi tercatat

BULK VERIFIKASI:
 → Pilih beberapa invoice yang sudah dikonfirmasi bayar
 → "Konfirmasi Semua" → batch update status + audit log
```

### Issue 5: Escalation ke Owner

```
Instructor/Guardian → "Hubungi Owner" (tombol di dashboard):

 → Form:
 Subjek: [dropdown: Jadwal, Pembayaran, Teknis, Keluhan]
 Pesan: [text area]
 Lampiran: [opsional]
 → Submit → Notifikasi ke Owner (email + WhatsApp)
 → Owner terima → Buka dashboard "Eskalasi Masuk"
 → Owner balas (via sistem atau langsung ke Instructor/Wali)
 → Status eskalasi: Open → In Progress → Resolved

Batasan:
 - Maks 3 eskalasi per minggu per pengguna
 - Eskalasi "Teknis" otomatis copy ke Admin juga
 - Eskalasi "Keluhan" hanya terlihat Owner (privasi)
```

### Issue 6: Deputasi Admin

```
Owner → "Deputasi Admin":

 → Pilih Admin A (yang akan berhalangan)
 → Pilih Admin B (yang akan menggantikan)
 → Set masa berlaku: [tanggal mulai] sampai [tanggal selesai]
 → Konfirmasi

 Efek:
 - Admin B mendapat akses SEMUA fitur Admin A (termasuk draft milik Admin A)
 - Admin B bisa login dengan akun sendiri (bukan akun Admin A)
 - Semua aksi Admin B tercatat di audit log dengan note "deputasi dari Admin A"
 - Setelah masa berlaku habis → akses deputasi otomatis dicabut

 Batasan:
 - Maks 1 deputasi aktif per Admin
 - Owner bisa cabut deputasi kapan saja
 - Admin yang dideputasi tetap bisa login (akun tidak ditutup)
```

---

## 7. REKOMENDASI TAMBAHAN

| # | Rekomendasi | Prioritas | Keterangan |
|---|------------|-----------|------------|
| 1 | Tambah fitur "Activity Log" di dashboard Admin | Low | Admin bisa lihat aktivitas terakhir sendiri |
| 2 | Tambah notifikasi "Session akan timeout" (2 menit sebelum) | Medium | Mencegah data hilang saat timeout |
| 3 | Tambah fitur "Undo" untuk operasi terakhir Admin | Low | Rollback cepat tanpa ke audit log |
| 4 | Tambah rate limit per role | Medium | Instructor: 10 absensi/jam, Admin: 100 aksi/jam |
| 5 | Tambah fitur "Quick Actions" di dashboard | Low | Shortcut untuk aksi sering digunakan |
| 6 | Tambah "Session History" untuk Owner | Low | Lihat kapan Admin terakhir login, dari mana |
