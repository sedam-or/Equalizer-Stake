---
fr:
  - FR-02
  - FR-12
uc:
  - UC-02
  - UC-05
status: Approved
module: penjadwalan
title: "Penjadwalan Kursus dengan Validasi Konflik Real-Time"
version: "2.0"
date: "2026-06-27"
---

# FR-02 & FR-12 — Penjadwalan Kursus

## 1. Ringkasan

| Field | Value |
|---|---|
| Kode | FR-02, FR-12 |
| Status | **Approved** |
| Aktor | Admin Operasional |
| Pemicu | Admin menyimpan jadwal baru / perubahan jadwal |
| Modul | Penjadwalan |
| UC Linked | UC-02, UC-05 |
| Verifikasi | Test |

---

## 2. FR-02 — Penjadwalan Kursus dengan Validasi Konflik Real-Time

### 2.1 Spesifikasi

Sistem shall memvalidasi ketersediaan slot (guru, ruangan, waktu) secara real-time saat Admin menyimpan jadwal baru atau melakukan perubahan jadwal. Konflik slot tidak boleh lolos ke penyimpanan data dalam keadaan apapun.

### 2.2 Validasi Konflik

| Dimensi | Rule |
|---|---|
| Ruangan | Waktu sesi + jeda 10 menit tidak tumpang tindih di ruangan yang sama |
| Guru | Guru tidak dijadwalkan dua sesi bersamaan |
| Performa | Validasi ≤ **1 detik** dari tombol Simpan |

### 2.3 Jam Operasional

| Hari | Jam |
|---|---|
| Senin–Jumat | 14:30 – 17:00 |
| Sabtu | 08:00 – 12:00 |
| Minggu | **Tutup** |
| Hari Libur Nasional | **Tidak ada penjadwalan** |

### 2.4 Concurrency Control

- Dua Admin tidak boleh menyimpan jadwal bentrok secara bersamaan
- Mekanisme: database-level row locking / optimistic concurrency (version field)
- Jika konflik terdeteksi: Admin kedua mendapat pesan "Slot sudah diambil, silakan refresh"

### 2.5 Logika Kredit Pembatalan

| Pembatalan oleh | Syarat | Kredit dikembalikan |
|---|---|---|
| **Admin** (sesi Locked) | Error / duplikasi / konflik | **100%** (refund penuh) |
| **Wali Murid** | Notifikasi ≥ 24 jam sebelum sesi | **50%** |
| **Wali Murid** | Notifikasi < 24 jam sebelum sesi | **0%** (hangus) |

Semua pembatalan wajib disertai **alasan** dan dicatat di **audit log**.

### 2.6 Draft Auto-Save

- Jadwal draft auto-save setiap **30 detik**
- Jika session timeout / disconnect, draft dipulihkan otomatis saat login ulang

---

## 3. FR-12 — Pengelolaan Libur Nasional dan Minggu ke-5

### 3.1 Minggu ke-5 — Non-Credit Theory Session

| Aspek | Detail |
|---|---|
| Durasi | **30 menit** |
| Tipe | Non-Credit Theory (tidak mengurangi kredit siswa) |
| Trigger | Bulan dengan 5 minggu, minggu ke-5 tidak ada sesi reguler |
| Konten | Teori musik, review materi, evaluasi |

### 3.2 Penanganan Hari Libur Nasional

1. Sistem mendeteksi jadwal yang jatuh pada hari libur nasional
2. **Reschedule otomatis**: pindahkan ke hari yang sama minggu depan
3. Jika minggu depan tidak ada slot tersedia → konversi ke **Theory Session** (30 menit, non-credit)
4. Admin dan Wali Murid mendapat notifikasi perubahan

---

## 4. Kriteria Penerimaan (Given-When-Then)

### 4.1 Jadwal Berhasil Disimpan

```
Given   Admin memilih slot waktu, guru, dan ruangan yang tersedia
When    Admin menekan tombol Simpan Jadwal
Then    Sistem memvalidasi ketersediaan ≤ 1 detik; jadwal tersimpan;
        notifikasi dikirim ke Guru
```

### 4.2 Konflik Terdeteksi

```
Given   Admin memilih slot yang bentrok dengan jadwal existing
When    Admin menekan tombol Simpan Jadwal
Then    Sistem menampilkan daftar slot alternatif yang tersedia;
        jadwal tidak tersimpan
```

### 4.3 Pembatalan oleh Admin

```
Given   Sesi berstatus Locked dan Admin membatalkan dengan alasan
When    Admin menekan Batalkan Sesi
Then    Kredit siswa dikembalikan 100%; pembatalan dicatat di audit log
```

### 4.4 Pembatalan oleh Wali Murid (≥ 24 jam)

```
Given   Wali Murid membatalkan ≥ 24 jam sebelum sesi
When    Konfirmasi pembatalan
Then    Kredit dikembalikan 50%; pembatalan dicatat di audit log
```

### 4.5 Pembatalan oleh Wali Murid (< 24 jam)

```
Given   Wali Murid membatalkan < 24 jam sebelum sesi
When    Konfirmasi pembatalan
Then    Kredit hangus (0% dikembalikan); pembatalan dicatat di audit log
```

### 4.6 Libur Nasional

```
Given   Jadwal jatuh pada hari libur nasional
When    Sistem memproses jadwal
Then    Reschedule ke hari sama minggu depan; jika tidak ada slot →
        konversi ke Theory Session 30 menit; notifikasi dikirim
```

---

## 5. Tahapan Alur (Flow Steps)

### 5.1 Normal Flow — Buat Jadwal

1. Admin pilih tanggal, jam, guru, ruangan
2. Sistem validasi real-time (guru, ruangan, waktu + 10min break) ≤ 1 detik
3. Jika tersedia → simpan jadwal, kirim notifikasi ke Guru
4. Jika konflik → tampilkan slot alternatif

### 5.2 Jalur Alternatif — Konflik Concurrent

1. Admin A dan Admin B submit jadwal bentrok bersamaan
2. Database lock mencegah double-write
3. Admin yang kalah mendapat pesan "Slot sudah diambil"
4. Admin refresh dan pilih slot lain

### 5.3 Jalur Alternatif — Libur Nasional

1. Sistem deteksi jadwal pada hari libur
2. Cari slot di hari sama minggu depan
3. Jika slot tersedia → reschedule otomatis
4. Jika tidak → konversi ke Theory Session (30 min)
5. Kirim notifikasi ke semua pihak terkait

---

## 6. Penanganan Error

| Kondisi | Penanganan |
|---|---|
| Validasi > 1 detik | Timeout error, minta Admin ulangi |
| Database lock timeout | Pesan "Sistem sibuk, coba lagi"; log insiden |
| Slot hilang saat validasi | Refresh otomatis, tampilkan slot terbaru |
| Draft save gagal | Retry 3x; jika gagal → alert Admin |
| Libur nasional tanpa slot | Auto-convert ke Theory Session |
