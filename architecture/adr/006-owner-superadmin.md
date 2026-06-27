# ADR-006: Owner as Superadmin with Admin Management

## Status
Accepted

## Context
SRS asli mendefinisikan Owner sebagai read-only (hanya lihat dashboard + ekspor). Namun dalam skenario bisnis nyata, Owner perlu:
- Mengelola akun Admin (buka/tutup akses)
- Override keputusan Admin yang salah
- Menghapus Admin yang dipecat
- Mengatur pengaturan sistem level tinggi

Tanpa kemampuan ini, Owner terlalu bergantung pada developer untuk perubahan administratif.

## Decision
Owner di-upgrade menjadi **Superadmin** dengan hak:

1. **Manajemen Admin**: Buat akun Admin baru, nonaktifkan, hapus (PII dianonimkan)
2. **Override Operasional**: Buka freeze, batalkan tagihan, revert jadwal
3. **Audit Full Access**: Lihat semua kategori log (termasuk keamanan)
4. **Pengaturan Sistem**: Maintenance window, alert threshold, WhatsApp gateway config
5. **Kelola Tarif**: Ubah tarif kursus/studio (bersama Admin)

## Consequences

### Positive
- Owner bisa operasional tanpa developer
- Admin yang bermasalah bisa di-nonaktifkan langsung
- Audit log keamanan bisa diakses Owner
- Tarif bisa diubah sesuai kebutuhan bisnis

### Negative
- Owner perlu training tambahan untuk fitur superadmin
- Risk: Owner salah hapus Admin → mitigasi: konfirmasi ketik username
- Audit log superadmin actions harus sangat detail

### Mitigation
- Hapus Admin: wajib ketik username untuk konfirmasi (mencegah accidental)
- Override: semua aksi tercatat di audit log kategori "keamanan"
- Owner tidak bisa hapus data siswa/wali murid (hanya Admin bisa)
- Force logout semua session saat Admin dihapus

## Related
- SRS BAB 2.3 (User Characteristics)
- SRS FR-01 (Authentication)
- SRS FR-13 (Owner Dashboard)
- `srs/11-user-interaction-flows.md`
