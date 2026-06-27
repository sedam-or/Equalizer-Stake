---
fr: FR-10
uc: UC-08
status: Approved
module: Multi-Student per Wali Murid
---

# SRS-09: Multi-Student per Wali Murid

## 1. Deskripsi Modul

Modul ini mengelola hubungan 1 Wali Murid ke N siswa. Menyediakan unified dashboard dengan fitur profile switching, penambahan siswa terpusat, invoice gabungan, dan penanganan pembekuan yang bersifat household-level.

## 2. Data Model

### 2.1 Relasi Guardian — Student

```
Wali Murid (1) ──── (N) Student
```

| Field | Tipe | Keterangan |
|-------|------|------------|
| `wali_murid_id` | PK | ID unik Wali Murid |
| `student_id` | PK | ID unik siswa |
| `relationship` | VARCHAR | Hubungan (Ayah, Ibu, Wali, dll) |
| `is_primary` | BOOLEAN | Wali Murid utama (untuk invoice & notifikasi) |
| `added_by` | FK | Admin yang menambahkan |
| `added_at` | TIMESTAMP | Waktu penambahan |

## 3. Unified Dashboard

### 3.1 Profile Switching

- Wali Murid melihat daftar semua siswa dalam 1 dashboard.
- Bisa beralih antar profil siswa menggunakan dropdown/tab selector.
- Setiap profil siswa menampilkan: jadwal, kehadiran, tagihan, status akun.
- Status akun aktif/frozen ditampilkan dengan badge visual.

### 3.2 Tampilan Badge Frozen

| Elemen | Detail |
|--------|--------|
| Badge | Label merah "BEKU" di samping nama siswa |
| Overlay | Profil siswa frozen menampilkan overlay semi-transparan |
| Info | Alasan pembekuan dan estimasi tanggal aktif kembali |
| Notifikasi | Banner khusus: "Akun [Nama Siswa] sedang dibekukan" |

## 4. Penambahan Siswa

### 4.1 Aturan Penambahan

- Hanya **Admin** yang dapat menambahkan siswa baru ke Wali Murid.
- Prasyarat: siswa sudah menyelesaikan registrasi + audition.
- Wali Murid **tidak bisa** menambahkan siswa sendiri.

### 4.2 Alur

```
Siswa daftar + audition
    ↓
Admin review hasil audition
    ↓
Admin tambahkan siswa ke Wali Murid yang sesuai
    ↓
Notifikasi ke Wali Murid: "Siswa [Nama] telah ditambahkan"
```

## 5. Invoice Gabungan

| Aspek | Detail |
|-------|--------|
| Cakupan | 1 invoice mencakup semua siswa di bawah 1 Wali Murid |
| Breakdown | Item per siswa tercantum dalam 1 invoice |
| Pembayaran | 1 kali bayar untuk semua siswa |
| Partial payment | TIDAK diizinkan — invoice harus dibayar penuh |
| Jatuh tempo | Berlaku sama untuk seluruh item dalam invoice |

## 6. Pembekuan (Freeze)

### 6.1 Aturan Pembekuan

- Pembekuan berlaku untuk **SEMUA siswa** di bawah 1 Wali Murid.
- Tidak ada pembekuan per siswa — bersifat household-level.
- Alasan pembekuan: keterlambatan pembayaran (sesuai SRS terkait).

### 6.2 Dampak Pembekuan

| Aspek | Dampak |
|-------|--------|
| Akses kelas | Semua siswa tidak bisa mengakses kelas |
| Booking studio | Diblokir untuk semua siswa |
| Dashboard | Badge "BEKU" pada semua profil siswa |
| Notifikasi | Notifikasi khusus ke Wali Murid: "Semua akun siswa Anda dibekukan" |

### 6.3 Penanganan Invoice Saat Freeze

- Invoice yang sudah ada tetap aktif dan harus dibayar.
- Invoice baru TIDAK dibuat selama masa pembekuan.
- Setelah unfreeze, invoice berikutnya dibuat sesuai siklus normal.

## 7. Constraint

- 1 siswa hanya bisa terikat ke 1 Wali Murid utama (`is_primary = TRUE`).
- Wali Murid bisa memiliki secondary guardian, tapi invoice dan notifikasi hanya ke primary.
- Penghapusan relasi guardian-student harus melalui proses audit (tercatat di audit log).
- Data siswa yang sudah dihapus dari Wali Murid tetap ada di sistem (soft delete) untuk keperluan audit.
