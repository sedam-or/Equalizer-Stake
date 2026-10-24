---
id: SRS-NFR
title: "Non-Functional Requirement (NFR)"
source: "BAB 3 §3.2"
last_updated: 2026-06-27
---

# 3.2 Non-Functional Requirement

## A. Kinerja (Performance)

| ID | Metrik | Target | Metode Ukur |
|----|--------|--------|-------------|
| NFR-PERF-01 | Waktu muat halaman | ≤2 detik (percentile 95) | Lighthouse CI, server-side APM |
| NFR-PERF-02 | Waktu respons validasi form | ≤1 detik | Client-side performance API |
| NFR-PERF-03 | Waktu kirim notifikasi (WhatsApp/Email) | ≤15 detik dari trigger | Queue monitoring, webhook timestamp |
| NFR-PERF-04 | Kapasitas pengguna simultan | ≤50 pengguna aktif | Load test (k6/Artillery) |
| NFR-PERF-05 | Proses batch invoice | ≤5 menit untuk 50 invoice | Server-side job timer |

### Detail

- **NFR-PERF-01**: Semua halaman (termasuk dashboard, form input, laporan) harus
 selesai dimuat dalam ≤2 detik pada koneksi 4G standar. Diukur dengan
 Time to Interactive (TTI) via Lighthouse CI pada pipeline.
- **NFR-PERF-02**: Validasi form sisi klien (format, required field) harus
 selesai ≤1 detik. Validasi sisi server (unik, referensi) ≤2 detik.
- **NFR-PERF-03**: Notifikasi WhatsApp dan Email harus terkirim ≤15 detik
 sejak event trigger. Jika gagal, retry sesuai NFR-REL-01.
- **NFR-PERF-04**: Sistem harus mendukung ≤50 pengguna aktif simultan tanpa
 degradasi performa >20% dari baseline.
- **NFR-PERF-05**: Batch generate invoice untuk 50 student harus selesai
 ≤5 menit. Monitoring via Celery/queue dashboard.

---

## B. Keamanan (Security)

| ID | Metrik | Target |
|----|--------|--------|
| NFR-SEC-01 | Enkripsi data at-rest | AES-256-GCM |
| NFR-SEC-02 | Enkripsi data in-transit | TLS 1.2+ |
| NFR-SEC-03 | Brute-force login protection | Maks 5 percobaan / 15 menit |
| NFR-SEC-04 | API rate limiting | 100 request/menit per user |
| NFR-SEC-05 | OWASP Top 10 coverage | A02 (Cryptographic Failures), A03 (Injection), A04 (Insecure Design), A05 (Security Misconfiguration), A06 (Vulnerable Components), A07 (Authentication Failures) |

### Detail

- **NFR-SEC-01**: Semua kolom PII (KTP, alamat, nama lengkap, foto) dienkripsi
 AES-256-GCM sebelum penyimpanan. Key management via KMS/Vault.
- **NFR-SEC-02**: Seluruh komunikasi client-server menggunakan TLS 1.2+.
 HSTS header aktif. HTTP redirect ke HTTPS.
- **NFR-SEC-03**: Setelah 5 percobaan login gagal dalam 15 menit, akun
 dikunci sementara 15 menit. Notifikasi ke admin.
- **NFR-SEC-04**: Rate limit 100 request/menit per IP/user. Melebihi batas →
 HTTP 429 Too Many Requests. Endpoint sensitif (login, payment) limit lebih ketat.
- **NFR-SEC-05**: Implementasi countermeasure OWASP A02–A07:
 - A02: Tidak menyimpan secret di source code; enkripsi at-rest
 - A03: Parameterized queries; ORM usage; no raw SQL
 - A04: Threat modeling pada fase desain
 - A05: Security headers (CSP, X-Frame-Options, X-Content-Type-Options)
 - A06: Dependency scanning (Dependabot/Snyk) pada CI
 - A07: MFA opsional untuk Owner; session timeout per role

---

## C. Kemudahan Penggunaan (Usability)

| ID | Metrik | Target |
|----|--------|--------|
| NFR-USA-01 | Standar aksesibilitas | WCAG 2.1 Level A |
| NFR-USA-02 | Responsiveness | Mobile-first, minimum 320px |
| NFR-USA-03 | Bahasa antarmuka | Indonesia (id-ID) |
| NFR-USA-04 | Waktu pelatihan admin | ≤2 jam untuk admin baru |

### Detail

- **NFR-USA-01**: Semua elemen interaktif memiliki label, kontras warna ≥4.5:1,
 navigasi keyboard, dan alt text pada gambar.
- **NFR-USA-02**: Layout menggunakan mobile-first CSS breakpoint. Minimum
 viewport 320px (iPhone SE). Tablet dan desktop breakpoint di 768px dan 1024px.
- **NFR-USA-03**: Seluruh label, pesan error, notifikasi, dan dokumentasi
 dalam Bahasa Indonesia.
- **NFR-USA-04**: Admin harus dapat menjalankan fungsi dasar (CRUD student,
 jadwal, tagihan) setelah ≤2 jam pelatihan.

---

## D. Keandalan (Reliability)

| ID | Metrik | Target |
|----|--------|--------|
| NFR-REL-01 | Retry external call | 3 kali dengan exponential backoff |
| NFR-REL-02 | Circuit breaker | Aktif untuk WhatsApp, Email, Payment gateway |
| NFR-REL-03 | Atomic booking | Transaksi booking tidak meninggalkan data setengah jadi |
| NFR-REL-04 | Backup | Harian (daily) otomatis |
| NFR-REL-05 | RTO (Recovery Time Objective) | ≤2 jam |
| NFR-REL-06 | RPO (Recovery Point Objective) | ≤1 jam |

### Detail

- **NFR-REL-01**: Semua panggilan eksternal (WhatsApp API, Email SMTP,
 Payment gateway) di-retry maksimal 3 kali dengan exponential backoff
 (1s, 2s, 4s). Setelah 3 kali gagal → circuit breaker trip.
- **NFR-REL-02**: Circuit breaker aktif pada integrasi pihak ketiga.
 State: Closed → Open (setelah threshold gagal) → Half-Open (setelah cooldown).
- **NFR-REL-03**: Proses booking jadwal menggunakan database transaction.
 Rollback jika terjadi error di tengah proses. Tidak ada slot terkunci
 tanpa booking tercatat.
- **NFR-REL-04**: Database di-backup otomatis setiap hari pukul 03:00 WIB.
 Backup disimpan 30 hari. Test restore dilakukan bulanan.
- **NFR-REL-05**: Jika sistem down, target recovery ≤2 jam dari waktu kejadian.
- **NFR-REL-06**: Data yang hilang pada skenario disaster ≤1 jam terakhir.

---

## E. Ketersediaan (Availability)

| ID | Metrik | Target |
|----|--------|--------|
| NFR-AVL-01 | Uptime | ≥99% per bulan (maks 7.3 jam downtime/bulan) |
| NFR-AVL-02 | Maintenance window | Senin 02:00–04:00 WIB |

### Detail

- **NFR-AVL-01**: Sistem harus tersedia ≥99% setiap bulan kalender. Downtime
 terencana (maintenance) tidak dihitung.
- **NFR-AVL-02**: Maintenance terjadwal dilakukan setiap Senin pukul 02:00–04:00
 WIB. Pengguna diberi notifikasi H-1.

---

## F. Trade-off Prioritas

Keputusan desain mengikuti urutan prioritas berikut ketika terjadi konflik:

| Prioritas | Prinsip | Contoh |
|-----------|---------|--------|
| 1 | **Keamanan > Kinerja** | Enkripsi PII menambah latency, tetapi wajib diterapkan |
| 2 | **Keandalan > Biaya** | Retry + circuit breaker menambah infrastruktur cost |
| 3 | **Keamanan > Kemudahan** | Session timeout memaksa re-login, mengurangi kenyamanan |
| 4 | **Ketersediaan > Skalabilitas** | Vertical scaling diprioritaskan sebelum horizontal |
