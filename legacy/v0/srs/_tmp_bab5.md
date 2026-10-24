**BAB 5 --- MODEL DATA SISTEM (ERD)**

**5.1 Identifikasi Entitas**

Berdasarkan analisis kebutuhan fungsional dan aturan bisnis Equalizer-Stake,
sistem Equalizer-Stake memiliki sebelas entitas data utama. Setiap
entitas merepresentasikan objek bisnis nyata yang perlu disimpan,
dikelola, dan dilacak oleh sistem.

Catatan Penting --- Relasi 1 Guardian : N Student: Satu akun Guardian
(Guardian) dapat terhubung ke banyak Student (Student), misalnya
kakak-adik dalam satu pengguna. Seluruh notifikasi, tagihan, dan
komunikasi sistem dikirimkan ke akun Guardian, bukan ke student secara
individual. Relasi ini bersifat fundamental dan memengaruhi desain
seluruh entitas yang berhubungan dengan student.

*Tabel 5.1 Identifikasi Entitas Sistem Equalizer-Stake*

 --------- -------------- ----------------------------------------------------
 **No.** **Nama **Deskripsi Bisnis**
 Entitas**

 1 User Tabel induk akun sistem untuk semua peran (Owner,
 Admin, Instructor, Guardian). Menyimpan kredensial login
 dan data peran.

 2 Guardian Profil Guardian yang terhubung ke akun User. Satu
 Guardian mengelola banyak Student (relasi 1:N).

 3 Student Data student yang terdaftar di Equalizer-Stake. Setiap student
 terhubung ke satu Guardian dan dapat memiliki banyak
 sesi terjadwal.

 4 Teacher Profil instructor yang terhubung ke akun User.
 Menyimpan data spesialisasi instrumen dan jadwal
 mengajar.

 5 Room Data ruang studio (course reguler atau
 rehearsal/recording). Menyimpan kapasitas dan tipe
 penggunaan ruangan.

 6 Session Jadwal sesi course atau reservasi studio yang telah
 terkonfirmasi. Entitas inti yang menghubungkan
 Student, Teacher, dan Room dalam satu slot waktu.

 7 Invoice Tagihan bulanan yang diterbitkan kepada Guardian.
 Mencakup semua biaya course untuk seluruh Student di
 bawah satu Guardian, dilengkapi kode unik 3 digit.

 8 Payment Rekaman setiap transaksi pembayaran yang masuk,
 terhubung ke Invoice. Mendukung pembayaran sebagian
 (DP) dan pembayaran penuh.

 9 Attendance Rekaman kehadiran per student per sesi. Menyimpan
 status kehadiran, bukti foto, dan hasil validasi
 GPS/waktu.

 10 Notification Log setiap notifikasi yang dikirimkan sistem
 (WhatsApp/email), termasuk status pengiriman,
 saluran yang digunakan, dan hasil percobaan ulang.

 11 AuditLog Rekaman permanen seluruh perubahan data kritis di
 sistem. Bersifat tidak dapat diubah (immutable),
 disimpan minimal 7 tahun.
 --------- -------------- ----------------------------------------------------

**5.2 Entity Relationship Diagram (ERD)**

Cara Membaca ERD: Notasi kardinalitas: \|\|\--o{ berarti \"satu ke
banyak (opsional)\", \|\|\--\|\| berarti \"tepat satu ke satu\".

**\[GAMBAR: ERD --- Entity Relationship Diagram Equalizer-Stake\]**

***Penjelasan Relasi dan Constraint ERD:***

Catatan Penting --- Constraint Referensial (Revisi sesuai Perubahan 5):

**Constraint Referensial (Revisi):**

- CASCADE DELETE TIDAK digunakan untuk data PII (User, Guardian,
 Student, Teacher) demi kepatuhan UU PDP dan retensi data historis.

- Ganti dengan Soft Delete menggunakan flag is_deleted (boolean) atau
 deleted_at (timestamp) di setiap tabel utama.

- Untuk entitas relasi (Session, Attendance, Payment), gunakan RESTRICT
 DELETE atau SET NULL untuk menjaga integritas data historis selama 7
 tahun sesuai kewajiban arsip.

- Penghapusan fisik (hard delete) hanya dilakukan setelah data berusia ≥
 7 tahun, melalui proses pembersihan data (data purging) terjadwal.

*Tabel 5.2 Penjelasan Relasi dan Constraint ERD*

 -------------- ------------------ ------------------------ ----------------------
 **Relasi** **Kardinalitas** **Constraint **Indeks Fungsional**
 Referensial**

 Guardian → 1 : N (satu wali, SOFT DELETE --- gunakan id_guardian pada tabel
 Student banyak student) flag Student
 is_deleted/deleted_at;
 CASCADE DELETE tidak
 digunakan demi kepatuhan
 UU PDP.

 Guardian → 1 : N (satu wali, RESTRICT DELETE --- id_guardian,
 Invoice banyak tagihan per Invoice tidak dapat issued_date, status
 bulan) dihapus jika masih dalam pada tabel Invoice
 status aktif.

 Invoice → 1 : N (satu RESTRICT DELETE --- id_invoice, paid_at
 Payment tagihan, bisa Payment tidak dihapus pada tabel Payment
 banyak cicilan) secara otomatis; data
 historis dipertahankan 7
 tahun.

 Session → 1 : 1 (satu sesi, RESTRICT --- Attendance id_session, id_student
 Attendance satu rekaman hanya dapat dibuat jika (UNIQUE bersama) pada
 absensi per student) Session berstatus Locked tabel Attendance
 atau Completed.

 User → 1 : N (satu SET NULL pada id_actor entity_name,
 AuditLog pengguna, banyak jika akun pengguna entity_id, logged_at
 rekaman perubahan) dianonimkan, referensi pada tabel AuditLog
 di AuditLog diubah ke
 NULL (log tetap ada).

 Session → rescheduled_from SET NULL --- jika sesi rescheduled_from pada
 Session (self) referensi sesi asal dihapus, referensi tabel Session
 sebelumnya dijadikan NULL.
 -------------- ------------------ ------------------------ ----------------------

**5.3 Kamus Data (Data Dictionary)**

Kamus data berikut mendokumentasikan seluruh atribut dari 11 entitas
sistem Equalizer-Stake. Setiap tabel memuat: nama atribut, deskripsi
bisnis, tipe data, constraint, contoh nilai, catatan perlindungan data
(PII), dan aturan siklus hidup data. Kolom PII menggunakan kode: KRITIS
= data sangat sensitif wajib enkripsi; RENDAH = data semi-sensitif;
NON-PII = tidak mengandung data pribadi.

***Entitas 1: USER --- Akun Login Sistem***

*Tabel 5.3 Kamus Data Entitas USER*

 ---------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama **Deskripsi **Tipe Data** **Constraint** **Contoh **PII** **Siklus Hidup**
 Atribut** Bisnis** Nilai**
 --------------- ------------------ ---------------------------------------------------- ------------------- --------------- --------- -------------------
 id_user Identitas unik uuid NOT NULL · PRIMARY 550e8400-\... NON-PII Dipertahankan
 pengguna di KEY (anonim) 7 tahun
 seluruh sistem; setelah akun
 tidak pernah ditutup.
 berubah sejak akun
 dibuat.

 username Nama akun login varchar(50) NOT NULL · UNIQUE · admin_eqs RENDAH Dianonimkan menjadi
 unik; digunakan CHECK ≥6 karakter, REDACTED-\[hash\]
 untuk autentikasi alphanumeric + jika hak
 dan tercatat di underscore penghapusan
 audit log. disetujui.

 password_hash Hash kredensial varchar(255) NOT NULL --- KRITIS Disimpan dengan
 login; tidak Argon2id + salt
 pernah disimpan unik. Di-reset
 sebagai teks hanya melalui
 biasa; tidak prosedur recovery.
 pernah ditampilkan Dihapus saat akun
 di antarmuka atau dianonimkan.
 log apapun.

 role Peran pengguna enum(\'owner\',\'admin\',\'teacher\',\'guardian\') NOT NULL admin / NON-PII Dapat diubah oleh
 yang menentukan guardian Admin; setiap
 hak akses seluruh perubahan dicatat
 fitur (RBAC). di AuditLog.
 Setiap perubahan
 dicatat di
 AuditLog.

 is_active Status aktivasi boolean NOT NULL · DEFAULT true / false NON-PII Dipertahankan
 akun; akun true selama akun ada;
 nonaktif tidak tidak dihapus saat
 dapat login ke anonimisasi.
 sistem.

 last_login Waktu login timestamp with time zone NULLABLE 2026-04-05 RENDAH Diperbarui setiap
 terakhir berhasil 14:32:10+07 login berhasil;
 berdasarkan jam dianonimkan saat
 server; digunakan akun dihapus.
 untuk deteksi akun
 tidak aktif.

 created_at Waktu akun pertama timestamp with time zone NOT NULL · DEFAULT 2026-01-10 NON-PII Tidak dapat diubah;
 kali dibuat di CURRENT_TIMESTAMP · 09:00:00+07 dipertahankan 7
 sistem; dicatat IMMUTABLE tahun.
 otomatis oleh
 server.
 ---------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 2: GUARDIAN --- Profil Guardian (1 Guardian : N Student)***

*Tabel 5.4 Kamus Data Entitas GUARDIAN*

 ----------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus Hidup**
 Bisnis**
 ---------------- ------------------ --------------------------------------------------------------- ------------------- ---------------- --------- -------------------
 id_guardian Identitas unik uuid NOT NULL · PRIMARY a1b2c3d4-\... NON-PII Dipertahankan
 Guardian; KEY (anonim) 7 tahun
 digunakan sebagai setelah akun
 referensi di ditutup.
 entitas Student,
 Invoice, dan
 Notification.

 id_user Referensi ke akun uuid NOT NULL · FK → 550e8400-\... NON-PII Mengikuti siklus
 login Guardian di User(id_user) · hidup User.
 tabel User. SOFT DELETE

 full_name Nama lengkap Wali varchar(150) NOT NULL Sari Wulandari KRITIS Dienkripsi
 Murid sesuai AES-256-GCM
 identitas resmi; at-rest.
 ditampilkan di Dianonimkan menjadi
 invoice dan REDACTED-\[hash\]
 notifikasi. jika hak
 penghapusan
 disetujui.

 phone_whatsapp Nomor WhatsApp varchar(20) NOT NULL · UNIQUE · +6281234567890 KRITIS Dienkripsi
 aktif; tujuan FORMAT: AES-256-GCM
 utama pengiriman +62XXXXXXXXXX at-rest.
 notifikasi Dianonimkan dan
 non-kritis dan indeks pencarian
 cadangan dihapus saat hak
 notifikasi kritis. penghapusan
 diproses.

 email Alamat email; varchar(255) NOT NULL · UNIQUE · sari@email.com KRITIS Dienkripsi
 saluran utama FORMAT: RFC 5322 AES-256-GCM
 untuk notifikasi at-rest.
 kritis (konfirmasi Dianonimkan saat
 pembayaran, hak penghapusan
 pembekuan akun). diproses.

 address Alamat tempat text NULLABLE Jl. Melati No. KRITIS Dienkripsi
 tinggal Wali 5, Sleman AES-256-GCM
 Murid; digunakan at-rest.
 untuk keperluan Dianonimkan saat
 administratif. hak penghapusan
 diproses.

 account_status Status akun yang enum(\'active\',\'frozen\',\'inactive\',\'pending_deletion\') NOT NULL · DEFAULT active / frozen NON-PII Setiap perubahan
 memengaruhi akses \'active\' dicatat di
 semua Student di AuditLog.
 bawah Guardian Dipertahankan 7
 ini. Diubah tahun.
 otomatis sistem
 atau manual Admin.

 created_at Waktu profil timestamp with time zone NOT NULL · DEFAULT 2026-01-15 NON-PII Tidak dapat diubah;
 Guardian pertama CURRENT_TIMESTAMP · 10:00:00+07 dipertahankan 7
 kali dibuat; IMMUTABLE tahun.
 dicatat otomatis
 saat pendaftaran.
 ----------------------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 3: STUDENT --- Data Student Course***

*Tabel 5.5 Kamus Data Entitas STUDENT*

 -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh **PII** **Siklus
 Bisnis** Nilai** Hidup**
 ------------------- ------------------ ---------------------------------------------------------------------------------------------------------- ----------------------- --------------- --------- ---------------
 id_student Identitas unik uuid NOT NULL · PRIMARY KEY c3d4e5f6-\... NON-PII Dipertahankan
 student; digunakan (anonim) 7
 di entitas tahun setelah
 Session, student tidak
 Attendance, dan aktif.
 kalkulasi
 kelayakan ujian.

 id_guardian Referensi ke Wali uuid NOT NULL · FK → a1b2c3d4-\... NON-PII Mengikuti
 Murid yang Guardian(id_guardian) · siklus hidup
 mengelola student RESTRICT DELETE Guardian.
 ini. Tidak boleh
 NULL; setiap student
 wajib memiliki
 satu Guardian
 aktif.

 full_name Nama lengkap varchar(150) NOT NULL Budi Santoso KRITIS Dienkripsi
 student; ditampilkan AES-256-GCM
 di jadwal, at-rest.
 absensi, dan Dianonimkan
 laporan kelayakan saat hak
 ujian. penghapusan
 disetujui.

 birth_date Tanggal lahir date NULLABLE · CHECK \<= 2020-03-15 KRITIS Dienkripsi
 student; digunakan CURRENT_DATE AES-256-GCM
 untuk validasi at-rest.
 format kelas. Dianonimkan
 saat hak
 penghapusan
 diproses.

 instrument Instrumen music enum(\'biola\',\'piano\',\'gitar\',\'drum\',\'keyboard\',\'flute\',\'saxophone\',\'vokal\',\'ensemble\') NOT NULL piano / gitar NON-PII Dapat diubah
 yang dipelajari Admin; setiap
 student; menentukan perubahan
 penugasan instructor dan dicatat di
 tarif course. AuditLog.

 class_format Format kelas yang enum(\'private\',\'semi_private\',\'kelompok_kecil\') NOT NULL private NON-PII Dapat diubah
 diikuti student; Admin;
 memengaruhi tarif perubahan
 dan kapasitas dicatat di
 ruangan. AuditLog.

 level Tingkat kemampuan enum(\'pemula\',\'dasar\',\'menengah\',\'lanjut\') NOT NULL pemula / NON-PII Dapat diubah
 student berdasarkan menengah Admin setelah
 hasil audisi audisi ulang;
 penempatan. perubahan
 dicatat di
 AuditLog.

 enrollment_status Status pendaftaran enum(\'active\',\'cuti\',\'inactive\',\'trial\') NOT NULL · DEFAULT active / cuti NON-PII Diubah Admin;
 yang memengaruhi \'trial\' setiap
 penagihan dan perubahan
 penjadwalan. Cuti dicatat di
 maks. 1 bulan. AuditLog.

 enrollment_date Tanggal resmi date NULLABLE 2026-02-01 NON-PII Tidak dapat
 student terdaftar diubah setelah
 sebagai peserta diisi;
 course aktif dipertahankan 7
 (setelah audisi tahun.
 dan pembayaran
 admin fee).

 trial_date Tanggal kelas date NULLABLE 2026-01-25 NON-PII Dipertahankan
 percobaan (trial sebagai rekam
 class) student; jejak
 maks. 2 instrumen onboarding
 per hari saat student.
 trial.
 -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 4: TEACHER --- Profil Instructor***

*Tabel 5.6 Kamus Data Entitas TEACHER*

 -------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus
 Bisnis** Hidup**
 ---------------------------- ------------------ ------------------------------- ---------------- ---------------------------- --------- ---------------
 id_teacher Identitas unik uuid NOT NULL · d4e5f6g7-\... NON-PII Dipertahankan
 instructor; digunakan PRIMARY KEY (anonim) 7
 sebagai referensi tahun setelah
 di entitas Session instructor tidak
 dan kalkulasi aktif.
 hutang jam.

 id_user Referensi ke akun uuid NOT NULL · FK → 550e8400-\... NON-PII Mengikuti
 login Instructor di User(id_user) · siklus hidup
 tabel User. SOFT DELETE User.

 full_name Nama lengkap instructor; varchar(150) NOT NULL Ahmad Fauzi KRITIS Dienkripsi
 ditampilkan di AES-256-GCM
 jadwal course dan at-rest.
 laporan kehadiran. Dianonimkan
 saat hak
 penghapusan
 disetujui.

 phone Nomor telepon varchar(20) NULLABLE · +62856789012 KRITIS Dienkripsi
 aktif instructor; FORMAT: AES-256-GCM
 digunakan untuk +62XXXXXXXXXX at-rest.
 koordinasi Dianonimkan
 internal oleh saat hak
 Admin. penghapusan
 diproses.

 instruments_specialization Daftar instrumen json (array of string) NOT NULL · MIN 1 \[\"piano\",\"keyboard\"\] NON-PII Dapat
 yang dikuasai item diperbarui
 instructor; digunakan Admin;
 sistem untuk perubahan
 validasi dicatat di
 kesesuaian AuditLog.
 penjadwalan
 instructor-student.

 employment_type Jenis hubungan enum(\'tetap\',\'freelance\') NOT NULL tetap / freelance NON-PII Dapat diubah
 kerja instructor dengan Admin;
 Equalizer-Stake; perubahan
 memengaruhi dicatat di
 kebijakan AuditLog.
 kompensasi jam.

 is_active Status aktif instructor; boolean NOT NULL · true / false NON-PII Diubah Admin;
 instructor tidak aktif DEFAULT true perubahan
 tidak dapat dicatat di
 dijadwalkan pada AuditLog.
 sesi baru.
 -------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 5: ROOM --- Data Ruang Studio***

*Tabel 5.7 Kamus Data Entitas ROOM*

 ----------------------------------------------------------------------------------------------------------------------------------------
 **Nama **Deskripsi **Tipe Data** **Constraint** **Contoh **PII** **Siklus
 Atribut** Bisnis** Nilai** Hidup**
 ----------- ------------------ ---------------------------------------------- ---------------- --------------- --------- ---------------
 id_room Identitas unik uuid NOT NULL · e5f6g7h8-\... NON-PII Dipertahankan
 ruangan; digunakan PRIMARY KEY selama ruangan
 sebagai referensi terdaftar di
 di entitas Session sistem.
 untuk validasi
 konflik slot.

 room_name Nama ruangan yang varchar(50) NOT NULL · Studio A / NON-PII Dapat diubah
 ditampilkan di UNIQUE Ruang Rekam 1 Admin;
 antarmuka perubahan
 penjadwalan dan dicatat di
 laporan utilisasi. AuditLog.

 room_type Jenis penggunaan enum(\'course\',\'rehearsal\',\'recording\') NOT NULL course / NON-PII Dapat diubah
 ruangan yang rehearsal Admin;
 menentukan aturan perubahan
 booking. Tipe dicatat di
 recording AuditLog.
 memerlukan
 approval Admin.

 capacity Jumlah maksimum int NOT NULL · CHECK 1 (private) / 4 NON-PII Dapat diubah
 orang di ruangan \> 0 (kelompok) Admin;
 secara bersamaan; perubahan
 digunakan untuk dicatat di
 validasi format AuditLog.
 kelas.

 is_active Status aktif boolean NOT NULL · true / false NON-PII Diubah Admin;
 ruangan; ruangan DEFAULT true perubahan
 tidak aktif tidak dicatat di
 dapat dipilih saat AuditLog.
 penjadwalan baru.
 ----------------------------------------------------------------------------------------------------------------------------------------

***Entitas 6: SESSION --- Jadwal Sesi Course & Reservasi Studio***

*Tabel 5.8 Kamus Data Entitas SESSION*

 ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh **PII** **Siklus
 Bisnis** Nilai** Hidup**
 --------------------- ------------------ ------------------------------------------------------------------------------------ --------------------- --------------- --------- ---------------
 id_session Identitas unik uuid NOT NULL · PRIMARY f6g7h8i9-\... NON-PII Dipertahankan 7
 sesi; digunakan KEY tahun sebagai
 sebagai referensi rekam jejak
 utama di entitas operasional.
 Attendance.

 id_student Referensi ke student uuid NOT NULL · FK → c3d4e5f6-\... NON-PII Mengikuti
 yang mengikuti Student(id_student) · siklus hidup
 sesi ini. RESTRICT DELETE Student.

 id_teacher Referensi ke instructor uuid NOT NULL · FK → d4e5f6g7-\... NON-PII Mengikuti
 yang mengampu sesi Teacher(id_teacher) · siklus hidup
 ini; digunakan RESTRICT DELETE Teacher.
 untuk validasi
 konflik jadwal
 mengajar.

 id_room Referensi ke uuid NOT NULL · FK → e5f6g7h8-\... NON-PII Mengikuti
 ruangan tempat Room(id_room) · siklus hidup
 sesi berlangsung; RESTRICT DELETE Room.
 digunakan untuk
 validasi konflik
 slot ruangan.

 session_date Tanggal date NOT NULL · CHECK: 2026-04-07 NON-PII Tidak dapat
 pelaksanaan sesi; bukan hari Minggu & diubah setelah
 tidak boleh jatuh bukan libur nasional status Locked;
 pada hari Minggu perubahan hanya
 atau hari libur melalui
 nasional. mekanisme
 Reschedule.

 start_time Waktu mulai sesi time NOT NULL · CHECK: 14:30 NON-PII Tidak dapat
 berdasarkan jam dalam jam operasional diubah setelah
 operasional yang Locked.
 berlaku.

 end_time Waktu selesai time NOT NULL · CHECK: 15:15 NON-PII Tidak dapat
 sesi; sistem end_time + 10 menit ≤ diubah setelah
 memastikan ada start_time sesi Locked.
 jeda wajib 10 berikutnya
 menit antar sesi
 di ruangan yang
 sama.

 session_type Jenis sesi yang enum(\'reguler\',\'teori\',\'rehearsal\',\'recording\',\'kompensasi\') NOT NULL reguler / teori NON-PII Tidak dapat
 menentukan apakah diubah setelah
 kredit bulanan Locked;
 student terpotong ditetapkan saat
 atau tidak. pembuatan sesi.

 status Status siklus enum(\'draft\',\'pending\',\'locked\',\'completed\',\'cancelled\',\'rescheduled\') NOT NULL · DEFAULT locked / NON-PII Setiap
 hidup sesi sesuai \'draft\' completed perubahan
 State Transition status dicatat
 Diagram. Slot di AuditLog.
 Locked hanya bisa
 dibuka kembali
 oleh Admin
 disertai alasan.

 is_credit Penanda apakah boolean NOT NULL · DEFAULT true (reguler) NON-PII Dipertahankan 7
 sesi ini memotong true / false (teori) tahun sebagai
 kredit bulanan dasar
 student. Bernilai perhitungan
 false untuk sesi tagihan
 teori minggu ke-5 historis.
 dan sesi
 kompensasi.

 cancellation_reason Alasan pembatalan text NULLABLE · Wajib Instructor sakit NON-PII Dipertahankan
 sesi; wajib diisi diisi jika status = mendadak bersama rekam
 Admin saat \'cancelled\' sesi; tidak
 mengubah status dapat dihapus.
 dari Locked ke
 Cancelled.

 rescheduled_from Referensi ke uuid NULLABLE · FK → f6g7h8i9-\... NON-PII SET NULL jika
 id_session sesi Session(id_session) · sesi asal
 asli jika sesi ini SET NULL dihapus; rantai
 merupakan hasil reschedule
 penjadwalan ulang. tetap terlacak.

 created_by ID Admin yang uuid NOT NULL · FK → 550e8400-\... NON-PII SET NULL jika
 membuat sesi ini; User(id_user) · SET akun Admin
 digunakan untuk NULL dianonimkan;
 audit jejak entri sesi
 pembuatan jadwal. tetap ada.

 created_at Waktu sesi pertama timestamp with time zone NOT NULL · DEFAULT 2026-04-01 NON-PII Tidak dapat
 kali dibuat di CURRENT_TIMESTAMP · 09:15:00+07 diubah;
 sistem; dicatat IMMUTABLE dipertahankan 7
 otomatis oleh tahun.
 server.
 ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 7: INVOICE --- Tagihan Bulanan Guardian***

*Tabel 5.9 Kamus Data Entitas INVOICE*

 ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus Hidup**
 Bisnis**
 ---------------- ------------------ ------------------------------------------------------------------------------ ----------------------- ------------------ --------- ----------------
 id_invoice Identitas unik uuid NOT NULL · PRIMARY KEY g7h8i9j0-\... NON-PII Dipertahankan 7
 tagihan; digunakan tahun sesuai
 sebagai referensi kewajiban arsip
 di entitas keuangan.
 Payment.

 id_guardian Referensi ke uuid NOT NULL · FK → a1b2c3d4-\... NON-PII RESTRICT:
 Guardian penerima Guardian(id_guardian) · Invoice tidak
 tagihan. Satu RESTRICT DELETE dapat dihapus
 tagihan mencakup selama masih
 semua Student di aktif.
 bawah Guardian
 ini.

 invoice_number Nomor tagihan yang varchar(20) NOT NULL · UNIQUE · INV-202604-00023 NON-PII Dipertahankan 7
 dapat dibaca FORMAT: tahun; tidak
 manusia; indeks INV-YYYYMM-XXXXX dapat diubah
 pencarian utama setelah
 Admin saat diterbitkan.
 verifikasi
 pembayaran.

 unique_code Kode 3 digit yang char(3) NOT NULL · CHECK: 042 / 317 NON-PII Berlaku selama
 ditambahkan ke numerik 000--999 · Invoice
 nominal transfer UNIQUE per Guardian per berstatus aktif;
 untuk identifikasi bulan aktif kedaluwarsa
 pembayaran manual. setelah paid.
 Jika collision,
 auto-regenerate
 maks. 3x; jika
 masih gagal,
 diberi flag untuk
 assignment manual
 Admin.

 total_amount Total biaya course int NOT NULL · CHECK \> 0 750000 (Rp NON-PII Tidak dapat
 seluruh Student di 750.000) diubah setelah
 bawah Guardian Invoice
 dalam bulan diterbitkan
 berjalan, sebelum kecuali revisi
 denda. Satuan: disetujui Owner.
 rupiah penuh.

 penalty_amount Nilai denda int NOT NULL · DEFAULT 0 · 0 / 100000 NON-PII Di-set otomatis
 keterlambatan yang CHECK IN (0, 100000) sistem pada
 diterapkan tanggal 11;
 otomatis pada dicatat di
 tanggal 11. Nilai AuditLog.
 selalu 0 atau Rp
 100.000 (flat).

 status Status pembayaran enum(\'unpaid\',\'partial_paid\',\'paid\',\'overdue\',\'frozen\',\'waived\') NOT NULL · DEFAULT unpaid / paid NON-PII Setiap perubahan
 tagihan mengikuti \'unpaid\' dicatat di
 State Transition AuditLog beserta
 Diagram. Transisi timestamp dan ID
 otomatis oleh aktor.
 sistem; pembukaan
 status frozen oleh
 Admin.

 issued_date Tanggal tagihan date NOT NULL · IMMUTABLE · 2026-04-05 NON-PII Tidak dapat
 diterbitkan; CHECK: hari ke-5 dari diubah setelah
 selalu tanggal 5 bulan berjalan dibuat.
 bulan berjalan
 pukul 00:00.

 due_date Batas waktu date NOT NULL · CHECK: = 2026-04-10 NON-PII Tidak dapat
 pembayaran; selalu issued_date + 5 hari diubah setelah
 tanggal 10 pukul diterbitkan.
 23:59 pada bulan
 penerbitan.

 paid_date Tanggal Admin date NULLABLE 2026-04-08 NON-PII Di-set saat
 mengkonfirmasi Admin
 pelunasan tagihan; mengkonfirmasi
 NULL selama belum pembayaran
 lunas. penuh; dicatat
 di AuditLog.

 created_at Waktu Invoice timestamp with time zone NOT NULL · DEFAULT 2026-04-05 NON-PII Tidak dapat
 dibuat oleh proses CURRENT_TIMESTAMP · 00:00:15+07 diubah;
 penagihan IMMUTABLE dipertahankan 7
 otomatis. tahun.
 ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 8: PAYMENT --- Rekaman Transaksi Pembayaran***

*Tabel 5.10 Kamus Data Entitas PAYMENT*

 -------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus
 Bisnis** Hidup**
 -------------------- ------------------ ------------------------------------- --------------------- ------------------------- --------- ---------------
 id_payment Identitas unik uuid NOT NULL · PRIMARY h8i9j0k1-\... NON-PII Dipertahankan 7
 transaksi KEY tahun sesuai
 pembayaran; satu kewajiban arsip
 Invoice bisa keuangan.
 memiliki banyak
 Payment (DP +
 pelunasan).

 id_invoice Referensi ke uuid NOT NULL · FK → g7h8i9j0-\... NON-PII Mengikuti
 Invoice yang Invoice(id_invoice) · siklus hidup
 dibayar melalui RESTRICT DELETE Invoice; data
 transaksi ini. historis
 dipertahankan 7
 tahun.

 verified_by ID Admin yang uuid NOT NULL · FK → 550e8400-\... NON-PII SET NULL jika
 melakukan User(id_user) · SET akun Admin
 konfirmasi NULL dianonimkan;
 pembayaran; rekaman Payment
 dicatat untuk tetap ada.
 akuntabilitas
 verifikasi manual.

 amount_paid Jumlah rupiah yang int NOT NULL · CHECK \> 0 525042 (Rp 525.000 + kode NON-PII Tidak dapat
 ditransfer dalam 042) diubah setelah
 transaksi ini, dikonfirmasi
 termasuk 3 digit Admin;
 kode unik di akhir dipertahankan 7
 nominal. Satuan: tahun.
 rupiah penuh.

 payment_type Jenis pembayaran enum(\'dp\',\'pelunasan\',\'full\') NOT NULL dp / full NON-PII Tidak dapat
 dalam transaksi diubah setelah
 ini; membedakan dikonfirmasi.
 pembayaran
 sebagian (DP)
 dengan pelunasan
 penuh.

 transfer_reference Nomor referensi varchar(100) NULLABLE TRF-BCA-20260405-001234 NON-PII Dipertahankan 7
 transfer dari tahun sebagai
 bank; digunakan bukti
 Admin untuk transaksi.
 verifikasi silang
 dengan mutasi
 rekening EQS
 music.

 paid_at Waktu transfer timestamp with time zone NOT NULL 2026-04-08 10:30:00+07 NON-PII Dipertahankan 7
 dilakukan oleh tahun; tidak
 Guardian (diinput dapat diubah
 Admin berdasarkan setelah
 bukti transfer). dikonfirmasi.

 verified_at Waktu Admin timestamp with time zone NOT NULL · Di-set 2026-04-08 11:05:22+07 NON-PII Dipertahankan 7
 mengkonfirmasi dan otomatis saat Admin tahun; tidak
 memvalidasi klik konfirmasi dapat diubah
 pembayaran ini di setelah diisi.
 sistem.
 -------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 9: ATTENDANCE --- Rekaman Kehadiran Student per Sesi***

**Aturan Dominasi Status (Perubahan 5):**

- Jika verification_status = pending_review, maka nilai status diabaikan
 (di-ignore) sampai Admin menyelesaikan verifikasi.

- Setelah verifikasi selesai: Jika Admin menyetujui →
 verification_status = manually_approved, status mengikuti keputusan
 Admin. Jika Admin menolak → verification_status = manually_rejected,
 status diubah menjadi absent_no_show atau absent_excused sesuai
 keputusan Admin.

- Jika verification_status = auto_approved, status berlaku normal.

*Tabel 5.11 Kamus Data Entitas ATTENDANCE*

 --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi Bisnis** **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus
 Hidup**
 --------------------- --------------------- -------------------------------------------------------------------------------------------------- --------------------- --------------------------------------------- --------- ---------------
 id_attendance Identitas unik uuid NOT NULL · PRIMARY i9j0k1l2-\... NON-PII Dipertahankan 7
 rekaman kehadiran; KEY tahun sebagai
 relasi 1:1 dengan rekam jejak
 pasangan id_session + kehadiran.
 id_student.

 id_session Referensi ke sesi uuid NOT NULL · FK → f6g7h8i9-\... NON-PII RESTRICT:
 yang diabsensikan. Session(id_session) · rekaman absensi
 RESTRICT DELETE · tidak dapat
 UNIQUE bersama dihapus selama
 id_student sesi ada.

 id_student Referensi ke student uuid NOT NULL · FK → c3d4e5f6-\... NON-PII Mengikuti
 yang diabsensikan Student(id_student) · siklus hidup
 pada sesi ini. RESTRICT DELETE · Student.
 UNIQUE bersama
 id_session

 recorded_by ID Instructor yang mengisi uuid NOT NULL · FK → 550e8400-\... NON-PII SET NULL jika
 dan mengupload User(id_user) · SET akun Instructor
 absensi sesi ini. NULL dianonimkan;
 rekaman absensi
 tetap ada.

 status Status kehadiran enum(\'present\',\'absent_excused\',\'absent_no_show\',\'pending_verification\',\'rescheduled\') NOT NULL present / absent_no_show NON-PII Hanya dapat
 student pada sesi ini; diubah Admin
 mengikuti State (koreksi);
 Transition Diagram setiap
 Kehadiran. Catatan: perubahan
 jika dicatat di
 verification_status = AuditLog.
 pending_review, nilai
 atribut ini diabaikan
 sampai Admin
 menyelesaikan
 verifikasi.

 photo_storage_path Path penyimpanan foto varchar(500) NULLABLE /storage/attendance/2026/04/07/att_uuid.jpg KRITIS Mengandung
 verifikasi kehadiran wajah student.
 di server; NULL jika Disimpan masa
 student absen dan tidak aktif student + 2
 ada foto. tahun. Dihapus
 permanen jika
 hak penghapusan
 disetujui.

 photo_hash Hash SHA-256 dari char(64) NULLABLE · FORMAT: a3f5b1c2d4\... NON-PII Dipertahankan 7
 file foto; SHA-256 hex string tahun bahkan
 dipertahankan sebagai (64 karakter) setelah foto
 jejak audit bahkan asli dihapus.
 setelah foto asli
 dihapus.

 gps_latitude Koordinat lintang GPS decimal(10,7) NULLABLE · RANGE: -7.7265000 NON-PII Dipertahankan
 perangkat Instructor saat -90.0 s.d. 90.0 bersama rekaman
 upload foto; absensi selama
 divalidasi dalam 7 tahun.
 radius 150 meter dari
 studio (-7.7265).

 gps_longitude Koordinat bujur GPS decimal(10,7) NULLABLE · RANGE: 110.4162000 NON-PII Dipertahankan
 perangkat Instructor saat -180.0 s.d. 180.0 bersama rekaman
 upload foto; absensi selama
 divalidasi dalam 7 tahun.
 radius 150 meter dari
 studio (110.4162).

 gps_valid Hasil validasi sistem boolean NOT NULL · DEFAULT true / false NON-PII Tidak dapat
 apakah koordinat GPS false · Di-set diubah setelah
 berada dalam radius otomatis oleh sistem diisi oleh
 150 meter dari saat upload sistem.
 koordinat studio EQS
 music (toleransi GPS
 indoor final ---
 TDB-05 RESOLVED).

 face_valid Hasil validasi sistem boolean NOT NULL · DEFAULT true / false NON-PII Tidak dapat
 apakah wajah student false · Di-set diubah setelah
 terdeteksi minimal otomatis oleh sistem diisi oleh
 30% dari area foto saat upload sistem.
 yang diunggah.

 server_timestamp Waktu penerimaan foto timestamp with time zone NOT NULL · Di-set 2026-04-07 14:35:22+07 NON-PII Tidak dapat
 di server berdasarkan otomatis server saat diubah;
 jam server (bukan jam menerima upload · dipertahankan 7
 perangkat Instructor); IMMUTABLE tahun.
 tidak dapat
 dimanipulasi
 pengguna.

 verification_status Status verifikasi enum(\'auto_approved\',\'pending_review\',\'manually_approved\',\'manually_rejected\') NOT NULL · DEFAULT auto_approved / pending_review NON-PII Setiap
 absensi; berubah ke \'auto_approved\' perubahan oleh
 pending_review jika Admin dicatat
 salah satu validasi di AuditLog.
 (GPS/wajah) gagal dan
 menunggu review
 Admin. Aturan
 dominasi:
 pending_review
 membuat nilai atribut
 \'status\' diabaikan.

 verified_by ID Admin yang uuid NULLABLE · FK → 550e8400-\... NON-PII SET NULL jika
 melakukan verifikasi User(id_user) · SET akun Admin
 manual; NULL jika NULL dianonimkan;
 absensi disetujui rekaman tetap
 otomatis oleh sistem. ada.

 verified_at Waktu Admin timestamp with time zone NULLABLE 2026-04-07 16:00:00+07 NON-PII Dipertahankan 7
 menyelesaikan tahun sebagai
 verifikasi manual bukti proses
 absensi; NULL jika verifikasi.
 disetujui otomatis.
 --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 10: NOTIFICATION --- Log Pengiriman Notifikasi***

*Tabel 5.12 Kamus Data Entitas NOTIFICATION*

 ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama Atribut** **Deskripsi **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus
 Bisnis** Hidup**
 ------------------ ------------------ --------------------------------------------------------------------------------------------------------------------------------- ---------------- ----------------------- --------- ---------------
 id_notification Identitas unik uuid NOT NULL · j0k1l2m3-\... NON-PII Dipertahankan 1
 entri log PRIMARY KEY tahun untuk
 notifikasi. keperluan
 analisis
 kegagalan;
 setelah itu
 dapat
 diarsipkan.

 id_recipient Referensi ke akun uuid NOT NULL · FK → 550e8400-\... NON-PII SET NULL jika
 User penerima User(id_user) · akun
 notifikasi SET NULL dianonimkan;
 (umumnya Guardian; log notifikasi
 dapat juga Instructor tetap ada.
 untuk notifikasi
 jadwal).

 channel Saluran pengiriman enum(\'whatsapp\',\'email\') NOT NULL whatsapp / email NON-PII Tidak dapat
 utama yang diubah setelah
 digunakan untuk notifikasi
 notifikasi ini. dikirim.

 category Kategori enum(\'reminder_jadwal\',\'konfirmasi_pembayaran\',\'tagihan\',\'freeze_unfreeze\',\'perubahan_jadwal\',\'verifikasi_absensi\') NOT NULL tagihan / NON-PII Dipertahankan
 notifikasi yang konfirmasi_pembayaran bersama entri
 menentukan log.
 prioritas saluran:
 kritis (email
 utama) atau
 non-kritis
 (WhatsApp utama).

 message_body Isi pesan text NOT NULL Pengingat: Sesi piano RENDAH Dianonimkan
 notifikasi yang Budi besok pukul 14.30. setelah 2 tahun
 dikirimkan; dapat jika memuat
 memuat nama nama pengguna.
 Guardian atau
 detail tagihan.

 status Status pengiriman enum(\'queued\',\'sent\',\'delivered\',\'failed\') NOT NULL · delivered / failed NON-PII Diperbarui
 notifikasi melalui DEFAULT otomatis
 saluran utama; \'queued\' sistem;
 diperbarui sistem dipertahankan 1
 setelah setiap tahun.
 percobaan.

 retry_count Jumlah percobaan int NOT NULL · 0 / 3 NON-PII Dipertahankan
 ulang yang telah DEFAULT 0 · bersama entri
 dilakukan sistem; CHECK 0--3 log selama 1
 maksimal 3 kali tahun.
 dengan jeda
 exponential (10,
 20, 40 detik).

 fallback_channel Saluran cadangan enum(\'email\',\'whatsapp\',\'none\') NOT NULL · email / none NON-PII Di-set otomatis
 yang digunakan DEFAULT \'none\' sistem saat
 jika saluran utama fallback
 gagal setelah 3 diaktifkan.
 percobaan.

 fallback_status Status pengiriman enum(\'not_used\',\'sent\',\'delivered\',\'failed\') NOT NULL · delivered / not_used NON-PII Diperbarui
 melalui saluran DEFAULT otomatis
 cadangan; bernilai \'not_used\' sistem;
 \'not_used\' dipertahankan 1
 selama saluran tahun.
 utama berhasil.

 dispatched_at Waktu notifikasi timestamp with time zone NOT NULL 2026-04-07 13:30:05+07 NON-PII Dipertahankan 1
 pertama kali tahun untuk
 diproses dan analisis
 dikirimkan oleh performa
 sistem (dari notifikasi.
 antrian).

 delivered_at Waktu konfirmasi timestamp with time zone NULLABLE 2026-04-07 13:30:18+07 NON-PII Dipertahankan 1
 pengiriman tahun;
 berhasil diterima diarsipkan
 dari gateway; NULL setelahnya.
 jika masih dalam
 proses atau gagal.
 ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

***Entitas 11: AUDITLOG --- Rekaman Perubahan Data Permanen***

*Tabel 5.13 Kamus Data Entitas AUDITLOG*

 ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 **Nama **Deskripsi **Tipe Data** **Constraint** **Contoh Nilai** **PII** **Siklus
 Atribut** Bisnis** Hidup**
 --------------- ------------------ ------------------------------------------------------------------------------------------------------------------------- ------------------- ------------------------- --------- ---------------
 id_log Identitas unik uuid NOT NULL · PRIMARY k1l2m3n4-\... NON-PII Dipertahankan
 entri audit log; KEY · IMMUTABLE permanen,
 tidak pernah minimal 7
 berubah dan tidak tahun. Tidak
 dapat dihapus oleh dapat dihapus.
 pengguna manapun.

 id_actor ID pengguna yang uuid NULLABLE · FK → 550e8400-\... (Admin) / NON-PII SET NULL saat
 melakukan User(id_user) · SET null (sistem) akun
 tindakan. Bernilai NULL dianonimkan;
 NULL jika tindakan log tetap
 dilakukan sistem dipertahankan 7
 otomatis atau jika tahun.
 akun aktor telah
 dianonimkan; entri
 log tetap ada.

 entity_name Nama entitas varchar(50) NOT NULL · Session / Invoice / NON-PII Tidak dapat
 (tabel) yang IMMUTABLE Guardian diubah setelah
 datanya diubah; dibuat.
 digunakan untuk
 memfilter log
 berdasarkan jenis
 data.

 entity_id ID baris data uuid NOT NULL · f6g7h8i9-\... (id sesi NON-PII Tidak dapat
 spesifik yang IMMUTABLE yang diubah) diubah setelah
 diubah; dibuat;
 dikombinasikan dipertahankan 7
 dengan entity_name tahun.
 untuk melacak
 rekaman tertentu.

 action_type Jenis tindakan enum(\'create\',\'update\',\'delete\',\'login\',\'logout\',\'freeze\',\'unfreeze\',\'verify_payment\',\'data_erasure\') NOT NULL · verify_payment / freeze NON-PII Tidak dapat
 yang dilakukan IMMUTABLE diubah setelah
 pada data; dibuat.
 mencakup
 operasional harian
 hingga tindakan
 kepatuhan.

 before_state Snapshot kondisi json NULLABLE · {\"status\":\"unpaid\"} KRITIS Field PII
 data sebelum IMMUTABLE · di-mask
 perubahan dalam before_state = NULL otomatis
 format JSON; NULL untuk action_type = setelah 2
 untuk tindakan \'create\' tahun. Metadata
 CREATE. struktur
 dipertahankan 7
 tahun.

 after_state Snapshot kondisi json NULLABLE · {\"status\":\"paid\"} KRITIS Field PII
 data setelah IMMUTABLE · di-mask
 perubahan dalam after_state = NULL otomatis
 format JSON; NULL untuk action_type = setelah 2
 untuk tindakan \'delete\' tahun. Metadata
 DELETE. struktur
 dipertahankan 7
 tahun.

 ip_address Alamat IP varchar(45) NULLABLE · FORMAT: 192.168.1.10 / RENDAH Dianonimkan
 perangkat aktor IPv4 atau IPv6 2001:db8::1 setelah 2
 saat tindakan tahun;
 dilakukan; dipertahankan
 digunakan untuk sebagai
 investigasi metadata anonim
 insiden keamanan. 7 tahun.

 log_category Kategori log untuk enum(\'operasional\',\'keamanan\',\'kepatuhan\',\'sistem\') NOT NULL · kepatuhan (hak hapus NON-PII Tidak dapat
 keperluan filter IMMUTABLE data) / keamanan (login diubah setelah
 dan akses berbasis gagal) dibuat.
 peran: Owner
 melihat semua
 kategori; Admin
 hanya melihat
 kategori
 operasional.

 logged_at Waktu entri log timestamp with time zone NOT NULL · DEFAULT 2026-04-08 11:05:22+07 NON-PII Tidak dapat
 dicatat di sistem CURRENT_TIMESTAMP · diubah;
 berdasarkan jam IMMUTABLE dipertahankan
 server; tidak permanen
 dapat dimanipulasi minimal 7
 pengguna. tahun.

 is_anonymized Penanda apakah boolean NOT NULL · DEFAULT false (baru) / true NON-PII Diubah ke true
 field PII dalam false (sudah anonim) otomatis
 entri ini sudah setelah 2 tahun
 dianonimkan oleh dari logged_at.
 proses otomatis
 sistem setelah 2
 tahun.
 ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

**BAB 6 --- TRACEABILITY MATRIX & KEPATUHAN**

**6.1 Matriks Traceability Kebutuhan**

Matriks ini memastikan setiap Functional Requirement (FR) dapat dilacak
ke use case yang relevan, memiliki kriteria penerimaan yang terukur, dan
terpetakan ke standar kepatuhan atau keamanan yang berlaku.
Ketidakhadiran pemetaan di salah satu kolom berarti requirement tersebut
belum siap untuk fase pengembangan.

Kode Kepatuhan: UU PDP = Undang-Undang No. 27/2022 tentang Perlindungan
Data Pribadi \| OWASP = Open Web Application Security Project Top 10
(2021) \| ISO 27001 = Standar Manajemen Keamanan Informasi

*Tabel 6.1 Matriks Traceability Kebutuhan Fungsional*

 ---------------------------------------------------------------------------------------------------------------
 **Kode **Nama Fitur** **Use Case **Test **Langkah-Langkah** **Hasil yang **Kepatuhan**
 FR** Terkait** Case Diharapkan**
 ID**
 -------- ----------------- ------------------- -------- --------------------- ----------------- ---------------
 FR-01 Login & UC-AUTH-01, TC-01 1\. Buka URL. 2. Sistem
 Autentikasi RBAC UC-AUTH-02 Input username & menerbitkan token
 password. 3. Validasi sesi, mencatat
 kredensial ≤2 detik. waktu login di
 4. Terbitkan JWT. 5. audit log, dan
 Catat audit log. 6. menampilkan
 Tampilkan dashboard dashboard sesuai
 sesuai peran. peran dalam ≤ 2
 detik

 FR-02 Penjadwalan + UC-SCHED-01, TC-02 1\. Buka form jadwal Sistem
 Validasi Konflik UC-SCHED-02 baru. 2. Pilih student, memvalidasi
 Real-Time instructor, ruangan. 3. ketersediaan ≤ 1
 Pilih tanggal & detik; jika
 waktu. 4. Validasi tersedia, jadwal
 konflik ≤1 dtk. 5. tersimpan dan
 Simpan status notifikasi
 Locked + catat audit dikirim ke Instructor;
 log. 6. Kirim jika konflik,
 notifikasi ke Instructor & tampilkan daftar
 Guardian. slot alternatif
 yang tersedia

 FR-03 Penerbitan UC-BILL-01, TC-03 1\. Cron job aktif Tagihan terbit
 Tagihan UC-BILL-02 tanggal 5 pukul untuk semua akun
 Otomatis + Kode 00:00. 2. Ambil Guardian aktif
 Unik daftar Guardian dalam ≤ 5 menit,
 aktif. 3. Hitung notifikasi
 total biaya per dikirim ke
 Guardian. 4. Generate WhatsApp/email,
 kode unik 3 digit setiap tagihan
 (000--999). 5. Cek memiliki kode
 duplikasi kode (maks. unik 3 digit yang
 3x). 6. Simpan tidak duplikat
 invoice. 7. Kirim
 notifikasi WhatsApp +
 email.

 FR-04 Denda Otomatis & UC-BILL-03, TC-04 1\. Cron job aktif Denda Rp 100.000
 Pembekuan Akun UC-BILL-04 tanggal 11 pukul ditambahkan, akun
 00:00. 2. Cek status dibekukan,
 tagihan semua notifikasi
 Guardian. 3. dikirim ke Wali
 Tambahkan denda Rp Murid (WhatsApp +
 100.000. 4. Ubah email), status
 status akun → Frozen. tercatat di audit
 5. Catat di audit log
 log. 6. Kirim
 notifikasi WhatsApp +
 email.

 FR-05 Absensi Digital + UC-ATTEND-01, TC-05 1\. Buka menu Absensi Sistem
 Validasi UC-ATTEND-02 Hari Ini. 2. Pilih memvalidasi
 Foto/GPS/Waktu sesi aktif. 3. Ambil waktu, GPS, dan
 foto student (wajah foto dalam ≤ 5
 ≥30% frame). 4. Tekan detik; jika semua
 Konfirmasi Absensi. valid → status
 5. Validasi waktu \'Hadir\'; jika
 server. 6. Validasi ada validasi
 GPS radius 150m. 7. gagal → status
 Validasi proporsi \'Menunggu
 wajah ≥30%. 8. Simpan Verifikasi\' dan
 status Hadir + potong Admin diberitahu
 kredit. 9. Kirim
 notifikasi ke Wali
 Murid.

 FR-06 Notifikasi UC-NOTIF-01, TC-06 1\. Event pemicu Notifikasi
 Multi-Saluran + UC-NOTIF-02 aktif. 2. Generate terkirim ke Wali
 Fallback pesan notifikasi. 3. Murid dalam ≤ 15
 Kirim via WhatsApp detik; status
 Gateway. 4. Terima pengiriman
 konfirmasi delivered. tercatat di log
 5. Catat status di yang dapat
 log notifikasi. dilihat Admin

 FR-07 Pelacakan UC-ATTEND-03 TC-07 1\. Admin/Owner buka Sistem
 Kelayakan Ujian laporan kelayakan menampilkan
 ujian. 2. Ambil data daftar student
 kehadiran 3 bulan beserta status
 terakhir per student. \'Siap
 3. Cek status akun Ujian\'/\'Belum
 (aktif/beku). 4. Siap Ujian\'
 Hitung % kehadiran dengan persentase
 per student. 5. Hitung kehadiran dan
 jumlah absen tanpa jumlah absen
 kabar. 6. Evaluasi tanpa kabar,
 semua 3 kondisi. 7. diperbarui
 Tampilkan status real-time
 dengan detail alasan.

 FR-08 Penyimpanan Draft UC-SYSTEM-01 TC-08 1\. Admin mengisi Sistem
 Otomatis & form. 2. Auto-save menampilkan
 Pemulihan Sesi lokal setiap 30 \'Draft
 detik. 3. Sesi sebelumnya
 timeout. 4. Admin ditemukan.
 login kembali. 5. Pulihkan?\' dan
 Sistem deteksi draft mengembalikan
 tersimpan. 6. semua data tanpa
 Tampilkan prompt kehilangan data
 \'Pulihkan draft?\' \> 1 menit
 7. Admin klik terakhir
 Pulihkan. 8. Form
 terisi kembali.

 FR-09 Hak Penghapusan UC-COMPLIANCE-01, TC-09 1\. Guardian isi Sistem UU PDP Pasal 19
 Data (UU PDP UC-COMPLIANCE-02 formulir hapus data. mengeksekusi
 Pasal 19) 2. Sistem cek anonimisasi PII,
 kelayakan (tidak ada menghapus foto,
 tagihan aktif). 3. mempertahankan
 Notifikasi ke Admin. metadata anonim
 4. Admin verifikasi di audit log, dan
 identitas pemohon. 5. mengirimkan
 Admin setujui konfirmasi ke
 permintaan. 6. Guardian dalam
 Anonimisasi data PII ≤ 14 hari kerja
 → REDACTED-\[hash\].
 7. Hapus permanen
 foto absensi. 8.
 Catat di audit log
 kategori \'Tindakan
 Kepatuhan\'. 9. Kirim
 konfirmasi ke Wali
 Murid (≤14 hari
 kerja).

 FR-10 Manajemen UC-GUARDIAN-01 TC-10 1\. Guardian login. Sistem
 Multi-Student oleh 2. Load daftar semua menampilkan
 Guardian student terdaftar. 3. jadwal, status
 Tampilkan dashboard kehadiran, dan
 terpadu. 4. Wali tagihan kedua
 Murid klik nama student student dalam satu
 untuk switch profil. tampilan terpadu,
 5. Dashboard refresh dengan kemampuan
 menampilkan data beralih antar
 student yang dipilih. profil student

 FR-11 Audit Log UC-AUDIT-01, TC-11 1\. Pengguna Sistem membuat ISO 27001 A.12;
 Permanen & UC-AUDIT-02 melakukan aksi entri audit log UU PDP Pasal 22
 Immutable perubahan data yang mencatat
 kritis. 2. Sistem waktu perubahan,
 capture before state. ID Admin, jadwal
 3. Proses perubahan sebelum dan
 data. 4. Tulis entri sesudah, dan
 audit log. 5. alasan Admin ---
 Konfirmasi log entri tidak dapat
 berhasil disimpan. 6. diedit atau
 Perubahan data dihapus
 dikonfirmasi ke
 pengguna.

 FR-12 Pengelolaan Libur UC-SCHED-03, TC-12 1\. Sistem deteksi Jadwal sesi
 Nasional & Minggu UC-SCHED-04 hari course = hari dipindah ke hari
 ke-5 libur nasional. 2. yang sama minggu
 Cari slot tersedia berikutnya; Wali
 minggu berikutnya. 3. Murid dan Instructor
 Pindahkan jadwal ke menerima
 slot baru. 4. Kredit notifikasi;
 student tidak dipotong. kredit student
 5. Kirim notifikasi tidak terpotong
 perubahan jadwal ke untuk sesi yang
 Instructor & Guardian. dipindah

 FR-13 Dashboard UC-REPORT-01, TC-13 1\. Owner login. 2. Sistem
 Monitoring Owner UC-REPORT-02 Load dashboard menampilkan semua
 (Read-Only) monitoring (≤3 dtk). ringkasan
 3. Tampilkan KPI: operasional
 pendapatan, diperbarui dalam
 kehadiran, akun beku, ≤ 3 detik, dengan
 utilisasi studio. 4. visualisasi data
 Owner buka tab audit yang mudah dibaca
 log ringkas. 5. tanpa keahlian
 Filter dan cari data teknis
 laporan. 6. Owner
 klik Ekspor Laporan.
 7. Unduh laporan
 format cetak.

 FR-14 Reservasi Studio UC-STUDIO-01, TC-14 1\. Guardian buka Slot terkunci,
 Rehearsal & UC-STUDIO-02 kalender studio. 2. notifikasi
 Recording Pilih tipe: konfirmasi
 Rehearsal. 3. Pilih dikirim ke Wali
 tanggal & jam. 4. Cek Murid, slot tidak
 ketersediaan lagi tersedia
 real-time. 5. untuk pengguna
 Konfirmasi reservasi. lain
 6. Kunci slot. 7.
 Kirim notifikasi
 konfirmasi ke Wali
 Murid.

 FR-15 Verifikasi UC-BILL-05 TC-15 1\. Guardian Status tagihan
 Pembayaran Manual transfer (nominal + berubah ke
 oleh Admin kode unik 3 digit). \'Lunas\', akun
 2. Guardian kirim aktif kembali
 bukti transfer ke (jika beku),
 Admin. 3. Admin buka notifikasi
 dashboard tagihan. 4. terkirim ke Wali
 Cocokkan kode unik Murid, dan
 dengan tagihan aktif. tindakan tercatat
 5. Klik Konfirmasi di audit log
 Pembayaran. 6. Status dalam ≤ 2 menit
 tagihan → Lunas. 7.
 Akun Guardian →
 Aktif. 8. Catat di
 audit log. 9. Kirim
 notifikasi konfirmasi
 ke Guardian.
 ---------------------------------------------------------------------------------------------------------------

***6.1.1 NFR Security Compliance Matrix (UU PDP & ISO 27001)***

Matriks berikut memetakan setiap Non-Functional Requirement (NFR)
Keamanan ke standar kepatuhan yang berlaku. Ini memastikan bahwa
kepatuhan tidak hanya pada prosedur penghapusan data (FR-09) tetapi juga
pada arsitektur keamanan data secara keseluruhan.

*Tabel 6.2 NFR Security Compliance Matrix (UU PDP & ISO 27001)*

 --------------------------------------------------------------------------
 **Kode **Deskripsi **Standar **Metode Verifikasi**
 NFR** Keamanan** Kepatuhan**
 -------- ------------------- ---------------- ----------------------------
 SEC-01 Enkripsi data PII UU PDP Pasal 20; Inspeksi konfigurasi
 at-rest ISO 27001 A.10 database; Audit enkripsi
 (AES-256-GCM) pada per-tabel
 User, Guardian,
 Student, Teacher

 SEC-02 Enkripsi data UU PDP Pasal 20; SSL Labs test; Wireshark
 in-transit (TLS ISO 27001 A.13 verifikasi
 1.2+) untuk semua
 koneksi

 SEC-03 Minimasi data UU PDP Pasal 16 Review skema database;
 (hanya mengumpulkan Penghapusan field yang tidak
 data esensial) perlu

 SEC-04 Pembatasan OWASP A07:2021; Uji penetrasi; Audit log
 percobaan login (5 ISO 27001 A.9 insiden
 gagal → kunci 15
 menit)

 SEC-05 Hak penghapusan UU PDP Pasal 19 Uji end-to-end prosedur
 data (prosedur penghapusan
 FR-09)

 SEC-06 Audit log UU PDP Pasal 22; Inspeksi tabel AuditLog;
 immutability & ISO 27001 A.12 Verifikasi tidak ada
 retensi 7 tahun mekanisme delete

 SEC-07 Anonimisasi data UU PDP Pasal 17 Verifikasi proses
 PII setelah 2 tahun anonimisasi otomatis

 SEC-08 Pembatasan akses ISO 27001 A.9 Uji akses per peran;
 berdasarkan peran Verifikasi JWT claims
 (RBAC)
 --------------------------------------------------------------------------

**6.2 Matriks Log Perubahan Dokumen**

Matriks berikut mencatat seluruh riwayat perubahan dokumen SRS sejak
versi draft awal hingga versi yang disetujui. Setiap perubahan pada
dokumen ini wajib dicatat di sini sebelum didistribusikan kembali kepada
pemangku kepentingan.

*Tabel 6.3 Matriks Log Perubahan Dokumen SRS*

 ----------------------------------------------------------------------------------------
 **Versi** **Tanggal** **Pemohon** **Bab yang **Alasan **Status
 Berubah** Perubahan** Persetujuan**
 ----------- ------------- ------------- ----------- ------------------- ----------------
 v0.1 Januari 2026 Kelompok 7 Semua Draft awal --- Draft (Belum
 struktur dokumen, Direview)
 template, dan
 pengumpulan konteks
 bisnis Equalizer-Stake

 v0.2 Februari 2026 Kelompok 7 BAB 1, 2, 3 Integrasi hasil Review Internal
 wawancara Kelompok
 stakeholder:
 penambahan aturan
 bisnis denda
 Rp100.000, validasi
 foto GPS, relasi
 1:N
 Guardian-Student,
 siklus tagihan
 tanggal 5--10

 v0.3 Februari 2026 Kelompok 7 BAB 3, 4 Penambahan FR-08 Review Internal
 (Draft Auto-Save), Kelompok
 FR-09 (Hak
 Penghapusan UU
 PDP), FR-12 (Libur
 Nasional);
 penyempurnaan State
 Transition Diagram

 v0.9 Maret 2026 Kelompok 7 BAB 5, 6, 7 Penambahan ERD Review
 lengkap, Data Pra-Submission
 Dictionary 11
 entitas,
 Traceability
 Matrix, dan
 Lampiran

 v1.0 Maret 2026 Kelompok 7 Semua Finalisasi dokumen Approved
 lengkap BAB 1--7; (Baseline) ---
 penyeragaman Menunggu Tanda
 format, bahasa, dan Tangan Dosen
 konsistensi
 antar-bab; siap
 untuk review
 akademis dan
 validasi teknis

 v2.0 Juni 2026 Kelompok 7 BAB 1, 2, Penerapan 7 Approved (Revisi
 3, 4, 5, 6, perubahan v2.0)
 7 komprehensif:
 Change Control
 Checklist (BAB
 1.5), penghapusan
 batasan shared
 hosting + catatan
 toleransi degradasi
 (BAB 2.4), fitur
 kirim password
 awal + logika
 kredit pembatalan +
 validasi wajah
 client-side (BAB
 3), penambahan
 Sequence Diagram +
 State Transition
 Student Profile
 (BAB 4),
 klarifikasi status
 Attendance + Soft
 Delete menggantikan
 CASCADE DELETE (BAB
 5), NFR Security
 Compliance Matrix
 (BAB 6), finalisasi
 TDB-05 radius
 150m + penghapusan
 TDB-01 (BAB 7)
 ----------------------------------------------------------------------------------------

**BAB 7 --- LAMPIRAN**

**7.1 Ringkasan Hasil Wawancara Stakeholder**

Wawancara dilaksanakan pada Februari 2026 dengan empat kelompok pemangku
kepentingan Equalizer-Stake: Owner, Admin Operasional, Instructor, dan Guardian.
Berikut poin-poin kunci yang menjadi dasar requirement dalam dokumen
ini.

***A. Wawancara dengan Owner Equalizer-Stake***

*Tabel 7.1 Ringkasan Wawancara Owner Equalizer-Stake*

 -----------------------------------------------------------------------
 **Topik** **Temuan Kunci** **FR Terkait**
 ----------------- -------------------------- --------------------------
 Masalah utama Tidak dapat memantau FR-13
 operasional harian dari
 jarak jauh; tidak ada
 laporan kehadiran dan
 pendapatan yang mudah
 dibaca.

 Kebutuhan akses Hanya butuh melihat; tidak FR-13
 ingin direpotkan input
 data. Kepercayaan penuh
 pada Admin untuk
 operasional harian.

 Prioritas bisnis Mengurangi no-show; FR-04, FR-06, FR-07
 memastikan tagihan tidak
 terlambat; transparansi
 perubahan jadwal.

 Kekhawatiran Keamanan data student dan FR-09, FR-11
 guardian; risiko
 penyalahgunaan data oleh
 pihak luar.
 -----------------------------------------------------------------------

***B. Wawancara dengan Admin Operasional***

*Tabel 7.2 Ringkasan Wawancara Admin Operasional*

 -----------------------------------------------------------------------
 **Topik** **Temuan Kunci** **FR Terkait**
 ----------------- -------------------------- --------------------------
 Masalah terbesar Konflik jadwal yang tidak FR-02, FR-06
 terdeteksi; mengingatkan
 jadwal dan tagihan secara
 manual via WhatsApp
 memakan 2--3 jam/hari.

 Proses pembayaran Sulitnya mencocokkan FR-03 (kode unik), FR-15
 transfer masuk dengan
 tagihan karena nominal
 serupa; sering terjadi
 kesalahan identifikasi.

 Absensi Instructor kadang lupa mengisi FR-05
 absensi atau mengisi
 terlambat; tidak ada
 verifikasi bahwa student
 benar-benar hadir.

 Permintaan fitur Sistem yang memperingatkan FR-04, FR-06, FR-11
 jika ada masalah (akun
 beku, notifikasi gagal)
 tanpa perlu dicek manual
 setiap saat.
 -----------------------------------------------------------------------

***C. Wawancara dengan Instructor***

*Tabel 7.3 Ringkasan Wawancara Instructor*

 -----------------------------------------------------------------------
 **Topik** **Temuan Kunci** **FR Terkait**
 ----------------- -------------------------- --------------------------
 Pengalaman saat Jadwal kadang berubah FR-02, FR-06, FR-12
 ini mendadak tanpa
 pemberitahuan; tidak ada
 rekam jejak perubahan yang
 bisa dirujuk.

 Absensi digital Setuju dengan foto absensi FR-05 (radius 150 meter
 asalkan proses tidak sebagai toleransi GPS
 memakan waktu lebih dari indoor --- TDB-05
 1--2 menit; khawatir RESOLVED)
 dengan sinyal GPS di dalam
 ruangan.

 Permintaan Notifikasi jadwal yang FR-06, FR-13
 jelas dan tepat waktu;
 kemudahan melihat jadwal
 mengajar seminggu ke
 depan.
 -----------------------------------------------------------------------

***D. Wawancara dengan Guardian***

*Tabel 7.4 Ringkasan Wawancara Guardian*

 -----------------------------------------------------------------------
 **Topik** **Temuan Kunci** **FR Terkait**
 ----------------- -------------------------- --------------------------
 Masalah utama Sering lupa jadwal anak FR-06 (reminder 1 jam
 dan tagihan; informasi sebelum)
 perubahan jadwal datang
 terlambat atau tidak
 sampai.

 Multi-anak Beberapa wali memiliki 2 FR-10 (1 akun untuk N
 anak yang les di EQS student)
 music; sangat merepotkan
 jika harus membayar
 terpisah dan memantau
 jadwal berbeda.

 Pembayaran Lebih suka transfer bank; FR-15, FR-06
 perlu konfirmasi cepat
 setelah membayar agar
 tidak khawatir pembayaran
 tidak tercatat.

 Privasi Ingin tahu bagaimana foto FR-09, FR-05
 anak digunakan dan
 disimpan.
 -----------------------------------------------------------------------

**7.2 Kontrak Integrasi Layanan Pihak Ketiga**

***A. WhatsApp Gateway (Tidak Resmi / Unofficial)***

Peringatan: Layanan WhatsApp gateway tidak resmi tidak memiliki jaminan
layanan (SLA) tertulis dan berpotensi diblokir oleh WhatsApp
sewaktu-waktu. Sistem wajib selalu memiliki mekanisme cadangan email
yang berfungsi.

*Tabel 7.5 Spesifikasi Integrasi WhatsApp Gateway*

 ----------------- -----------------------------------------------------
 **Parameter** **Detail**

 Tipe Integrasi REST API (HTTP/HTTPS)

 Autentikasi API Key dikirim di header permintaan: X-API-Key:
 \[kunci_rahasia\]; kunci disimpan di variabel
 lingkungan server, tidak di kode sumber.

 Endpoint POST /send-message
 Pengiriman

 Struktur Payload {\"to\": \"+628XXXXXXXXX\", \"message\": \"teks
 (JSON) pesan\", \"type\": \"text\"}

 Kode Error yang 200 OK --- berhasil dikirim ke gateway \| 429 Too
 Ditangani Many Requests --- batas kirim tercapai; sistem
 menunggu 60 detik sebelum mencoba ulang \| 5xx Server
 Error --- percobaan ulang dengan jeda bertambah (10,
 20, 40 detik); setelah 3 gagal → fallback ke email \|
 401 Unauthorized --- kunci API tidak valid;
 notifikasi kritis ke Admin

 Batas Pengiriman Disesuaikan dengan paket gateway yang digunakan;
 Admin memantau kuota melalui dashboard gateway.

 Eskalasi Jika gateway tidak tersedia \>1 jam: Admin menerima
 Kegagalan notifikasi sistem; semua notifikasi dialihkan ke
 email hingga gateway pulih.
 ----------------- -----------------------------------------------------

***B. Email SMTP (Saluran Kritis Utama)***

*Tabel 7.6 Spesifikasi Integrasi Email SMTP*

 ----------------- -----------------------------------------------------
 **Parameter** **Detail**

 Penyedia SMTP penyedia hosting (atau layanan SMTP
 transaksional seperti Brevo/Mailgun jika anggaran
 memungkinkan)

 Port & Enkripsi Port 587 dengan STARTTLS, atau Port 465 dengan
 SSL/TLS

 Autentikasi Username dan password SMTP; disimpan di variabel
 lingkungan server

 Pengirim noreply@equalizer-stake.id (dikonfigurasi Admin)

 Penggunaan Komunikasi kritis (konfirmasi pembayaran,
 freeze/unfreeze akun, aktivasi paket, hak penghapusan
 data) dan cadangan jika WhatsApp gagal

 Penanganan Log bounced email dicatat; Admin menerima notifikasi
 Bounced Email jika email Guardian bounced lebih dari 2 kali
 ----------------- -----------------------------------------------------

**7.3 Daftar Kode Error dan Pesan Pengguna Terstandarisasi**

Seluruh pesan error yang ditampilkan kepada pengguna menggunakan bahasa
Indonesia yang jelas, tidak menyebut detail teknis internal, dan selalu
menyertakan panduan tindakan berikutnya.

*Tabel 7.7 Daftar Kode Error dan Pesan Pengguna Terstandarisasi*

 -----------------------------------------------------------------------------
 **Kode **Kondisi **Pesan kepada **Tindakan Sistem**
 Error** Pemicu** Pengguna (Bahasa
 Indonesia)**
 ------------ ---------------- ---------------------- ------------------------
 AUTH_001 Kredensial login Username atau kata Tambah hitungan
 salah sandi yang Anda percobaan gagal; catat
 masukkan tidak sesuai. di audit log keamanan.
 Silakan coba lagi.

 AUTH_002 Akun terkunci Akun Anda dikunci Kirim notifikasi email
 sementara (5 sementara selama 15 ke Admin; catat insiden
 percobaan gagal) menit karena terlalu keamanan.
 banyak percobaan
 login. Silakan coba
 lagi setelah 15 menit.

 AUTH_003 Akun nonaktif Akun Anda saat ini Hentikan proses login;
 tidak aktif. Hubungi tidak memberikan detail
 Admin Equalizer-Stake di alasan penonaktifan.
 \[nomor kontak\].

 SCHED_001 Konflik slot Slot yang Anda pilih Tampilkan 3 slot
 jadwal sudah terisi. Berikut alternatif terdekat;
 beberapa pilihan waktu tidak simpan jadwal.
 lain yang tersedia:
 \[daftar slot\].

 SCHED_002 Di luar jam Jadwal yang dipilih Tolak penyimpanan; tidak
 operasional berada di luar jam ubah data.
 operasional Equalizer-Stake
 (Senin--Jumat
 14.30--17.00, Sabtu
 08.00--12.00).

 ATTEND_001 GPS di luar Lokasi Anda terdeteksi Simpan absensi dengan
 radius 150 meter di luar area studio. status
 Absensi ini akan PendingVerification;
 menunggu verifikasi kirim notifikasi ke
 dari Admin. Admin.

 ATTEND_002 Wajah tidak Wajah student belum Tampilkan panduan
 terdeteksi ≥30% terlihat jelas di pengambilan foto; tidak
 foto. Pastikan wajah kirim foto ke server.
 mengisi minimal
 sepertiga area foto,
 lalu coba lagi.

 BILL_001 Kode unik tidak Kode tagihan tidak Tampilkan daftar tagihan
 ditemukan ditemukan. Pastikan 3 aktif Guardian yang
 digit terakhir pada relevan untuk membantu
 nominal transfer Admin.
 sesuai dengan tagihan
 aktif.

 BILL_002 Akun dalam Akun Anda saat ini Blokir akses ke fitur
 status beku dibekukan karena ada penjadwalan dan absensi;
 tagihan yang belum akses laporan tetap
 diselesaikan. Silakan diizinkan.
 hubungi Admin EQS
 music untuk informasi
 lebih lanjut.

 WA_429 Batas pengiriman (Tidak ditampilkan ke Tunggu 60 detik,
 WhatsApp pengguna akhir) --- percobaan ulang; jika
 tercapai Dicatat di log sistem; gagal lagi → fallback
 fallback ke email email.
 diaktifkan.

 WA_5XX Server WhatsApp (Tidak ditampilkan ke Exponential backoff
 gateway error pengguna akhir) --- 10/20/40 detik; setelah
 Dicatat di log; 3 percobaan → fallback
 eskalasi ke Admin jika email; catat di
 \>5 kali dalam 1 jam. Notification log.

 GPS_OUT GPS tidak dapat Lokasi GPS tidak dapat Simpan absensi dengan
 dideteksi dideteksi di perangkat status
 Anda. Pastikan GPS PendingVerification.
 aktif dan coba lagi,
 atau hubungi Admin
 untuk absensi manual.

 DRAFT_001 Draft ditemukan Kami menemukan data Tampilkan pilihan; jika
 setelah sesi yang belum tersimpan diabaikan, hapus draft
 berakhir dari sesi sebelumnya. lokal.
 Apakah Anda ingin
 melanjutkan dari data
 tersebut? \[Lanjutkan
 / Abaikan\]

 SYS_001 Server tidak Sistem sedang tidak Tampilkan data cache
 dapat dijangkau dapat dijangkau. terakhir jika tersedia
 Silakan coba beberapa (PWA offline mode).
 saat lagi atau hubungi
 Admin jika masalah
 berlanjut.
 -----------------------------------------------------------------------------

**7.4 Panduan Pemantauan dan Peringatan Sistem**

***A. Endpoint Pemantauan Kesehatan Sistem (Health Check)***

*Tabel 7.8 Endpoint Pemantauan Kesehatan Sistem*

 --------------------------------------------------------------------------------------------------------------------------
 **Endpoint** **Fungsi** **Respons Normal** **Respons Bermasalah**
 --------------- ----------------- ---------------------------------------- -----------------------------------------------
 GET /health Status 200 OK {\"status\":\"healthy\"} 503
 keseluruhan {\"status\":\"degraded\",\"detail\":\"\...\"}
 sistem

 GET /health/db Koneksi basis 200 OK {\"db\":\"connected\"} 503 {\"db\":\"disconnected\"}
 data

 GET Status layanan 200 OK 200
 /health/notif notifikasi {\"whatsapp\":\"ok\",\"email\":\"ok\"} {\"whatsapp\":\"degraded\",\"email\":\"ok\"}
 --------------------------------------------------------------------------------------------------------------------------

***B. Struktur Log Audit Sistem***

Setiap entri log audit memiliki struktur JSON yang konsisten untuk
memudahkan pencarian, filter, dan analisis:

{ \"id_log\": \"uuid\", \"logged_at\": \"2026-04-07T14:35:22+07:00\",
\"id_actor\": \"uuid-pengguna \| null (sistem)\", \"actor_role\":
\"admin \| system \| teacher \| guardian\", \"ip_address\":
\"192.168.x.x\", \"entity_name\": \"Session \| Invoice \| Attendance \|
User \| \...\", \"entity_id\": \"uuid-entitas-yang-diubah\",
\"action_type\": \"create \| update \| delete \| freeze \|
verify_payment \| \...\", \"log_category\": \"operasional \| keamanan \|
kepatuhan \| sistem\", \"before_state\": { \... }, \"after_state\": {
\... }, \"is_anonymized\": false }

***C. Ambang Batas Peringatan Otomatis (Alert Thresholds)***

*Tabel 7.9 Ambang Batas Peringatan Otomatis (Alert Thresholds)*

 -----------------------------------------------------------------------------------
 **Kondisi Pemicu **Ambang Batas** **Penerima **Saluran**
 Peringatan** Notifikasi**
 ---------------------- --------------------- ---------------- ---------------------
 Kegagalan notifikasi \> 5 kegagalan dalam Admin Email sistem
 berulang 1 jam

 Penyimpanan server \> 80% kapasitas disk Admin & Owner Email sistem
 hampir penuh

 Gagal login berulang \> 10 percobaan gagal Admin Email sistem + catat
 (brute force) dari IP yang sama audit log kategori
 dalam 15 menit keamanan

 Proses penagihan Penagihan tanggal 5 Admin Email sistem
 otomatis gagal tidak selesai dalam
 15 menit

 Waktu respons sistem Waktu respons Admin Email sistem
 lambat rata-rata \> 3 detik
 selama 5 menit
 berturut-turut

 Gateway WhatsApp tidak 3 kegagalan Admin Email sistem;
 tersedia berturut-turut ke aktifkan mode
 endpoint gateway email-only otomatis

 Absensi menunggu \> 2 jam sejak status Admin Notifikasi di
 verifikasi lama PendingVerification dashboard Admin
 dibuat
 -----------------------------------------------------------------------------------

**7.5 Daftar Periksa Peluncuran MVP dan Prosedur Pembatalan**

***A. Daftar Periksa Pra-Peluncuran (Pre-Launch Checklist)***

*Tabel 7.10 Daftar Periksa Pra-Peluncuran MVP*

 -----------------------------------------------------------------------------
 **No.** **Item Pemeriksaan** **Penanggung **Status**
 Jawab**
 --------- ---------------------------------- --------------- ----------------
 1 Semua Functional Requirement QA + Admin EQS ☐
 (FR-01 s.d. FR-15) telah melewati music
 pengujian penerimaan (UAT) dengan
 hasil lulus

 2 Backup data awal (data master: DevOps ☐
 akun, jadwal, tarif) telah dibuat
 dan dapat dipulihkan

 3 Enkripsi TLS 1.2+ aktif pada DevOps ☐
 seluruh endpoint dan diverifikasi
 menggunakan SSL Labs Grade A

 4 WhatsApp gateway aktif, DevOps + Admin ☐
 terverifikasi, dan uji pengiriman
 ke 3 nomor berbeda berhasil

 5 SMTP email aktif; uji pengiriman DevOps ☐
 email kritis ke alamat Owner dan
 Admin berhasil

 6 Health check endpoint (/health, DevOps ☐
 /health/db, /health/notif)
 merespons dengan benar

 7 Semua akun pengguna (Owner, Admin, Admin Equalizer-Stake ☐
 minimal 1 Instructor, minimal 1
 Guardian) telah dibuat dan diuji
 login

 8 Data kalender libur nasional tahun Admin Equalizer-Stake ☐
 berjalan telah diinput oleh Admin

 9 Prosedur hak penghapusan data QA + Kelompok 7 ☐
 (FR-09) telah diuji end-to-end
 dengan data uji

 10 Dokumen SRS v2.0 telah disetujui Kelompok 7 ☐
 dan didistribusikan ke seluruh
 pemangku kepentingan

 11 Rencana pemantauan 7 hari pertama Kelompok 7 + ☐
 setelah peluncuran telah DevOps
 ditetapkan (siaga tim teknis)
 -----------------------------------------------------------------------------

***B. Prosedur Pembatalan / Rollback***

Jika sistem mengalami kegagalan kritis setelah peluncuran, prosedur
berikut diaktifkan oleh Admin atau tim teknis:

1. Deteksi Kegagalan: Admin atau sistem monitoring mendeteksi masalah
 kritis (downtime \>15 menit, data tidak konsisten, atau pelanggaran
 keamanan).

2. Eskalasi Segera: Admin menghubungi tim teknis (Kelompok 7) melalui
 kontak darurat yang tercantum di dokumen deployment internal.

3. Pengumuman Pemeliharaan: Sistem menampilkan halaman pemeliharaan;
 Admin mengirimkan pemberitahuan ke Instructor dan Guardian melalui
 WhatsApp grup bahwa sistem sedang dalam perbaikan.

4. Rollback Data: Tim teknis memulihkan sistem dari backup terakhir
 (RPO maksimal 1 jam data hilang). Proses pemulihan ditargetkan
 selesai dalam ≤2 jam (RTO).

5. Operasional Manual Sementara: Selama sistem tidak tersedia, Admin
 menggunakan prosedur manual (spreadsheet sementara) untuk mencatat
 kehadiran dan pembayaran yang masuk.

6. Verifikasi Pasca-Rollback: Tim teknis dan Admin memverifikasi
 integritas data dan fungsionalitas sistem sebelum mengaktifkan
 kembali akses pengguna.

7. Laporan Insiden: Tim teknis menyusun laporan insiden dalam 24 jam
 yang mencakup: akar masalah, dampak, tindakan yang diambil, dan
 langkah pencegahan.

***C. Pemantauan 7 Hari Pertama Pasca-Peluncuran***

*Tabel 7.11 Pemantauan 7 Hari Pertama Pasca-Peluncuran*

 ----------------------------------------------------------------------------
 **Hari** **Fokus Pemantauan** **Tindakan jika Ada **Catatan**
 Masalah**
 ---------- ------------------------ ------------------------ ---------------
 Hari 1--2 Login seluruh peran; Perbaikan hotfix oleh
 pengiriman notifikasi tim teknis dalam ≤2 jam
 pertama; validasi
 absensi pertama

 Hari 3--4 Performa sistem di bawah Optimasi konfigurasi
 beban nyata; kecepatan server jika diperlukan
 respons; log error

 Hari 5 Proses penerbitan Trigger ulang proses
 tagihan otomatis jika gagal; notifikasi
 (tanggal 5) berjalan manual ke Admin
 dengan benar

 Hari 6--7 Review log audit; cek Catat sebagai backlog
 alert threshold; umpan perbaikan untuk versi
 balik dari Admin dan v2.1
 Instructor
 ----------------------------------------------------------------------------

**7.6 Daftar To Be Determined (TDB List)**

Tabel berikut mengidentifikasi parameter teknis atau kebijakan
operasional yang belum dapat difinalisasi pada versi SRS v1.0, beserta
penanggung jawab dan target penyelesaian.

Catatan: TDB-01 (Batas harian pengiriman pesan WhatsApp Gateway tidak
resmi) telah dihapus dari daftar ini karena batas harian WhatsApp
gateway bukanlah constraint desain sistem, melainkan risiko operasional
yang dikelola melalui mekanisme fallback email. TDB-05 telah
diselesaikan melalui uji lapangan (status: RESOLVED).

*Tabel 7.12 Daftar To Be Determined (TDB List)*

 ---------------------------------------------------------------------------------------
 **ID **Item yang **Dampak **Penanggung **Target **Status**
 TBD** Belum terhadap Jawab** Penyelesaian**
 Ditetapkan** Sistem**
 -------- ---------------- ---------------- -------------- ---------------- ------------
 TDB-02 Spesifikasi Skalabilitas Owner + Sebelum UAT Open
 upgrade server dan biaya Kelompok 7 (Minggu ke-8)
 infrastruktur operasional fase
 cloud jika beban pasca-MVP
 \>50 pengguna
 aktif simultan

 TDB-03 Mekanisme resmi Akurasi log Admin + DevOps Minggu ke-3 Open
 pelaporan bounce notifikasi pengembangan
 email ke kritis dan
 provider SMTP kepatuhan data

 TDB-04 Format ekspor Kompatibilitas Owner Minggu ke-5 Proposed
 laporan (CSV vs dengan perangkat pengembangan
 PDF vs Excel) lunak akuntansi
 untuk dashboard internal
 Owner

 TDB-05 Toleransi GPS Akurasi validasi Tim QA + Selesai (Minggu **RESOLVED
 indoor final absensi di Instructor + Admin ke-2 --- Final:
 (penyempurnaan lantai 2/3 pengembangan) 150m**
 dari 100m) gedung studio --- Setelah uji
 lapangan,
 ditetapkan
 radius 150 meter
 untuk
 mengakomodasi
 GPS drift di
 dalam gedung.
 ---------------------------------------------------------------------------------------

**Pernyataan Kesiapan Dokumen**

Dokumen Software Requirements Specification (SRS) Equalizer-Stake versi
2.0 ini telah memuat seluruh spesifikasi kebutuhan yang diperlukan untuk
memulai fase desain teknis, pengembangan, pengujian penerimaan, dan
audit kepatuhan data. Semua Functional Requirement telah dilengkapi
dengan kriteria penerimaan yang terukur (Given-When-Then), dapat diuji
secara independen, dan terpetakan ke standar kepatuhan yang berlaku (UU
PDP No. 27/2022, ISO/IEC 27001, OWASP Top 10).

Dokumen ini siap untuk didistribusikan kepada: Tim Pengembang (sebagai
dasar desain teknis dan implementasi), Tim QA (sebagai dasar penyusunan
test case dan UAT), DevOps (sebagai acuan konfigurasi infrastruktur dan
deployment), dan Auditor Kepatuhan (sebagai bukti perencanaan
perlindungan data pribadi sesuai regulasi).

Setiap perubahan terhadap dokumen ini setelah status Approved (Baseline)
wajib melalui prosedur Change Control yang tercantum di BAB 1.5 dan
dicatat dalam Matriks Log Perubahan di BAB 6.2.
