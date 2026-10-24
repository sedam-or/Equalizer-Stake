**BAB 3 --- KEBUTUHAN SPESIFIK**

**3.1 Functional Requirement**

Setiap kebutuhan fungsional di bawah memuat: kode unik, nama fitur,
spesifikasi perilaku terukur, aktor utama, batasan logika bisnis,
kriteria penerimaan dalam format Given-When-Then, Tahapan Alur (Flow
Steps), dan penanganan error. Kode status: Approved = kebutuhan final;
Proposed = dalam evaluasi.

**FR-01 Login dan Autentikasi Berbasis Peran (RBAC) ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Owner, Admin, Instructor, Guardian

 **Pemicu** Pengguna membuka URL sistem

 **Modul** Autentikasi

 **Verifikasi** Test

 **UC Linked** UC-01
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall menyediakan halaman login tunggal yang mengarahkan pengguna
ke dashboard sesuai perannya (Owner, Admin, Instructor, Guardian) setelah
autentikasi berhasil. Token sesi (JWT) diterbitkan dengan masa berlaku
sesuai batas waktu per peran.

***Batasan Logika***

- Percobaan login gagal maksimal 5 kali dalam 15 menit sebelum akun
 dikunci sementara.

- Batas waktu sesi tidak aktif: Owner 30 menit, Admin 15 menit, Instructor 30
 menit, Guardian 60 menit.

- Setelah batas waktu, pengguna diminta login ulang; draft tersimpan
 dipulihkan otomatis.

**Fitur Kirim Password Awal (Onboarding) --- Perubahan v2.0:**

Ketika Admin membuat akun baru untuk Instructor atau Guardian, sistem wajib
mengirimkan email berisi:

- Link akses sistem

- Username (sesuai yang dibuat Admin)

- Password sementara (auto-generate, minimal 8 karakter, alphanumeric)

- Instruksi untuk mengubah password setelah login pertama

Password sementara berlaku 7 hari; jika tidak digunakan, Admin dapat
mengirim ulang melalui dashboard.

**Fitur Ubah Password (di dashboard) --- Perubahan v2.0:**

Setelah login, setiap pengguna (kecuali Owner) wajib dapat mengubah
password sendiri melalui menu \"Pengaturan Akun\" atau \"Profil\".
Perubahan password dicatat di audit log.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Pengguna memiliki akun aktif dan kredensial yang benar

 When Pengguna memasukkan username dan password lalu menekan
 tombol Masuk

 Then Sistem menerbitkan token sesi, mencatat waktu login di
 audit log, dan menampilkan dashboard sesuai peran dalam ≤
 2 detik
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Buka URL sistem

2\. Masukkan username

3\. Masukkan password

4\. Validasi kredensial ≤ 2 detik

5\. Terbitkan JWT sesuai peran

6\. Catat login di audit log

7\. Tampilkan dashboard sesuai peran

**Jalur Alternatif --- Kredensial Salah:**

Masukkan username/password salah → Increment fail counter → Tampilkan
pesan \"Kredensial tidak valid\" → Ulangi login (maks. 5x)

**Jalur Alternatif --- Percobaan ke-5 Gagal (Brute Force):**

Percobaan login ke-5 gagal → Kunci akun 15 menit → Kirim notifikasi
email ke Admin → Tampilkan pesan waktu tunggu

**Jalur Alternatif --- Akun Nonaktif:**

Login dengan akun nonaktif → Tampilkan \"Akun tidak aktif. Hubungi
Admin\" → Hentikan proses, tidak terbitkan JWT

**Jalur Alternatif --- Login Pertama (Password Sementara):**

Pengguna baru login dengan password sementara → Sistem deteksi flag
first_login → Redirect ke halaman wajib ubah password → Password baru
disimpan, flag direset, audit log dicatat

***Penanganan Error & Cadangan***

- Kredensial salah: Tampilkan \"Username atau password tidak valid\"
 tanpa menyebutkan mana yang salah.

- Percobaan ke-5 gagal: Kunci akun 15 menit, kirim notifikasi email ke
 Admin.

- Akun nonaktif: Tampilkan pesan \"Akun Anda tidak aktif. Hubungi Admin
 Equalizer-Stake.\"

- Password sementara kedaluwarsa (\>7 hari): Tampilkan pesan \"Link
 telah kedaluwarsa. Hubungi Admin untuk mengirim ulang.\"

- Cadangan: Jika layanan token sesi gangguan, tampilkan pesan
 pemeliharaan dan catat insiden di log sistem.

**FR-02 Penjadwalan Course dengan Validasi Konflik Real-Time ●
Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Admin

 **Pemicu** Admin menyimpan jadwal baru

 **Modul** Penjadwalan

 **Verifikasi** Test

 **UC Linked** UC-02
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall memvalidasi ketersediaan slot (instructor, ruangan, waktu) secara
real-time saat Admin menyimpan jadwal baru atau melakukan perubahan
jadwal. Konflik slot tidak boleh lolos ke penyimpanan data dalam keadaan
apapun.

***Batasan Logika***

- Validasi mencakup: waktu sesi + jeda 10 menit tidak tumpang tindih di
 ruangan yang sama; instructor tidak dijadwalkan dua sesi bersamaan.

- Jadwal hanya dalam jam operasional (Senin--Jumat 14.30--17.00, Sabtu
 08.00--12.00).

- Tidak ada penjadwalan di hari libur nasional atau hari Minggu.

- Concurrency control wajib: mencegah dua Admin menyimpan jadwal bentrok
 secara bersamaan.

**Logika Kredit untuk Pembatalan Sesi --- Perubahan v2.0:**

**Pembatalan oleh Admin (sesi berstatus Locked):** Kredit student wajib
dikembalikan penuh (refund kredit) dan dicatat di audit log disertai
alasan pembatalan. Berlaku untuk pembatalan akibat error penjadwalan,
duplikasi, atau konflik.

**Pembatalan oleh Guardian (notifikasi ≥ 24 jam sebelum sesi):**
Kredit dikembalikan 50%.

**Pembatalan oleh Guardian (notifikasi \< 24 jam sebelum sesi):**
Kredit hangus (tidak dikembalikan).

Semua pembatalan wajib disertai alasan dan dicatat di audit log.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Admin memilih slot waktu, instructor, dan ruangan tertentu

 When Admin menekan tombol Simpan Jadwal

 Then Sistem memvalidasi ketersediaan ≤ 1 detik; jika tersedia,
 jadwal tersimpan dan notifikasi dikirim ke Instructor; jika
 konflik, tampilkan daftar slot alternatif yang tersedia
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Buka form jadwal baru

2\. Pilih student, instructor, ruangan

3\. Pilih tanggal & waktu sesi

4\. Validasi konflik real-time ≤ 1 detik

5\. Tampilkan konfirmasi \"Slot tersedia\"

6\. Klik \"Simpan Jadwal\"

7\. Simpan status Locked + catat audit log

8\. Kirim notifikasi ke Instructor & Guardian

**Jalur Alternatif --- Konflik Slot Terdeteksi:**

Validasi mendeteksi konflik → Tampilkan detail konflik → Tampilkan 3
slot alternatif terdekat → Admin pilih alternatif atau batalkan

**Jalur Alternatif --- Concurrency (2 Admin bersamaan):**

Admin kedua coba simpan slot sama → Concurrency lock aktif → Pesan
\"Data baru saja diperbarui, periksa kembali\" → Admin kedua pilih ulang
slot

**Jalur Alternatif --- Pembatalan Sesi Locked oleh Admin:**

Admin pilih \"Batalkan Sesi\" pada sesi berstatus Locked → Sistem
wajibkan input alasan → Kredit student dikembalikan penuh → Catat di audit
log → Notifikasi ke Guardian & Instructor

**Jalur Alternatif --- Pembatalan oleh Guardian (≥ 24 jam):**

Guardian ajukan pembatalan → Sistem cek selisih waktu ≥ 24 jam →
Kredit dikembalikan 50% → Catat di audit log

**Jalur Alternatif --- Pembatalan oleh Guardian (\< 24 jam):**

Guardian ajukan pembatalan → Sistem cek selisih waktu \< 24 jam →
Tampilkan konfirmasi \"Kredit akan hangus\" → Jika setuju, kredit hangus
→ Catat di audit log

***Penanganan Error & Cadangan***

- Konflik slot: Tampilkan detail konflik (\"Ruang A sudah digunakan
 \[Instructor X\] pukul 15.00--16.00\") + 3 slot alternatif.

- Concurrency: \"Data jadwal baru saja diperbarui. Silakan periksa
 kembali sebelum menyimpan.\"

- Pembatalan tanpa alasan: Sistem menolak, field alasan wajib diisi
 sebelum konfirmasi.

- Cadangan: Draft jadwal disimpan lokal setiap 30 detik; dapat
 dipulihkan saat kembali online.

**FR-03 Penerbitan Tagihan Otomatis dan Kode Unik ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Sistem (otomatis); Admin (verifikasi)

 **Pemicu** Tanggal 5 pukul 00:00

 **Modul** Penagihan

 **Verifikasi** Test

 **UC Linked** UC-05
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall menerbitkan tagihan secara otomatis kepada seluruh Wali
Murid aktif setiap tanggal 5 pukul 00:00. Setiap tagihan dilengkapi kode
unik 3 digit (000--999) yang ditambahkan ke nominalnya untuk memudahkan
identifikasi pembayaran manual.

***Batasan Logika***

- Kode unik tidak boleh duplikat untuk tagihan aktif di bulan yang sama.

- Jika terjadi duplikasi: coba kode baru maks. 3 kali; jika masih gagal,
 tandai untuk assignment manual Admin.

- Satu tagihan per Guardian mencakup seluruh student yang dikelola
 (relasi 1:N).

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Tanggal sistem menunjukkan tanggal 5 pukul 00:00

 When Proses penagihan otomatis berjalan

 Then Tagihan terbit untuk semua akun Guardian aktif dalam ≤ 5
 menit, notifikasi dikirim ke WhatsApp/email, setiap
 tagihan memiliki kode unik 3 digit yang tidak duplikat
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Cron job aktif tanggal 5 pukul 00:00

2\. Ambil daftar Guardian aktif

3\. Hitung total biaya per Guardian (semua student)

4\. Generate kode unik 3 digit (000--999)

5\. Cek duplikasi kode (maks. 3x percobaan)

6\. Simpan invoice ke database

7\. Kirim notifikasi WhatsApp + email ke Guardian

**Jalur Alternatif --- Kode Unik Bentrok (setelah 3x percobaan):**

Generate kode gagal 3x berturut → Tandai invoice \"Perlu Kode Manual\" →
Kirim notifikasi ke Admin untuk assignment manual

**Jalur Alternatif --- Notifikasi WhatsApp Gagal:**

WhatsApp gagal terkirim → Retry 3x (10/20/40 detik) → Fallback ke email
→ Catat status di log notifikasi

***Penanganan Error & Cadangan***

- Kode unik bentrok 3x: Status \"Perlu Kode Manual\", Admin menerima
 notifikasi.

- Penagihan gagal parsial: Catat tagihan gagal, retry dalam 30 menit;
 Admin diberitahu via dashboard jika masih gagal.

- Cadangan notifikasi: Jika WhatsApp gagal, otomatis kirim via email.

**FR-04 Penerapan Denda Otomatis dan Pembekuan Akun ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Sistem (otomatis); Admin (unfreeze)

 **Pemicu** Tanggal 11 pukul 00:00, tagihan belum lunas

 **Modul** Penagihan

 **Verifikasi** Test

 **UC Linked** UC-07
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall secara otomatis menerapkan denda flat Rp 100.000 dan
membekukan akun Guardian yang tagihan bulannya belum terlunasi pada
tanggal 11 pukul 00:00.

***Batasan Logika***

- Pembekuan memblokir seluruh akses student di bawah Guardian tersebut.

- DP ≥ 70% mengizinkan kehadiran hingga tanggal 10; setelah tanggal 11
 tetap diblokir.

- State machine: unpaid → partial_paid → overdue → frozen.

- Pembukaan akun hanya oleh Admin setelah verifikasi pelunasan penuh.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Tagihan Guardian belum lunas pada tanggal 11 pukul 00:00

 When Proses otomatis berjalan

 Then Denda Rp 100.000 ditambahkan, akun dibekukan, notifikasi
 dikirim ke Guardian (WhatsApp + email), status tercatat
 di audit log
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Cron job aktif tanggal 11 pukul 00:00

2\. Cek status tagihan semua Guardian

3\. Tambahkan denda Rp 100.000 ke tagihan

4\. Ubah status akun → Frozen

5\. Catat di audit log

6\. Kirim notifikasi WhatsApp + email ke Guardian

**Jalur Alternatif --- Pembukaan Akun (Unfreeze) setelah Pelunasan:**

Guardian transfer (pokok + denda) → Kirim bukti ke Admin → Admin
verifikasi → Ubah status → Aktif (Paid) → Catat audit log → Kirim
konfirmasi ke Guardian

***Penanganan Error & Cadangan***

- Proses otomatis gagal: Catat kegagalan, Admin menerima peringatan di
 dashboard, proses dapat dijalankan ulang manual.

- Guardian mengklaim sudah bayar: Admin dapat menangguhkan pembekuan
 sementara sambil verifikasi, dengan catatan di audit log.

**FR-05 Absensi Digital dengan Validasi Foto, GPS, dan Waktu ●
Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Instructor; Admin (verifikasi)

 **Pemicu** Instructor membuka form absensi sesi aktif

 **Modul** Kehadiran

 **Verifikasi** Test

 **UC Linked** UC-03
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall memvalidasi tiga aspek secara bersamaan: (a) waktu
berdasarkan jam server, (b) lokasi GPS dalam radius 150 meter dari
studio (koordinat: -7.7265, 110.4162) --- radius final pasca uji
lapangan, (c) wajah student terlihat minimal 30% dari area foto. Gagal
satu aspek → status pending_verification, bukan langsung ditolak.

***Batasan Logika***

- Tiga validasi bersifat kumulatif; satu gagal → pending_verification,
 Admin menerima notifikasi.

- Instructor tidak dapat mengubah data absensi setelah dikirim; hanya Admin
 yang dapat koreksi (dengan audit log).

- Status PendingVerification wajib direview Admin ≤ 2 jam.

**Validasi Wajah Client-Side --- Perubahan v2.0:**

Validasi proporsi wajah (≥ 30% frame) dilakukan sebelum foto diunggah ke
server, menggunakan library JavaScript di sisi client (misal:
face-api.js atau MediaPipe Face Detection). Tujuan:

- Mengurangi beban server --- mencegah bottleneck di cloud hosting

- Memberikan umpan balik instan kepada Instructor (≤ 1 detik)

- Mengurangi penggunaan bandwidth --- hanya foto yang lolos validasi
 yang dikirim ke server

Server tetap melakukan validasi redundant (verifikasi ulang) untuk
keamanan, namun proses utama tetap di client.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Instructor berada di lokasi studio dan sesi course sedang
 berlangsung

 When Instructor mengunggah foto student dan menekan tombol Konfirmasi
 Absensi

 Then Sistem memvalidasi waktu, GPS, dan foto dalam ≤ 5 detik;
 jika semua valid → status \"Hadir\"; jika ada validasi
 gagal → status \"Menunggu Verifikasi\" dan Admin
 diberitahu
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Buka menu \"Absensi Hari Ini\"

2\. Pilih sesi aktif

3\. Ambil foto student via kamera

4\. Client-side: validasi proporsi wajah ≥ 30% (face-api.js / MediaPipe)
≤ 1 detik

5\. Jika wajah tidak lolos → tampilkan panduan retake, TIDAK kirim ke
server

6\. Jika wajah lolos → ambil koordinat GPS perangkat

7\. Kirim (foto + GPS + timestamp) ke Backend

8\. Backend validasi GPS radius 150m + timestamp server

9\. Simpan status \"Hadir\" + potong kredit

10\. Kirim notifikasi hadir ke Guardian

**Jalur Alternatif --- Wajah Tidak Lolos Validasi Client-Side:**

face-api.js gagal deteksi wajah ≥ 30% → Tampilkan panduan \"Pastikan
wajah mengisi minimal 1/3 area foto\" → Instructor ambil ulang foto → Tidak
ada request ke server

**Jalur Alternatif --- GPS Luar Radius (Validasi Server):**

GPS/foto/waktu gagal validasi server → Status → \"Menunggu Verifikasi\"
→ Notifikasi ke Admin untuk review manual ≤ 2 jam

**Jalur Alternatif --- Koneksi Terputus Saat Upload:**

Internet terputus → Simpan foto di perangkat lokal → Auto-upload saat
koneksi kembali (≤ 5 menit) → Timestamp server saat upload diterima

***Penanganan Error & Cadangan***

- Wajah tidak terdeteksi (client-side): \"Wajah student belum terlihat
 jelas. Pastikan wajah mengisi minimal sepertiga area foto.\" --- tidak
 ada request ke server.

- GPS luar radius 150m: \"Lokasi Anda berada di luar area studio.
 Absensi perlu verifikasi Admin.\"

- Koneksi terputus: Foto tersimpan di perangkat, upload otomatis saat
 kembali online.

- Server validasi wajah gagal (redundant check): Catat sebagai
 PendingVerification; tidak langsung ditolak.

**FR-06 Notifikasi Multi-Saluran dengan Sistem Cadangan ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Sistem (otomatis); Admin (konfigurasi)

 **Pemicu** Event bisnis (jadwal, pembayaran, freeze)

 **Modul** Notifikasi

 **Verifikasi** Test

 **UC Linked** UC-04
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall mengirimkan notifikasi melalui WhatsApp (non-kritis) dan
email (kritis) dengan mekanisme percobaan ulang otomatis dan peralihan
saluran jika pengiriman gagal. Semua notifikasi dikirim ke Guardian,
bukan per student.

***Batasan Logika***

- Kritis (konfirmasi pembayaran, freeze/unfreeze): email utama, WhatsApp
 cadangan.

- Non-kritis (reminder 1 jam sebelum sesi): WhatsApp utama, email
 cadangan.

- Retry 3x: jeda 10 detik → 20 detik → 40 detik sebelum fallback ke
 saluran cadangan.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Pemicu notifikasi terjadi (contoh: 1 jam sebelum sesi)

 When Sistem memproses antrian notifikasi

 Then Notifikasi terkirim ke Guardian dalam ≤ 15 detik; status
 pengiriman tercatat di log yang dapat dilihat Admin
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Event pemicu aktif (misal: H-1 jam sesi)

2\. Generate pesan notifikasi

3\. Kirim via WhatsApp Gateway

4\. Terima konfirmasi delivered

5\. Catat status \"Terkirim\" di log notifikasi

**Jalur Alternatif --- WhatsApp Gagal (Fallback Email):**

WhatsApp gagal → Retry ke-1 (10 dtk) → Retry ke-2 (20 dtk) → Retry ke-3
(40 dtk) → Fallback kirim via email → Catat \"Fallback ke Email\" di log

**Jalur Alternatif --- Email Gagal Total:**

Semua saluran gagal → Peringatan di dashboard Admin → Admin kirim ulang
manual

***Penanganan Error & Cadangan***

- WhatsApp gagal 3x: Otomatis kirim via email, catat \"Fallback ke
 Email\" di log.

- Email gagal: Admin menerima peringatan dashboard; notifikasi dapat
 dikirim ulang manual.

- Gateway tidak tersedia: Blokir notifikasi non-kritis; alihkan semua
 komunikasi kritis ke email.

**FR-07 Pelacakan Kelayakan Ujian Student ● Proposed**

 ---------------- ------------------------------------------------------
 **Aktor** Sistem (otomatis); Admin, Owner

 **Pemicu** Admin/Owner membuka laporan kelayakan

 **Modul** Kehadiran

 **Verifikasi** Analysis

 **UC Linked** UC-09
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall secara otomatis menghitung dan menampilkan status kelayakan
ujian setiap student berdasarkan rekam kehadiran dalam 3 bulan terakhir.
Kriteria \"Siap Ujian\": akun aktif + kehadiran ≥ 90% + absen tanpa
kabar ≤ 1 kali.

***Batasan Logika***

- Semua tiga kondisi harus terpenuhi sekaligus; satu gagal → \"Belum
 Siap Ujian\" dengan alasan tampil.

- Student baru (data \< 3 bulan): ditampilkan dengan catatan \"Data
 kehadiran belum lengkap (X bulan)\".

- Absensi berstatus PendingVerification: status kelayakan sementara
 dengan tanda peringatan.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Admin atau Owner mengakses laporan kelayakan ujian

 When Halaman laporan dibuka

 Then Sistem menampilkan daftar student beserta status \"Siap
 Ujian\"/\"Belum Siap Ujian\" dengan persentase kehadiran
 dan jumlah absen tanpa kabar, diperbarui real-time
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Admin/Owner buka laporan kelayakan ujian

2\. Ambil data kehadiran 3 bulan terakhir per student

3\. Cek status akun (aktif/beku)

4\. Hitung % kehadiran per student

5\. Hitung jumlah absen tanpa kabar

6\. Evaluasi semua 3 kondisi

7\. Tampilkan status \"Siap Ujian\" / \"Belum Siap\" dengan detail
alasan

***Penanganan Error & Cadangan***

- Student baru (\< 3 bulan): Tampilkan \"Data kehadiran belum lengkap (X
 bulan)\", hitung dari data tersedia.

- Absensi PendingVerification: Tampilkan status sementara dengan tanda
 peringatan.

**FR-08 Penyimpanan Draft Otomatis dan Pemulihan Sesi ● Proposed**

 ---------------- ------------------------------------------------------
 **Aktor** Admin, Instructor

 **Pemicu** Sesi berakhir saat pengisian form aktif

 **Modul** Penjadwalan / Umum

 **Verifikasi** Test

 **UC Linked** UC-13
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall menyimpan draft input pengguna (jadwal, tagihan, formulir)
secara otomatis setiap 30 detik. Jika sesi berakhir karena batas waktu
tidak aktif, draft dipulihkan otomatis setelah pengguna login kembali
tanpa kehilangan data \> 1 menit terakhir.

***Batasan Logika***

- Draft disimpan di penyimpanan lokal perangkat + sinkronisasi ke server
 saat ada jaringan.

- Draft permanen (setelah klik \"Simpan\") tidak dapat dikembalikan ke
 draft.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Admin sedang mengisi formulir jadwal \> 30 detik dan sesi
 berakhir karena tidak aktif

 When Admin login kembali

 Then Sistem menampilkan \"Draft sebelumnya ditemukan.
 Pulihkan?\" dan mengembalikan semua data tanpa kehilangan
 data \> 1 menit terakhir
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Admin mengisi form

2\. Auto-save lokal setiap 30 detik

3\. Sesi timeout karena tidak aktif

4\. Admin login kembali

5\. Sistem deteksi draft tersimpan

6\. Tampilkan prompt \"Pulihkan draft?\"

7\. Admin klik \"Pulihkan\"

8\. Form terisi kembali dengan data draft

***Penanganan Error & Cadangan***

- Penyimpanan lokal penuh: Beri peringatan, minta pengguna membersihkan
 cache browser.

- Sinkronisasi server gagal: Draft tetap tersimpan lokal; tanda \"Belum
 Tersinkronisasi\" di antarmuka.

**FR-09 Prosedur Hak Penghapusan Data Pribadi (UU PDP Pasal 19) ●
Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Guardian/Instructor (pengaju); Admin (verifikator)

 **Pemicu** Guardian/Instructor mengajukan permintaan hapus data

 **Modul** Kepatuhan / Governance

 **Verifikasi** Inspection

 **UC Linked** UC-12
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall menyediakan mekanisme pengajuan permintaan penghapusan data
pribadi sesuai UU PDP Pasal 19. Setelah Admin menyetujui: data PII
dianonimkan, foto absensi dihapus permanen, metadata audit tetap 7 tahun
dalam bentuk anonim.

***Batasan Logika***

- Data PII dianonimkan dengan pola REDACTED-\[kode hash\].

- Permintaan ditolak jika akun masih memiliki tagihan aktif; alasan
 wajib dikirim ke pemohon.

- Seluruh proses tercatat di audit log dengan kategori \"Tindakan
 Kepatuhan\".

- Eksekusi penghapusan ≤ 14 hari kerja setelah persetujuan Admin.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Guardian mengajukan permintaan penghapusan data melalui
 formulir sistem

 When Admin memverifikasi identitas dan menyetujui permintaan

 Then Sistem mengeksekusi anonimisasi PII, menghapus foto,
 mempertahankan metadata anonim di audit log, dan
 mengirimkan konfirmasi ke Guardian dalam ≤ 14 hari kerja
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Guardian isi formulir hapus data

2\. Sistem cek kelayakan (tidak ada tagihan aktif)

3\. Notifikasi ke Admin untuk review

4\. Admin verifikasi identitas pemohon

5\. Admin setujui permintaan

6\. Anonimisasi data PII → REDACTED-\[hash\]

7\. Hapus permanen foto absensi dari server

8\. Catat di audit log kategori \"Tindakan Kepatuhan\"

9\. Kirim konfirmasi ke Guardian (≤ 14 hari kerja)

**Jalur Alternatif --- Permintaan Ditolak (Tagihan Aktif):**

Tagihan aktif belum lunas terdeteksi → Generate notifikasi penolakan +
alasan → Kirim ke Guardian dengan panduan langkah selanjutnya

***Penanganan Error & Cadangan***

- Ditolak (tagihan aktif): Kirim notifikasi penolakan + alasan + panduan
 langkah selanjutnya.

- Penghapusan foto gagal teknis: Catat insiden di log, eskalasi ke tim
 teknis, informasikan Admin dengan tenggat pemulihan.

**FR-10 Manajemen Akun Multi-Student oleh Guardian ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Guardian, Admin

 **Pemicu** Guardian masuk ke dashboard

 **Modul** Manajemen Akun

 **Verifikasi** Test

 **UC Linked** UC-08
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall memungkinkan satu akun Guardian mengelola informasi,
jadwal, tagihan, dan riwayat kehadiran untuk banyak student (relasi 1
Guardian : N Student) melalui antarmuka terpadu dengan switching antar
profil student.

***Batasan Logika***

- Penambahan student ke akun Guardian hanya oleh Admin (setelah
 pendaftaran + audisi level).

- Satu invoice mencakup semua student di bawah satu Guardian.

- Pembekuan akun Guardian berlaku untuk semua student di bawahnya.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Guardian memiliki dua anak yang terdaftar di Equalizer-Stake

 When Guardian masuk ke dashboard

 Then Sistem menampilkan jadwal, status kehadiran, dan tagihan
 kedua student dalam satu tampilan terpadu, dengan kemampuan
 beralih antar profil student
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Guardian login ke sistem

2\. Load daftar semua student terdaftar di akun ini

3\. Tampilkan dashboard terpadu (jadwal + kehadiran + tagihan)

4\. Guardian klik nama student untuk switch profil

5\. Dashboard refresh menampilkan data student yang dipilih

**Jalur Alternatif --- Salah Satu Student Akun Beku:**

Student A aktif, Student B beku → Dashboard tampilkan status per student
(badge berbeda) → Notifikasi spesifik mengenai student yang akunnya beku

***Penanganan Error & Cadangan***

- Satu student beku, satu aktif: Dashboard tampilkan status per student
 dengan notifikasi khusus mengenai yang beku.

**FR-11 Audit Log Permanen dan Tidak Dapat Diubah (Immutable) ●
Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Sistem (pencatat); Owner, Admin (pembaca)

 **Pemicu** Setiap perubahan data kritis

 **Modul** Audit / Governance

 **Verifikasi** Inspection

 **UC Linked** UC-11
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall mencatat seluruh perubahan data kritis ke dalam audit log
yang bersifat permanen dan tidak dapat diubah atau dihapus oleh siapapun
termasuk Admin. Jika pencatatan log gagal, perubahan data di-rollback.

***Batasan Logika***

- Setiap entri memuat: timestamp server, ID pengguna pelaku, tindakan,
 before state, after state.

- Disimpan 7 tahun; data PII dianonimkan setelah 2 tahun.

- Owner: akses seluruh log; Admin: akses log operasional (bukan log
 keamanan khusus Owner).

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Admin mengubah jadwal sesi yang sudah terkunci

 When Perubahan disimpan

 Then Sistem membuat entri audit log yang mencatat waktu
 perubahan, ID Admin, jadwal sebelum dan sesudah, dan
 alasan Admin --- entri tidak dapat diedit atau dihapus
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Pengguna melakukan aksi perubahan data kritis

2\. Sistem capture before state

3\. Proses perubahan data

4\. Tulis entri audit log (timestamp, ID aktor, before, after)

5\. Konfirmasi log berhasil disimpan

6\. Perubahan data dikonfirmasi ke pengguna

**Jalur Alternatif --- Pencatatan Log Gagal (Rollback):**

Penulisan audit log gagal → Rollback perubahan data ke before state →
Tampilkan error ke pengguna → Catat insiden di log sistem terpisah

***Penanganan Error & Cadangan***

- Pencatatan log gagal: Perubahan data di-rollback; operasi dikembalikan
 ke before state.

- Kapasitas storage \> 80%: Kirim peringatan otomatis ke Admin dan
 Owner.

**FR-12 Pengelolaan Libur Nasional dan Minggu ke-5 ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Admin (konfigurasi); Sistem (eksekusi)

 **Pemicu** Kalender memuat hari libur / minggu ke-5

 **Modul** Penjadwalan

 **Verifikasi** Test

 **UC Linked** UC-05
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall mengelola penyesuaian jadwal secara otomatis untuk hari
libur nasional dan minggu ke-5 sesuai aturan bisnis Equalizer-Stake:
reschedule otomatis, Kelas Teori Non-Kredit di minggu ke-5, dan
prioritas libur nasional di atas Kelas Teori.

***Batasan Logika***

- Daftar libur nasional diinput Admin di awal tahun ajaran.

- Sesi di hari libur → dipindah ke minggu berikutnya dalam bulan yang
 sama; jika tidak ada slot → Sesi Teori Pengganti.

- Minggu ke-5 → otomatis Non-Kredit; slot Kelas Teori 30 menit dibuka.

- Libur nasional di minggu ke-5 → libur nasional diprioritaskan; Kelas
 Teori dibatalkan.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Kalender memuat hari libur nasional pada hari course
 reguler

 When Sistem memproses kalender bulanan

 Then Jadwal sesi dipindah ke hari yang sama minggu berikutnya;
 Guardian dan Instructor menerima notifikasi; kredit student
 tidak terpotong untuk sesi yang dipindah
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Sistem deteksi hari course = hari libur nasional

2\. Cari slot tersedia minggu berikutnya (hari yang sama)

3\. Pindahkan jadwal ke slot baru

4\. Kredit student tidak dipotong

5\. Kirim notifikasi perubahan jadwal ke Instructor & Guardian

**Jalur Alternatif --- Tidak Ada Slot Reschedule:**

Slot tersedia tidak ditemukan dalam bulan yang sama → Catat otomatis
sebagai Sesi Teori Pengganti → Admin diberitahu untuk konfirmasi manual

***Penanganan Error & Cadangan***

- Tidak ada slot reschedule: Otomatis jadi Sesi Teori Pengganti; Admin
 diberitahu.

- Konflik reschedule: Sistem tampilkan konflik, minta Admin pilih slot
 alternatif manual.

**FR-13 Dashboard Monitoring untuk Owner (Read-Only) ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Owner

 **Pemicu** Owner membuka dashboard monitoring

 **Modul** Pelaporan

 **Verifikasi** Demonstration

 **UC Linked** UC-10
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall menyediakan dashboard monitoring yang menampilkan ringkasan
operasional Equalizer-Stake secara real-time untuk Owner. Owner hanya dapat
melihat dan mengekspor; tidak dapat mengubah data operasional apapun.

***Batasan Logika***

- Dashboard menampilkan: total pendapatan berjalan, tingkat kehadiran,
 jumlah akun beku, utilisasi studio, audit log ringkas.

- Owner dapat ekspor laporan ke format cetak; Owner tidak dapat
 input/ubah jadwal, tagihan, atau data pengguna.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Owner masuk ke sistem

 When Owner membuka dashboard monitoring

 Then Sistem menampilkan semua ringkasan operasional diperbarui
 dalam ≤ 3 detik, dengan visualisasi data yang mudah dibaca
 tanpa keahlian teknis
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Owner login ke sistem

2\. Load dashboard monitoring (≤ 3 dtk)

3\. Tampilkan KPI: pendapatan, kehadiran, akun beku, utilisasi studio

4\. Owner buka tab audit log ringkas

5\. Filter dan cari data laporan

6\. Owner klik \"Ekspor Laporan\"

7\. Unduh laporan format cetak

***Penanganan Error & Cadangan***

- Data laporan gagal dimuat: \"Data sedang diperbarui. Coba lagi dalam
 beberapa saat\"; tampilkan data terakhir yang tersimpan.

**FR-14 Reservasi Studio Rehearsal dan Recording ● Proposed**

 ---------------- ------------------------------------------------------
 **Aktor** Guardian/Pengguna (pengaju); Admin (persetujuan
 recording)

 **Pemicu** Guardian memilih slot studio

 **Modul** Penjadwalan

 **Verifikasi** Test

 **UC Linked** UC-06
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall mendukung reservasi ruang studio untuk dua jenis
penggunaan: rehearsal (booking mandiri per jam) dan recording
(memerlukan konsultasi harga dan persetujuan Admin sebelum
dikonfirmasi).

***Batasan Logika***

- Studio rehearsal: booking mandiri 09.00--21.00, validasi konflik
 real-time.

- Studio recording: memerlukan alur konsultasi dan persetujuan Admin
 sebelum slot dikonfirmasi.

- Tarif studio sebagai data master; hanya Admin yang dapat ubah.

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Guardian memilih slot studio rehearsal yang tersedia

 When Guardian mengkonfirmasi reservasi

 Then Slot terkunci, notifikasi konfirmasi dikirim ke Wali
 Murid, slot tidak lagi tersedia untuk pengguna lain
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Guardian buka kalender studio

2\. Pilih tipe: Rehearsal

3\. Pilih tanggal & jam

4\. Cek ketersediaan real-time

5\. Konfirmasi reservasi

6\. Kunci slot

7\. Kirim notifikasi konfirmasi ke Guardian

**Jalur Alternatif --- Flow Recording (Perlu Persetujuan Admin):**

Pilih tipe: Recording → Sistem buka alur konsultasi harga → Admin review
& setujui → Slot dikunci setelah approval → Konfirmasi ke Guardian

**Jalur Alternatif --- Slot Baru Saja Dipesan (Concurrency):**

Slot diambil pengguna lain bersamaan → Pesan \"Slot ini baru saja
dipesan\" → Tampilkan slot tersedia terdekat

***Penanganan Error & Cadangan***

- Slot baru dipesan (concurrency): \"Slot ini baru saja dipesan. Silakan
 pilih waktu lain\" + tampilkan slot tersedia terdekat.

**FR-15 Verifikasi Pembayaran Manual oleh Admin ● Approved**

 ---------------- ------------------------------------------------------
 **Aktor** Admin

 **Pemicu** Admin menerima bukti transfer dari Guardian

 **Modul** Penagihan

 **Verifikasi** Test

 **UC Linked** UC-08
 ---------------- ------------------------------------------------------

***Spesifikasi***

Sistem shall mendukung Admin memverifikasi pembayaran transfer bank
menggunakan kode unik 3 digit pada nominal transfer sebagai identifikasi
tagihan. Setiap tindakan verifikasi dicatat di audit log dengan ID Admin
dan timestamp.

***Batasan Logika***

- Admin cocokkan nominal transfer (termasuk 3 digit unik) dengan tagihan
 di sistem.

- Setelah verifikasi: status tagihan → \"Lunas\", akun aktif kembali
 (jika sebelumnya beku).

- Admin tidak dapat mengubah nominal tagihan tanpa persetujuan Owner.

- Idempotency: konfirmasi ganda pada tagihan yang sama dicegah (unique
 constraint invoice_id + unique_code).

***Kriteria Penerimaan (Given-When-Then)***

 ------------------------------------------------------------------------
 **Kondisi** **Pernyataan**
 ------------- ----------------------------------------------------------
 Given Guardian telah melakukan transfer dengan nominal yang
 mengandung kode unik yang benar

 When Admin memilih tagihan di dashboard dan menekan tombol
 \"Konfirmasi Pembayaran\"

 Then Status tagihan berubah ke \"Lunas\", akun aktif kembali
 (jika beku), notifikasi terkirim ke Guardian, dan
 tindakan tercatat di audit log dalam ≤ 2 menit
 ------------------------------------------------------------------------

***Tahapan Alur (Flow Steps)***

**Normal Flow:**

1\. Guardian transfer (nominal + kode unik 3 digit)

2\. Guardian kirim bukti transfer ke Admin

3\. Admin buka dashboard tagihan

4\. Cocokkan kode unik dengan tagihan aktif

5\. Klik \"Konfirmasi Pembayaran\"

6\. Status tagihan → \"Lunas\"

7\. Akun Guardian → Aktif (jika sebelumnya beku)

8\. Catat di audit log (ID Admin + timestamp)

9\. Kirim notifikasi konfirmasi ke Guardian

**Jalur Alternatif --- Kode Unik Tidak Cocok:**

Admin input kode tidak ditemukan → Peringatan \"Kode tidak ditemukan.
Periksa ulang nominal\" → Admin hubungi Guardian untuk klarifikasi

**Jalur Alternatif --- Konfirmasi Ganda (Idempotency Guard):**

Admin coba konfirmasi tagihan yang sudah lunas → Sistem cegah duplikasi
→ Tampilkan detail konfirmasi pertama yang sudah ada

***Penanganan Error & Cadangan***

- Kode tidak cocok: \"Kode tidak ditemukan. Periksa kembali nominal
 transfer atau hubungi Guardian.\"

- Konfirmasi ganda: Sistem mencegah duplikasi; tampilkan detail
 konfirmasi awal yang sudah ada.

**3.1.1 Ringkasan Prioritas FR (Analytical Scoring)**

Skala Wiegers: Benefit/Penalty/Cost/Risk (1--9). Skor = Value% /
(Cost% + Risk%). Status dan Linked UC tercantum untuk traceability.

**Tabel 3.1 Ringkasan Prioritas Functional Requirement**

 -----------------------------------------------------------------------------------------------------------
 **Kode** **Deskripsi \"shall\" **Status** **Nilai** **Biaya** **Risiko** **Skor** **UC**
 Behavior**
 ---------- ------------------------- ------------ ----------- ----------- ------------ ---------- ---------
 FR-01 Validasi kredensial, Approved 5 2 1 1.67 UC-01
 terbitkan JWT, onboarding
 password awal

 FR-02 Validasi slot real-time + Approved 5 3 2 1.00 UC-02
 logika kredit pembatalan

 FR-03 Terbitkan tagihan Approved 4 2 1 1.33 UC-05
 otomatis dengan kode unik

 FR-04 Terapkan denda & freeze Approved 4 1 1 1.00\* UC-07
 tanggal 11

 FR-05 Validasi absensi (GPS + Approved 5 3 3 0.42 UC-03
 Waktu + Wajah
 client-side)

 FR-06 Notifikasi Approved 4 2 2 0.50 UC-04
 multi-saluran + fallback

 FR-07 Hitung kelayakan ujian Proposed 3 1 1 0.75 UC-09
 otomatis

 FR-08 Auto-save draft & Proposed 2 2 1 0.33 UC-13
 pemulihan sesi

 FR-09 Hak penghapusan data (UU Approved 5 3 4 0.36 UC-12
 PDP) ⚠

 FR-10 Dashboard multi-student Approved 4 2 1 1.33 UC-08
 terpadu

 FR-11 Audit log permanen & Approved 5 1 1 1.67 UC-11
 immutable

 FR-12 Kelola libur nasional & Approved 3 2 2 0.38 UC-05
 minggu ke-5

 FR-13 Dashboard read-only Owner Approved 3 1 1 0.75 UC-10

 FR-14 Reservasi studio Proposed 3 3 2 0.30 UC-06
 rehearsal/recording

 FR-15 Verifikasi pembayaran Approved 5 1 1 1.67 UC-08
 manual Admin
 -----------------------------------------------------------------------------------------------------------

*⚠ FR-09: Skor analitis rendah (0.36) namun wajib legal (UU PDP No.
27/2022 Pasal 19). Tidak dapat di-defer. Rekomendasi: pisahkan kolom
\"Mandatory Compliance\" dari scoring prioritas bisnis.*

*\* FR-04: Skor disesuaikan karena Penalty bisnis jika tidak
diimplementasi sangat tinggi (churn, sengketa) melebihi scoring numerik
awal.*

**3.2 Non-Functional Requirement**

***A. Kinerja (Performance)***

**Tabel 3.2 Non-Functional Requirement --- Kinerja**

 ------------------------------------------------------------------------
 **Metrik** **Target** **Toleransi** **Cara
 Pemantauan**
 ------------------- ---------------- ----------------- -----------------
 Waktu respons ≤ 2 detik untuk Maksimal 5 detik Log waktu respons
 halaman 95% permintaan untuk 99% server;
 permintaan peringatan jika
 \> 3 detik

 Waktu respons ≤ 1 detik Maksimal 3 detik Log API endpoint
 validasi jadwal penjadwalan

 Pengiriman ≤ 15 detik Maksimal 60 detik Log antrian
 notifikasi setelah pemicu notifikasi;
 peringatan jika
 \> 30 detik

 Beban pengguna ≤ 50 pengguna Degradasi Monitor CPU &
 bersamaan aktif bertahap di atas memori server
 50; tidak crash setiap 5 menit

 Penerbitan tagihan ≤ 5 menit untuk Maksimal 15 menit Log proses
 massal 50 tagihan penagihan
 otomatis tanggal
 5
 ------------------------------------------------------------------------

***B. Keamanan (Security)***

**Tabel 3.3 Non-Functional Requirement --- Keamanan**

 ------------------------------------------------------------------------
 **Aspek** **Spesifikasi** **Mitigasi Risiko**
 ----------------- ------------------------- ----------------------------
 Enkripsi data AES-256-GCM untuk semua Mengurangi risiko kebocoran
 tersimpan data PII (nama, nomor HP, data (OWASP A02:2021)
 foto, alamat)

 Enkripsi data TLS 1.2 atau lebih baru Mencegah penyadapan data
 dikirim untuk semua koneksi saat transit (OWASP
 A02:2021)

 Pembatasan Maksimal 5 percobaan Mencegah serangan tebak kata
 percobaan login gagal dalam 15 menit; sandi (OWASP A07:2021)
 kunci akun sementara

 Validasi input Semua input pengguna Mencegah injeksi kode
 divalidasi di sisi server berbahaya (OWASP A03:2021)
 sebelum diproses

 Token sesi (JWT) Masa berlaku sesuai Mencegah pembajakan sesi
 peran; tidak disimpan di (OWASP A07:2021)
 URL; diperbarui setelah
 login ulang

 Kepatuhan UU PDP Minimasi data (Pasal 16), Audit tahunan kepatuhan;
 keamanan data (Pasal 20), prosedur penghapusan data
 hak penghapusan (Pasal terdokumentasi
 19), transfer data (Pasal
 17)

 Pembatasan laju Maksimal 100 Mencegah penyalahgunaan API
 permintaan permintaan/menit per (OWASP A04:2021)
 pengguna untuk endpoint
 kritis
 ------------------------------------------------------------------------

***C. Kemudahan Penggunaan (Usability)***

- Antarmuka memenuhi standar aksesibilitas WCAG 2.1 Level A.

- Desain mobile-first: dioptimalkan untuk layar HP (320px--480px),
 tablet (768px), dan laptop (1024px ke atas).

- Pesan error menggunakan bahasa Indonesia yang mudah dipahami,
 konsisten, dan selalu menyertakan langkah tindakan yang disarankan.

- Admin baru dapat menguasai fungsi utama sistem dalam ≤ 2 jam
 pelatihan.

- Seluruh teks tombol, label, dan navigasi menggunakan bahasa Indonesia,
 kecuali istilah teknis yang tercantum di glosarium.

***D. Keandalan (Reliability)***

**Tabel 3.4 Non-Functional Requirement --- Keandalan**

 -----------------------------------------------------------------------
 **Aspek** **Spesifikasi**
 ------------------------- ---------------------------------------------
 Percobaan ulang Otomatis 3 kali dengan jeda bertambah (10,
 notifikasi 20, 40 detik) sebelum beralih ke saluran
 cadangan.

 Pemutus sirkuit (circuit Jika WhatsApp gateway gagal \> 5 kali
 breaker) berturut-turut, sistem otomatis menonaktifkan
 sementara dan beralih ke email, mencegah
 kegagalan berantai.

 Konsistensi data booking Operasi booking bersifat atomik --- berhasil
 sepenuhnya atau dibatalkan sepenuhnya; tidak
 ada kondisi data setengah tersimpan.

 Pencadangan data Backup harian otomatis; data dapat dipulihkan
 ke kondisi 24 jam sebelumnya jika terjadi
 kegagalan sistem.

 Pemulihan setelah Sistem kembali beroperasi dalam ≤ 2 jam
 gangguan setelah gangguan server (RTO); kehilangan
 data maksimal 1 jam terakhir (RPO).
 -----------------------------------------------------------------------

***E. Ketersediaan (Availability)***

- Target ketersediaan sistem: 99% per bulan (maksimal ≈ 7,2 jam downtime
 per bulan).

- Jendela pemeliharaan terjadwal: Senin pukul 02.00--04.00 WIB, di luar
 jam operasional course.

- Halaman pemeliharaan ditampilkan saat sistem dalam perbaikan, dengan
 estimasi waktu selesai.

- Endpoint pemantauan kesehatan sistem (health check) tersedia untuk
 monitoring otomatis.

- Notifikasi pemeliharaan dikirimkan ke Admin dan Owner minimal 24 jam
 sebelum pemeliharaan terjadwal.

***F. Analisis Trade-Offs Atribut Kualitas***

**Tabel 3.5 Analisis Trade-Offs Atribut Kualitas**

 -------------------------------------------------------------------------
 **Atribut **Skenario Konflik **Prioritas **Mitigasi**
 Bentrok** Desain** Keputusan**
 -------------- ------------------- ---------------- ---------------------
 Keamanan vs Enkripsi Keamanan Cache read-only untuk
 Kinerja AES-256-GCM pada diprioritaskan dashboard; enkripsi
 PII & validasi GPS asinkron pada
 menambah latency background worker
 \~150ms/request

 Keandalan vs Retry notifikasi Keandalan SMTP sebagai fallback
 Biaya 3x + fallback SMTP diprioritaskan utama pesan kritis;
 meningkatkan biaya WhatsApp hanya untuk
 gateway & storage reminder non-kritis
 log

 Kemudahan vs Session timeout Keamanan Implementasi \"ingat
 Keamanan ketat (Admin 15 diprioritaskan perangkat\" dengan
 menit) mengganggu token refresh aman &
 alur kerja panjang re-autentikasi
 biometrik/PIN lokal

 Kinerja vs Beban puncak \> 50 Ketersediaan Auto-scaling
 Skalabilitas pengguna bersamaan diprioritaskan dikonfigurasi;
 pada infrastruktur matikan fitur
 cloud non-kritis (animasi
 dashboard, ekspor PDF
 real-time) saat CPU
 \> 80%
 -------------------------------------------------------------------------

**3.3 External Interface Requirement**

***A. Antarmuka Pengguna (User Interface)***

- Sistem diakses melalui browser modern (Chrome 90+, Safari 14+, Firefox
 88+, Edge 90+) tanpa instalasi tambahan.

- Antarmuka dibangun sebagai PWA --- pengguna dapat \"memasang\"
 aplikasi di layar utama HP seperti aplikasi biasa.

- Fungsi offline dasar tersedia melalui PWA: melihat jadwal terakhir
 yang tersimpan, bahkan tanpa koneksi internet.

- Strategi penyimpanan cache: data jadwal dan profil pengguna di-cache
 untuk akses cepat dan penggunaan offline.

***B. Perangkat (Hardware)***

**Tabel 3.6 Spesifikasi Perangkat**

 -----------------------------------------------------------------------
 **Komponen** **Spesifikasi Minimum**
 ------------------------- ---------------------------------------------
 Server aplikasi 2 vCPU, 4 GB RAM, 40 GB penyimpanan SSD
 (cloud scalable); auto-scaling dikonfigurasi
 jika pengguna melampaui 50 orang aktif.

 Perangkat klien Smartphone Android 8+ atau iOS 13+; laptop
 (pengguna) dengan browser modern; koneksi internet
 minimal 4G/WiFi.

 Pertumbuhan penyimpanan Estimasi 500 MB/tahun untuk foto absensi (50
 student × 4 sesi/bulan × 12 bulan × rata-rata
 \~200KB/foto); ditinjau setiap 6 bulan.
 -----------------------------------------------------------------------

***C. Perangkat Lunak dan Integrasi (Software)***

**Tabel 3.7 Spesifikasi Integrasi Layanan Eksternal**

 -----------------------------------------------------------------------
 **Layanan** **Spesifikasi Integrasi**
 ------------------- ---------------------------------------------------
 WhatsApp Gateway Endpoint REST API; autentikasi via API key di
 (tidak resmi) header; payload JSON; kode error yang ditangani:
 429 (batas kirim) → tunda 60 detik; 5xx →
 exponential backoff 3x. Kontak admin gateway untuk
 eskalasi.

 Email SMTP Server SMTP penyedia cloud; port 587 (TLS STARTTLS)
 (cadangan kritis) atau 465 (SSL); autentikasi username & password;
 digunakan untuk komunikasi kritis dan cadangan
 WhatsApp.

 Token sesi (JWT) Struktur: header (algoritma HS256), payload (ID
 pengguna, peran, waktu terbit, waktu kedaluwarsa),
 tanda tangan. Token diperbarui saat login dan tidak
 disimpan di URL.

 Webhook notifikasi Sistem mendukung pengiriman webhook ke Admin Panel
 saat terjadi kegagalan notifikasi berulang (\> 5
 kali/hari); payload berisi detail kegagalan dan
 saran tindakan.
 -----------------------------------------------------------------------

***D. Peta Navigasi Antarmuka (Dialog Map)***

Peta navigasi berikut menjamin setiap layar memiliki back-out route yang
jelas dan mencegah dead-end state pada antarmuka PWA.

**\[GAMBAR: Peta Navigasi Antarmuka (Dialog Map) PWA\]**

**Aturan Navigasi:**

- Tombol \"Batal\" pada form input selalu mengembalikan ke layar
 sebelumnya tanpa menyimpan data parsial.

- Tombol \"Kembali\" pada dashboard memicu konfirmasi logout jika sesi
 hampir habis.

- Tidak ada layar yang berada di leaf node tanpa opsi navigasi ke
 dashboard utama atau aksi pembatalan.
