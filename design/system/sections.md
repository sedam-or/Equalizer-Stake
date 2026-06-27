# Sections — Landing Page BMD Music School

Detail 7 section landing page beserta elemen dan komponen per section.

---

## Section 1: Hero (Full-Screen)

**Background**: FloatingLines (gradient #08CB00 → #000000)
**Overlay**: SplashCursor (transparan)
**Padding**: 64px all sides

### Elemen

| # | Elemen | Komponen | Posisi | Style |
|---|--------|----------|--------|-------|
| 1 | Brand name | ShinyText | Top center | "BMD MUSIC SCHOOL", #EEEEEE, shine #08CB00 |
| 2 | Headline | BlurText | Center | "Sistem Penjadwalan Kursus Musik & Reservasi Studio", Poppins Bold 48px, putih |
| 3 | Sub-headline | RotatingText | Center, bawah headline | "Gitar" → "Piano" → "Vokal" → "Drum" → "Bass" → "Keyboard", #08CB00 |
| 4 | CTA button | StarBorder | Center, bawah sub-headline | "Jadwalkan Sekarang", #08CB00 |
| 5 | Scroll indicator | Animated chevron | Bottom center | ↓ putih, bounce animation |

### Layout

```
┌─────────────────────────────────────────┐
│ [GooeyNav: Logo | Beranda | Layanan |   │
│  Studio | Hubungi | [Masuk]]            │
├─────────────────────────────────────────┤
│                                         │
│         BMD MUSIC SCHOOL                │ ← ShinyText
│                                         │
│    Sistem Penjadwalan Kursus Musik      │ ← BlurText
│         & Reservasi Studio              │
│                                         │
│            [Gitar]                      │ ← RotatingText
│                                         │
│    ┌─────────────────────────┐          │
│    │ Jadwalkan Sekarang      │          │ ← StarBorder
│    └─────────────────────────┘          │
│                                         │
│                 ↓                       │ ← Scroll indicator
└─────────────────────────────────────────┘
```

---

## Section 2: Layanan Kami (Bento Grid)

**Background**: #000000
**Padding**: 96px 24px

### Elemen

| # | Elemen | Komponen | Posisi | Style |
|---|--------|----------|--------|-------|
| 1 | Heading | BlurText | Top center | "Layanan Kami", Poppins SemiBold 32px, putih |
| 2 | Grid | MagicBento | Center | 6 cards, 3 kolom (desktop), 1 kolom (mobile) |

### Cards

| # | Judul | Deskripsi | Ikon |
|---|-------|-----------|------|
| 1 | Jadwal Kursus | Penjadwalan otomatis tanpa konflik | 📅 |
| 2 | Tagihan Otomatis | Invoice & pembayaran manual | 💰 |
| 3 | Absensi Digital | GPS + face detection | 📷 |
| 4 | Notifikasi | WhatsApp & email otomatis | 🔔 |
| 5 | Studio Rekaman | Reservasi studio & ruang | 🎙️ |
| 6 | Laporan & Audit | Dashboard owner & audit log | 📊 |

### Layout

```
┌─────────────────────────────────────────┐
│            Layanan Kami                  │ ← Heading
├─────────────┬─────────────┬─────────────┤
│  Jadwal     │  Tagihan    │  Absensi    │
│  Kursus     │  Otomatis   │  Digital    │
│             │             │             │
│  [icon]     │  [icon]     │  [icon]     │
├─────────────┼─────────────┼─────────────┤
│  Notif      │  Studio     │  Laporan    │
│  WhatsApp   │  Rekaman    │  & Audit    │
│             │             │             │
│  [icon]     │  [icon]     │  [icon]     │
└─────────────┴─────────────┴─────────────┘
```

---

## Section 3: Cara Kerja (3 Langkah)

**Background**: #253900 (hijau gelap)
**Padding**: 96px 24px

### Elemen

| # | Elemen | Komponen | Posisi | Style |
|---|--------|----------|--------|-------|
| 1 | Heading | BlurText | Top center | "Mulai dalam 3 Langkah", Poppins SemiBold 32px, putih |
| 2 | 3 cards | GlassSurface | Horizontal (desktop), stack (mobile) | Transparan putih |

### Cards

| # | Angka | Judul | Deskripsi |
|---|-------|-------|-----------|
| 1 | 01 | Daftar & Pilih Instrumen | Buat akun dan pilih kursus musik yang diinginkan |
| 2 | 02 | Pilih Jadwal Otomatis | Sistem memilihkan jadwal terbaik tanpa konflik |
| 3 | 03 | Pantau Progress Anak | Monitoring kehadiran dan pembayaran real-time |

### Layout

```
┌─────────────────────────────────────────┐
│         Mulai dalam 3 Langkah           │ ← Heading
├─────────────┬─────────────┬─────────────┤
│     01      │     02      │     03      │ ← ShinyText (hijau)
│             │             │             │
│  Daftar &   │  Pilih      │  Pantau     │
│  Pilih      │  Jadwal     │  Progress   │
│  Instrumen  │  Otomatis   │  Anak       │
│             │             │             │
│  [deskripsi]│  [deskripsi]│  [deskripsi]│
└─────────────┴─────────────┴─────────────┘
```

---

## Section 4: Studio Showcase (Gallery)

**Background**: #000000
**Padding**: 96px 24px

### Elemen

| # | Elemen | Komponen | Posisi | Style |
|---|--------|----------|--------|-------|
| 1 | Heading | BlurText | Top center | "Studio Kami", Poppins SemiBold 32px, putih |
| 2 | Gallery | CircularGallery | Center | bend: 3, textColor: #EEEEEE |

### Items

| # | Text | Image |
|---|------|-------|
| 1 | Studio Gitar | (kosong) |
| 2 | Studio Piano | (kosong) |
| 3 | Studio Vokal | (kosong) |
| 4 | Studio Drum | (kosong) |
| 5 | Ruang Rekaman | (kosong) |
| 6 | Ruang Latihan | (kosong) |

### Layout

```
┌─────────────────────────────────────────┐
│            Studio Kami                   │ ← Heading
│                                         │
│         ╭──────────────────╮            │
│        ╱   Studio Gitar     ╲           │
│       │    Studio Piano      │          │ ← CircularGallery
│        ╲   Studio Vokal     ╱           │
│         ╰──────────────────╯            │
│                                         │
└─────────────────────────────────────────┘
```

---

## Section 5: Testimonial

**Background**: #EEEEEE (neutral)
**Padding**: 96px 24px

### Elemen

| # | Elemen | Komponen | Posisi | Style |
|---|--------|----------|--------|-------|
| 1 | Heading | BlurText | Top center | "Apa Kata Mereka", Poppins SemiBold 32px, #000000 |
| 2 | 3 cards | GlassSurface (terang) | Horizontal (desktop), scroll (mobile) | Glass terang |

### Cards

| # | Nama | Role | Kutipan |
|---|------|------|---------|
| 1 | Ananda Putra | Orang Tua | "Sangat membantu mengatur jadwal les anak saya" |
| 2 | Rizky Pratama | Guru Musik | "Sistem absensi memudahkan monitoring kelas" |
| 3 | Putri Wulandari | Wali Murid | "Tagihan otomatis dan notifikasi sangat praktis" |

### Layout

```
┌─────────────────────────────────────────┐
│          Apa Kata Mereka                │ ← Heading (hitam)
├─────────────┬─────────────┬─────────────┤
│  [Avatar]   │  [Avatar]   │  [Avatar]   │
│  Ananda     │  Rizky      │  Putri      │
│  Putra      │  Pratama    │  Wulandari  │
│  Orang Tua  │  Guru Musik │  Wali Murid │
│             │             │             │
│  "Sangat    │  "Sistem    │  "Tagihan   │
│   membantu  │   absensi   │   otomatis  │
│   mengatur  │   memudah-  │   dan notif │
│   jadwal"   │   kan..."   │   praktis"  │
└─────────────┴─────────────┴─────────────┘
```

---

## Section 6: CTA

**Background**: FloatingLines (gradient #253900 → #000000)
**Padding**: 96px 24px

### Elemen

| # | Elemen | Komponen | Posisi | Style |
|---|--------|----------|--------|-------|
| 1 | Heading | ShinyText | Center | "Siap Mulai?", Poppins Bold 48px, #08CB00 shine |
| 2 | Deskripsi | Inter regular | Center | "Daftar sekarang dan nikmati kemudahan penjadwalan kursus musik", putih, 16px |
| 3 | CTA button | StarBorder | Center | "Hubungi Kami", #08CB00 |

### Layout

```
┌─────────────────────────────────────────┐
│                                         │
│           Siap Mulai?                   │ ← ShinyText
│                                         │
│   Daftar sekarang dan nikmati           │ ← Deskripsi
│   kemudahan penjadwalan                 │
│   kursus musik                          │
│                                         │
│    ┌─────────────────────────┐          │
│    │ Hubungi Kami            │          │ ← StarBorder
│    └─────────────────────────┘          │
│                                         │
└─────────────────────────────────────────┘
```

---

## Section 7: Footer

**Background**: #000000
**Padding**: 48px 24px

### Elemen

| # | Elemen | Posisi | Style |
|---|--------|--------|-------|
| 1 | Logo + nama | Left | "BMD MUSIC SCHOOL", Poppins Bold 20px, #08CB00 |
| 2 | Tagline | Left | "Sistem Penjadwalan Kursus Musik", Inter 14px, #EEEEEE |
| 3 | Alamat | Left | "Seturan, Yogyakarta", Inter 14px, #EEEEEE |
| 4 | WhatsApp | Left | "+62 xxx-xxxx-xxxx", Inter 14px, #08CB00 |
| 5 | Email | Left | "info@bmdmusic.com", Inter 14px, #08CB00 |
| 6 | Navigation links | Center | "Masulayanan | Studio | Hubungi", Inter 14px, #EEEEEE |
| 7 | Copyright | Bottom | "© 2026 BMD Music School. All rights reserved.", Inter 12px, #888 |

### Layout

```
┌─────────────────────────────────────────┐
│ BMD MUSIC SCHOOL                        │
│ Sistem Penjadwalan Kursus Musik         │
│                                         │
│ Alamat: Seturan, Yogyakarta             │
│ WhatsApp: +62 xxx-xxxx-xxxx             │
│ Email: info@bmdmusic.com                │
│                                         │
│ [Masuk] [Layanan] [Studio] [Hubungi]   │
│                                         │
│ © 2026 BMD Music School. All rights.    │
└─────────────────────────────────────────┘
```
