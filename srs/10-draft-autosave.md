---
id: SRS-FR-08
title: "Penyimpanan Draft Otomatis dan Pemulihan Sesi"
status: Proposed
module: Sistem
uc_linked: UC-13
priority: Medium
complexity: 2
risk: 1
effort: 0.33
source: "BAB 3 §3.1 FR-08, BAB 4 §UC-13"
last_updated: 2026-06-27
---

# FR-08 — Penyimpanan Draft Otomatis dan Pemulihan Sesi

## 1. Ringkasan

Sistem shall menyimpan draft input pengguna (jadwal, tagihan, formulir)
secara otomatis setiap 30 detik. Jika sesi berakhir karena batas waktu
tidak aktif, draft dipulihkan otomatis setelah pengguna login kembali.
Kehilangan data maksimal 1 menit.

## 2. Aktor

| Aktor | Peran |
|-------|-------|
| Admin | Mengisi formulir jadwal, tagihan |
| Instructor | Mengisi formulir terkait jadwal mengajar |

## 3. Spesifikasi Perilaku

### 3.1 Auto-save

- Draft disimpan ke **local storage** perangkat setiap **30 detik**.
- Setiap simpan lokal juga memicu **sinkronisasi ke server** (async).
- Frekuensi: ≤30 detik → **data loss ≤1 menit** pada skenario terburuk.

### 3.2 Pemulihan Sesi

- Setelah sesi timeout dan pengguna login kembali, sistem **mendeteksi
 draft tersimpan** di local storage.
- Prompt: **"Pulihkan draft?"** ditampilkan.
- Jika Ya → form terisi kembali dengan data draft.
- Jika Tidak → draft dihapus dari local storage.

### 3.3 Penanganan Local Storage Penuh

- Jika local storage penuh, sistem menampilkan **peringatan**:
 *"Penyimpanan lokal penuh. Simpan pekerjaan Anda segera."*
- Auto-save lokal dinonaktifkan; hanya server sync yang berjalan
 (jika koneksi tersedia).

### 3.4 Penanganan Server Sync Gagal

- Jika sinkronisasi ke server gagal, indikator **"Belum Tersinkronisasi"**
 ditampilkan di antarmuka.
- Retry otomatis hingga 3 kali dengan exponential backoff.
- Draft tetap tersimpan di local storage sebagai cadangan.

### 3.5 Simpan Permanen

- Setelah pengguna menekan tombol **"Simpan"**, data tersimpan permanen
 ke database.
- Data permanen **tidak dapat dikembalikan ke status draft**.
- Draft dihapus dari local storage dan server draft table.

## 4. Batasan Logika

- Draft hanya berlaku untuk form yang belum disimpan permanen.
- Draft tidak menggantikan validasi server-side.
- Setiap draft memiliki `draft_id` unik dan `updated_at` timestamp.
- TTL draft di server: 7 hari sejak update terakhir.
- Draft yang melewati TTL dihapus otomatis (cron job).

## 5. Kriteria Penerimaan (Given-When-Then)

| # | Given | When | Then |
|---|-------|------|------|
| 1 | Admin sedang mengisi formulir jadwal >30 detik | 30 detik berlalu sejak perubahan terakhir | Draft tersimpan di local storage dan dikirim ke server |
| 2 | Sesi Admin timeout karena tidak aktif | Admin login kembali | Prompt "Pulihkan draft?" muncul; form terisi jika Ya |
| 3 | Local storage perangkat penuh | Auto-save trigger | Peringatan "Penyimpanan lokal penuh" ditampilkan |
| 4 | Server tidak dapat dijangkau | Auto-save trigger | Indikator "Belum Tersinkronisasi" muncul |
| 5 | Admin menekan tombol "Simpan" | Data valid | Data tersimpan permanen; draft dihapus; tidak bisa revert ke draft |

## 6. Tahapan Alur (Flow Steps)

### Normal Flow

1. Pengguna membuka form (jadwal/tagihan/formulir)
2. Pengguna mengisi data
3. Setiap perubahan → debounce 30 detik dimulai
4. Timer 30 detik habis → save ke local storage
5. Kirim draft ke server (async)
6. Jika server berhasil → status "Tersinkronisasi"
7. Jika server gagal → status "Belum Tersinkronisasi", retry 3x
8. Pengguna menekan "Simpan" → data disimpan permanen
9. Draft dihapus dari local storage dan server

### Jalur Alternatif — Sesi Timeout

1. Sesi timeout (idle > batas per peran)
2. Pengguna diarahkan ke halaman login
3. Login berhasil
4. Sistem cek local storage untuk draft
5. Jika ada draft → tampilkan prompt "Pulihkan draft?"
6. Ya → form diisi; Tidak → draft dihapus

### Jalur Error — Local Storage Penuh

1. Auto-save trigger
2. Gagal tulis ke local storage (quota exceeded)
3. Tampilkan peringatan "Penyimpanan lokal penuh"
4. Tetap coba server sync jika online

## 7. Error Handling

| Error | Penanganan |
|-------|------------|
| LocalStorage quota exceeded | Warning UI; server-only sync |
| Server unreachable | "Belum Tersinkronisasi" indicator; retry 3x |
| Draft corrupt di local storage | Hapus draft; log error; mulai baru |
| Conflict draft lokal vs server | Bandingkan `updated_at`; gunakan yang terbaru; konfirmasi ke user |

## 8. Tautan ke UC

- **UC-13**: Restore Draft Setelah Login Ulang (extends UC-01)
