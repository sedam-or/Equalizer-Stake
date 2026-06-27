# Mobile Rules — Landing Page BMD Music School

Perubahan layout dan komponen untuk perangkat mobile (320px - 768px).

---

## Breakpoints

| Name | Min Width | Max Width | Layout |
|------|-----------|-----------|--------|
| Mobile | 320px | 767px | Single column, stack |
| Tablet | 768px | 1023px | 2 columns |
| Desktop | 1024px | — | Full layout |

---

## Perubahan Per Elemen

### Navigation

| Desktop | Mobile |
|---------|--------|
| GooeyNav horizontal (4 items) | Hamburger menu (slide-in dari kiri) |
| Items: Beranda, Layanan, Studio, Hubungi | Items: sama + tombol Masuk di bawah |
| Visible | Hidden, buka via hamburger |

### Hero

| Desktop | Mobile |
|---------|--------|
| H1: 48px | H1: 36px |
| Sub: 32px | Sub: 24px |
| CTA: inline button | CTA: full-width button |
| FloatingLines: lineCount [8,12,16] | FloatingLines: lineCount [4,6,8] |
| SplashCursor: aktif | SplashCursor: disabled |
| Padding: 64px | Padding: 24px |

### Layanan (MagicBento)

| Desktop | Mobile |
|---------|--------|
| 3 kolom | 1 kolom (stack) |
| 2 baris | 6 baris |
| Spotlight: aktif | Spotlight: disabled |
| Tilt: disabled | Tilt: disabled |
| Particles: aktif | Particles: disabled |

### Cara Kerja

| Desktop | Mobile |
|---------|--------|
| 3 kolom horizontal | 1 kolom (stack) |
| Cards: inline | Cards: full-width |
| Angka: 48px | Angka: 36px |

### Studio (CircularGallery)

| Desktop | Mobile |
|---------|--------|
| bend: 3 | bend: 2 (lebih pipih) |
| height: 600px | height: 400px |
| font: 24px | font: 18px |

### Testimonial

| Desktop | Mobile |
|---------|--------|
| 3 cards horizontal | 1 card visible, horizontal scroll |
| Cards: 33% width | Cards: 80% width |
| Gap: 16px | Gap: 8px |

### CTA

| Desktop | Mobile |
|---------|--------|
| H1: 48px | H1: 36px |
| CTA: inline button | CTA: full-width button |
| FloatingLines: lineCount [6,10,14] | FloatingLines: lineCount [3,5,7] |

### Footer

| Desktop | Mobile |
|---------|--------|
| Multi-column layout | Single column stack |
| Navigation links: horizontal | Navigation links: vertical |

---

## Touch Rules

| Rule | Nilai |
|------|-------|
| Min touch target | 44x44px |
| Min spacing between targets | 8px |
| Tap feedback | Opacity 0.8 + scale 0.98 |
| Tap delay | touch-action: manipulation |

---

## Performance

| Optimasi | Desktop | Mobile |
|----------|---------|--------|
| FloatingLines | Full | lineCount dikurangi 50% |
| SplashCursor | Aktif | Disabled |
| MagicBento particles | Aktif | Disabled |
| MagicBento spotlight | Aktif | Disabled |
| CircularGallery bend | 3 | 2 |
| Image lazy load | Yes | Yes |
| Font loading | swap | swap |
