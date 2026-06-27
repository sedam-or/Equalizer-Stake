---
fr: FR-09, FR-11
uc: UC-12, UC-11
status: Approved
module: Audit Log & Tata Kelola Data
---

# SRS-07: Audit Log & Tata Kelola Data

## 1. Deskripsi Modul

Modul audit log menyediakan pencatatan immutable atas semua perubahan data dan aksi pengguna. Modul ini juga menangani retensi data, anonimisasi PII, dan proses penghapusan data sesuai UU PDP Pasal 19.

## 2. Immutable Audit Log (FR-09 / UC-12)

### 2.1 Struktur Record Audit

| Field | Tipe | Keterangan |
|-------|------|------------|
| `audit_id` | UUID | ID unik record audit |
| `timestamp` | TIMESTAMP | Waktu kejadian (UTC, presisi milidetik) |
| `actor_id` | FK | ID pengguna yang melakukan aksi |
| `actor_role` | ENUM | `OWNER`, `ADMIN`, `WALI_MURID`, `SYSTEM` |
| `action` | VARCHAR | Jenis aksi (CREATE, UPDATE, DELETE, LOGIN, dll) |
| `entity_type` | VARCHAR | Jenis entity (Invoice, Student, Schedule, dll) |
| `entity_id` | FK | ID entity yang terdampak |
| `before_state` | JSONB | Snapshot data sebelum perubahan (nullable untuk CREATE) |
| `after_state` | JSONB | Snapshot data sesudah perubahan (nullable untuk DELETE) |
| `ip_address` | INET | Alamat IP pelaku |
| `category` | VARCHAR | Kategori log (operasional, keamanan, kepatuhan) |

### 2.2 Immutability & Integrity

- Record audit **TIDAK BISA** diubah atau dihapus oleh role manapun.
- Jika penulisan audit log gagal, **rollback seluruh perubahan data** yang terkait.
- Audit log disimpan di storage terpisah dari data operasional.

### 2.3 Retensi & Anonimisasi

| Aturan | Detail |
|--------|--------|
| Retensi | 7 tahun sejak tanggal pencatatan |
| Anonimisasi PII | Data PII (nama, email, telepon, alamat) dianonimisasi setelah 2 tahun |
| Format anonim | PII di-replace dengan `REDACTED-[SHA256 hash]` |
| Non-PII dipertahankan | `actor_id`, `action`, `entity_type`, `entity_id`, `timestamp` tetap tersimpan |

### 2.4 Akses Control

| Role | Akses |
|------|-------|
| Owner | Semua audit log (operasional, keamanan, kepatuhan) |
| Admin | Audit log kategori operasional saja |

## 3. Storage Monitoring

- Sistem memonitor penggunaan storage audit log.
- Jika penggunaan >80% dari kapasitas alokasi, kirim alert ke Owner dan Admin.
- Alert mencakup: persentase penggunaan, estimasi waktu hingga penuh, rekomendasi rotasi.

## 4. Penghapusan Data — UU PDP Pasal 19 (FR-11 / UC-11)

### 4.1 Alur Penghapusan

```
Wali Murid ajukan form permintaan penghapusan
    ↓
Sistem cek: tidak ada invoice aktif?
    ├─ Ya → lanjut ke verifikasi
    └─ Tidak → tolak, tampilkan pesan: "Masih ada tagihan aktif"
    ↓
Admin verifikasi identitas Wali Murid
    ↓
Sistem proses:
    1. Anonimisasi PII → REDACTED-[SHA256 hash]
    2. Hapus foto profil secara permanen (physical delete)
    3. Catat di audit log dengan category = 'Tindakan Kepatuhan'
    ↓
Selesai — konfirmasi ke Wali Murid
```

### 4.2 SLA

| Aspek | Detail |
|-------|--------|
| Batas waktu proses | ≤14 hari kerja sejak permintaan diterima |
| Verifikasi identitas | Dilakukan oleh Admin dalam 3 hari kerja pertama |
| Pemrosesan data | Otomatis setelah verifikasi disetujui |

### 4.3 Data yang Dianonimisasi

| Field | Perlakuan |
|-------|-----------|
| `nama_lengkap` | `REDACTED-[hash]` |
| `email` | `REDACTED-[hash]` |
| `telepon` | `REDACTED-[hash]` |
| `alamat` | `REDACTED-[hash]` |
| Foto profil | Dihapus permanen (physical delete) |
| Data transaksi | Dipertagihkan (bukan PII, diperlukan untuk pembukuan) |

### 4.4 Constraint

- Permintaan penghapusan hanya dapat diajukan oleh Wali Murid yang terverifikasi.
- Jika ada invoice aktif, proses penghapusan ditolak secara otomatis.
- Semua langkah proses penghapusan dicatat di audit log.
- Data yang sudah dianonimisasi TIDAK BISA dikembalikan.
