---
id: SRS-ENTITIES
title: "Kamus Data & Model Entitas"
module: data-model
source: "BAB 5 §5.1–5.13"
last_updated: 2026-06-27
---

# Data Model — 11 Entitas Sistem Equalizer-Stake

> Source: BAB 5, Tabel 5.1–5.13. Semua entitas menggunakan soft-delete
> (`deleted_at TIMESTAMP NULL`) untuk data ber-PII. Relasi menggunakan
> `ON DELETE RESTRICT` kecuali stated otherwise.

---

## 1. User (Tabel 5.3)

Tabel induk akun sistem untuk semua peran.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_user` | UUID | PK, DEFAULT gen_random_uuid() | Primary key |
| `username` | VARCHAR(50) | UNIQUE, NOT NULL | Login identifier |
| `email` | VARCHAR(255) | UNIQUE, NOT NULL | Email akun |
| `password_hash` | VARCHAR(255) | NOT NULL | bcrypt (cost factor ≥12) |
| `role` | ENUM | NOT NULL | `owner`, `admin`, `teacher`, `guardian` |
| `is_active` | BOOLEAN | DEFAULT true | Status aktif akun |
| `last_login_at` | TIMESTAMP | NULL | Waktu login terakhir |
| `refresh_token` | TEXT | NULL | JWT refresh token (hashed) |
| `token_expires_at` | TIMESTAMP | NULL | Expiry refresh token |
| `failed_login_attempts` | INT | DEFAULT 0 | Counter brute-force |
| `locked_until` | TIMESTAMP | NULL | Lock sementara setelah 5 gagal |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |
| `deleted_at` | TIMESTAMP | NULL | Soft delete |

**Constraint**: `password_hash` tidak boleh plaintext. bcrypt cost factor ≥12.

---

## 2. Guardian (Tabel 5.4)

Profil Wali Murid. Relasi 1:N ke Student (satu guardian bisa punya banyak siswa).

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_guardian` | UUID | PK | |
| `id_user` | UUID | FK → User(id_user), UNIQUE, NOT NULL | 1:1 dengan User |
| `full_name` | TEXT | NOT NULL | **KRITIS** — AES-256-GCM encrypted |
| `phone_whatsapp` | VARCHAR(20) | UNIQUE, NOT NULL | Format: `+62xxxxxxxxxx` |
| `email` | VARCHAR(255) | NULL | Email kontak (bisa beda dari User.email) |
| `address` | TEXT | NULL | Alamat (boleh encrypted) |
| `account_status` | ENUM | NOT NULL, DEFAULT `active` | `active`, `frozen`, `inactive`, `pending_deletion` |
| `frozen_at` | TIMESTAMP | NULL | Kapan dibekukan |
| `frozen_reason` | TEXT | NULL | Alasan pembekuan |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |
| `deleted_at` | TIMESTAMP | NULL | Soft delete |

**Constraint**:
- `phone_whatsapp` harus match regex `^\+62\d{9,13}$`.
- `full_name` KRITIS — wajib dienkripsi AES-256-GCM sebelum insert/update.
- `account_status = 'frozen'` artinya tagihan belum dibayar (lihat SRS-02).

---

## 3. Student (Tabel 5.5)

Data siswa terdaftar. Setiap student terhubung ke satu Guardian.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_student` | UUID | PK | |
| `id_guardian` | UUID | FK → Guardian(id_guardian), NOT NULL | Relasi 1 Guardian : N Student |
| `full_name` | TEXT | NOT NULL | **KRITIS** — AES-256-GCM encrypted |
| `date_of_birth` | DATE | NULL | |
| `instrument` | ENUM | NOT NULL | `biola`, `piano`, `gitar`, `drum`, `keyboard`, `flute`, `saxophone`, `vokal`, `ensemble` |
| `level` | ENUM | NOT NULL, DEFAULT `pemula` | `pemula`, `dasar`, `menengah`, `lanjut` |
| `enrollment_status` | ENUM | NOT NULL, DEFAULT `active` | `active`, `cuti`, `inactive`, `trial` |
| `enrollment_date` | DATE | NOT NULL | Tanggal mulai |
| `trial_end_date` | DATE | NULL | Akhir masa trial (jika enrollment_status = trial) |
| `eligible_for_exam` | BOOLEAN | DEFAULT false | Kelayakan ujian (computed dari kehadiran) |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |
| `deleted_at` | TIMESTAMP | NULL | Soft delete |

**Constraint**:
- `full_name` KRITIS — enkripsi AES-256-GCM.
- `enrollment_status` transitions: `trial` → `active` (setelah bayar), `active` → `cuti` (ajukan), `cuti` → `active` (aktifkan kembali), `active`/`cuti` → `inactive` (keluar).

---

## 4. Teacher (Tabel 5.6)

Profil guru yang terhubung ke akun User.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_teacher` | UUID | PK | |
| `id_user` | UUID | FK → User(id_user), UNIQUE, NOT NULL | 1:1 dengan User |
| `full_name` | VARCHAR(255) | NOT NULL | Nama guru |
| `instruments_specialization` | JSONB | NOT NULL | Array instrumen, e.g. `["biola","piano"]` |
| `employment_type` | ENUM | NOT NULL | `tetap`, `freelance` |
| `max_students_per_session` | INT | DEFAULT 1 | Kapasitas per sesi |
| `is_active` | BOOLEAN | DEFAULT true | |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |
| `deleted_at` | TIMESTAMP | NULL | Soft delete |

**Constraint**:
- `instruments_specialization` harus JSON array valid, minimal 1 elemen.
- Hanya user dengan role `teacher` boleh punya record Teacher.

---

## 5. Room (Tabel 5.7)

Data ruangan/studio di BMD Music.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_room` | UUID | PK | |
| `room_name` | VARCHAR(100) | UNIQUE, NOT NULL | Nama ruangan |
| `room_type` | ENUM | NOT NULL | `kursus`, `rehearsal`, `recording` |
| `capacity` | INT | NOT NULL, DEFAULT 1 | Kapasitas siswa |
| `is_available` | BOOLEAN | DEFAULT true | Status ketersediaan |
| `location_notes` | TEXT | NULL | Catatan lokasi |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |

**Constraint**: Room tidak punya soft-delete. Hapus fisik hanya jika tidak ada session terkait (`ON DELETE RESTRICT`).

---

## 6. Session (Tabel 5.8)

Jadwal sesi kursus/rekaman.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_session` | UUID | PK | |
| `id_teacher` | UUID | FK → Teacher(id_teacher), NOT NULL | |
| `id_student` | UUID | FK → Student(id_student), NOT NULL | |
| `id_room` | UUID | FK → Room(id_room), NOT NULL | |
| `session_date` | DATE | NOT NULL | Tanggal sesi |
| `start_time` | TIME | NOT NULL | Jam mulai |
| `end_time` | TIME | NOT NULL | Jam selesai |
| `session_type` | ENUM | NOT NULL, DEFAULT `reguler` | `reguler`, `teori`, `rehearsal`, `recording`, `kompensasi` |
| `status` | ENUM | NOT NULL, DEFAULT `draft` | `draft`, `pending`, `locked`, `completed`, `cancelled`, `rescheduled` |
| `rescheduled_to` | UUID | FK → Session(id_session), NULL | Jika rescheduled |
| `cancellation_reason` | TEXT | NULL | |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |

### State Machine: Session

```
draft ──→ pending      (submit jadwal)
pending ──→ locked     (konfirmasi / H-1)
locked ──→ completed   (absensi guru tercatat)
locked ──→ cancelled   (batal)
locked ──→ rescheduled (jadwal ulang)
pending ──→ cancelled  (batal sebelum lock)
draft ──→ cancelled    (batal draft)
```

**Constraint**:
- `end_time` > `start_time`.
- Satu guru tidak boleh punya 2 session overlap di waktu sama.
- Satu room tidak boleh punya 2 session overlap di waktu sama.
- Satu siswa tidak boleh punya 2 session overlap di waktu sama.
- `status = 'locked'` tidak bisa diubah kecuali oleh Admin/Owner.

---

## 7. Invoice (Tabel 5.9)

Tagihan otomatis berdasarkan sesi.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_invoice` | UUID | PK | |
| `id_guardian` | UUID | FK → Guardian(id_guardian), NOT NULL | Tagihan ke Wali Murid |
| `unique_code` | CHAR(3) | NOT NULL, UNIQUE per invoice | 3 digit unik untuk verifikasi transfer |
| `billing_period` | VARCHAR(7) | NOT NULL | Format: `YYYY-MM` |
| `total_amount` | DECIMAL(12,2) | NOT NULL | Total tagihan |
| `due_date` | DATE | NOT NULL | Jatuh tempo |
| `status` | ENUM | NOT NULL, DEFAULT `draft` | `draft`, `issued`, `paid`, `overdue`, `cancelled` |
| `issued_at` | TIMESTAMP | NULL | Kapan diterbitkan |
| `paid_at` | TIMESTAMP | NULL | Kapan dibayar |
| `notes` | TEXT | NULL | |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |

### State Machine: Invoice

```
draft ──→ issued     (terbitkan tagihan)
issued ──→ paid      (pembayaran terverifikasi)
issued ──→ overdue   (lewat jatuh tempo)
issued ──→ cancelled (batal)
overdue ──→ paid     (bayar setelah jatuh tempo + denda)
draft ──→ cancelled  (batal draft)
```

**Constraint**:
- `unique_code` 3 digit, unik per billing period per guardian. Digunakan Wali Murid sebagai kode verifikasi transfer.
- `due_date` = tanggal 10 bulan berikutnya (default).
- `overdue` di-set otomatis oleh cron job harian jika `due_date < today AND status = 'issued'`.

---

## 8. Payment (Tabel 5.10)

Catatan pembayaran yang masuk.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_payment` | UUID | PK | |
| `id_invoice` | UUID | FK → Invoice(id_invoice), NOT NULL | |
| `amount` | DECIMAL(12,2) | NOT NULL | Jumlah bayar |
| `method` | ENUM | NOT NULL | `transfer`, `cash`, `dp` |
| `payment_date` | TIMESTAMP | NOT NULL | |
| `proof_url` | TEXT | NULL | URL bukti transfer (jika transfer) |
| `unique_code_used` | CHAR(3) | NOT NULL | Kode unik yang digunakan |
| `is_verified` | BOOLEAN | DEFAULT false | |
| `verified_by` | UUID | FK → User(id_user), NULL | Admin yang verifikasi |
| `verified_at` | TIMESTAMP | NULL | |
| `notes` | TEXT | NULL | |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |

**Constraint**:
- `unique_code_used` harus match `Invoice.unique_code`.
- Payment tidak boleh dihapus (audit trail). Soft-delete opsional.
- `verified_by` hanya boleh user dengan role `admin` atau `owner`.

---

## 9. Attendance (Tabel 5.11)

Catatan kehadiran guru per sesi.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_attendance` | UUID | PK | |
| `id_session` | UUID | FK → Session(id_session), UNIQUE, NOT NULL | 1 session = 1 attendance |
| `id_teacher` | UUID | FK → Teacher(id_teacher), NOT NULL | |
| `photo_url` | TEXT | NOT NULL | **KRITIS** — AES-256-GCM encrypted |
| `gps_lat` | DECIMAL(10,7) | NOT NULL | Latitude saat submit |
| `gps_lng` | DECIMAL(10,7) | NOT NULL | Longitude saat submit |
| `gps_distance_m` | FLOAT | NOT NULL | Jarak dari titik studio |
| `face_valid` | BOOLEAN | NOT NULL | Hasil face detection server |
| `face_confidence` | FLOAT | NULL | Confidence score (0.0–1.0) |
| `client_timestamp` | TIMESTAMP | NOT NULL | Waktu dari device guru |
| `server_timestamp` | TIMESTAMP | NOT NULL, DEFAULT NOW() | Waktu server (authoritative) |
| `verification_status` | ENUM | NOT NULL, DEFAULT `pending_review` | `verified`, `pending_review`, `rejected` |
| `rejection_reason` | TEXT | NULL | Alasan reject |
| `reviewed_by` | UUID | FK → User(id_user), NULL | Admin yang review |
| `reviewed_at` | TIMESTAMP | NULL | |
| `is_offline_submission` | BOOLEAN | DEFAULT false | |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |
| `updated_at` | TIMESTAMP | DEFAULT NOW() | |

### State Machine: Attendance

```
pending_review ──→ verified   (semua validasi lolos / admin approve)
pending_review ──→ rejected   (admin reject)
```

**Constraint**:
- `photo_url` KRITIS — enkripsi AES-256-GCM.
- GPS: distance dari (-7.7265, 110.4162) ≤150m untuk auto-verified.
- Face: area ≥30% frame untuk auto-verified.
- Jika salah satu gagal → `pending_review` (bukan reject otomatis).
- Guru TIDAK BOLEH edit setelah submit (immutable).
- Admin review SLA: ≤2 jam.

---

## 10. Notification (Tabel 5.12)

Log semua notifikasi yang dikirim sistem.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_notification` | UUID | PK | |
| `recipient_id` | UUID | FK → User(id_user), NOT NULL | Penerima |
| `channel` | ENUM | NOT NULL | `whatsapp`, `email`, `sms` |
| `category` | ENUM | NOT NULL | `critical`, `non_critical` |
| `subject` | VARCHAR(255) | NULL | Subjek (untuk email) |
| `body` | TEXT | NOT NULL | Isi pesan |
| `status` | ENUM | NOT NULL, DEFAULT `pending` | `pending`, `sent`, `failed`, `retrying` |
| `retry_count` | INT | DEFAULT 0 | Jumlah retry |
| `error_message` | TEXT | NULL | Error detail jika gagal |
| `sent_at` | TIMESTAMP | NULL | Waktu berhasil kirim |
| `related_entity_type` | VARCHAR(50) | NULL | e.g. `invoice`, `session` |
| `related_entity_id` | UUID | NULL | ID entity terkait |
| `created_at` | TIMESTAMP | DEFAULT NOW() | |

**Constraint**:
- Notifikasi tidak boleh dihapus (append-only log).
- Retry max 3x exponential backoff sebelum fallback channel.

---

## 11. AuditLog (Tabel 5.13)

Log audit trail untuk semua perubahan data.

| Field | Tipe | Constraint | Keterangan |
|-------|------|------------|------------|
| `id_audit` | UUID | PK | |
| `table_name` | VARCHAR(100) | NOT NULL | Nama tabel yang berubah |
| `record_id` | UUID | NOT NULL | ID record yang berubah |
| `action` | ENUM | NOT NULL | `INSERT`, `UPDATE`, `DELETE` |
| `before_state` | JSONB | NULL | State sebelum perubahan |
| `after_state` | JSONB | NULL | State setelah perubahan |
| `changed_by` | UUID | FK → User(id_user), NOT NULL | Siapa yang ubah |
| `ip_address` | INET | NULL | IP address pelaku |
| `user_agent` | TEXT | NULL | User agent pelaku |
| `created_at` | TIMESTAMP | DEFAULT NOW(), NOT NULL | |

### Constraint & Policy

- **IMMUTABLE**: Record AuditLog TIDAK BOLEH diupdate atau dihapus.
- **Retention**: Data disimpan minimal **7 tahun**.
- **PII Masking**: Setelah **2 tahun**, field ber-PII di `before_state` dan `after_state` di-mask (hash/placeholder).
- **No FK cascade**: AuditLog tidak punya FK ke tabel lain (independent log).
- Hanya INSERT yang diizinkan pada tabel ini (via trigger atau application logic).

---

## 12. Relational Constraint Summary

| Relasi | FK | On Delete | On Update |
|--------|----|-----------|-----------|
| Guardian → User | `id_user` | RESTRICT | CASCADE |
| Student → Guardian | `id_guardian` | RESTRICT | CASCADE |
| Teacher → User | `id_user` | RESTRICT | CASCADE |
| Session → Teacher | `id_teacher` | RESTRICT | CASCADE |
| Session → Student | `id_student` | RESTRICT | CASCADE |
| Session → Room | `id_room` | RESTRICT | CASCADE |
| Session → Session | `rescheduled_to` | SET NULL | CASCADE |
| Invoice → Guardian | `id_guardian` | RESTRICT | CASCADE |
| Payment → Invoice | `id_invoice` | RESTRICT | CASCADE |
| Payment → User | `verified_by` | RESTRICT | CASCADE |
| Attendance → Session | `id_session` | RESTRICT | CASCADE |
| Attendance → Teacher | `id_teacher` | RESTRICT | CASCADE |
| Attendance → User | `reviewed_by` | SET NULL | CASCADE |
| Notification → User | `recipient_id` | RESTRICT | CASCADE |
| AuditLog → User | `changed_by` | RESTRICT | CASCADE |

**Policy**: `ON DELETE RESTRICT` = tidak boleh hapus parent jika masih ada child. Data ber-PII menggunakan soft-delete (`deleted_at`), bukan hard delete.

---

## 13. Soft Delete Policy

| Entitas | Soft Delete | Reason |
|---------|-------------|--------|
| User | ✅ | PII — kredensial |
| Guardian | ✅ | PII — nama, telepon, alamat |
| Student | ✅ | PII — nama |
| Teacher | ✅ | PII — nama |
| Room | ❌ | Non-PII, RESTRICT cukup |
| Session | ❌ | Referensi historis |
| Invoice | ❌ | Financial record, immutable |
| Payment | ❌ | Financial record, immutable |
| Attendance | ❌ | Bukti kehadiran, immutable |
| Notification | ❌ | Append-only log |
| AuditLog | ❌ | IMMUTABLE — tidak boleh dihapus |

Untuk entitas dengan soft delete, query default menambahkan `WHERE deleted_at IS NULL`. Hapus permanen hanya melalui proses `UC-20 Proses Hapus PII` (compliance) dengan approval Owner.
