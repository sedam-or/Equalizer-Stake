**BAB 2 --- DESKRIPSI UMUM**

**2.1 Perspektif Sistem (AS-IS vs TO-BE)**

**Kondisi Saat Ini (AS-IS)**

BMD Musik saat ini mengelola seluruh operasional penjadwalan, kehadiran,
dan penagihan secara manual menggunakan kombinasi catatan kertas,
spreadsheet, dan komunikasi informal melalui grup WhatsApp. Kondisi ini
menghasilkan sejumlah masalah operasional yang terukur:

**Tabel 2.1 --- Masalah Operasional AS-IS**

  -----------------------------------------------------------------------
  **Masalah Operasional**         **Dampak Terukur**
  ------------------------------- ---------------------------------------
  Penjadwalan manual rentan       Insiden double-booking 3--5x per bulan
  konflik slot (double-booking)   

  Pengingat jadwal dilakukan      Waktu kerja Admin 2--3 jam/hari hanya
  manual oleh Admin via WhatsApp  untuk notifikasi

  Absensi dicatat di kertas,      Tingkat ketidakhadiran tanpa kabar
  tanpa verifikasi digital        (no-show) 20--30%

  Tagihan dibuat manual per murid Kesalahan tagihan dan keterlambatan
  setiap bulan                    penagihan terjadi reguler

  Tidak ada rekam jejak perubahan Sulit melacak perubahan jadwal dan
  data yang terpusat              riwayat pembayaran
  -----------------------------------------------------------------------

**Kondisi yang Dituju (TO-BE)**

Sistem Equalizer-Stake dirancang untuk menggantikan proses manual dengan
otomatisasi berbasis web yang dapat diakses dari HP maupun laptop tanpa
instalasi aplikasi. Target peningkatan terukur setelah implementasi:

**Tabel 2.2 --- Target Peningkatan AS-IS ke TO-BE**

  -----------------------------------------------------------------------
  **Indikator**                **AS-IS**         **TO-BE (Target)**
  ---------------------------- ----------------- ------------------------
  Waktu kerja Admin untuk      2--3 jam/hari     ≤ 45 menit/hari
  operasional harian                             

  Tingkat no-show siswa        20--30%           ≤ 8%

  Insiden double-booking       3--5x/bulan       0 (nol)

  Kecepatan pengiriman         Manual, tidak     ≤ 15 detik setelah
  notifikasi                   terukur           pemicu

  Tingkat keberhasilan         Tidak terukur     ≥ 98% via WhatsApp
  pengiriman notifikasi                          

  Ketersediaan audit log       Tidak ada         Permanen 7 tahun
  perubahan data                                 
  -----------------------------------------------------------------------

**2.2 Fungsi Produk**

Sistem Equalizer-Stake terdiri dari enam modul fungsional utama:

**Modul 1: Penjadwalan (Scheduling)**

- Input dan pengelolaan jadwal kursus reguler oleh Admin.

- Validasi konflik slot secara real-time saat penyimpanan jadwal.

- Pengelolaan slot minggu ke-5 sebagai Kelas Teori Non-Kredit.

- Penanganan otomatis libur nasional: reschedule atau pembatalan.

- Reservasi ruang studio (rehearsal dan recording) oleh Admin.

- Penguncian slot setelah konfirmasi; hanya Admin yang dapat membuka
  kembali dengan rekam jejak.

**Modul 2: Penagihan dan Pembayaran (Billing)**

- Penerbitan tagihan otomatis setiap tanggal 5 kepada seluruh Wali Murid
  aktif.

- Pemberian kode unik 3 digit pada setiap tagihan untuk verifikasi
  pembayaran manual.

- Penerimaan bukti pembayaran dan verifikasi manual oleh Admin.

- Penerapan denda otomatis Rp 100.000 pada tanggal 11 pukul 00:00 untuk
  tagihan yang belum lunas.

- Pembekuan otomatis akun Wali Murid yang melewati batas waktu tanpa
  pelunasan.

- Dukungan pembayaran sebagian (DP ≥70%) dengan akses terbatas hingga
  tanggal 10.

**Modul 3: Kehadiran (Attendance)**

- Input absensi digital oleh Guru melalui antarmuka web.

- Validasi foto absensi: timestamp server, koordinat GPS dalam radius
  100 meter, wajah siswa ≥30% frame.

- Pencatatan kompensasi sesi jika Guru berhalangan hadir.

- Pelacakan kelayakan ujian siswa (kehadiran ≥90%, maks. 1 absen tanpa
  kabar per 3 bulan).

**Modul 4: Notifikasi (Notification)**

- Pengingat jadwal otomatis 1 jam sebelum sesi via WhatsApp
  (non-kritis).

- Konfirmasi pembayaran, pembekuan/pembukaan akun via email (kritis)
  dengan cadangan WhatsApp.

- Sistem percobaan ulang otomatis (3x dengan jeda bertambah: 10, 20, 40
  detik) jika pengiriman gagal.

- Catatan status pengiriman per notifikasi dan per saluran
  (WhatsApp/email).

- Panel toggle multi-saluran di dashboard Admin.

- Semua notifikasi dikirim ke kontak Wali Murid, bukan per siswa
  individual.

**Modul 5: Pelaporan (Reporting)**

- Dashboard monitoring kehadiran, pendapatan, dan utilisasi studio untuk
  Owner.

- Laporan bulanan rekap kehadiran per siswa, per kelas, dan per guru.

- Laporan status tagihan dan riwayat pembayaran.

- Ekspor laporan dalam format yang dapat dicetak.

**Modul 6: Audit dan Tata Kelola (Governance)**

- Audit log permanen seluruh perubahan data kritis (jadwal, tagihan,
  akun, absensi).

- Penyimpanan metadata log selama 7 tahun; anonimisasi data pribadi
  setelah 2 tahun.

- Prosedur hak penghapusan data pribadi sesuai UU PDP Pasal 19.

- Penyimpanan otomatis draft input setiap 30 detik dan pemulihan draft
  setelah sesi berakhir.

**Modul 7: Dokumentasi Pengguna (User Documentation)**

- Admin Operation Manual: Mencakup prosedur penjadwalan, verifikasi
  pembayaran, penanganan alert notifikasi, dan eskalasi pending
  verification.

- Guru Quick Guide: Panduan singkat input absensi, validasi GPS, dan
  penjadwalan ulang via WhatsApp (jika sistem down).

- Wali Murid FAQ: Pertanyaan umum mengenai kode unik, kebijakan denda,
  cara membaca dashboard multi-siswa, dan prosedur penghapusan data.

- Online Help Tooltip: Sistem shall menampilkan petunjuk kontekstual
  (tooltip) pada form input kritis (jadwal, absensi, verifikasi) yang
  dapat dinonaktifkan oleh pengguna.

**2.3 Karakteristik Pengguna**

Catatan Penting: Satu akun Wali Murid mengelola banyak siswa (relasi 1
Wali : N Siswa, contoh: kakak dan adik dalam satu keluarga). Seluruh
notifikasi dikirim ke kontak Wali Murid, bukan per siswa. Tidak tersedia
login terpisah per siswa.

**Tabel 2.3 --- Karakteristik Pengguna Sistem**

  -------------------------------------------------------------------------------------------------------
  **Pengguna**   **Kemampuan        **Hak Akses Utama**     **Session  **Product         **Otoritas
                 Teknis**                                   Timeout**  Champion**        Keputusan**
  -------------- ------------------ ---------------------- ----------- ----------------- ----------------
  Owner          Menengah           Dashboard monitoring,   30 menit   \[Nama Pemilik    Final approval
                                    audit log,                         BMD Musik\]       strategi harga,
                                    aktivasi/nonaktifkan               (Founder/Owner)   hak akses admin,
                                    admin, approval revisi                               kebijakan cuti &
                                    tagihan.                                             denda

  Admin          Menengah--Tinggi   Input/ubah jadwal,      15 menit   \[Nama Kepala     Penjadwalan
  Operasional                       verifikasi pembayaran,             Admin BMD Musik\] harian, validasi
                                    manajemen akun,                    (Operational      absensi,
                                    monitoring notifikasi.             Lead)             eskalasi bentrok
                                                                                         jadwal

  Guru Musik     Dasar--Menengah    Lihat kalender          30 menit   \[Nama            Standar
                                    pribadi, input                     koordinator       penilaian
                                    absensi + foto,                    guru\] (Teaching  kehadiran,
                                    konfirmasi kehadiran,              Lead)             persetujuan
                                    terima reminder.                                     reschedule,
                                                                                         kelayakan level
                                                                                         siswa

  Wali Murid     Dasar              Lihat jadwal            60 menit   \[Nama Perwakilan Batas toleransi
                                    multi-anak, ajukan                 Wali Murid\]      keterlambatan,
                                    booking (pending),                 (Parent Liaison)  format laporan
                                    terima tagihan &                                     progres, kanal
                                    reminder, riwayat                                    komunikasi resmi
                                    kehadiran.                                           
  -------------------------------------------------------------------------------------------------------

**2.4 Batasan Sistem**

**Batasan Teknis**

- Sistem dirancang untuk berjalan di atas infrastruktur cloud scalable
  (misal: AWS, GCP, atau VPS dengan spesifikasi minimal 2 vCPU, 4 GB
  RAM). Kapasitas beban awal dirancang untuk ≤ 50 pengguna aktif
  bersamaan dengan kemampuan auto-scaling ke beban lebih tinggi. Biaya
  infrastruktur tidak menjadi batasan desain; prioritas utama adalah
  performa, keandalan, dan keamanan data.

- Enkripsi data pribadi menggunakan AES-256-GCM untuk data tersimpan
  (at-rest) dan TLS 1.2+ untuk data dalam pengiriman (in-transit).

- Sistem tidak menggunakan aplikasi mobile native; antarmuka berbasis
  PWA yang dapat diakses di browser HP dan laptop.

- Kapasitas beban awal: ≤ 50 pengguna aktif bersamaan dengan kemampuan
  auto-scaling.

- Penyimpanan data PII diprioritaskan di server dengan lokasi data (data
  residency) di region Jakarta.

**Catatan Toleransi Degradasi (Masa MVP)**

Pada masa MVP, fitur berat seperti ekspor laporan (report export) dan
pencarian data historis (search) mungkin memiliki latensi lebih tinggi
(≤ 5 detik) karena optimalisasi query belum sepenuhnya dilakukan. Owner
dan Admin BMD Musik telah menyetujui toleransi ini selama masa uji coba
3 bulan pertama.

**Batasan Bisnis**

- Tidak ada integrasi payment gateway otomatis pada fase MVP; pembayaran
  diverifikasi manual oleh Admin.

- Timeline pengembangan MVP: 10 minggu.

- Tidak ada fitur kecerdasan buatan (AI/ML) di MVP.

- Sistem hanya mendukung satu lokasi studio (BMD Musik, Jl. Seturan No.
  15, Yogyakarta).

**Batasan Kepatuhan**

- Seluruh pengelolaan data pribadi wajib memenuhi UU PDP No. 27 Tahun
  2022 (minimasi data, hak penghapusan, keamanan data).

- Audit log wajib disimpan minimal 7 tahun; tidak dapat dihapus oleh
  pengguna manapun termasuk Admin.

- Foto absensi dihapus permanen dari server setelah hak penghapusan data
  diverifikasi; metadata anonim tetap dipertahankan untuk keperluan
  audit.

**Batasan Layanan Pihak Ketiga**

- WhatsApp gateway menggunakan layanan tidak resmi (unofficial);
  berpotensi gangguan layanan dan tidak memiliki SLA tertulis.

- Sistem wajib menyediakan mekanisme cadangan email SMTP untuk
  komunikasi kritis jika WhatsApp gateway tidak tersedia.

**2.5 Asumsi**

- Jaringan: Pengguna memiliki akses internet yang stabil (minimum 4G)
  untuk mengakses sistem. Fungsi offline dasar tersedia melalui PWA
  untuk keperluan melihat jadwal terakhir.

- WhatsApp Gateway: Layanan WhatsApp gateway pihak ketiga tersedia
  minimal 98% waktu operasional. Jika terjadi gangguan, sistem beralih
  otomatis ke email.

- Volume Data: Jumlah siswa aktif tidak melebihi 50 orang per bulan dan
  jumlah guru tidak melebihi 10 orang selama fase MVP.

- Perilaku Pengguna: Wali Murid dan Guru mengikuti alur onboarding yang
  telah ditetapkan Admin; perubahan data master (jadwal, tarif) hanya
  dilakukan oleh Admin.

- Perangkat: Pengguna mengakses sistem menggunakan browser modern
  (Chrome 90+, Safari 14+, atau setara).

- Pembayaran: Wali Murid membayar melalui transfer bank; bukti transfer
  dikirimkan ke Admin untuk verifikasi manual.

- Kalender: Daftar hari libur nasional diperbarui manual oleh Admin di
  awal setiap tahun ajaran.

**2.6 Aturan Bisnis Formal**

**A. Durasi Sesi dan Jeda Wajib**

- Durasi sesi: 30 menit (pemula/kelas khusus) atau 45--50 menit (kelas
  reguler).

- Jeda wajib antar-sesi: 10 menit untuk persiapan ruang dan pergantian
  guru; sistem tidak boleh mengizinkan booking yang melanggar jeda ini.

- Jam operasional kursus: Senin--Jumat 14.30--17.00 WIB; Sabtu
  08.00--12.00 WIB.

- Sistem tidak menjadwalkan kursus pada hari Minggu dan hari libur
  nasional.

**B. Kebijakan Kehadiran dan Kompensasi**

- Siswa absen tanpa kabar: Sesi hangus, kredit tetap terpotong, tidak
  ada reschedule atau refund.

- Guru berhalangan hadir: Sesi dijadwalkan ulang (reschedule), kredit
  siswa tidak terpotong, dicatat sebagai hutang jam guru.

- Keterlambatan siswa: Sesi tetap berakhir pada waktu yang ditentukan;
  tidak ada perpanjangan waktu.

- Cuti siswa: Maksimal 1 bulan; jika melebihi 1 bulan, siswa wajib
  mendaftar ulang sebagai siswa baru.

**C. Siklus Tagihan dan Denda Keterlambatan**

- Tagihan diterbitkan otomatis setiap tanggal 5 pukul 00:00.

- Batas pembayaran: tanggal 10 pukul 23:59.

- Pada tanggal 11 pukul 00:00: sistem otomatis menerapkan denda flat Rp
  100.000 dan membekukan akun Wali Murid.

- Pembayaran sebagian (DP ≥ 70% dari total tagihan) diperbolehkan untuk
  menghadiri sesi hingga tanggal 10; setelah tanggal 11, akses diblokir
  hingga pelunasan penuh + denda diverifikasi Admin.

**D. Penguncian Slot Booking**

- Slot yang telah dikonfirmasi Admin berstatus Terkunci (Locked).

- Hanya Admin yang dapat membuka kembali slot terkunci; setiap pembukaan
  wajib disertai alasan dan dicatat di audit log.

**E. Minggu ke-5 dan Hari Libur Nasional**

- Minggu ke-5: Otomatis berstatus Non-Kredit. Slot dibuka untuk Kelas
  Teori (30 menit, tidak memotong kuota 4 pertemuan reguler).

- Jika libur nasional jatuh di minggu ke-5: Aturan libur nasional
  berlaku prioritas; Kelas Teori dibatalkan tanpa reschedule.

- Libur nasional di hari reguler: Sistem otomatis menjadwalkan ulang
  sesi ke hari yang sama pada minggu berikutnya dalam bulan yang sama.
  Jika tidak tersedia slot, sesi dicatat sebagai Sesi Teori Pengganti
  (tidak memotong kredit).

**F. Penyesuaian Tarif Kursus**

- Tarif kursus bervariasi berdasarkan kombinasi: instrumen, format kelas
  (Private/Semi-Private/Kelompok Kecil), dan level siswa.

- Tarif studio rehearsal: per jam, dapat dipesan tanpa konsultasi. Tarif
  studio recording: memerlukan konsultasi dan persetujuan Admin sebelum
  konfirmasi.

- Daftar tarif resmi dikelola sebagai data master oleh Admin dan Owner;
  tidak dapat diubah oleh pengguna lain.

**2.7 Lingkungan Pengoperasian**

**Tabel 2.4 --- Lingkungan Pengoperasian Sistem**

+-------------------------+-------------------------+--------------------------+
| **Kategori Lingkungan** | **Kondisi /             | **Implikasi Desain       |
|                         | Spesifikasi**           | Sistem**                 |
+:========================+:========================+:=========================+
| **Lingkungan Fisik & Ergonomis**                                             |
+-------------------------+-------------------------+--------------------------+
| Tingkat Kebisingan      | Studio musik bising     | Antarmuka mengandalkan   |
|                         | saat sesi berlangsung   | indikator visual kontras |
|                         |                         | & jelas; tidak           |
|                         |                         | bergantung pada          |
|                         |                         | notifikasi suara         |
+-------------------------+-------------------------+--------------------------+
| Mobilitas Pengguna      | Guru berpindah antar    | Responsif pada mode      |
|                         | ruang/berdiri saat      | mobile portrait; tombol  |
|                         | mengajar                | aksi minimal 44×44       |
|                         |                         | piksel untuk hindari     |
|                         |                         | salah ketuk              |
+-------------------------+-------------------------+--------------------------+
| Variasi Pencahayaan     | Ruang tunggu/admin      | Dukungan mode kontras    |
|                         | terang matahari hingga  | tinggi sesuai standar    |
|                         | redup                   | WCAG 2.1 Level AA        |
+-------------------------+-------------------------+--------------------------+
| **Kondisi Perangkat Klien & Platform**                                       |
+-------------------------+-------------------------+--------------------------+
| Admin & Owner           | Laptop/PC Windows 10+   | Optimasi layout desktop; |
|                         | atau macOS 12+,         | dukungan multi-window &  |
|                         | resolusi minimal        | export data              |
|                         | 1366×768                |                          |
+-------------------------+-------------------------+--------------------------+
| Guru                    | Smartphone Android 10+  | Fitur absensi dengan     |
|                         | / iOS 14+, kamera       | foto + geotagging;       |
|                         | belakang aktif, sensor  | validasi perangkat       |
|                         | GPS tersedia            | sebelum input            |
+-------------------------+-------------------------+--------------------------+
| Wali Murid/Siswa        | Smartphone Android 10+  | Antarmuka mobile-first;  |
|                         | / iOS 14+, akses via    | notifikasi push via      |
|                         | browser atau PWA        | web/app                  |
+-------------------------+-------------------------+--------------------------+
| Browser Wajib           | Chrome 90+, Firefox     | Browser lawas dialihkan  |
|                         | 88+, Safari 14+         | ke halaman               |
|                         |                         | kompatibilitas dengan    |
|                         |                         | peringatan fitur         |
|                         |                         | terbatas                 |
+-------------------------+-------------------------+--------------------------+
| **Kondisi Jaringan & Infrastruktur**                                         |
+-------------------------+-------------------------+--------------------------+
| Koneksi Studio          | WiFi stabil minimal 10  | Sinkronisasi real-time   |
|                         | Mbps down / 5 Mbps up   | untuk jadwal &           |
|                         |                         | pembayaran               |
+-------------------------+-------------------------+--------------------------+
| Koneksi Minimum         | 3G (min. 2 Mbps)        | Strategi caching lokal   |
|                         |                         | via IndexedDB untuk form |
|                         |                         | absensi & draft jadwal   |
+-------------------------+-------------------------+--------------------------+
| Lokasi Server           | Region Jakarta          | Kepatuhan data residency |
|                         | (Indonesia)             | sesuai UU PDP; latency   |
|                         |                         | rendah untuk pengguna    |
|                         |                         | lokal                    |
+-------------------------+-------------------------+--------------------------+
| Mode Offline            | Tidak ada toleransi     | Fitur baca jadwal &      |
|                         | offline penuh untuk     | riwayat kehadiran        |
|                         | absensi & penagihan     | tersedia offline         |
|                         |                         | terbatas setelah         |
|                         |                         | sinkronisasi terakhir    |
+-------------------------+-------------------------+--------------------------+
