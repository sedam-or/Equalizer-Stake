**PROGRAM STUDI REKAYASA KEAMANAN SIBER**

Mata Kuliah: Rekayasa Perangkat Lunak

**SOFTWARE REQUIREMENT SPECIFICATION (SRS)**

**EQUALIZER-STAKE**

Sistem Penjadwalan Kursus & Reservasi Studio Berbasis Web BMD Musik ---
Yogyakarta

**\[GAMBAR: Logo / Header Dokumen\]**

Disusun oleh

Juansyah Nursedam 2526214002

Nu\'man Adiansyah 2526214019

Ade Galih B. 2526214022

Habibie Septiarsya 2526214013

Tahun 2026 Versi Dokumen v2.0

**LEMBAR PERSETUJUAN**

Dokumen Software Requirements Specification (SRS) ini telah diperiksa
dan disetujui sebagai spesifikasi kebutuhan perangkat lunak sistem
Equalizer-Stake untuk BMD Musik, Yogyakarta.

  -------------------------- --------------------------------------------
  **Nama Sistem**            Equalizer-Stake

  **Versi Dokumen**          v2.0 (Revised)

  **Tanggal Persetujuan**    \...\... / \...\... / 2026

  **Status**                 ☐ Draft ☐ Review ☐ Approved
  -------------------------- --------------------------------------------

Dosen Pengampu

( \...\...\...\...\...\...\...\...\...\...\...\...\...\...\...\... )

**DAFTAR ISI**

\[Daftar Isi --- dibuat otomatis oleh Microsoft Word: klik kanan →
Update Field\]

**DAFTAR TABEL**

\[Daftar Tabel --- update otomatis via Word\]

**DAFTAR GAMBAR**

\[Daftar Gambar --- update otomatis via Word\]

**BAB 1 --- PENDAHULUAN**

**1.1 Tujuan Dokumen**

Dokumen Software Requirements Specification (SRS) ini disusun
berdasarkan standar ISO/IEC/IEEE 29148:2018 dan berfungsi sebagai
satu-satunya sumber acuan resmi bagi seluruh pemangku kepentingan sistem
Equalizer-Stake. Sistem ini adalah aplikasi penjadwalan kursus musik dan
reservasi studio berbasis web yang dirancang khusus untuk operasional
BMD Musik (BMD Music School), Yogyakarta. Tujuan utama dokumen ini
adalah memastikan semua pihak --- pemilik bisnis, pengelola operasional,
tim pengembang, penguji kualitas (QA), dan auditor kepatuhan ---
memiliki pemahaman yang sama, terukur, dan tidak ambigu mengenai:

- Perilaku sistem yang diharapkan pada setiap skenario penggunaan.

- Batasan teknis, bisnis, dan kepatuhan yang mengikat pengembangan.

- Standar kualitas yang wajib dipenuhi sebelum sistem dinyatakan siap
  produksi.

- Prosedur kepatuhan terhadap Undang-Undang Perlindungan Data Pribadi
  (UU PDP No. 27 Tahun 2022).

Dokumen ini menjadi dasar kontrak teknis antara pemilik produk dan tim
pengembang, serta acuan utama untuk fase desain, pengembangan, pengujian
penerimaan, dan audit kepatuhan data.

**1.1.1 Kriteria Keberhasilan Bisnis (Success Criteria)**

Dokumen ini menetapkan target kuantitatif bernilai bisnis di luar
indikator operasional teknis. Keberhasilan sistem Equalizer-Stake akan
diukur melalui pencapaian berikut dalam 12 bulan pasca-peluncuran MVP:

**Tabel 1.1 --- Kriteria Keberhasilan Bisnis**

  -----------------------------------------------------------------------
  **Indikator            **Target 12 Bulan Pasca-MVP**
  Keberhasilan**         
  ---------------------- ------------------------------------------------
  Efisiensi Biaya        Mengurangi biaya koordinasi dan administrasi
  Operasional            manual sebesar ≥30% melalui otomatisasi
                         penjadwalan, penagihan, dan notifikasi.

  Retensi Siswa          Meningkatkan tingkat retensi siswa aktif sebesar
                         ≥15% dengan mengurangi friksi administrasi,
                         menurunkan no-show ke ≤8%, dan meningkatkan
                         transparansi jadwal serta tagihan.

  Perputaran Kas (Cash   Mempercepat siklus penerimaan pembayaran dari
  Flow)                  rata-rata 14 hari (manual) menjadi ≤7 hari
                         melalui penerbitan tagihan otomatis, kode unik
                         identifikasi, dan notifikasi jatuh tempo
                         terukur.

  Kepatuhan & Risiko     Menurunkan insiden sengketa jadwal atau tagihan
                         sebesar ≥90% melalui audit log permanen,
                         validasi real-time, dan rekam jejak perubahan
                         yang tidak dapat dimanipulasi.
  -----------------------------------------------------------------------

**1.2 Ruang Lingkup Sistem**

Nama Sistem: Equalizer-Stake --- Sistem Penjadwalan Kursus & Reservasi
Studio Berbasis Web untuk BMD Musik.

**Pengguna Sistem:**

**Tabel 1.2 --- Pengguna Sistem**

  -----------------------------------------------------------------------
  **Peran**           **Deskripsi Singkat**
  ------------------- ---------------------------------------------------
  Owner (Pemilik)     Pengawas strategis; memantau laporan keuangan,
                      kehadiran, dan audit perubahan data.

  Admin Operasional   Pengelola harian; mengelola jadwal, akun,
                      pembayaran, dan notifikasi.

  Guru Musik          Pengajar; mengakses jadwal pribadi dan mengisi
                      absensi digital.

  Wali Murid          Pengguna utama; satu akun mengelola banyak siswa
                      (relasi 1 Wali : N Siswa).
  -----------------------------------------------------------------------

**Fungsi Utama Sistem**

1.  Penjadwalan kursus real-time dengan pencegahan konflik slot
    otomatis.

2.  Reservasi ruang studio untuk latihan (rehearsal) dan rekaman.

3.  Notifikasi otomatis melalui WhatsApp dan email (dengan sistem
    cadangan).

4.  Absensi digital oleh guru, dilengkapi validasi foto, lokasi GPS, dan
    waktu server.

5.  Penagihan bulanan otomatis, pelacakan pembayaran, dan penerapan
    denda keterlambatan.

6.  Pembekuan akun otomatis untuk murid yang menunggak melewati batas
    waktu.

7.  Pelacakan kelayakan ujian siswa berdasarkan rekam kehadiran.

8.  Laporan operasional untuk Owner dan Admin.

9.  Audit log permanen untuk seluruh perubahan data penting.

**Yang Tidak Termasuk dalam Sistem (Out-of-Scope)**

- Aplikasi mobile native (Android/iOS) --- sistem menggunakan PWA
  (Aplikasi Web yang bisa diakses seperti aplikasi HP).

- Integrasi payment gateway otomatis --- pembayaran diverifikasi secara
  manual oleh Admin pada fase MVP.

- Fitur kecerdasan buatan (AI/ML) atau analitik prediktif.

- Integrasi sistem akuntansi eksternal.

- Manajemen multi-cabang.

- Fitur streaming atau rekaman sesi kursus daring.

- Manajemen inventaris peralatan musik.

**1.2.1 Diagram Konteks**

Diagram berikut mendefinisikan batas sistem (system boundary) dan aliran
data makro antara Equalizer-Stake dengan entitas eksternal.

**\[GAMBAR: Diagram Konteks Sistem Equalizer-Stake\]**

**1.3 Definisi Istilah dan Glosarium**

**Tabel 1.3 --- Glosarium Istilah**

  -----------------------------------------------------------------------
  **Istilah**            **Padanan Bisnis / Penjelasan Operasional**
  ---------------------- ------------------------------------------------
  Admin                  Staf operasional BMD Musik yang mengelola
                         jadwal, akun, dan pembayaran sehari-hari.

  Owner                  Pemilik atau pimpinan BMD Musik; pengawas
                         strategis yang memantau laporan, tidak terlibat
                         input harian.

  Wali Murid             Orang tua atau wali resmi yang memiliki satu
                         akun untuk mengelola data semua anak yang
                         terdaftar (relasi 1:N).

  Siswa / Murid          Peserta kursus yang dijadwalkan dan dikelola
                         melalui akun Wali Murid.

  Slot                   Unit waktu kursus atau studio yang tersedia
                         untuk dipesan, dengan durasi tetap dan jeda
                         antar-sesi.

  Booking                Proses pemesanan slot oleh Wali Murid atau
                         Admin, dengan siklus status dari awal hingga
                         selesai.

  Rolling Credits        Sistem kredit sesi yang terikat pada kalender
                         reguler; 4 pertemuan per bulan, tidak bisa
                         dikumpulkan antarbulan kecuali aturan kompensasi
                         berlaku.

  Freeze (Pembekuan      Status akun murid yang diblokir aksesnya secara
  Akun)                  otomatis karena tagihan belum terlunasi melewati
                         batas waktu.

  Invoice                Tagihan bulanan yang diterbitkan sistem secara
                         otomatis setiap tanggal 5.

  Unique Code            Kode unik 3 digit (000--999) yang ditambahkan ke
                         nominal tagihan untuk memudahkan identifikasi
                         pembayaran manual.

  RBAC                   Sistem Kontrol Akses Berbasis Peran ---
                         membatasi hak akses pengguna sesuai peran yang
                         ditetapkan (Owner/Admin/Guru/Wali Murid).

  JWT (Token Sesi)       Token keamanan digital yang digunakan sistem
                         untuk memvalidasi identitas pengguna yang sedang
                         aktif, tanpa perlu menyimpan data sesi di
                         server.

  PWA                    Aplikasi Web yang Bisa Dipasang --- website yang
                         bisa diakses dan diinstal seperti aplikasi HP
                         tanpa perlu mengunduh dari app store.

  Concurrency Control    Penguncian data saat proses bersamaan ---
                         mekanisme yang mencegah dua pengguna memesan
                         slot yang sama di waktu yang sama.

  Idempotency            Sifat proses yang aman diulang --- jika terjadi
                         gangguan jaringan, pengiriman ulang data tidak
                         menghasilkan duplikasi booking atau pembayaran.

  Circuit Breaker        Sistem pemulihan otomatis saat layanan pihak
                         ketiga (WhatsApp/Email) mengalami gangguan,
                         mencegah kegagalan berantai.

  AES-256-GCM            Metode enkripsi data tingkat militer yang
                         digunakan untuk melindungi data pribadi saat
                         tersimpan di server.

  TLS 1.2+               Protokol keamanan yang mengenkripsi data saat
                         dikirim antara perangkat pengguna dan server
                         (seperti gembok pada koneksi internet).

  Audit Log              Catatan permanen seluruh perubahan data penting
                         di sistem, tidak dapat dihapus atau diubah,
                         disimpan minimal 7 tahun.

  Exam Eligibility       Status kelayakan ujian siswa berdasarkan rekam
                         kehadiran ≥90% dan maksimal 1 absen tanpa kabar
                         dalam 3 bulan terakhir.

  PII                    Informasi Pribadi Identifiable --- data yang
                         dapat mengidentifikasi seseorang, seperti nama,
                         nomor telepon, dan foto (dilindungi UU PDP).

  UU PDP                 Undang-Undang Nomor 27 Tahun 2022 tentang
                         Perlindungan Data Pribadi, yang mengatur hak dan
                         kewajiban dalam pengelolaan data warga negara
                         Indonesia.

  MVP                    Versi Minimum yang Dapat Diluncurkan --- versi
                         pertama sistem dengan fitur inti yang cukup
                         untuk mulai digunakan, dikembangkan dalam 10
                         minggu.
  -----------------------------------------------------------------------

**1.4 Referensi**

**Tabel 1.4 --- Referensi Dokumen**

  ----------------------------------------------------------------------------
   **No.**  **Sumber**                **Keterangan**
  --------- ------------------------- ----------------------------------------
      1     Website BMD Musik         Informasi profil, layanan, dan jadwal
                                      operasional BMD Music School Yogyakarta.

      2     Hasil Wawancara           Wawancara dengan Owner, Admin, Guru, dan
            Stakeholder               Wali Murid BMD Musik (Februari--Maret
                                      2026); dokumen lengkap di Lampiran 7.1.

      3     UU No. 27 Tahun 2022      Undang-Undang Perlindungan Data Pribadi
                                      Republik Indonesia acuan kepatuhan
                                      pengelolaan data pengguna.

      4     ISO/IEC/IEEE 29148:2018   Standar internasional untuk penyusunan
                                      kebutuhan perangkat lunak dan sistem.

      5     OWASP Top 10 (2021)       Daftar 10 risiko keamanan aplikasi web
                                      paling kritis --- acuan mitigasi
                                      keamanan sistem.

      6     ISO/IEC 27001:2022        Standar manajemen keamanan informasi ---
                                      acuan kontrol keamanan data.

      7     WCAG 2.1 Level A          Panduan aksesibilitas konten web ---
                                      acuan kemudahan penggunaan antarmuka.
  ----------------------------------------------------------------------------

**1.5 Pengelolaan Perubahan & Versi Dokumen**

Dokumen ini dikelola menggunakan skema penomoran versi semantik
(semantic versioning): vX.Y, di mana X adalah nomor versi utama
(perubahan besar pada ruang lingkup atau arsitektur) dan Y adalah nomor
versi minor (penambahan atau koreksi kebutuhan tanpa mengubah arsitektur
dasar).

**Prosedur Revisi:**

- Setiap permintaan perubahan wajib diajukan secara tertulis oleh
  pemohon kepada ketua kelompok atau dosen pengampu.

- Perubahan minor (v1.Y) dapat disetujui oleh ketua kelompok; perubahan
  major (v2.0+) wajib mendapat persetujuan dosen pengampu.

- Seluruh perubahan dicatat dalam tabel log di bawah dan di Matriks
  Traceability BAB 6.

**Change Control Checklist (Kriteria Objektif)**

Untuk membedakan perubahan \"Minor\" (v1.Y) dan \"Major\" (v2.0+),
gunakan kriteria berikut:

**Tabel 1.5 --- Change Control Checklist**

  ---------------------------------------------------------------------------
  **Kategori**     **Kriteria**                                    **Form**
  ---------------- ----------------------------------------------- ----------
  Minor (v1.Y)     Perubahan yang hanya memengaruhi 1 tabel        CR Form
                   database, atau penambahan field opsional, atau  
                   perbaikan typo/format, atau perubahan UI yang   
                   tidak mengubah logika bisnis.                   

  Major (v2.0+)    Perubahan yang memengaruhi 3 tabel atau lebih,  CR Form +
                   atau perubahan arsitektur sistem, atau          Dosen
                   perubahan logika bisnis inti (siklus tagihan,   
                   denda, pembekuan), atau perubahan integrasi     
                   dengan layanan pihak ketiga.                    
  ---------------------------------------------------------------------------

Setiap perubahan wajib diisi formulir Change Request yang mencakup: ID
Perubahan, Deskripsi, Dampak, Estimasi Effort, dan Persetujuan (Ketua
Kelompok untuk Minor, Dosen Pengampu untuk Major).

**Tabel 1.6 --- Log Versi Dokumen**

  ------------------------------------------------------------------------------
  **Versi**   **Tanggal**   **Pemohon**    **Alasan Perubahan**   **Status**
  ----------- ------------- -------------- ---------------------- --------------
  v0.1        Januari 2026  Kelompok 7     Draft awal ---         Draft
                                           struktur dokumen dan   
                                           konteks bisnis         

  v0.2        Februari 2026 Kelompok 7     Penambahan hasil       Review
                                           wawancara stakeholder  Internal
                                           dan validasi           
                                           requirement            

  v1.0        Maret 2026    Kelompok 7     Dokumen lengkap BAB    Approved
                                           1--7 siap untuk review (Baseline)
                                           akademis dan validasi  
                                           teknis                 

  v2.0        Juni 2026     Kelompok 7     Revisi komprehensif: 7 Approved
                                           perubahan diterapkan   (Revised)
                                           (Change Control, Cloud 
                                           Infrastructure, FR     
                                           tambahan, Diagram UML, 
                                           ERD Soft Delete, NFR   
                                           Security Compliance    
                                           Matrix, TDB            
                                           finalisasi)            
  ------------------------------------------------------------------------------
