---
fr: FR-14
uc: UC-06
status: Proposed
module: Reservasi Studio
---

# SRS-08: Reservasi Studio

## 1. Deskripsi Modul

Modul reservasi studio menyediakan pemesanan studio untuk dua keperluan: latihan (rehearsal) dan rekaman (recording). Masing-masing memiliki alur dan aturan berbeda.

## 2. Tipe Reservasi

### 2.1 Rehearsal (Latihan)

| Aspek | Detail |
|-------|--------|
| Booking | Self-booking oleh Guardian via dashboard |
| Jam operasional | 09:00 — 21:00 WIB |
| Conflict check | Real-time saat proses booking |
| Durasi minimum | 1 jam |
| Durasi maksimum | 4 jam per sesi |
| Booking window | Maksimal 14 hari ke depan |

### 2.2 Recording (Rekaman)

| Aspek | Detail |
|-------|--------|
| Booking | Memerlukan konsultasi + persetujuan Admin |
| Alur | Guardian ajukan jadwal → Admin konsultasi → Admin setujui/tolak |
| Jam operasional | 09:00 — 21:00 WIB |
| Durasi | Ditentukan saat konsultasi dengan Admin |
| Booking window | Maksimal 30 hari ke depan |

## 3. Tarif Studio

- Tarif studio dikelola sebagai **master data**.
- Hanya **Admin** yang dapat mengubah tarif.
- Tarif berlaku per jam, per studio.
- Perubahan tarif tercatat di audit log.
- Tarif yang berlaku adalah tarif saat booking dikonfirmasi, bukan saat sesi berlangsung.

### 3.1 Struktur Tarif

| Field | Tipe | Keterangan |
|-------|------|------------|
| `studio_id` | FK | ID studio |
| `rate_per_hour` | DECIMAL | Tarif per jam (IDR) |
| `rate_type` | ENUM | `REHEARSAL`, `RECORDING` |
| `effective_date` | DATE | Tanggal mulai berlaku |
| `updated_by` | FK | Admin yang mengubah |
| `updated_at` | TIMESTAMP | Waktu perubahan |

## 4. Manajemen Konflik & Concurrency

### 4.1 Real-Time Conflict Check

- Saat Guardian memilih slot waktu, sistem melakukan pengecekan real-time.
- Jika slot sudah ditempati, tampilkan pesan: "Slot sudah dipesan" beserta **alternatif terdekat**.

### 4.2 Alternatif Terdekat

Sistem menampilkan hingga 3 alternatif:

| Prioritas | Kriteria Alternatif |
|-----------|---------------------|
| 1 | Studio berbeda, jam sama |
| 2 | Studio sama, jam berdekatan (±1 jam) |
| 3 | Studio berbeda, jam berdekatan (±1 jam) |

### 4.3 Race Condition Handling

- Gunakan optimistic locking (version field) atau database-level `SELECT ... FOR UPDATE`.
- Jika dua user memperebutkan slot yang sama, yang pertama commit menang.
- User yang kalah mendapat notifikasi: "Slot baru saja dipesan oleh pengguna lain" + daftar alternatif.

## 5. Constraint

- Studio tidak dapat dipesan di luar jam operasional (09:00—21:00).
- Booking rehearsal yang sudah dikonfirmasi dapat dibatalkan H-1 sebelum sesi.
- Booking recording yang sudah disetujui oleh Admin tidak dapat dibatalkan tanpa persetujuan Admin.
- Semua transaksi booking tercatat di audit log.
- Double-booking pada slot yang sama TIDAK BOLEH terjadi (database constraint unique pada `studio_id` + `slot_start` + `slot_end`).
