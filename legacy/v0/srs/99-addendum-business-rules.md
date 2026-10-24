# SRS Addendum — Koreksi Business Rules Berdasarkan Data Asli EQS
**Tanggal:** 2026-07-01
**Sumber:** Data asli Equalizer-Stake (website 2014 + konfirmasi owner)
**Status:** APPROVED — override SRS asli jika ada konflik

---

## 1. Koreksi: Jatuh Tempo Pembayaran

**SRS lama (salah):** Auto-invoice tanggal 5, denda tanggal 11
**Koreksi:**
- Tanggal 5: Kirim reminder pembayaran via WhatsApp (pihak ketiga) + email backup
- Tanggal 10, 23:59 WIB: Batas pembayaran
- Tanggal 11: Denda Rp 100.000 + freeze akun

## 2. Koreksi: Kebijakan Absensi Student

**SRS lama (salah):** Ada credit refund tier (>24h=100%, 1-24h=50%, <1h=0%)
**Koreksi:**
- Student absen → jam HANGUS, tidak bisa diganti, uang tidak bisa ditarik
- Instructor absen → diganti oleh instructor pengganti atau hari lain
- Libur nasional → tidak ada penggantian jam
- Minggu ke-5 dalam bulan → libur, diganti 1x kelas teori tambahan

## 3. Koreksi: Kebijakan Cuti

**SRS lama:** Tidak ada aturan cuti jelas
**Koreksi:**
- Cuti maksimal 1 bulan (kena bea administrasi)
- Lebih dari 1 bulan → dianggap resign, wajib daftar ulang

## 4. Koreksi: Studio Reservation

**SRS lama (FR-08):** Ada fitur booking studio rehearsal/recording
**Koreksi:**
- HAPUS fitur studio reservation dari sistem
- Studio = tempat course, bukan layanan sewa terpisah
- Booking studio = langsung WhatsApp admin (di luar sistem)
- Room hanya sebagai lokasi kelas course

## 5. Koreksi: Instrumen

**SRS lama:** biola, piano, gitar, drum, keyboard, flute, saxophone, vokal, ensemble
**Koreksi (tambah):**
- Drum (sudah ada di SRS, konfirmasi tersedia)
- Flute (sudah ada di SRS, konfirmasi tersedia)
- Saxophone (sudah ada di SRS, konfirmasi tersedia)
- Bass (gitar bass, bukan instrumen terpisah)

## 6. Koreksi: Level per Instrumen

**SRS lama:** pemula, dasar, menengah, lanjut (flat untuk semua)
**Koreksi:**

| Instrumen | Level |
|-----------|-------|
| Piano Klasik | pra_elementary, elementary, grade_1, grade_2, grade_3, grade_4, grade_5, grade_6 |
| Piano Pop | beginner, intermediate, intermediate_2 |
| Vokal | basic, intermediate, advance |
| Biola | basic, intermediate, advance |
| Gitar | basic, intermediate, advance |
| Keyboard | basic, intermediate, advance |
| Drum | basic, intermediate, advance |
| Flute | basic, intermediate, advance |
| Saxophone | basic, intermediate, advance |

## 7. Tipe Kelas (BARU)

**SRS lama:** Tidak ada tipe kelas
**Koreksi:**

| Tipe | Jumlah Student | Keterangan |
|------|-------------|------------|
| private | 1 | Kelas privat 1-on-1 |
| intensif_2 | 2 | Kelas intensif 2 student |
| intensif_3 | 3 | Kelas intensif 3 student |
| semi_intensif | 4-6 | Kelas semi intensif (max 6 tahun) |
| expatriate_private | 1 | Kelas expatriate (bahasa Inggris) |
| expatriate_2 | 2 | Kelas expatriate 2 student |
| expatriate_3 | 3 | Kelas expatriate 3 student |
| tunas | 1 | Kelas balita (min usia 3.5 tahun) |
| paket_liburan | 1-4 | Kelas paket liburan |

## 8. Harga (Update 2024 — asumsi +20% dari 2014)

### Registration Fee
| Instrumen | Biaya Daftar |
|-----------|-------------|
| Piano Klasik | Rp 420.000 |
| Piano Pop | Rp 420.000 |
| Vokal | Rp 420.000 |
| Biola | Rp 540.000 |
| Gitar Akustik | Rp 420.000 |
| Keyboard | Rp 480.000 |
| Drum | Rp 420.000 |
| Flute | Rp 420.000 |
| Saxophone | Rp 420.000 |
| Paket Liburan | Rp 60.000 |

### Vokal
| Tipe | SPP/Bulan | Durasi |
|------|----------|--------|
| Private (1 student) | Rp 600.000 | 45 menit |
| Intensif (2 student) | Rp 390.000 | 45 menit |
| Intensif (3 student) | Rp 330.000 | 45 menit |
| Semi Intensif (4 student) | Rp 300.000 | 45 menit |
| Expatriate Private | Rp 840.000 | 45 menit |
| Expatriate 2 student | Rp 660.000 | 45 menit |
| Expatriate 3 student | Rp 600.000 | 45 menit |

### Piano Klasik
| Tipe | SPP/Bulan | Durasi |
|------|----------|--------|
| Beginner | Rp 420.000 | 30 menit |
| Grade 1 | Rp 480.000 | 45 menit |
| Grade 2 | Rp 480.000 | 45 menit |
| Grade 3 | Rp 510.000 | 45 menit |
| Grade 4 | Rp 540.000 | 45 menit |
| Grade 5 | Rp 570.000 | 45 menit |
| Grade 6 | Rp 630.000 | 45 menit |
| Anak Aktif 30" | Rp 600.000 | 30 menit |
| Anak Aktif 45" | Rp 720.000 | 45 menit |

### Piano Expatriate
| Tipe | SPP/Bulan | Durasi |
|------|----------|--------|
| Beginner | Rp 540.000 | 45 menit |
| Grade 1-6 | Rp 600.000 - 840.000 | 45 menit |

### Piano Pop
| Tipe | SPP/Bulan | Durasi |
|------|----------|--------|
| Umum | Rp 600.000 | 45 menit |
| Expatriate | Rp 720.000 | 45 menit |

### Instrumen Lain
| Instrumen | SPP/Bulan | Durasi |
|-----------|----------|--------|
| Biola | Rp 420.000 | 45 menit |
| Gitar Akustik | Rp 420.000 | 45 menit |
| Keyboard | Rp 420.000 | 45 menit |
| Drum | Rp 420.000 | 45 menit |
| Flute | Rp 420.000 | 45 menit |
| Saxophone | Rp 420.000 | 45 menit |

### Paket Liburan
| Tipe | SPP | Durasi |
|------|-----|--------|
| Kelas A (1 student) | Rp 780.000 | 45 menit |
| Kelas B (2 student) | Rp 900.000 | 45 menit |
| Kelas C (3 student) | Rp 1.020.000 | 45 menit |
| Kelas D (4 student) | Rp 1.260.000 | 45 menit |

## 9. Syarat Usia
| Instrumen | Usia Minimal |
|-----------|-------------|
| Piano, Drum, Vokal | 3.5 tahun |
| Biola, Gitar, Flute, Keyboard, Saxophone | 7 tahun |

## 10. Waktu Operasional
| Hari | Jam |
|------|-----|
| Senin - Jumat | 14:30 - 17:00 WIB |
| Sabtu | 08:00 - 12:00 WIB |
| Minggu | Libur |

## 11. Durasi Frekuensi
- Standar: 45 menit per sesi, 1x per minggu
- Pengecualian: Piano Beginner = 30 menit
- Evaluasi: minimal 3 bulan belajar boleh ikut ujian

---

## 12. Foto Profil (BARU)
- Setiap user (owner, admin, instructor, guardian) WAJIB punya foto profil
- Default: avatar lingkaran dengan inisial nama (warna random)
- Upload: klik foto, modal crop, simpan
- Format: JPEG/PNG, max 2MB
- Disimpan: encrypted (AES-256-GCM) di server
- Hapus profil = hapus foto (UU PDP Pasal 19)

## 13. Feedback Instructor ke Guardian (BARU)
- Instructor memberikan feedback progress student ke guardian
- Kategori: Kemajuan, Perilaku, Kehadiran, Rekomendasi
- Rating: 1-5 per kategori
- Komentar: textarea, opsional
- Wali murid bisa lihat feedback di halaman "Anak Saya"
- Feedback tersimpan di audit log

## 14. Penilaian Guardian ke Instructor (BARU)
- Wali murid memberikan penilaian ke instructor course anak
- Rating: 1-5 bintang
- Komentar: textarea, opsional
- Hanya bisa menilai instructor yang mengajar anaknya
- Rating tersimpan di audit log
- Admin bisa lihat rating instructor di dashboard

## 15. Page Detail Instrumen dan Kelas (BARU)
- Public page: /instruments
- Menjelaskan 9 instrumen dengan detail
- Per instrumen: deskripsi, level, tipe kelas, harga, durasi, usia minimal
- CTA "Daftar Sekarang" per instrumen
- Tabel ringkasan harga semua instrumen
- FAQ: "Bisa ganti instrumen?", "Ada trial?", "Diskon paket?"

## 16. Page Bantuan (BARU)
- Public: /help - FAQ accordion, search bar, kontak WhatsApp
- Dashboard: /dashboard/help - panduan per role
- Kategori FAQ: Umum, Akun, Pembayaran, Jadwal, Absensi, Privasi
- Jam operasional: Senin-Jumat 14:30-17:00, Sabtu 08:00-12:00

## 17. Email Reset Password Pages (BARU)
- /reset-password/success - "Password berhasil diubah!" + tombol Login
- /reset-password/error - "Link tidak valid atau sudah kedaluwarsa" + tombol Kirim Ulang
- Token valid: 1 jam
- Rate limit: max 3 reset per jam per email

## 18. Mobile-First Guardian (BARU)
- NFR-USA-02: mobile-first, min 320px
- 60% user akses via HP
- Bottom navigation (bukan sidebar) di mobile
- Kartu besar, tombol besar, font besar
- Sidebar hanya di desktop (>=768px)
