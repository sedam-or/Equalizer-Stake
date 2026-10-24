# User Acceptance Test (UAT) — Equalizer-Stake
**Tanggal:** 2026-07-01
**Tujuan:** Patokan test setiap alur. Jika semua PASS = sistem siap. Jika ada FAIL = ada yang belum selesai.
**Prinsip:** Trust by Design — setiap aksi bisa di-trace ke siapa, kapan, bagaimana, dan mengapa.

---

## Cara Pakai
- Setiap alur punya: **SIAPA** (role), **APA** (aksi), **BAGAIMANA** (step), **KAPAN** (trigger), **MENGAPA** (tujuan bisnis)
- Test secara manual: login sebagai role → ikuti step → verifikasi hasil
- Centang (✅) jika PASS, coret (❌) jika FAIL, catat alasan

---

## ALUR 1: Visitor Mendaftar Course
**Tujuan:** Calon student baru bisa mendaftar course melalui sistem

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 1.1 | Buka landing page | Visitor | Akses URL utama | Landing page tampil, ada tombol "Daftar Course" |
| 1.2 | Isi form pendaftaran | Visitor | Nama, WhatsApp, email, alamat, pilih instrumen, pilih level, pilih tipe kelas | Form valid, semua field wajib terisi |
| 1.3 | Submit pendaftaran | Visitor | Klik "Daftar" | Status: "menunggu konfirmasi", notifikasi masuk ke admin |
| 1.4 | Admin review pendaftaran | Admin | Buka dashboard → menu Pendaftaran → lihat daftar pending | Data pendaftaran lengkap terlihat |
| 1.5 | Admin approve + assign instructor | Admin | Pilih instructor, pilih jadwal, klik "Approve" | Status berubah → akun guardian dibuat otomatis |
| 1.6 | Guardian terima notifikasi | Guardian | WhatsApp/email masuk | Pesan: "Akun Anda aktif, silakan login" |

---

## ALUR 2: Guardian Login & Lihat Anak
**Tujuan:** Guardian bisa melihat data anak dan jadwal course

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 2.1 | Login | Guardian | Masuk dengan username + password dari notifikasi | Berhasil masuk ke dashboard guardian |
| 2.2 | Lihat daftar anak | Guardian | Buka menu "Anak Saya" | Daftar anak tampil (nama, instrumen, level, instructor, status) |
| 2.3 | Lihat jadwal anak | Guardian | Buka menu "Jadwal Saya" | Jadwal course anak tampil (hari, jam, ruangan, instructor) |
| 2.4 | Lihat tagihan | Guardian | Buka menu "Pembayaran Saya" | Daftar invoice tampil (periode, jumlah, status, kode unik) |

---

## ALUR 3: Admin Membuat Jadwal Course
**Tujuan:** Admin bisa menjadwalkan sesi course untuk instructor + student

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 3.1 | Buka menu Jadwal | Admin | Dashboard → Jadwal | Halaman jadwal tampil dengan kalender/tabel |
| 3.2 | Buat sesi baru | Admin | Pilih instructor, student, ruangan, hari, jam mulai, jam selesai | Form valid, tidak ada konflik jadwal |
| 3.3 | Konflik detection | Admin | Coba buat jadwal bentrok | Sistem menolak: "Instructor/student/ruangan sudah dipakai jam tersebut" |
| 3.4 | Simpan jadwal | Admin | Klik "Simpan" | Jadwal tersimpan, muncul di daftar |
| 3.5 | Instructor lihat jadwal | Instructor | Login → Jadwal Saya | Jadwal baru yang dibuat admin muncul |

---

## ALUR 4: Instructor Submit Absensi
**Tujuan:** Instructor membuktikan kehadiran dengan foto + GPS

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 4.1 | Buka jadwal hari ini | Instructor | Login → Jadwal Saya → filter hari ini | Daftar sesi hari ini tampil |
| 4.2 | Klik "Absen" | Instructor | Pilih sesi → klik tombol Absen | Form absensi terbuka |
| 4.3 | Ambil foto | Instructor | Kamera browser atau upload foto | Foto ter-preview, ukuran ≤2MB |
| 4.4 | GPS auto-detect | Sistem | Browser minta izin lokasi | Koordinat terisi otomatis |
| 4.5 | Validasi GPS | Sistem | Hitung jarak ke studio (≤150m) | Jarak tampil, status ✓ atau ✗ |
| 4.6 | Submit absensi | Instructor | Klik "Kirim" | Status: "menunggu verifikasi admin" |
| 4.7 | Admin lihat pending | Admin | Dashboard → Absensi → filter pending | Absensi instructor tampil dengan foto + GPS |

---

## ALUR 5: Admin Verifikasi Absensi
**Tujuan:** Admin memverifikasi kehadiran instructor berdasarkan bukti foto + GPS

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 5.1 | Buka detail absensi | Admin | Klik baris absensi pending | Modal: foto bukti, GPS, jarak, face confidence |
| 5.2 | Review foto | Admin | Lihat foto, pastikan wajah instructor terlihat | Foto jelas, wajah terdeteksi ≥30% frame |
| 5.3 | Review GPS | Admin | Lihat koordinat + jarak | Jarak ≤150m dari studio |
| 5.4 | Verifikasi | Admin | Klik "Verifikasi" | Status → "verified", instructor dianggap hadir |
| 5.5 | Atau Tolak | Admin | Klik "Tolak" + isi alasan | Status → "rejected", notifikasi ke instructor |

---

## ALUR 6: Billing Auto-Generate (Tanggal 5)
**Tujuan:** Sistem otomatis membuat tagihan dan mengingatkan guardian

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 6.1 | Cron job jalan | Sistem | Tanggal 5, 00:05 WIB | Invoice dibuat untuk semua guardian aktif |
| 6.2 | Hitung SPP | Sistem | Ambil harga dari master data (instrumen × tipe × level) | Jumlah benar sesuai harga asli |
| 6.3 | Generate kode unik | Sistem | Random 3 digit, unik per invoice | Kode unik ter-generate, tidak duplikat |
| 6.4 | Kirim reminder | Sistem | WhatsApp (utama) + email (backup) | Guardian terima pesan: "Tagihan bulan X, jatuh tempo 10" |
| 6.5 | Guardian lihat tagihan | Guardian | Login → Pembayaran Saya | Invoice baru tampil dengan kode unik |

---

## ALUR 7: Guardian Bayar + Upload Bukti
**Tujuan:** Guardian membayar tagihan dan mengirim bukti transfer

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 7.1 | Lihat invoice | Guardian | Login → Pembayaran Saya → klik invoice | Detail: jumlah, kode unik, jatuh tempo |
| 7.2 | Transfer ke rekening | Guardian | Di luar sistem (bank/e-wallet/tunai) | Transfer berhasil (di luar sistem) |
| 7.3 | Upload bukti | Guardian | Klik "Bayar" → upload foto/screenshot bukti | Foto ter-preview |
| 7.4 | Input jumlah | Guardian | Masukkan nominal yang ditransfer | Jumlah sesuai invoice |
| 7.5 | Submit | Guardian | Klik "Kirim Bukti" | Status → "menunggu verifikasi admin" |

---

## ALUR 8: Admin Verifikasi Pembayaran (Kode Unik)
**Tujuan:** Admin mencocokkan kode unik dari bukti transfer dengan invoice

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 8.1 | Buka menu Billing | Admin | Dashboard → Tagihan → filter pending | Daftar pembayaran pending tampil |
| 8.2 | Review bukti | Admin | Klik baris → modal: foto bukti, detail invoice | Bukti transfer terlihat jelas |
| 8.3 | Input kode unik | Admin | Ketik 3 digit kode dari bukti transfer | Input validasi real-time: cocok/tidak |
| 8.4 | Verifikasi | Admin | Klik "Verifikasi" (hanya aktif jika kode cocok) | Invoice → lunas, payment → verified |
| 8.5 | Atau Tolak | Admin | Klik "Tolak" | Status → ditolak, guardian diberitahu |
| 8.6 | Duplikat detection | Sistem | Jika kode sudah pernah dipakai | Alert: "Pembayaran sudah diverifikasi sebelumnya" |

---

## ALUR 9: Denda & Freeze (Tanggal 11)
**Tujuan:** Sistem mengenakan denda dan membekukan akun yang belum bayar

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 9.1 | Cron job jalan | Sistem | Tanggal 11, 00:05 WIB | Invoice overdue terdeteksi |
| 9.2 | Tambah denda | Sistem | Tambah Rp 100.000 ke invoice | Invoice amount bertambah Rp 100.000 |
| 9.3 | Freeze akun | Sistem | Set guardian status → frozen | Guardian tidak bisa login |
| 9.4 | Guardian coba login | Guardian | Masuk dengan credentials | Error: "Akun Anda dibekukan. Silakan hubungi Admin." |
| 9.5 | Admin unfreeze | Admin | Dashboard → review → unfreeze setelah bayar | Guardian bisa login lagi |

---

## ALUR 10: Notifikasi Otomatis
**Tujuan:** Sistem mengirim notifikasi di event penting

| # | Event | Channel | Tujuan |
|---|-------|---------|--------|
| 10.1 | Invoice diterbitkan (tgl 5) | WA + email | Reminder pembayaran |
| 10.2 | Jatuh tempo besok (tgl 9) | WA | Reminder terakhir |
| 10.3 | Denda + freeze (tgl 11) | WA + email | Informasi denda |
| 10.4 | Absensi instructor pending >24 jam | WA ke admin | Reminder review |
| 10.5 | Pembayaran berhasil diverifikasi | WA + email | Konfirmasi lunas |
| 10.6 | Jadwal besok | WA | Reminder ke instructor + guardian |

---

## ALUR 11: Data Deletion (UU PDP Pasal 19)
**Tujuan:** Guardian bisa minta hapus data pribadi

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 11.1 | Ajukan penghapusan | Guardian | Profile → "Hapus Data Saya" | Status → pending_deletion |
| 11.2 | Cek tagihan aktif | Sistem | Jika ada invoice belum lunas | Tolak: "Lunasi tagihan terlebih dahulu" |
| 11.3 | Admin review | Admin | Dashboard → menu Privacy | Data deletion request tampil |
| 11.4 | Proses penghapusan | Admin | Klik "Proses" | PII dianonimkan (nama, email, phone → REDACTED) |
| 11.5 | Audit log | Sistem | Catat semua langkah di audit_log | Immutable, 7 tahun retention |

---

## RINGKASAN UAT

| # | Alur | Status | Catatan |
|---|------|--------|---------|
| 1 | Visitor Daftar | ⬜ | Belum ada flow |
| 2 | Guardian Login & Lihat Anak | ⬜ | Dummy data |
| 3 | Admin Buat Jadwal | ⬜ | Dummy data |
| 4 | Instructor Submit Absensi | ⬜ | Belum ada page |
| 5 | Admin Verifikasi Absensi | ⬜ | Dummy data |
| 6 | Billing Auto-Generate | ⬜ | Backend ada, belum teruji |
| 7 | Guardian Bayar + Upload | ⬜ | Belum ada flow |
| 8 | Admin Verifikasi Pembayaran | ⬜ | Dummy data |
| 9 | Denda & Freeze | ⬜ | Backend ada, belum teruji |
| 10 | Notifikasi Otomatis | ⬜ | Backend ada, belum teruji |
| 11 | Data Deletion | ⬜ | Backend ada, belum teruji |
| 12 | Instructor Feedback → Wali | ⬜ | Belum ada |
| 13 | Wali Rating → Instructor | ⬜ | Belum ada |
| 14 | Foto Profil | ⬜ | Belum ada |
| 15 | Page Instrumen Detail | ⬜ | Belum ada |
| 16 | Page Bantuan | ⬜ | Belum ada |
| 17 | Lupa Password Flow | ⬜ | Belum ada |
| 18 | Mobile-First Wali | ⬜ | Belum ada |

**Semua 18 alur = ⬜ BELUM PASS**

Target: Semua ✅ = sistem siap produksi.

---

## ALUR 12: Instructor Feedback ke Guardian
**Tujuan:** Instructor memberikan feedback progress student ke guardian

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 12.1 | Buka menu Feedback | Instructor | Login → sidebar → Feedback | Halaman feedback tampil |
| 12.2 | Pilih student | Instructor | Dropdown daftar student yang diajar | Student terpilih, data tampil |
| 12.3 | Isi feedback | Instructor | Rating 1-5 per kategori (Kemajuan, Perilaku, Kehadiran, Rekomendasi) + komentar | Form valid |
| 12.4 | Submit feedback | Instructor | Klik "Kirim Feedback" | Success message, feedback masuk riwayat |
| 12.5 | Wali lihat feedback | Wali | Login → Anak Saya → klik anak → tab "Feedback Instructor" | Feedback instructor tampil dengan rating + komentar |
| 12.6 | Audit log | Sistem | Setiap feedback tercatat di audit_log | Log tampil di audit log admin |

---

## ALUR 13: Guardian Rating ke Instructor
**Tujuan:** Wali murid memberikan penilaian ke instructor course anak

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 13.1 | Buka menu Penilaian Instructor | Wali | Login → sidebar → Penilaian Instructor | Halaman rating tampil |
| 13.2 | Pilih instructor | Wali | Dropdown hanya tampil instructor yang mengajar anaknya (auto-detect dari jadwal) | Instructor terpilih |
| 13.3 | Beri rating | Wali | Rating bintang 1-5 + komentar (opsional) | Form valid |
| 13.4 | Submit | Wali | Klik "Kirim Penilaian" | Success message, rating masuk riwayat |
| 13.5 | Admin lihat rating | Admin | Dashboard → menu Rating Instructor | Rating per instructor tampil (rata-rata + detail) |
| 13.6 | Validasi | Sistem | Hanya bisa menilai instructor yang mengajar anaknya | Error: "Anda tidak bisa menilai instructor ini" jika validasi gagal |

---

## ALUR 14: Upload Foto Profil
**Tujuan:** Setiap user bisa upload foto profil

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 14.1 | Buka Profil | User (semua role) | Login → sidebar → Profil | Halaman profil tampil |
| 14.2 | Klik foto | User | Klik avatar/foto profil | Modal crop terbuka |
| 14.3 | Upload foto | User | Pilih file (JPEG/PNG, max 2MB) → crop → simpan | Foto ter-upload, tampil di profil |
| 14.4 | Foto di sidebar | Sistem | Foto tampil di sidebar/header sebagai avatar | Foto konsisten di semua halaman |
| 14.5 | Default avatar | Sistem | Jika belum upload → inisial nama dengan warna random | Avatar inisial tampil |
| 14.6 | Hapus foto | User | Klik "Hapus Foto" → kembali ke inisial | Foto terhapus |

---

## ALUR 15: Page Detail Instrumen
**Tujuan:** Visitor bisa melihat detail instrumen, level, tipe kelas, dan harga

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 15.1 | Buka /instruments | Visitor | Klik "Lihat Instrumen" di landing page atau navbar | Page tampil dengan 9 instrumen |
| 15.2 | Scroll per instrumen | Visitor | Scroll ke bawah, setiap instrumen punya section sendiri | Deskripsi, level, tipe kelas, harga tampil |
| 15.3 | Lihat harga | Visitor | Tabel harga per kombinasi (tipe kelas × level) | Harga sesuai data asli EQS |
| 15.4 | Klik CTA | Visitor | Klik "Daftar Sekarang" pada instrumen yang dipilih | Redirect ke /register-course?instrument=piano_klasik |
| 15.5 | FAQ | Visitor | Scroll ke bawah → FAQ accordion | FAQ tampil, bisa buka/tutup |

---

## ALUR 16: Page Bantuan
**Tujuan:** User bisa mencari bantuan dan FAQ

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 16.1 | Buka /help | Visitor/User | Klik "Bantuan" di footer atau sidebar | Page tampil dengan search + FAQ |
| 16.2 | Search | Visitor/User | Ketik kata kunci di search bar | FAQ terfilter sesuai kata kunci |
| 16.3 | Buka FAQ | Visitor/User | Klik kategori → accordion terbuka | Jawaban tampil |
| 16.4 | Kontak WhatsApp | Visitor/User | Klik tombol "Hubungi WhatsApp" | Redirect ke wa.me/62xxx |
| 16.5 | Dashboard help | User (login) | Buka /dashboard/help → panduan per role | Panduan sesuai role tampil |

---

## ALUR 17: Lupa Password Flow
**Tujuan:** User bisa reset password via email

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 17.1 | Klik "Lupa Password?" | User | Login page → klik link | Redirect ke /forgot-password |
| 17.2 | Input email | User | Masukkan email terdaftar → submit | "Jika email terdaftar, link reset telah dikirim" |
| 17.3 | Cek email | User | Buka email → klik link reset | Redirect ke /reset-password?token=xxx |
| 17.4 | Set password baru | User | Input password baru + konfirmasi → submit | "Password berhasil diubah!" page |
| 17.5 | Login | User | Klik tombol "Login" → masuk dengan password baru | Login berhasil |
| 17.6 | Token expired | User | Tunggu 1 jam → klik link | "Link tidak valid atau sudah kedaluwarsa" page |
| 17.7 | Rate limit | Sistem | Minta reset 4x dalam 1 jam | Error: "Terlalu banyak permintaan" |

---

## ALUR 18: Mobile-First Guardian
**Tujuan:** Dashboard guardian optimal di mobile (320px+)

| # | Step | Siapa | Bagaimana | Verifikasi |
|---|------|-------|-----------|------------|
| 18.1 | Buka di mobile | Wali | Akses /dashboard di HP (320px viewport) | Layout mobile tampil |
| 18.2 | Bottom navigation | Wali | Navigation di bawah layar (bukan sidebar) | Nav tampil, ikon + label |
| 18.3 | Kartu besar | Wali | Kartu Anak, Tagihan, Jadwal cukup besar untuk tap | Tidak ada elemen terlalu kecil |
| 18.4 | Scroll vertikal | Wali | Semua konten scroll vertikal (tidak horizontal) | Tidak ada horizontal scroll |
| 18.5 | Form mobile | Wali | Form Bayar, Rating, dll full-width di mobile | Form mudah diisi di HP |
| 18.6 | Tablet (768px) | Wali | Layout berubah ke 2 kolom, sidebar muncul | Layout tablet tampil |
| 18.7 | Desktop (1024px+) | Wali | Layout penuh, sidebar + konten | Layout desktop tampil |
