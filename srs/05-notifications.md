---
fr: FR-06
uc: UC-04
status: Approved
module: Notifikasi Multi-Channel
---

# SRS-05: Notifikasi Multi-Channel

## 1. Deskripsi Modul

Sistem notifikasi multi-channel yang mengirim pesan melalui WhatsApp dan email kepada Wali Murid. Notifikasi dikategorikan berdasarkan urgensi dengan strategi channel prioritas berbeda.

## 2. Kategori Notifikasi

### 2.1 Kritis (Pembayaran, Pembekuan)

| Aspek | Detail |
|-------|--------|
| Channel utama | Email |
| Channel cadangan | WhatsApp |
| Trigger | Invoice jatuh tempo, pembekuan akun, pembatalan kelas |

### 2.2 Non-Kritis (Pengingat 1 Jam)

| Aspek | Detail |
|-------|--------|
| Channel utama | WhatsApp |
| Channel cadangan | Email |
| Trigger | Pengingat jadwal kelas 1 jam sebelum sesi |

## 3. Strategi Pengiriman

### 3.1 Retry Policy

- Maksimal 3 kali retry per channel sebelum fallback.
- Interval retry: 10 detik → 20 detik → 40 detik (exponential backoff).
- Setelah 3 kali gagal di channel utama, otomatis fallback ke channel cadangan.
- Channel cadangan mengikuti retry policy yang sama.

### 3.2 Circuit Breaker

- Jika WhatsApp mengalami >5 kegagalan berturut-turut, nonaktifkan sementara.
- Sistem mencatat waktu mulai nonaktif.
- Auto-recovery: coba kirim ulang setelah cooldown period (default 30 menit).
- Admin dapat memaksa reset circuit breaker via dashboard.

## 4. Penerima Notifikasi

- Semua notifikasi dikirim ke **Wali Murid**, bukan per siswa.
- Jika 1 Wali Murid memiliki N siswa, notifikasi digabungkan menjadi 1 pesan.
- Format pesan mencantumkan nama semua siswa terkait.

## 5. Admin Dashboard — Panel Toggle

| Fitur | Detail |
|-------|--------|
| Lokasi | Admin Dashboard → Pengaturan Notifikasi |
| Fungsi | Toggle on/off per channel (WhatsApp, Email) |
| Granularitas | Per kategori notifikasi (Kritis, Non-Kritis) |
| Akses | Hanya Admin |

## 6. Delivery Status Logging

Setiap notifikasi dicatat dengan field berikut:

| Field | Tipe | Keterangan |
|-------|------|------------|
| `notification_id` | UUID | ID unik notifikasi |
| `wali_murid_id` | FK | Penerima |
| `category` | ENUM | `CRITICAL`, `NON_CRITICAL` |
| `channel` | ENUM | `WHATSAPP`, `EMAIL` |
| `status` | ENUM | `PENDING`, `SENT`, `DELIVERED`, `FAILED`, `FALLBACK` |
| `retry_count` | INT | Jumlah retry yang dilakukan |
| `error_message` | TEXT | Pesan error jika gagal (nullable) |
| `sent_at` | TIMESTAMP | Waktu pengiriman berhasil |
| `created_at` | TIMESTAMP | Waktu notifikasi dibuat |

## 7. Constraint

- Notifikasi kritis TIDAK BOLEH gagal total — jika kedua channel gagal, log sebagai `CRITICAL_FAILURE` dan trigger alert ke Admin.
- Rate limit WhatsApp: sesuai ketentuan provider (max 100 pesan/detik default).
- Semua notifikasi harus tercatat di audit log (lihat SRS-07).
