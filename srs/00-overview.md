---
module: overview
title: "Equalizer-Stake — Sistem Penjadwalan Kursus & Reservasi Studio BMD Musik Yogya"
standard: "ISO/IEC/IEEE 29148:2018"
version: "2.0"
date: "2026-06-27"
---

# Equalizer-Stake — Overview

## 1. Identitas Proyek

| Field | Value |
|---|---|
| Nama Sistem | Equalizer-Stake |
| Klien | BMD Music School Yogyakarta |
| Standar | ISO/IEC/IEEE 29148:2018 |
| Cakupan | Penjadwalan kursus musik, reservasi studio, billing, kehadiran |

## 2. Peran Pengguna

| Peran | Fungsi | Session Timeout |
|---|---|---|
| **Owner / Superadmin** | Superadmin: dashboard, manajemen Admin, override operasional, audit full, pengaturan sistem | 30 menit |
| **Admin Operasional** | Manajemen harian (jadwal, tagihan, siswa) | 30 menit |
| **Guru Musik** | Absensi, lihat jadwal mengajar | 30 menit |
| **Wali Murid** | Portal 1:N siswa, booking, pembayaran | 60 menit |

## 3. Modul Sistem

| # | Modul | Keterangan |
|---|---|---|
| 1 | **Scheduling** | Penjadwalan kursus, reservasi studio, validasi konflik real-time |
| 2 | **Billing** | Invoice otomatis, kode unik, denda, pembekuan |
| 3 | **Attendance** | Absensi siswa per sesi, rekap kehadiran |
| 4 | **Notification** | Pengingat jadwal, tagihan, perubahan (WA/email) |
| 5 | **Reporting** | Laporan keuangan, utilisasi guru/ruangan, KPI Owner |
| 6 | **Audit** | Trail semua transaksi, retensi 7 tahun |
| 7 | **User Docs** | Panduan pengguna per peran |

## 4. Kriteria Keberhasilan

| Metrik | Target |
|---|---|
| Biaya admin | **-30%** dari baseline manual |
| Retensi siswa | **+15%** dari baseline |
| Cash flow lag | **≤ 7 hari** dari tagihan ke terima |
| Sengketa pembayaran | **-90%** dari baseline |

## 5. Out of Scope

- Native mobile app (iOS/Android)
- Payment gateway integration
- AI/ML (prediksi churn, rekomendasi)
- Multi-cabang / franchise
- Live streaming / video lesson
- Inventory manajemen alat musik

## 6. Lingkungan Operasional

| Aspek | Spesifikasi |
|---|---|
| Infrastruktur | Cloud Jakarta |
| Server | 2 vCPU / 4 GB RAM |
| Kapasitas | ≤ 50 concurrent users |
| Delivery | PWA (Progressive Web App) |
| Browser | Chrome 90+, Safari 14+ |

## 7. AS-IS vs TO-BE

### AS-IS (Manual)

| Masalah | Dampak |
|---|---|
| Double-booking | 3–5 kejadian/bulan |
| No-show | 20–30% dari total sesi |
| Beban admin | 2–3 jam/hari untuk penjadwalan & tagihan |
| Sengketa pembayaran | Frequent, manual reconciliation |

### TO-BE (Equalizer-Stake)

| Metrik | Target |
|---|---|
| Double-booking | **0** (validasi real-time) |
| No-show | **≤ 8%** (pengingat otomatis) |
| Beban admin | **≤ 45 menit/hari** |
| Notifikasi terkirim | **≤ 15 detik** dari trigger |
| Audit trail | **7 tahun** retensi |
