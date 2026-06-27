---
fr: FR-01
uc: UC-01
status: Approved
module: autentikasi
title: "Login dan Autentikasi Berbasis Peran (RBAC)"
version: "2.0"
date: "2026-06-27"
---

# FR-01 — Login dan Autentikasi Berbasis Peran (RBAC)

## 1. Ringkasan

| Field | Value |
|---|---|
| Kode | FR-01 |
| Status | **Approved** |
| Aktor | Owner, Admin Operasional, Guru Musik, Wali Murid |
| Pemicu | Pengguna membuka URL sistem |
| Modul | Autentikasi |
| UC Linked | UC-01 |
| Verifikasi | Test |

## 2. Spesifikasi

Sistem shall menyediakan halaman login tunggal yang mengarahkan pengguna ke dashboard sesuai perannya (Owner, Admin, Guru, Wali Murid) setelah autentikasi berhasil. Token sesi (JWT) diterbitkan dengan masa berlaku sesuai batas waktu per peran.

### 2.1 Token & Sesi

| Aspek | Spesifikasi |
|---|---|
| Token | JWT HS256, per-role payload |
| Owner | 30 menit timeout |
| Admin Operasional | 30 menit timeout |
| Guru Musik | 30 menit timeout |
| Wali Murid | 60 menit timeout |

Setelah timeout, pengguna diminta login ulang. Draft tersimpan dipulihkan otomatis.

### 2.2 Brute Force Protection

- Maksimal **5 percobaan login gagal** dalam **15 menit**
- Setelah percobaan ke-5 gagal: akun dikunci sementara 15 menit
- Notifikasi email dikirim ke Admin saat akun terkunci

### 2.3 Onboarding Password

Ketika Admin membuat akun baru untuk Guru atau Wali Murid:

| Fitur | Detail |
|---|---|
| Password | Auto-generate, **8 karakter alphanumeric** |
| Pengiriman | Email berisi link akses, username, password sementara |
| Kedaluwarsa | **7 hari** dari penerbitan |
| First login | **Wajib ubah password** sebelum akses dashboard |
| Resend | Admin dapat mengirim ulang via dashboard jika expired |

### 2.4 Ubah Password

- Setiap pengguna (kecuali Owner) dapat mengubah password via menu **Pengaturan Akun**
- Perubahan password dicatat di **audit log**
- Berlaku setelah login, bukan saat first-login (yang sudah wajib ganti)

## 3. Kriteria Penerimaan (Given-When-Then)

### 3.1 Login Berhasil

```
Given   Pengguna memiliki akun aktif dan kredensial yang benar
When    Pengguna memasukkan username dan password lalu menekan tombol Masuk
Then    Sistem menerbitkan token sesi, mencatat waktu login di audit log,
        dan menampilkan dashboard sesuai peran dalam ≤ 2 detik
```

### 3.2 Kredensial Salah

```
Given   Pengguna memasukkan username atau password yang salah
When    Menekan tombol Masuk
Then    Sistem menampilkan "Username atau password tidak valid"
        tanpa menyebutkan field mana yang salah; fail counter bertambah +1
```

### 3.3 Akun Terkunci (Brute Force)

```
Given   Sudah 4 percobaan login gagal dalam 15 menit terakhir
When    Percobaan ke-5 gagal
Then    Akun dikunci 15 menit; notifikasi email dikirim ke Admin;
        pesan "Akun terkunci. Coba lagi dalam 15 menit."
```

### 3.4 Login Pertama (Password Sementara)

```
Given   Pengguna baru dengan password sementara yang belum expired
When    Login berhasil dengan password sementara
Then    Sistem redirect ke halaman wajib ubah password;
        password baru disimpan; flag first_login direset; audit log dicatat
```

### 3.5 Password Sementara Kedaluwarsa

```
Given   Password sementara sudah > 7 hari sejak penerbitan
When    Pengguna mencoba login
Then    Pesan "Link telah kedaluwarsa. Hubungi Admin untuk mengirim ulang."
```

## 4. Tahapan Alur (Flow Steps)

### 4.1 Normal Flow

1. Buka URL sistem
2. Masukkan username
3. Masukkan password
4. Validasi kredensial ≤ 2 detik
5. Terbitkan JWT sesuai peran
6. Catat login di audit log
7. Tampilkan dashboard sesuai peran

### 4.2 Jalur Alternatif — Kredensial Salah

1. Input username/password salah
2. Increment fail counter
3. Tampilkan "Kredensial tidak valid"
4. Ulangi login (maks. 5x)

### 4.3 Jalur Alternatif — Percobaan ke-5 Gagal

1. Percobaan login ke-5 gagal
2. Kunci akun 15 menit
3. Kirim notifikasi email ke Admin
4. Tampilkan pesan waktu tunggu

### 4.4 Jalur Alternatif — Akun Nonaktif

1. Login dengan akun nonaktif
2. Tampilkan "Akun tidak aktif. Hubungi Admin"
3. Hentikan proses, tidak terbitkan JWT

### 4.5 Jalur Alternatif — Login Pertama

1. Pengguna baru login dengan password sementara
2. Sistem deteksi flag `first_login`
3. Redirect ke halaman wajib ubah password
4. Password baru disimpan, flag direset, audit log dicatat

## 5. Penanganan Error

| Kondisi | Penanganan |
|---|---|
| Kredensial salah | "Username atau password tidak valid" (tanpa spesifik) |
| Percobaan ke-5 gagal | Kunci 15 menit, notifikasi email ke Admin |
| Akun nonaktif | "Akun Anda tidak aktif. Hubungi Admin BMD Musik." |
| Password kedaluwarsa | "Link telah kedaluwarsa. Hubungi Admin untuk mengirim ulang." |
| Layanan token gangguan | Pesan pemeliharaan; catat insiden di log sistem |
