---
fr:
 - FR-03
 - FR-04
 - FR-15
uc:
 - UC-05
 - UC-07
 - UC-08
status: Approved
module: penagihan
title: "Penagihan Otomatis, Denda, dan Verifikasi Pembayaran"
version: "2.0"
date: "2026-06-27"
---

# FR-03, FR-04, FR-15 — Penagihan

## 1. Ringkasan

| Field | Value |
|---|---|
| Kode | FR-03, FR-04, FR-15 |
| Status | **Approved** |
| Aktor | Admin Operasional, Guardian |
| Modul | Penagihan |
| UC Linked | UC-05, UC-07, UC-08 |
| Verifikasi | Test |

---

## 2. FR-03 — Penerbitan Tagihan Otomatis dan Kode Unik

### 2.1 Spesifikasi

Sistem shall menerbitkan tagihan secara otomatis setiap tanggal 5 pukul 00:00 untuk semua Guardian yang memiliki student aktif. Setiap tagihan dilengkapi kode unik 3 digit untuk verifikasi pembayaran.

### 2.2 Aturan Tagihan

| Aspek | Detail |
|---|---|
| Penerbitan | Otomatis tanggal **5 pukul 00:00** |
| Scope | **1 invoice per Guardian** (1:N student digabung) |
| Kode Unik | 3 digit (000–999), di-generate otomatis |
| Collision | Max **3 retry** generate → status "Perlu Kode Manual", notifikasi ke Admin |
| DP | Guardian boleh bayar **≥ 70%** sebagai DP sampai tanggal 10 |

### 2.3 Kode Unik

| Rule | Detail |
|---|---|
| Format | 3 digit numerik (000–999) |
| Generasi | Otomatis, random |
| Collision handling | Max 3 retries → manual assignment by Admin |
| Idempotency | Unique constraint: `invoice_id + unique_code` |
| Tujuan | Verifikasi kecocokan saat pembayaran masuk |

### 2.4 Notifikasi

- Tagihan dikirim via WhatsApp
- Jika WhatsApp gagal → fallback ke **email**
- Admin diberitahu via dashboard jika pengiriman gagal

---

## 3. FR-04 — Penerapan Denda Otomatis dan Pembekuan Akun

### 3.1 Spesifikasi

Sistem shall menerapkan denda otomatis dan membekukan akses Guardian yang belum melunasi tagihan setelah batas waktu.

### 3.2 Timeline Denda & Freeze

| Tanggal | Trigger | Aksi |
|---|---|---|
| **Tanggal 5** 00:00 | Invoice diterbitkan | Status: `unpaid` |
| **Tanggal 10** 23:59 | Deadline DP / pelunasan | Jika < 70% → tetap `unpaid` |
| **Tanggal 11** 00:00 | Tagihan belum lunas | Denda **Rp 100.000** + status → `overdue` + **akun dibekukan** |

### 3.3 State Machine

```
unpaid → partial_paid → overdue → frozen

Detail:
- unpaid : Invoice diterbitkan, belum ada pembayaran
- partial_paid : Sudah bayar sebagian (≥ 1 pembayaran, belum lunas)
- overdue : Lewat tanggal 10, belum lunas → denda diterapkan
- frozen : Lewat tanggal 11 → akses dibekukan
```

### 3.4 Pembekuan

| Aspek | Detail |
|---|---|
| Trigger | Tanggal 11 pukul 00:00, tagihan belum lunas |
| Dampak | Guardian tidak bisa akses booking, jadwal student di-freeze |
| Unfreeze | **Hanya Admin** setelah verifikasi pembayaran **lunas** |

---

## 4. FR-15 — Verifikasi Pembayaran Manual oleh Admin

### 4.1 Spesifikasi

Sistem shall menyediakan antarmuka bagi Admin untuk memverifikasi bukti transfer yang dikirim oleh Guardian, dengan pencocokan kode unik.

### 4.2 Proses Verifikasi

1. Guardian kirim bukti transfer ke Admin (WhatsApp / upload)
2. Admin buka dashboard verifikasi pembayaran
3. Admin cocokkan **kode unik 3 digit** dari bukti transfer dengan invoice
4. Jika cocok → Admin approve → status invoice berubah
5. Jika tidak cocok → Admin tolak / minta klarifikasi

### 4.3 Idempotency

| Rule | Detail |
|---|---|
| Constraint | Unique: `(invoice_id, unique_code)` |
| Tujuan | Mencegah double-apply pembayaran yang sama |
| Duplicate detection | Jika kode unik sudah dipakai → alert Admin "Pembayaran sudah diverifikasi" |

---

## 5. Kriteria Penerimaan (Given-When-Then)

### 5.1 Invoice Otomatis Terbit

```
Given Tanggal 5 pukul 00:00 dan terdapat Guardian dengan student aktif
When Cron job penerbitan berjalan
Then Invoice diterbitkan untuk setiap Guardian dengan kode unik 3 digit;
 notifikasi terkirim via WhatsApp/email ≤ 15 detik
```

### 5.2 Kode Unik Collision

```
Given Sistem sudah 3x generate kode unik yang bentrok
When Retry ke-3 gagal
Then Invoice ditandai "Perlu Kode Manual"; Admin menerima notifikasi
```

### 5.3 Denda Otomatis

```
Given Tanggal 11 pukul 00:00 dan tagihan belum lunas
When Cron job denda berjalan
Then Denda Rp 100.000 ditambahkan; status → overdue; akun Guardian dibekukan;
 notifikasi dikirim
```

### 5.4 Unfreeze oleh Admin

```
Given Guardian sudah melunasi semua tagihan + denda
When Admin verifikasi pembayaran dan klik Unfreeze
Then Status → lunas; akun Guardian aktif kembali; audit log dicatat
```

### 5.5 Verifikasi Pembayaran

```
Given Admin menerima bukti transfer dengan kode unik 234
When Admin input kode 234 dan cocokkan dengan invoice
Then Invoice status berubah sesuai jumlah bayar; audit log dicatat
```

### 5.6 Duplicate Payment

```
Given Kode unik sudah pernah diverifikasi sebelumnya
When Admin mencoba verifikasi kode yang sama
Then Alert "Pembayaran sudah diverifikasi sebelumnya"; tidak ada perubahan status
```

---

## 6. Tahapan Alur (Flow Steps)

### 6.1 Normal Flow — Siklus Tagihan

1. Tanggal 5, 00:00: cron terbitkan invoice + kode unik
2. Notifikasi WA/email ke Guardian
3. Guardian bayar (full atau DP ≥ 70%)
4. Guardian kirim bukti transfer
5. Admin verifikasi kode unik → approve
6. Status berubah (`partial_paid` atau lunas)

### 6.2 Jalur Alternatif — Deadline Lewat

1. Tanggal 10: deadline DP/lewat
2. Tanggal 11, 00:00: denda Rp 100k + freeze
3. Notifikasi ke Guardian
4. Guardian bayar lunas + denda
5. Admin verifikasi → unfreeze

### 6.3 Jalur Alternatif — WA Gagal

1. Notifikasi WA gagal terkirim
2. Sistem auto-retry via email
3. Admin diberitahu via dashboard jika email juga gagal

---

## 7. Penanganan Error

| Kondisi | Penanganan |
|---|---|
| Kode unik collision 3x | Status "Perlu Kode Manual"; Admin notifikasi |
| Penagihan gagal parsial | Catat tagihan gagal; retry 30 menit; Admin alert jika masih gagal |
| WA gagal kirim | Fallback ke email otomatis |
| Unfreeze tanpa verifikasi penuh | Diblokir oleh sistem; Admin harus verifikasi lunas |
| Duplicate payment | Alert "sudah diverifikasi"; tidak ada perubahan |
| Cron job gagal | Retry 3x; alert Admin; catat di log sistem |
