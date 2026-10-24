---
module: use-cases
source: BAB4 (SRS_Equalizer-Stake_BAB4.md)
---

# Use Case Diagrams — Equalizer-Stake

## 20 Use Cases across 7 Modules

### Autentikasi
| UC | Nama | Aktor |
|----|------|-------|
| UC-01 | Login & Role-Based Auth | Owner, Admin, Instructor, Guardian |
| UC-02 | Kelola Profil | Semua |

### Penjadwalan
| UC | Nama | Aktor |
|----|------|-------|
| UC-03 | Input Jadwal Course | Admin |
| UC-04 | Validasi Konflik Slot | Sistem (otomatis) |
| UC-05 | Kelola Libur & Minggu ke-5 | Admin, Sistem |
| UC-06 | Reservasi Studio | Guardian, Admin |

### Penagihan
| UC | Nama | Aktor |
|----|------|-------|
| UC-07 | Terbitkan Tagihan Otomatis | Sistem |
| UC-08 | Verifikasi Pembayaran Manual | Admin |
| UC-09 | Terapkan Denda & Bekukan | Sistem |
| UC-10 | Buka Pembekuan Akun | Admin |

### Kehadiran
| UC | Nama | Aktor |
|----|------|-------|
| UC-11 | Input Absensi + Foto | Instructor |
| UC-12 | Validasi GPS & Waktu | Sistem |
| UC-13 | Lacak Kelayakan Ujian | Sistem, Admin, Owner |

### Notifikasi
| UC | Nama | Aktor |
|----|------|-------|
| UC-14 | Kirim WhatsApp | Sistem |
| UC-15 | Email Cadangan | Sistem |

### Pelaporan
| UC | Nama | Aktor |
|----|------|-------|
| UC-16 | Dashboard Monitoring | Owner |
| UC-17 | Ekspor Laporan | Owner |
| UC-18 | Audit Log | Owner, Admin |

### Kepatuhan
| UC | Nama | Aktor |
|----|------|-------|
| UC-19 | Ajukan Hapus Data | Guardian, Instructor |
| UC-20 | Proses Hapus PII | Sistem, Admin |

---

## Include Relations (mandatory)

| Dari | Ke | Alasan |
|------|-----|--------|
| UC-03 | UC-04 | Setiap input jadwal → validasi konflik otomatis |
| UC-07 | UC-14 | Setiap invoice → notifikasi WhatsApp ke Guardian |
| UC-08 | UC-09 | Setiap verifikasi pembayaran → cek denda terutang |
| UC-11 | UC-12 | Setiap absensi → validasi GPS + waktu server |

## Extend Relations (conditional)

| Dari | Ke | Kondisi |
|------|-----|---------|
| UC-12 | UC-13 | Jika kehadiran memenuhi threshold ≥10 berturut-turut |
| UC-14 | UC-15 | Jika WhatsApp gagal (timeout 30s atau API error) |

---

## Actor Access Matrix

| Aktor | UC yang Bisa Diakses |
|-------|---------------------|
| Owner | UC-01, UC-02, UC-16, UC-17, UC-18 |
| Admin | UC-01..UC-10, UC-14..UC-20 (kecuali UC-11 Instructor) |
| Instructor | UC-01, UC-02, UC-11 |
| Guardian | UC-01, UC-02, UC-06, UC-10 (view), UC-19 |

---

## 3 Detailed Scenarios

### SCR-001: Login (UC-01)

**Prasyarat**: Akun aktif, kredensial valid, browser accessible, internet stabil.

**Normal Flow:**
1. Buka URL sistem
2. Masukkan username
3. Masukkan password
4. Tekan tombol Masuk
5. Validasi kredensial ≤2 detik
6. Terbitkan JWT sesuai peran
7. Catat login di audit log
8. Tampilkan dashboard sesuai peran

**Alternatif — Kredensial Salah:**
Increment fail counter → Tampilkan "Username atau password tidak valid" → Ulangi (maks 5x)

**Alternatif — Percobaan ke-5 Gagal:**
Kunci akun 15 menit → Kirim notifikasi email ke Admin → Tampilkan pesan waktu tunggu

**Alternatif — Akun Nonaktif:**
Tampilkan "Akun tidak aktif. Hubungi Admin." → Hentikan proses

**Alternatif — Login Pertama (Password Sementara):**
Redirect ke halaman wajib ubah password → Password baru disimpan, flag direset, audit log

**Error Handling:**
- Password kedaluwarsa (>7 hari): "Link telah kedaluwarsa. Hubungi Admin untuk mengirim ulang."
- Token service gangguan: Tampilkan pesan pemeliharaan, catat insiden di log

---

### SCR-003: Input Jadwal (UC-03 + UC-04)

**Prasyarat**: Admin login, data instructor/ruang/student tersedia, hari libur dikonfigurasi, jam operasional aktif.

**Normal Flow:**
1. Buka form jadwal baru
2. Pilih student, instructor, ruangan
3. Pilih tanggal & waktu sesi
4. Validasi konflik real-time ≤1 detik
5. Tampilkan "Slot tersedia"
6. Klik "Simpan Jadwal"
7. Simpan status Locked + catat audit log
8. Kirim notifikasi ke Instructor & Guardian

**Alternatif — Konflik Slot Terdeteksi:**
Validasi deteksi konflik → Tampilkan detail konflik → 3 slot alternatif terdekat → Admin pilih atau batalkan

**Alternatif — Concurrency (2 Admin):**
Admin kedua coba simpan slot sama → Concurrency lock aktif → "Data baru saja diperbarui, periksa kembali"

**Alternatif — Pembatalan Sesi Locked:**
Admin pilih "Batalkan Sesi" → Wajib input alasan → Kredit dikembalikan penuh → Audit log

**Error Handling:**
- Konflik: "Ruang A sudah digunakan oleh [Instructor X] pukul 15.00-16.00" + 3 alternatif
- Draft auto-save 30 detik via IndexedDB

---

### SCR-011: Absensi Digital (UC-11 + UC-12)

**Prasyarat**: Instructor login, sesi aktif, di lokasi studio, GPS aktif, kamera accessible.

**Normal Flow:**
1. Buka menu "Absensi Hari Ini"
2. Pilih sesi aktif
3. Ambil foto student via kamera
4. Client-side: validasi proporsi wajah ≥30% (face-api.js) ≤1 detik
5. Ambil koordinat GPS perangkat
6. Kirim (foto + GPS + timestamp) ke Backend
7. Backend validasi GPS radius 150m + timestamp server
8. Simpan status "Hadir" + potong kredit
9. Kirim notifikasi hadir ke Guardian

**Alternatif — Wajah Tidak Lolos (Client-Side):**
face-api.js gagal → "Pastikan wajah mengisi minimal 1/3 area foto" → Instructor ambil ulang → Tidak ada request ke server

**Alternatif — GPS Luar Radius:**
Validasi server gagal → Status "Menunggu Verifikasi" → Notifikasi ke Admin untuk review ≤2 jam

**Alternatif — Koneksi Terputus:**
Foto tersimpan di perangkat lokal → Auto-upload saat koneksi kembali (≤5 menit)

**Error Handling:**
- Wajah tidak terdeteksi: Tidak ada request ke server
- GPS luar radius: "Lokasi Anda berada di luar area studio. Absensi perlu verifikasi Admin."
- Server validasi wajah gagal: Status PendingVerification, tidak ditolak

---

## State Transitions

### Booking
| Dari | Ke | Trigger | Side Effect |
|------|-----|---------|-------------|
| Draft | Pending | Data lengkap, dikirim | Tahan slot sementara |
| Pending | Locked | Admin konfirmasi | Permanen, notifikasi, audit log |
| Locked | Rescheduled | Instructor berhalangan / libur | Kredit TIDAK dipotong |
| Locked | Cancelled | Admin batalkan (alasan) | Slot dilepas, kredit sesuai kebijakan |
| Locked | Completed | Absensi dikonfirmasi | Kredit dipotong, hitung kelayakan |

### Payment
| Dari | Ke | Trigger | Side Effect |
|------|-----|---------|-------------|
| Unpaid | Overdue | Tgl 11 00:00 | Denda Rp 100k otomatis |
| Overdue | Frozen | Otomatis | Semua akses student diblokir |
| Frozen | Paid | Admin verifikasi pelunasan | Aktif kembali, notifikasi, audit log |

### Attendance
| Status | Dampak Kredit | Dampak Kelayakan |
|--------|--------------|-----------------|
| Present | Terpotong (sesi dihitung) | Positif |
| AbsentExcused | Terpotong (kebijakan EQS) | Terhitung absen |
| AbsentNoShow | Terpotong + sesi hangus | >1x/3bln = tidak layak ujian |
| Rescheduled (Instructor) | TIDAK terpotong | TIDAK terhitung absen |

### Student Profile
| Dari | Ke | Trigger | Side Effect |
|------|-----|---------|-------------|
| Trial | Active | Admin verifikasi admin fee + audisi | enrollment_date diisi |
| Active | Cuti | Guardian ajukan (maks 1 bln) | Sesi ditahan, tagihan jeda |
| Cuti | Active | Kembali ≤1 bulan | Kembali ke jadwal |
| Cuti | Inactive | >1 bulan tanpa kembali | Auto-inactive, harus daftar ulang |
| Active | Inactive | Admin nonaktifkan | PII dianonimasi, audit log |
