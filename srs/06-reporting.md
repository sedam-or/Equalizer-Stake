---
fr: FR-07, FR-13
uc: UC-09, UC-10
status: Proposed
module: Reporting & Dashboard
---

# SRS-06: Reporting & Dashboard

## 1. Deskripsi Modul

Modul reporting menyediakan dashboard untuk Owner dan fitur pengecekan eligibility ujian. Semua laporan bersifat read-only dengan opsi export cetak. Target performa: load ≤3 detik.

## 2. Exam Eligibility Check (FR-07 / UC-09)

### 2.1 Kriteria Eligibility

Siswa memenuhi syarat ujian jika **semua** kondisi berikut terpenuhi:

| No | Kriteria | Field | Threshold |
|----|----------|-------|-----------|
| 1 | Akun aktif | `account_status` | `ACTIVE` |
| 2 | Kehadiran ≥90% | `attendance_rate` | ≥ 0.90 |
| 3 | Max 1 no-show per 3 bulan | `no_show_count` | ≤ 1 (rolling 3 bulan) |

### 2.2 Penanganan Data Tidak Lengkap

| Kondisi | Perlakuan |
|---------|-----------|
| Siswa baru (<3 bulan terdaftar) | Tampilkan badge "Data Tidak Lengkap" dengan catatan: periode observasi belum selesai |
| Status `PendingVerification` | Tampilkan warning flag kuning: "Verifikasi belum selesai — eligibility tidak dapat ditentukan" |

### 2.3 Business Rule

- Perhitungan attendance_rate berdasarkan total sesi hadir / total sesi terjadwal sejak pendaftaran.
- No-show dihitung per event (bukan per kelas). Reset rolling window setiap 3 bulan.
- Hasil eligibility ditampilkan per siswa di halaman profil dan di dashboard Owner.

## 3. Owner Dashboard (FR-13 / UC-10)

### 3.1 Komponen Dashboard

| Widget | Data | Refresh |
|--------|------|---------|
| **Revenue** | Total pemasukan bulanan, tren 6 bulan terakhir | Real-time |
| **Attendance** | Rata-rata kehadiran per studio, per kelas | Harian |
| **Frozen Accounts** | Jumlah akun beku, alasan, durasi | Real-time |
| **Studio Utilization** | Persentase pemakaian per studio per hari/minggu | Harian |
| **Audit Summary** | Jumlah event audit hari ini, flag anomali | Real-time |

### 3.2 Aturan Dashboard

- Semua data **read-only** — Owner tidak dapat mengubah data dari dashboard.
- Export: format cetak (printable HTML/PDF).
- Filter: rentang tanggal (default: bulan berjalan).
- Performa: seluruh dashboard harus load dalam ≤3 detik.

## 4. Constraint

- Data revenue tidak menyertakan detail per-transaksi (privasi), hanya agregat.
- Attendance data yang belum lengkap (< 5 sesi) ditandai dengan asterisk (*).
- Dashboard hanya dapat diakses oleh role Owner.
- Semua akses dashboard dicatat di audit log.
