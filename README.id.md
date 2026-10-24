<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="brand/assets/banner-dark.svg">
    <img alt="Ritma: irama sekolah Anda" src="brand/assets/banner-light.svg" width="100%">
  </picture>
</p>

<h3 align="center">Sistem operasi open-source untuk lembaga kursus dan studio.</h3>

<p align="center">
  Jadwal les, pendaftaran, SPP bulanan, absensi dan pesan ke wali murid dalam satu tempat.<br>
  Siap dalam tiga langkah. Jalankan di PC sendiri, server sendiri, atau cloud kami.
</p>

<p align="center">
  <a href="https://github.com/sedam-or/Equalizer-Stake/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/sedam-or/Equalizer-Stake/actions/workflows/ci.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="Lisensi: AGPL-3.0" src="https://img.shields.io/badge/license-AGPL--3.0-005EB9"></a>
  <a href="ROADMAP.md"><img alt="Status: pra-rilis" src="https://img.shields.io/badge/status-pre--release-AA5F04"></a>
</p>

<p align="center"><a href="README.md">Read in English</a></p>

> [!IMPORTANT]
> **Ritma masih pra-rilis. Belum ada yang bisa dipasang.** Spesifikasinya sudah lengkap dan implementasi dimulai dari milestone M0. Demo, cloud dan unduhan di bawah aktif mulai v1.0. Pantau progres di [ROADMAP.md](ROADMAP.md) dan [tasks/INDEX.md](tasks/INDEX.md), atau *watch* repositori ini untuk rilis.

**Coba demo** · **Mulai gratis di cloud** · **[Pasang sendiri](#pasang-sendiri)** · Unduh untuk Windows / macOS / Linux *(mulai v1.0)*

<!-- GIF setup 3 langkah ditaruh di sini setelah wizard setup ada (milestone M3, brand/BRAND.md §8). -->

## Kenapa Ritma

Kebanyakan sekolah kecil berjalan dengan WhatsApp, spreadsheet dan buku absen kertas. Semuanya lancar sampai dua les jatuh di ruang yang sama, wali murid membayar dengan nominal yang salah, atau satu-satunya orang yang paham spreadsheet sedang cuti. Ritma mengganti tumpukan itu dengan satu sistem yang mengikuti aturan **Anda**.

- **Siapkan sekolah Anda dalam tiga langkah.** Pilih template, nyalakan fitur yang dibutuhkan, cek aturannya. Setiap aturan punya nilai bawaan yang aman, jadi "Lanjut, Lanjut, Selesai" sudah menghasilkan sekolah yang siap jalan. Targetnya: les pertama terjadwal dalam kurang dari 10 menit. → [spec 01](spec/01-product.md), [spec 05](spec/05-presets-and-setup.md)
- **Tidak ada jadwal bentrok, dijaga oleh database.** Constraint *exclusion* di PostgreSQL membuat bentrok mustahil terjadi, bukan sekadar jarang. Sebuah tes mengirim 50 pemesanan bersamaan ke satu slot di setiap pull request: tepat satu yang berhasil. → [scheduling](spec/modules/scheduling.md), [TST-05](spec/12-testing.md)
- **Aturan Anda, dalam bahasa sehari-hari.** SPP, tanggal jatuh tempo, denda, jam operasional, kredit pembatalan dan radius absensi adalah pengaturan, bukan kode. Setiap layar aturan menjelaskan artinya dalam satu kalimat, dan tagihan lama tetap memakai aturan saat tagihan itu dibuat. → [spec 04](spec/04-policy-engine.md)
- **Dibuat untuk cara kerja sekolah kecil.** Pengingat WhatsApp dengan cadangan email dan notifikasi di aplikasi, satu tagihan per wali untuk semua anaknya, kode referensi transfer, serta Bahasa Indonesia dan Inggris sejak hari pertama. → [billing](spec/modules/billing.md), [notifikasi](spec/modules/notify.md)
- **Data Anda, Anda yang pilih tempatnya.** Software yang sama berjalan di PC resepsionis, server sendiri atau cloud kami. Ekspor semua data kapan saja dan pindah di antaranya. → [spec 14](spec/14-distribution.md)

## Untuk siapa

Sekolah dan studio independen dengan satu sampai tiga lokasi dan 20 sampai 500 murid: **musik, tari, bela diri, bimbel, seni, renang**, dan studio yang menyewakan ruang per jam.

| Anda adalah | Ritma memberi Anda |
|---|---|
| **Pemilik** | Sekolah yang berjalan sesuai aturan Anda, dasbor pemasukan dan kehadiran, dan perubahan aturan yang tidak pernah mengubah tagihan lama |
| **Admin / resepsionis** | Satu daftar "perlu perhatian" sebagai ganti 40 pesan, ubah jadwal dengan sekali klik, pembayaran dicocokkan dengan tagihan |
| **Guru** | Jadwal hari ini di HP, absen masuk dalam hitungan detik (meski sinyal lemah), absensi murid dan catatan perkembangan |
| **Wali murid** | Daftar dari rumah, semua anak di satu tempat, satu tagihan, pengingat lewat WhatsApp |
| **Orang IT** | Satu binary dan PostgreSQL, HTTPS otomatis, backup terenkripsi, image bertanda tangan, update tanpa migrasi manual |

Ikuti hari lengkap masing-masing di [perjalanan pengguna](spec/28-user-journeys.md) (bahasa Inggris).

## Siap dalam tiga langkah

| Langkah | Anda | Ritma |
|---|---|---|
| **1. Pilih template** | Pilih *Sekolah musik*, *Tari atau bela diri*, *Bimbel*, *Sewa studio* atau *Kosong* | Memuat fitur, aturan, istilah ("guru" atau "pelatih") dan mata uang untuk jenis sekolah Anda |
| **2. Pilih fitur** | Nyalakan atau matikan modul | Menjelaskan modul mana yang saling bergantung, supaya tidak ada yang rusak |
| **3. Cek aturan** | Baca ringkasannya dan ubah yang tidak cocok | Menyimpan aturan versi 1 dan membuka dasbor dengan daftar periksa singkat |

Semua bisa diubah nanti di layar yang sama, dengan tiga langkah yang sama.

## Fitur

Nyalakan hanya yang Anda butuhkan. Modul yang tidak dipakai tidak muncul di menu.

| Modul | Fungsinya |
|---|---|
| **Jadwal** *(selalu aktif)* | Ruang, kursus, les mingguan berulang, pemesanan tanpa bentrok dengan saran alternatif, hari libur |
| **Pendaftaran** | Pendaftaran online, wali dengan beberapa anak, level, cuti, kredit les pengganti |
| **Tagihan** | Tagihan bulanan, kode referensi transfer, unggah dan verifikasi bukti bayar, denda, bekukan dan buka akun |
| **Absensi** | Absen masuk guru berdasarkan waktu, lokasi dan foto, absensi murid, antrean offline |
| **Sewa studio** | Pemesanan ruang per jam dengan persetujuan, DP dan penahanan slot yang kedaluwarsa |
| **Notifikasi** | WhatsApp, email dan di aplikasi, dengan pengulangan, cadangan dan jam tenang |
| **Laporan** | Dasbor pemilik, pemakaian ruang, piutang, laporan siap cetak |
| **Umpan balik** | Catatan perkembangan untuk wali murid, penilaian guru |

## Perbandingan

| | Spreadsheet + WhatsApp | SaaS sekolah global | Aplikasi booking janji temu | **Ritma** |
|---|---|---|---|---|
| Jadwal tanpa bentrok | Manual | Biasanya | Ya | **Ya, dijaga oleh database** |
| Les berulang, pendaftaran, SPP bulanan | Manual | Ya | Tidak | **Ya** |
| Wali dengan beberapa anak, satu tagihan | Manual | Tergantung | Tidak | **Ya** |
| Mata uang lokal, kode transfer bank, WhatsApp | Sebagian | Jarang | Jarang | **Ya** |
| Pasang sendiri, atau bawa keluar data Anda | Ya | Tidak | Sebagian | **Ya, ekspor semua data** |
| Harga | Gratis | Langganan bulanan | Tergantung | **Gratis jika dipasang sendiri**, cloud opsional |
| Open source | — | Tidak | Sebagian | **AGPL-3.0** |

## Pasang sendiri

Satu binary ditambah PostgreSQL. Pilih yang paling cocok (tersedia mulai v1.0):

- **Docker satu baris** di server Linux mana pun: `curl -fsSL https://get.<domain>/ | sh`. Perintah ini menyiapkan Ritma, PostgreSQL dan HTTPS otomatis, lalu menampilkan tautan setup Anda.
- **Template sekali klik:** Railway, Render, DigitalOcean, Coolify, CasaOS, YunoHost.
- **Aplikasi desktop** untuk PC sekolah (Windows, macOS, Linux), dengan akses aman dari HP guru dan wali murid.

Semua pilihan berakhir di setup tiga langkah yang sama. Setiap rilis ditandatangani dengan cosign dan dilengkapi SBOM serta *build provenance* ([spec 26](spec/26-containers-and-caching.md)).

## Dibangun dengan

Rust (axum, sqlx) · PostgreSQL · React, Vite dan Tailwind · Tauri untuk aplikasi desktop.

Dibangun di atas tool open-source yang sudah dikenal, bukan buatan sendiri ([spec 24](spec/24-open-source-stack.md)). Keamanan dirancang sejak awal: data pribadi terenkripsi, log audit yang ketahuan jika diubah, sesi berumur pendek dan matriks hak akses yang dipublikasikan ([spec 18](spec/18-security-by-design.md), [spec 25](spec/25-auth-and-access-control.md)). Baca [arsitektur dan spesifikasi](spec/00-index.md) serta [catatan keputusan](spec/adr/).

## Peta jalan

| Milestone | Yang Anda dapat |
|---|---|
| M0–M2 | Mesinnya: akun, aturan, modul dan jadwal tanpa bentrok (tes 50 → 1 berhasil) |
| M3 | **Sekolah siap dalam tiga langkah**, dengan aplikasi web dan layar jadwal |
| M4–M6 | Pendaftaran, tagihan, absensi, notifikasi, sewa, laporan, alat privasi |
| M7 | **Rilis publik v1.0**: Docker, template sekali klik, dokumentasi dan demo langsung |
| M8 / M9 | Aplikasi desktop / Ritma Cloud |

Detail di [ROADMAP.md](ROADMAP.md).

## Kontribusi

- **Tambahkan template untuk jenis sekolah Anda cukup dengan YAML, tanpa Rust.** Sekolah renang, kursus bahasa, klub coding: [CONTRIBUTING.md](CONTRIBUTING.md) menjelaskan caranya.
- **Terjemahkan** Ritma ke bahasa Anda.
- **Tulis kode:** pekerjaan dipecah menjadi file task kecil yang berdiri sendiri. Ambil satu dari [tasks/INDEX.md](tasks/INDEX.md) atau issue `good first issue`, lalu ikuti [WORKFLOW.md](WORKFLOW.md).

Mohon baca [Kode Etik](CODE_OF_CONDUCT.md), [GOVERNANCE.md](GOVERNANCE.md) dan [SECURITY.md](SECURITY.md) (laporkan celah keamanan secara privat).

## Cerita

Ritma berawal dari proyek capstone universitas untuk sebuah sekolah musik sungguhan di Indonesia, yang jadwalnya ada di WhatsApp dan SPP-nya di spreadsheet. Kini dibangun ulang sebagai open source supaya lembaga kursus mana pun, di mana pun, bisa memakainya.

## Lisensi

Ritma v2 berlisensi [AGPL-3.0](LICENSE). Arsip capstone v0 di `legacy/` tetap MIT. Nama dan logo adalah merek dagang proyek; lihat [TRADEMARK.md](TRADEMARK.md) dan [panduan merek](brand/GUIDELINES.md).
