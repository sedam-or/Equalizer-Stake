**BAB 4 --- MODEL ANALISIS SISTEM (UML)**

**4.1 Use Case Diagram**

Diagram berikut menggambarkan seluruh aktor dan fungsi utama sistem
Equalizer-Stake, dikelompokkan per modul. Relasi include menunjukkan
fungsi yang selalu dipanggil, sedangkan relasi extend menunjukkan fungsi
opsional yang aktif dalam kondisi tertentu.

**\[GAMBAR: Use Case Diagram Equalizer-Stake --- seluruh aktor dan
modul\]**

**Tabel 4.1 --- Keterangan Use Case per Modul**

 -----------------------------------------------------------------------
 **Modul** **Use Case (Kode + Nama Singkat)**
 ---------------- ------------------------------------------------------
 Autentikasi UC1: Login dan Autentikasi UC2: Kelola Profil

 Penjadwalan UC3: Input Jadwal Course UC4: Validasi Konflik Slot
 UC5: Kelola Libur & Minggu ke-5 UC6: Reservasi Studio

 Penagihan UC7: Terbitkan Tagihan Otomatis UC8: Verifikasi
 Pembayaran Manual UC9: Terapkan Denda & Bekukan UC10:
 Buka Pembekuan Akun

 Kehadiran UC11: Input Absensi + Foto UC12: Validasi GPS & Waktu
 UC13: Lacak Kelayakan Ujian

 Notifikasi UC14: Kirim WhatsApp UC15: Email Cadangan

 Pelaporan UC16: Dashboard Monitoring UC17: Ekspor Laporan UC18:
 Audit Log

 Kepatuhan UC19: Ajukan Hapus Data UC20: Proses Hapus PII
 -----------------------------------------------------------------------

Keterangan Diagram:

- Owner hanya memiliki akses baca: dashboard monitoring, ekspor laporan,
 dan audit log. Tidak dapat mengubah data operasional.

- Admin adalah aktor utama operasional: mengelola jadwal, memverifikasi
 pembayaran, membekukan/membuka akun, dan memproses permintaan
 penghapusan data.

- Instructor hanya mengakses jadwal pribadi dan mengisi absensi; tidak dapat
 mengubah data pengguna lain.

- Guardian memiliki akses terbatas: melihat jadwal semua anak, memesan
 studio, dan mengajukan hak penghapusan data.

**Relasi Include**

Include adalah hubungan wajib antara dua use case. Use case dasar (base)
selalu memanggil atau menyertakan use case yang di-include setiap kali
use case dasar dijalankan. Use case yang di-include tidak berdiri
sendiri; ia menjadi bagian tak terpisahkan dari alur use case dasar.

**A. UC3 \<\<include\>\> UC4**

- Base: UC3 Input Jadwal Course \| Included: UC4 Validasi Konflik Slot

- Setiap admin menginput jadwal, sistem wajib memvalidasi slot agar
 tidak bentrok. Validasi konflik adalah langkah wajib dalam alur input
 jadwal. Pemicu: Otomatis setiap kali UC3 dijalankan.

**B. UC7 \<\<include\>\> UC14**

- Base: UC7 Terbitkan Tagihan Otomatis \| Included: UC14 Kirim WhatsApp

- Setiap tagihan yang diterbitkan otomatis, wajib dikirimkan notifikasi
 ke WhatsApp Guardian. UC14 adalah sub-proses tetap dari UC7. Pemicu:
 Otomatis setiap UC7 berhasil generate tagihan.

**C. UC8 \<\<include\>\> UC9**

- Base: UC8 Verifikasi Pembayaran Manual \| Included: UC9 Terapkan Denda
 & Bekukan

- Saat admin memverifikasi pembayaran masuk, sistem wajib mengecek
 status pembayaran. Jika terlambat, sistem otomatis menghitung denda.
 Pemicu: Otomatis setiap UC8 dijalankan, meskipun hasilnya tidak ada
 denda (denda = 0).

**D. UC11 \<\<include\>\> UC12**

- Base: UC11 Input Absensi + Foto \| Included: UC12 Validasi GPS & Waktu

- Setiap instructor mengisi absensi dengan mengunggah foto, sistem wajib
 memvalidasi metadata foto (GPS dan timestamp). Validasi adalah langkah
 wajib dalam proses absensi. Pemicu: Otomatis setiap UC11 dijalankan.

**Relasi Extend**

Extend adalah hubungan opsional/bersyarat antara dua use case. Use case
dasar (base) dapat berdiri sendiri tanpa use case yang di-extend. Use
case yang di-extend hanya dijalankan jika kondisi tertentu terpenuhi.

**A. UC12 \<\<extend\>\> UC13**

- Base: UC12 Validasi GPS & Waktu \| Extending: UC13 Lacak Kelayakan
 Ujian

- Hanya jika data kehadiran menunjukkan student memenuhi jumlah minimum
 kehadiran berturut-turut (misal 10x), maka sistem akan menandai student
 \"layak ujian kenaikan grade\". UC12 tetap berfungsi tanpa UC13.

- Kondisi Pemicu: Jumlah kehadiran student \>= batas kelayakan ujian.

**B. UC14 \<\<extend\>\> UC15**

- Base: UC14 Kirim WhatsApp \| Extending: UC15 Kirim Email Cadangan

- Hanya jika pengiriman WhatsApp gagal (tidak terkirim dalam 30 detik
 atau kode error dari API WhatsApp), maka sistem akan mengirim
 notifikasi via email sebagai fallback. Jika WhatsApp berhasil, UC15
 tidak dijalankan.

- Kondisi Pemicu: Status pengiriman WhatsApp = gagal / timeout.

**4.2 Use Case Scenario**

**Skenario 1: Login dan Autentikasi Berbasis Peran**

**Tabel 4.2 --- Use Case Scenario: Login dan Autentikasi Berbasis
Peran**

 -----------------------------------------------------------------------
 **Aspek** **Deskripsi**
 ------------------- ---------------------------------------------------
 Use Story Sebagai pengguna sistem (Owner/Admin/Instructor/Wali
 Murid), saya ingin login ke sistem agar dapat
 mengakses fitur sesuai peran saya.

 Description Proses autentikasi pengguna untuk masuk ke sistem
 Equalizer-Stake dengan validasi kredensial dan
 penerbitan token sesi (JWT) sesuai peran.

 Screen# SCR-001: Halaman Login

 Trigger Pengguna membuka URL sistem dan memasukkan username
 serta password.

 Preconditions 1\. Pengguna memiliki akun aktif di sistem. 2.
 Kredensial (username & password) valid. 3. Halaman
 login dapat diakses melalui browser. 4. Koneksi
 internet stabil.

 Postconditions 1\. Token sesi (JWT) diterbitkan dengan masa
 berlaku sesuai peran. 2. Waktu login tercatat di
 audit log. 3. Pengguna diarahkan ke dashboard
 sesuai peran. 4. Session timer dimulai.

 Normal Flow 1\. Membuka URL sistem di browser. 2. Memasukkan
 username dan password. 3. Menekan tombol \"Masuk\".
 → Sistem memvalidasi kredensial ke basis data (≤2
 detik). → Menerbitkan token sesi dengan masa
 berlaku sesuai peran. → Mencatat login berhasil di
 audit log (waktu, ID pengguna, alamat IP). →
 Menampilkan dashboard sesuai peran pengguna.

 Alternative Flows A1: Memasukkan password salah → Sistem menampilkan
 \"Username atau password tidak valid\", increment
 fail counter. A2: Login dengan akun nonaktif →
 Sistem menampilkan \"Akun Anda tidak aktif. Hubungi
 Admin Equalizer-Stake\" dan hentikan proses. A3: Login
 saat sesi masih aktif di perangkat lain → Sistem
 mengakhiri sesi lama dan membuat sesi baru.

 Exceptions Percobaan login gagal ≥5 kali dalam 15 menit →
 Sistem mengunci akun 15 menit, menampilkan pesan
 waktu tunggu, mengirim notifikasi email ke Admin,
 dan mencatat insiden di audit log keamanan.

 Business Rules 1\. Batas percobaan login gagal: maksimal 5 kali
 dalam 15 menit. 2. Masa berlaku sesi: Owner 30 mnt,
 Admin 15 mnt, Instructor 30 mnt, Guardian 60 mnt. 3.
 Password minimal 6 karakter, alphanumeric +
 underscore. 4. Token JWT tidak disimpan di URL. 5.
 Setiap login wajib tercatat di audit log.

 Assumptions 1\. Pengguna memiliki akses internet stabil
 (minimum 4G). 2. Browser yang digunakan modern
 (Chrome 90+, Safari 14+, atau setara). 3. Server
 waktu sinkron dengan NTP server. 4. Database
 tersedia dan responsif.

 Notes and Issues Jika layanan token sesi gangguan → Tampilkan pesan
 pemeliharaan dan catat insiden di log sistem. User
 lupa password → Sediakan fitur \"Lupa Password\"
 dengan reset link via email (di luar scope MVP).
 Multiple login dari lokasi berbeda → Implementasi
 notifikasi keamanan ke email pengguna (future
 enhancement).
 -----------------------------------------------------------------------

**Skenario 2: Input Jadwal Course dengan Validasi Konflik Slot**

**Tabel 4.3 --- Use Case Scenario: Input Jadwal Course dengan Validasi
Konflik Slot**

 -----------------------------------------------------------------------
 **Aspek** **Deskripsi**
 ------------------- ---------------------------------------------------
 User Story Sebagai Admin operasional, saya ingin input jadwal
 course baru agar student dapat dijadwalkan dengan
 instructor dan ruangan yang sesuai tanpa konflik.

 Description Admin menginput jadwal course baru dengan validasi
 real-time terhadap ketersediaan instructor, ruangan, dan
 waktu untuk mencegah double-booking.

 Screen# SCR-003: Form Input Jadwal

 Trigger Admin memilih menu \"Tambah Jadwal Baru\" di modul
 Penjadwalan.

 Preconditions 1\. Admin sudah login dengan role Admin. 2. Data
 instructor, ruangan, dan student sudah tersedia di sistem.
 3. Kalender libur nasional sudah diinput. 4. Jam
 operasional sudah dikonfigurasi.

 Postconditions 1\. Jadwal tersimpan dengan status \"Terkunci
 (Locked)\". 2. Instructor menerima notifikasi jadwal
 baru. 3. Guardian menerima notifikasi. 4. Entri
 dicatat di audit log. 5. Slot terkunci untuk
 booking lain.

 Normal Flow 1\. Membuka modul Penjadwalan. 2. Memilih \"Tambah
 Jadwal Baru\". 3. Memilih: student, instructor, ruangan,
 tanggal, waktu mulai, dan durasi sesi. 4. Melihat
 konfirmasi \"Slot tersedia\" dengan ringkasan
 jadwal. 5. Mengkonfirmasi dan menekan tombol
 \"Simpan Jadwal\". → Sistem memeriksa ketersediaan
 (≤1 detik): ketersediaan instructor, ruangan, waktu +
 jeda 10 menit, bukan hari Minggu/libur nasional,
 dalam jam operasional. → Menyimpan jadwal dengan
 status \"Terkunci\". → Mencatat di audit log. →
 Mengirim notifikasi ke Instructor dan Guardian.

 Alternative Flows A1: Konflik slot terdeteksi → Tampilkan detail
 konflik dan 3 slot alternatif terdekat. A2: Instructor
 tidak tersedia (libur/cuti) → Sistem memperingatkan
 dan meminta Admin memilih instructor pengganti. A3:
 Jadwal di luar jam operasional → Sistem menolak dan
 menampilkan jam operasional yang berlaku. A4:
 Duplikasi booking bersamaan → Sistem hanya
 menyetujui yang pertama (concurrency control);
 Admin kedua menerima pesan \"Slot baru saja
 dipesan\".

 Exceptions 1\. Database connection lost saat penyimpanan →
 Sistem membatalkan transaksi dan rollback. 2.
 Validasi konflik gagal (timeout \>3 detik) → Sistem
 tampilkan warning \"Validasi mengalami gangguan,
 periksa manual\". 3. Notifikasi gagal terkirim ke
 semua saluran → Jadwal tetap tersimpan, notifikasi
 masuk antrian retry (3x exponential backoff).

 Business Rules 1\. Durasi sesi: 30 menit (pemula) atau 45--50
 menit (reguler). 2. Jeda wajib antar-sesi: 10
 menit. 3. Jam operasional: Senin--Jumat
 14.30--17.00, Sabtu 08.00--12.00. 4. Tidak ada
 jadwal di hari Minggu dan libur nasional. 5. Slot
 yang sudah Locked hanya bisa diubah Admin dengan
 audit trail. 6. Validasi harus selesai ≤1 detik. 7.
 Draft auto-save setiap 30 detik.

 Notes and Issues Konflik validasi karena timezone berbeda → Sistem
 menggunakan server time (WIB) sebagai acuan
 tunggal. Admin input jadwal untuk tanggal yang
 sudah lewat → Sistem validasi: tanggal harus ≥ hari
 ini + 1 hari. Perubahan jadwal mendadak (instructor
 sakit) → Gunakan fitur Reschedule dengan alasan
 wajib (audit trail).
 -----------------------------------------------------------------------

**Skenario 3: Absensi Digital dengan Validasi Foto, GPS, dan Waktu**

**Tabel 4.4 --- Use Case Scenario: Absensi Digital dengan Validasi Foto,
GPS, dan Waktu**

 -----------------------------------------------------------------------
 **Aspek** **Deskripsi**
 ------------------- ---------------------------------------------------
 User Story Sebagai Instructor music, saya ingin mengisi absensi
 digital dengan foto agar kehadiran student tercatat
 secara valid dan otomatis.

 Description Instructor mengisi absensi melalui antarmuka web dengan
 mengunggah foto verifikasi. Sistem memvalidasi 3
 aspek: waktu server, lokasi GPS (radius 100m), dan
 proporsi wajah (≥30% frame).

 Screen# SCR-011: Form Absensi Digital

 Trigger Instructor membuka menu \"Absensi Hari Ini\" dan memilih
 sesi yang sedang berlangsung.

 Preconditions 1\. Instructor sudah login dengan role Instructor. 2. Sesi
 course sedang aktif sesuai jadwal. 3. Instructor berada
 di lokasi studio Equalizer-Stake. 4. GPS perangkat
 diaktifkan. 5. Kamera perangkat memiliki izin
 akses.

 Postconditions 1\. Absensi student tercatat sebagai \"Hadir\" atau
 \"Menunggu Verifikasi\". 2. Kredit student diperbarui
 (jika status Hadir). 3. Notifikasi dikirim ke Wali
 Murid. 4. Data absensi tersimpan dengan metadata
 (foto, GPS, timestamp). 5. Kelayakan ujian dihitung
 ulang.

 Normal Flow 1\. Membuka menu \"Absensi Hari Ini\". 2. Memilih
 sesi yang sedang berlangsung. 3. Menekan tombol
 \"Ambil Foto Kehadiran\". 4. Mengambil foto student
 (wajah terlihat minimal 30%). 5. Menekan
 \"Konfirmasi Absensi\". → Sistem memvalidasi 3
 aspek (≤5 detik): waktu berdasarkan jam server,
 koordinat GPS dalam radius 100m dari studio
 (-7.7265, 110.4162), proporsi wajah ≥30%. → Jika
 semua valid → absensi tercatat \"Hadir\", kredit
 student terpotong. → Notifikasi hadir dikirim ke Wali
 Murid. → Update kelayakan ujian real-time.

 Alternative Flows A1: GPS di luar radius 100 meter → Absensi
 berstatus \"Menunggu Verifikasi\", Admin menerima
 notifikasi untuk review. A2: Wajah tidak terdeteksi
 ≥30% → Sistem meminta Instructor mengambil ulang foto
 dengan panduan posisi. A3: Student tidak hadir → Instructor
 memilih opsi \"Student Absen\" dengan keterangan
 (absen dengan/tanpa kabar). A4: Koneksi internet
 terputus saat upload → Foto disimpan di perangkat,
 upload otomatis saat koneksi kembali (≤5 menit).
 A5: Server timestamp berbeda \>5 menit → Sistem
 gunakan server time, tampilkan warning
 sinkronisasi.

 Exceptions 1\. Instructor mencoba absensi untuk sesi yang tidak ada
 di jadwalnya → Sistem menolak dengan pesan \"Anda
 tidak memiliki sesi terjadwal saat ini\". 2. Upload
 foto gagal berulang (\>3x) → Sistem simpan draft
 lokal, notifikasi Admin untuk absensi manual. 3.
 GPS tidak dapat dideteksi → Absensi status
 \"PendingVerification\", Instructor input manual
 koordinat. 4. Foto corrupted/tidak valid →
 Tampilkan error \"Foto tidak valid, silakan ambil
 ulang\".

 Business Rules 1\. Validasi GPS: radius maksimal 100 meter dari
 koordinat studio. 2. Validasi wajah: minimal 30%
 dari area foto. 3. Timestamp menggunakan jam server
 (bukan jam perangkat). 4. Instructor tidak dapat mengubah
 absensi setelah dikirim. 5. Hanya Admin yang dapat
 koreksi absensi (dengan audit trail). 6. Foto
 disimpan dengan enkripsi AES-256-GCM. 7. Status
 \"PendingVerification\" wajib direview Admin ≤2
 jam. 8. Student no-show: kredit terpotong, sesi
 hangus.

 Notes and Issues GPS kurang akurat di dalam ruangan → Toleransi
 radius 100m untuk mengakomodasi indoor GPS drift.
 Student tidak mau difoto → Gunakan mode \"Student
 Absen\" dengan keterangan, perlu verifikasi Admin.
 Foto blur/gelap → Sistem validasi kualitas foto
 minimal, prompt retake jika gagal. Privasi foto
 student → Foto dienkripsi, dihapus permanen jika hak
 penghapusan disetujui (UU PDP).
 -----------------------------------------------------------------------

**4.3 Activity Diagram**

Diagram berikut menggambarkan alur lengkap proses booking course dari
permintaan awal hingga konfirmasi dan notifikasi, termasuk jalur
cadangan email jika notifikasi WhatsApp gagal.

**\[GAMBAR: Activity Diagram --- Alur Booking Course (Jalur Normal)\]**

**\[GAMBAR: Activity Diagram --- Alur Booking Course (Jalur Gagal)\]**

Keterangan Diagram:

**A. Jalur Utama Booking Berhasil**

- Autentikasi: Guardian melakukan login ke sistem; Sistem memvalidasi
 kredensial dan menampilkan dashboard sesuai peran Guardian.

- Pengajuan Booking: Guardian memilih student dan slot waktu yang
 diinginkan (mengajukan booking baru).

- Validasi & Penahanan Slot: Sistem menerima draft booking dan melakukan
 validasi slot secara real-time. Jika slot tersedia, sistem mengubah
 status menjadi Pending (slot ditahan sementara) dan mengirimkan
 notifikasi ke Admin.

- Review & Konfirmasi Admin: Admin menerima notifikasi, me-review detail
 booking, dan mengklik tombol konfirmasi.

- Finalisasi & Notifikasi: Sistem mengubah status slot menjadi Locked
 (terkunci); mencatat perubahan ke dalam audit log; mengirimkan
 notifikasi kepada Instructor (jadwal baru) dan Guardian (konfirmasi).

- Selesai: Proses booking dinyatakan selesai, dan Guardian menerima
 konfirmasi akhir di dashboard mereka.

**B. Jalur Gagal**

- F1 --- Login Gagal 5x Berturut-turut: Sistem memvalidasi kegagalan,
 mengunci akun selama 15 menit, dan mengirimkan notifikasi waktu tunggu
 ke Guardian.

- F2 --- Akun Guardian Dibekukan: Jika Guardian berhasil login namun
 mencoba mengakses fitur booking saat tagihan melewati tanggal 11,
 sistem mendeteksi akun beku dan memblokir semua akses.

- F3 --- Slot Konflik / Double Booking: Jika terjadi konflik slot, Admin
 dapat memilih slot alternatif. Sistem menampilkan detail konflik
 beserta 3 slot alternatif.

- F4 --- Slot di Luar Jam Operasional & Hari Libur: Sistem menolak
 validasi, membatalkan booking, dan menampilkan pesan error kepada Wali
 Murid.

- F5 --- Concurrency (Dua Admin Menyimpan Slot Bersamaan): Sistem
 menerapkan concurrency lock. Admin A diterima duluan, sedangkan Admin
 B ditolak dan diwajibkan memilih ulang slot/waktu yang berbeda.

- F6 --- Admin Menolak Booking: Sistem secara otomatis membebaskan slot
 yang ditahan dan mengirimkan notifikasi penolakan kepada Guardian.

- F7 --- Database Error Saat Penyimpanan: Sistem membatalkan perubahan
 data (rollback transaksi) untuk mencegah data korup, dan Admin
 menerima pesan error.

**4.4 Sequence Diagram**

Diagram berikut menggambarkan komunikasi antar komponen sistem saat Wali
Murid menerima tagihan dan melakukan pembayaran, termasuk mekanisme
percobaan ulang notifikasi.

**\[GAMBAR: Sequence Diagram --- Penerbitan Tagihan dan Verifikasi
Pembayaran\]**

Keterangan Diagram:

- Fragment loop menunjukkan proses penagihan yang berjalan untuk setiap
 akun Guardian secara berurutan.

- Fragment alt/else menggambarkan mekanisme cadangan email yang aktif
 jika WhatsApp gateway gagal setelah 3 percobaan ulang.

- Untuk notifikasi kritis (konfirmasi pembayaran), email adalah saluran
 utama dan WhatsApp berfungsi sebagai cadangan --- kebalikan dari
 notifikasi non-kritis.

- Timeout Handling: Retry dengan exponential backoff (10s → 20s → 40s)
 sebelum fallback.

- Circuit Breaker: Monitoring gateway otomatis; nonaktifkan WhatsApp
 sementara jika error \>5x berturut-turut.

- Audit Logging: Setiap tindakan Admin/verifikasi dicatat otomatis ke
 database audit log.

- Idempotency: Verifikasi pembayaran ganda dicegah dengan unique
 constraint pada invoice_id + unique_code.

**Sequence Diagram Tambahan 1: Input Jadwal dengan Validasi Konflik**

Diagram berikut menggambarkan alur komunikasi antar komponen sistem pada
saat Admin menginput jadwal baru, termasuk mekanisme deteksi konflik dan
penawaran slot alternatif.

**\[GAMBAR: Sequence Diagram --- Input Jadwal dengan Validasi
Konflik\]**

**Tabel 4.5 --- Alur Sequence Diagram: Input Jadwal dengan Validasi
Konflik**

+:----------------------------------------------------------------------+
| Aktor : Admin |
| |
| Komponen : Frontend → Backend → Database → Notification Service |
| |
| Alur Normal: |
| |
| 1\. Admin memilih student, instructor, ruangan, tanggal, waktu. |
| |
| 2\. Frontend mengirim request ke Backend. |
| |
| 3\. Backend query Database untuk mengecek konflik |
| |
| (instructor, ruangan, waktu + jeda 10 menit). |
| |
| 4a. \[KONFLIK\] Backend kirim response \"Konflik terdeteksi\" |
| |
| dengan daftar 3 slot alternatif terdekat. |
| |
| 4b. \[TIDAK KONFLIK\] Backend simpan jadwal dengan status |
| |
| \"Pending\" (belum Locked). |
| |
| 5\. Backend kirim notifikasi ke Instructor dan Guardian |
| |
| via Notification Service. |
| |
| 6\. Instructor/Guardian konfirmasi → Backend ubah status |
| |
| menjadi \"Locked\". |
| |
| 7\. Audit log dicatat (timestamp, ID Admin, before/after state). |
+-----------------------------------------------------------------------+

**Sequence Diagram Tambahan 2: Absensi dengan Validasi Foto dan GPS**

Diagram berikut menggambarkan alur komunikasi antar komponen sistem pada
saat Instructor mengisi absensi digital, termasuk validasi client-side dan
server-side serta penanganan kasus GPS gagal.

**\[GAMBAR: Sequence Diagram --- Absensi dengan Validasi Foto dan
GPS\]**

**Tabel 4.6 --- Alur Sequence Diagram: Absensi dengan Validasi Foto dan
GPS**

+:----------------------------------------------------------------------+
| Aktor : Instructor |
| |
| Komponen : Frontend (Client-Side Validation) → Backend → Database |
| |
| Alur Normal: |
| |
| 1\. Instructor buka form absensi, pilih sesi aktif. |
| |
| 2\. Frontend akses kamera, ambil foto. |
| |
| 3\. Frontend validasi wajah (client-side, face-api.js / |
| |
| MediaPipe) → jika gagal, minta ulang (≤1 detik, no upload). |
| |
| 4\. Frontend ambil koordinat GPS perangkat. |
| |
| 5\. Frontend kirim (foto + GPS + timestamp) ke Backend. |
| |
| 6\. Backend validasi GPS (radius 100m dari koordinat studio) |
| |
| dan timestamp server. |
| |
| 7a. \[SEMUA VALID\] → simpan status \"Hadir\", potong kredit, |
| |
| kirim notifikasi ke Guardian. |
| |
| 7b. \[GPS GAGAL --- luar radius\] → status \"Pending Verification\", |
| |
| notifikasi ke Admin untuk review manual (≤2 jam). |
| |
| 7c. \[FOTO GAGAL --- wajah \<30%\] → tolak, kirim pesan |
| |
| panduan ke Instructor, minta ulang foto. |
| |
| 8\. Audit log dicatat (timestamp server, ID Instructor, |
| |
| GPS coords, gps_valid, face_valid, verification_status). |
+-----------------------------------------------------------------------+

**4.5 State Transition Diagram**

Bagian ini mendefinisikan siklus hidup status (mesin status) untuk empat
entitas kritis sistem: Booking, Pembayaran, Kehadiran, dan Profil Student.

**A. State Transition: Booking (Pemesanan Sesi)**

**\[GAMBAR: State Transition Diagram --- Booking (Draft → Pending →
Locked → Completed/Cancelled/Rescheduled)\]**

**Tabel 4.7 --- Keterangan Transisi Booking**

 ------------------------------------------------------------------------
 **Dari **Ke Status** **Pemicu** **Efek Samping**
 Status**
 -------------- ------------- ---------------------- --------------------
 Draft Pending Data booking lengkap, Notifikasi ke Admin;
 dikirim untuk slot ditahan
 konfirmasi sementara

 Pending Locked Admin mengkonfirmasi Slot terkunci
 permanen; notifikasi
 ke Instructor & Wali;
 dicatat di audit log

 Locked Rescheduled Instructor berhalangan atau Kredit tidak
 libur nasional terpotong;
 notifikasi perubahan
 jadwal ke Wali &
 Instructor

 Locked Cancelled Pembatalan oleh Admin Slot dibebaskan;
 (dengan alasan) kredit terpotong
 (jika student yang
 membatalkan); audit
 log wajib

 Locked Completed Absensi dikonfirmasi Kredit terpotong;
 oleh Instructor rekam kehadiran
 diperbarui;
 kelayakan ujian
 dihitung ulang
 ------------------------------------------------------------------------

**B. State Transition: Pembayaran (Payment)**

**\[GAMBAR: State Transition Diagram --- Pembayaran (Unpaid →
Partial_Paid → Overdue → Frozen → Paid)\]**

**Tabel 4.8 --- Keterangan Transisi Pembayaran**

 ------------------------------------------------------------------------
 **Dari **Ke Status** **Pemicu** **Efek Samping**
 Status**
 -------------- ------------- ---------------------- --------------------
 Unpaid Overdue Tanggal 11 pukul Denda Rp 100.000
 00:00, belum ada otomatis
 pembayaran ditambahkan; akun
 masuk antrian
 pembekuan

 Overdue Frozen Proses otomatis Seluruh akses student
 setelah denda diblokir; notifikasi
 diterapkan WhatsApp + email ke
 Guardian

 Frozen Paid Admin memverifikasi Akun aktif kembali;
 pelunasan (pokok + notifikasi
 denda) konfirmasi ke Wali
 Murid; audit log
 dicatat
 ------------------------------------------------------------------------

**C. State Transition: Kehadiran (Attendance)**

**\[GAMBAR: State Transition Diagram --- Kehadiran (Pending Verification
→ Present / AbsentExcused / AbsentNoShow / Rescheduled)\]**

**Tabel 4.9 --- Keterangan Transisi Kehadiran**

 -----------------------------------------------------------------------
 **Status** **Dampak pada Kredit **Dampak pada Kelayakan
 Student** Ujian**
 ----------------- ---------------------- ------------------------------
 Present Kredit terpotong (sesi Menambah hitungan kehadiran
 terhitung) (positif)

 AbsentExcused Kredit terpotong Dihitung sebagai absen
 (kebijakan Equalizer-Stake) (berisiko jika \>10% dari
 sesi)

 AbsentNoShow Kredit terpotong, sesi Dihitung sebagai absen tanpa
 hangus kabar; \>1x/3 bulan = tidak
 layak ujian

 Rescheduled Kredit tidak terpotong Tidak dihitung sebagai absen
 (Instructor) student
 -----------------------------------------------------------------------

**D. State Transition: Student Profile (Profil Student)**

Diagram berikut menggambarkan siklus hidup status profil student, mulai
dari masa percobaan (Trial) hingga berbagai kondisi aktif, cuti, dan
tidak aktif. Setiap transisi mencatat perubahan di audit log dan memicu
notifikasi yang relevan.

**\[GAMBAR: State Transition Diagram --- Student Profile (Trial → Active
→ Cuti / Inactive)\]**

**Tabel 4.10 --- Alur State Transition Student Profile (Mermaid
Notation)**

+:----------------------------------------------------------------------+
| stateDiagram-v2 |
| |
| \[\*\] \--\> Trial |
| |
| Trial \--\> Active : Pembayaran admin fee & audisi diverifikasi |
| |
| Trial \--\> Inactive : Trial gagal / tidak lanjut |
| |
| Active \--\> Cuti : Cuti diajukan (maks. 1 bulan) |
| |
| Cuti \--\> Active : Kembali dari cuti (≤ 1 bulan) |
| |
| Cuti \--\> Inactive : Cuti \> 1 bulan (wajib daftar ulang) |
| |
| Active \--\> Inactive : Student berhenti / drop-out |
| |
| Inactive \--\> Active : Daftar ulang (proses Trial lagi) |
+-----------------------------------------------------------------------+

**Tabel 4.11 --- Keterangan Transisi Student Profile**

 ------------------------------------------------------------------------
 **Dari **Ke Status** **Pemicu** **Efek Samping**
 Status**
 -------------- ------------- ------------------- -----------------------
 Trial Active Admin memverifikasi Student masuk jadwal
 pembayaran admin reguler; Guardian
 fee & hasil audisi menerima notifikasi
 konfirmasi;
 enrollment_date di-set.

 Trial Inactive Trial gagal / student Data student tetap ada;
 tidak melanjutkan status
 pendaftaran enrollment_status =
 \'inactive\'; tidak ada
 penagihan.

 Active Cuti Guardian mengajukan Notifikasi ke Admin;
 cuti (maksimal 1 sesi aktif ditahan;
 bulan); disetujui tidak ada penagihan
 Admin selama masa cuti.

 Cuti Active Guardian Student kembali ke jadwal
 mengkonfirmasi aktif; notifikasi ke
 kembali aktif (≤ 1 Instructor; penagihan
 bulan dari tanggal berjalan kembali bulan
 mulai cuti) berikutnya.

 Cuti Inactive Masa cuti melebihi Sistem otomatis
 1 bulan tanpa mengubah status ke
 konfirmasi kembali Inactive; student wajib
 dari Guardian daftar ulang; data
 histori tetap
 tersimpan.

 Active Inactive Admin menonaktifkan Data PII student
 atas permintaan dianonimkan sesuai UU
 Guardian (drop-out PDP setelah periode
 / berhenti) retensi; notifikasi ke
 Guardian; audit log
 dicatat.

 Inactive Active Student daftar ulang Status kembali ke Trial
 (melalui proses terlebih dahulu;
 Trial baru) riwayat histori
 sebelumnya tetap
 tertaut via id_student
 lama.
 ------------------------------------------------------------------------
