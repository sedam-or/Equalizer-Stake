---
fr: FR-10
uc: UC-08
status: Approved
module: Multi-Student per Guardian
---

# SRS-09: Multi-Student per Guardian

## 1. Deskripsi Modul

Modul ini mengelola hubungan 1 Guardian ke N student. Menyediakan unified dashboard dengan fitur profile switching, penambahan student terpusat, invoice gabungan, dan penanganan pembekuan yang bersifat household-level.

## 2. Data Model

### 2.1 Relasi Guardian — Student

```
Guardian (1) ──── (N) Student
```

| Field | Tipe | Keterangan |
|-------|------|------------|
| `guardian_id` | PK | ID unik Guardian |
| `student_id` | PK | ID unik student |
| `relationship` | VARCHAR | Hubungan (Ayah, Ibu, Wali, dll) |
| `is_primary` | BOOLEAN | Guardian utama (untuk invoice & notifikasi) |
| `added_by` | FK | Admin yang menambahkan |
| `added_at` | TIMESTAMP | Waktu penambahan |

## 3. Unified Dashboard

### 3.1 Profile Switching

- Guardian melihat daftar semua student dalam 1 dashboard.
- Bisa beralih antar profil student menggunakan dropdown/tab selector.
- Setiap profil student menampilkan: jadwal, kehadiran, tagihan, status akun.
- Status akun aktif/frozen ditampilkan dengan badge visual.

### 3.2 Tampilan Badge Frozen

| Elemen | Detail |
|--------|--------|
| Badge | Label merah "BEKU" di samping nama student |
| Overlay | Profil student frozen menampilkan overlay semi-transparan |
| Info | Alasan pembekuan dan estimasi tanggal aktif kembali |
| Notifikasi | Banner khusus: "Akun [Nama Student] sedang dibekukan" |

## 4. Penambahan Student

### 4.1 Aturan Penambahan

- Hanya **Admin** yang dapat menambahkan student baru ke Guardian.
- Prasyarat: student sudah menyelesaikan registrasi + audition.
- Guardian **tidak bisa** menambahkan student sendiri.

### 4.2 Alur

```
Student daftar + audition
 ↓
Admin review hasil audition
 ↓
Admin tambahkan student ke Guardian yang sesuai
 ↓
Notifikasi ke Guardian: "Student [Nama] telah ditambahkan"
```

## 5. Invoice Gabungan

| Aspek | Detail |
|-------|--------|
| Cakupan | 1 invoice mencakup semua student di bawah 1 Guardian |
| Breakdown | Item per student tercantum dalam 1 invoice |
| Pembayaran | 1 kali bayar untuk semua student |
| Partial payment | TIDAK diizinkan — invoice harus dibayar penuh |
| Jatuh tempo | Berlaku sama untuk seluruh item dalam invoice |

## 6. Pembekuan (Freeze)

### 6.1 Aturan Pembekuan

- Pembekuan berlaku untuk **SEMUA student** di bawah 1 Guardian.
- Tidak ada pembekuan per student — bersifat household-level.
- Alasan pembekuan: keterlambatan pembayaran (sesuai SRS terkait).

### 6.2 Dampak Pembekuan

| Aspek | Dampak |
|-------|--------|
| Akses kelas | Semua student tidak bisa mengakses kelas |
| Booking studio | Diblokir untuk semua student |
| Dashboard | Badge "BEKU" pada semua profil student |
| Notifikasi | Notifikasi khusus ke Guardian: "Semua akun student Anda dibekukan" |

### 6.3 Penanganan Invoice Saat Freeze

- Invoice yang sudah ada tetap aktif dan harus dibayar.
- Invoice baru TIDAK dibuat selama masa pembekuan.
- Setelah unfreeze, invoice berikutnya dibuat sesuai siklus normal.

## 7. Constraint

- 1 student hanya bisa terikat ke 1 Guardian utama (`is_primary = TRUE`).
- Guardian bisa memiliki secondary guardian, tapi invoice dan notifikasi hanya ke primary.
- Penghapusan relasi guardian-student harus melalui proses audit (tercatat di audit log).
- Data student yang sudah dihapus dari Guardian tetap ada di sistem (soft delete) untuk keperluan audit.
