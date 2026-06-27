---
fr: FR-05
uc: UC-03
status: Approved
module: kehadiran
---

# SRS-04: Validasi Kehadiran Guru

## 1. Deskripsi Modul

Sistem absensi guru dengan validasi berlapis (GPS, waktu server, verifikasi wajah) untuk memastikan kehadiran fisik guru di studio BMD Music. Setiap submission menghasilkan foto bukti yang diverifikasi secara lokal (client-side) dan redundan di server. Gagal validasi tidak langsung menolak — status menjadi `pending_verification` untuk review Admin.

## 2. Validasi Kumulatif

Validasi berjalan secara **kumulatif** — semua tiga syarat harus lolos untuk status `verified`.

### 2.1 Validasi GPS (Radius 150m)

| Parameter | Nilai |
|-----------|-------|
| Titik referensi studio | Latitude: `-7.7265`, Longitude: `110.4162` |
| Radius maksimum | 150 meter |
| Algoritma | Haversine formula |
| Sumber koordinat | Browser Geolocation API (`enableHighAccuracy: true`) |
| Tolerance | GPS jitter ≤15m dikompensasi (effective radius 165m) |

**Rule**: Jarak guru dari titik referensi > 150m → `pending_verification`.

### 2.2 Validasi Waktu Server

| Parameter | Nilai |
|-----------|-------|
| Sumber waktu | Server timestamp (bukan client clock) |
| Toleransi keterlambatan | +15 menit dari jadwal sesi mulai |
| Toleransi kedatangan awal | -30 menit dari jadwal sesi mulai |
| Timezone | WIB (UTC+7), hardcoded server-side |

**Rule**: Submission di luar window [-30min, +15min] dari `session.start_time` → `pending_verification`.

### 2.3 Validasi Wajah (Face Detection)

| Parameter | Nilai |
|-----------|-------|
| Minimum face area | ≥30% dari frame foto |
| Detection engine (client) | face-api.js atau MediaPipe Face Detection |
| Performa target | ≤1 detik untuk deteksi di perangkat klien |
| Server re-check | Redundant face detection di server (tidak bergantung client result) |
| Format foto | JPEG, ≤2MB, minimal 640×480px |

**Rule**: Face area < 30% frame atau wajah tidak terdeteksi → `pending_verification`.

## 3. Alur Client-Side Validation

```
┌─────────────────────┐
│  Guru buka form     │
│  absensi            │
└──────────┬──────────┘
           │
     ┌─────▼─────┐
     │ Ambil GPS  │──── Gagal ──→ Tampilkan error, minta aktifkan lokasi
     └─────┬─────┘
           │ OK
     ┌─────▼──────────┐
     │ Ambil foto via  │
     │ kamera          │
     └─────┬──────────┘
           │
     ┌─────▼───────────────┐
     │ face-api.js/MediaPipe│
     │ detect face ≥30%?    │
     └─────┬───────────────┘
           │ Yes
     ┌─────▼──────────┐
     │ Kirim ke server │
     │ (GPS + foto +   │
     │  timestamp)     │
     └─────┬──────────┘
           │
     ┌─────▼──────────────┐
     │ Server validates:   │
     │ 1. GPS distance     │
     │ 2. Time window      │
     │ 3. Face re-check    │
     └─────┬──────────────┘
           │
     ┌─────▼──────────┐
     │ Semua lolos?    │
     └──┬──────────┬──┘
        │ Yes      │ No
   ┌────▼───┐ ┌───▼──────────────┐
   │verified│ │pending_verification│
   └────────┘ └────────────────────┘
```

## 4. Status Kehadiran

### 4.1 State Machine

```
pending_verification ──→ verified      (Admin approve)
pending_verification ──→ rejected      (Admin reject, ≤2h SLA)
```

| Status | Keterangan |
|--------|------------|
| `verified` | Semua validasi lolos atau Admin approve |
| `pending_verification` | Minimal 1 validasi gagal — menunggu review Admin |
| `rejected` | Admin menolak setelah review |

### 4.2 Admin Review SLA

- Admin harus mereview `pending_verification` dalam **≤2 jam**.
- Sistem kirim notifikasi ke Admin saat item baru masuk queue.
- Reminder otomatis jika >1 jam belum direview.
- Jika >2 jam, eskalasi ke Owner.

## 5. Constraint Guru

| Constraint | Detail |
|------------|--------|
| Edit setelah submit | **TIDAK BOLEH** — data absensi immutable setelah guru menekan submit |
| Submit ulang | Tidak bisa — satu sesi = satu submission |
| Bukti foto | Disimpan sebagai `photo_url` (KRITIS, dienkripsi AES-256-GCM) |

## 6. Mode Offline

| Aspek | Detail |
|-------|--------|
| Penyimpanan lokal | IndexedDB / localStorage (encrypted) |
| Data tersimpan | GPS coords, foto (base64), timestamp client, session_id |
| Auto-upload | Ketika koneksi pulih, otomatis upload dalam **≤5 menit** |
| Conflict handling | Jika sesi sudah ada attendance record → log duplicate, skip upload |
| Notification | Guru diberi notif "Absensi tersimpan offline, akan dikirim otomatis" |

### 6.1 Offline Flow

```
┌────────────────┐
│ Submit absensi │
└───────┬────────┘
        │
  ┌─────▼───────┐
  │ Online?      │
  └──┬────────┬──┘
     │ Yes    │ No
     │   ┌────▼──────────────┐
     │   │ Save to IndexedDB │
     │   │ + base64 foto     │
     │   │ Show toast:       │
     │   │ "Tersimpan offline"│
     │   └────┬──────────────┘
     │        │
     │   ┌────▼──────────────┐
     │   │ Service Worker    │
     │   │ background sync   │
     │   │ ≤5 menit          │
     │   └────┬──────────────┘
     │        │
     └────┬───┘
          ▼
   Normal server validation
```

## 7. Given-When-Then Scenarios

### Scenario 7.1: Absensi Berhasil (Semua Validasi Lolos)

```gherkin
Given guru berada dalam radius 150m dari studio (-7.7265, 110.4162)
And waktu saat ini dalam window [-30min, +15min] dari jadwal sesi
And guru mengambil foto dengan wajah terdeteksi ≥30% frame
When guru menekan tombol "Submit Absensi"
Then sistem menyimpan data dengan status "verified"
And foto disimpan terenkripsi di storage
And audit log mencatat attendance verified
```

### Scenario 7.2: GPS Di Luar Radius

```gherkin
Given guru berada 200m dari studio (-7.7265, 110.4162)
And waktu dan foto memenuhi syarat
When guru menekan tombol "Submit Absensi"
Then sistem menyimpan data dengan status "pending_verification"
And notifikasi dikirim ke Admin
And guru melihat pesan "Absensi menunggu verifikasi admin"
```

### Scenario 7.3: Face Tidak Terdeteksi

```gherkin
Given guru berada dalam radius 150m dari studio
And waktu memenuhi syarat
And foto yang diambil tidak memiliki wajah yang terdeteksi ≥30% frame
When guru menekan tombol "Submit Absensi"
Then sistem menolak submission di sisi klien
And menampilkan pesan "Wajah tidak terdeteksi. Pastikan wajah terlihat jelas."
And data TIDAK dikirim ke server
```

### Scenario 7.4: Mode Offline

```gherkin
Given guru berada di studio tanpa koneksi internet
And GPS, waktu, dan foto memenuhi syarat
When guru menekan tombol "Submit Absensi"
Then data tersimpan di IndexedDB (terenkripsi)
And guru melihat notifikasi "Tersimpan offline"
When koneksi internet pulih dalam 5 menit
Then sistem otomatis mengupload data ke server
And server memproses validasi normal
```

### Scenario 7.5: Admin Review

```gherkin
Given terdapat absensi dengan status "pending_verification"
And admin login ke dashboard
When admin membuka detail absensi
Then admin melihat foto, GPS coords, dan alasan pending
When admin menekan "Approve"
Then status berubah menjadi "verified"
And guru menerima notifikasi konfirmasi
```

## 8. Error Handling

| Error | Penanganan |
|-------|------------|
| GPS tidak tersedia | Tampilkan pesan: "Aktifkan lokasi untuk absensi". Block submission. |
| GPS timeout (>10s) | Retry 1x. Jika tetap gagal → `pending_verification` dengan flag `gps_unavailable`. |
| Kamera tidak tersedia | Tampilkan pesan: "Kamera diperlukan untuk absensi". Block submission. |
| Foto corrupt/too large | Validasi di client sebelum kirim. Tolak jika >2MB atau corrupt. |
| Face detection timeout (>1s) | Kirim ke server tanpa client face result. Server lakukan full check. |
| Server unreachable | Simpan ke IndexedDB, auto-upload saat online (lihat §6). |
| Server error (5xx) | Retry 3x exponential backoff. Jika tetap gagal → simpan offline. |
| Duplicate submission | Server cek: jika attendance untuk session_id sudah ada → HTTP 409, log. |
| Session tidak ditemukan | HTTP 404, tampilkan "Sesi tidak valid" ke guru. |
| Waktu server mismatch | Gunakan server time. Jika client-server clock skew >5min → log warning. |

## 9. Data Model — Attendance Record

| Field | Tipe | Keterangan |
|-------|------|------------|
| `attendance_id` | UUID | PK |
| `session_id` | UUID FK | Relasi ke Session |
| `teacher_id` | UUID FK | Relasi ke Teacher |
| `photo_url` | TEXT | KRITIS — dienkripsi AES-256-GCM |
| `gps_lat` | DECIMAL(10,7) | Latitude guru saat submit |
| `gps_lng` | DECIMAL(10,7) | Longitude guru saat submit |
| `gps_distance_m` | FLOAT | Jarak dari titik studio (computed) |
| `face_valid` | BOOLEAN | Hasil face detection server-side |
| `face_confidence` | FLOAT | Confidence score face detection |
| `client_timestamp` | TIMESTAMP | Waktu dari device guru |
| `server_timestamp` | TIMESTAMP | Waktu server saat terima (authoritative) |
| `verification_status` | ENUM | `verified`, `pending_review`, `rejected` |
| `rejection_reason` | TEXT | Alasan reject (nullable) |
| `reviewed_by` | UUID FK | Admin yang review (nullable) |
| `reviewed_at` | TIMESTAMP | Waktu review (nullable) |
| `is_offline_submission` | BOOLEAN | True jika awalnya disimpan offline |
| `created_at` | TIMESTAMP | Auto |
| `updated_at` | TIMESTAMP | Auto |

## 10. API Endpoint

| Method | Path | Deskripsi |
|--------|------|-----------|
| `POST` | `/api/v1/attendance` | Submit absensi (GPS + foto + session_id) |
| `GET` | `/api/v1/attendance/:id` | Detail absensi |
| `GET` | `/api/v1/attendance?status=pending_verification` | List pending (Admin) |
| `PATCH` | `/api/v1/attendance/:id/review` | Admin approve/reject |

## 11. Security

- Foto disimpan terenkripsi AES-256-GCM (field KRITIS).
- GPS coords disimpan plaintext (non-PII).
- Endpoint submit memerlukan JWT token dengan role `teacher`.
- Endpoint review memerlukan JWT token dengan role `admin` atau `owner`.
- Rate limit: max 5 submission per teacher per hari.
- Semua aksi tercatat di AuditLog.
