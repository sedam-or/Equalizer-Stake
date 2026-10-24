<p align="center">
  <b>Ritma</b> · <i>irama sekolah Anda</i><br/>
  <b>Sistem operasi open-source untuk lembaga kursus & studio.</b><br/>
  Jadwal · pendaftaran · SPP bulanan · absensi · pesan ke wali murid. Siap dalam 3 langkah.
</p>

> **🚧 Status: v2 sedang dibangun ulang dari spesifikasi.** Versi capstone awal (v0) diarsipkan di `legacy/`.
> Pantau progres di [ROADMAP.md](ROADMAP.md) dan [tasks/INDEX.md](tasks/INDEX.md).

## Kenapa

- **Siap dipakai dalam 3 langkah.** Pilih template, nyalakan fitur, cek aturan. Kurang dari 10 menit.
- **Dijamin tidak ada jadwal bentrok.** Dijaga langsung oleh database, dan dibuktikan dengan uji 50 permintaan bersamaan → hanya 1 yang berhasil.
- **Aturan sekolah Anda, bukan aturan kami.** SPP, tanggal jatuh tempo, denda, jam operasional, kredit pembatalan dan radius absensi semuanya bisa diatur, lengkap dengan ringkasan bahasa sehari-hari.
- **Dibuat untuk cara kerja sekolah kecil.** Pengingat via WhatsApp, satu tagihan per wali untuk semua anak, kode unik transfer, Bahasa Indonesia.
- **Data Anda, Anda yang pilih tempatnya.** Di PC resepsionis, server sendiri, atau cloud kami. Ekspor semua data kapan saja.

## Pasang sendiri (mulai v1.0)

- **Docker satu baris** di server Linux
- **1-klik**: Railway · Render · DigitalOcean · Coolify · CasaOS · YunoHost
- **Aplikasi desktop** untuk PC sekolah, dengan akses aman dari HP

## Kontribusi

Tambahkan template untuk jenis sekolah Anda **cukup dengan YAML, tanpa Rust**, bantu terjemahan, atau ambil issue `good first issue`. Lihat [CONTRIBUTING.md](CONTRIBUTING.md).

## Lisensi

[AGPL-3.0](LICENSE) untuk v2. Arsip v0 di `legacy/` berlisensi MIT.
