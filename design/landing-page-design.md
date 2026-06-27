# Landing Page Design — BMD Music School

Dokumen referensi pembuatan frontend landing page BMD Music School.

---

## Index

| # | File | Deskripsi |
|---|------|-----------|
| 1 | [system/design-tokens.md](system/design-tokens.md) | Palette, typography, spacing, breakpoints, shadows, z-index |
| 2 | [system/components.md](system/components.md) | 11 komponen React Bits + konfigurasi lengkap |
| 3 | [system/sections.md](system/sections.md) | 7 sections (Hero → Footer) + layout ASCII |
| 4 | [system/animations.md](system/animations.md) | Scroll animations, micro-interactions, reduced motion |
| 5 | [system/mobile.md](system/mobile.md) | Mobile rules (320px-768px), touch rules, performance |
| 6 | [system/accessibility.md](system/accessibility.md) | WCAG 2.1 Level AA checklist |
| 7 | [system/file-structure.md](system/file-structure.md) | Frontend file structure + dependencies |

---

## Quick Reference

### Palette

| Token | Hex | Kegunaan |
|-------|-----|----------|
| Primary | `#000000` | Background utama |
| Secondary | `#08CB00` | Aksen hijau, CTA |
| Tertiary | `#253900` | Section gelap |
| Neutral | `#EEEEEE` | Section terang |

### Typography

| Elemen | Font | Weight |
|--------|------|--------|
| Headings | Poppins | Bold/SemiBold |
| Body | Inter | Regular |

### Sections

1. **Hero** — FloatingLines + BlurText + RotatingText + StarBorder
2. **Layanan** — MagicBento (6 cards) + BorderGlow
3. **Cara Kerja** — 3 GlassSurface cards
4. **Studio** — CircularGallery
5. **Testimonial** — 3 GlassSurface cards (terang)
6. **CTA** — FloatingLines + ShinyText + StarBorder
7. **Footer** — Static

### Components

FloatingLines, BlurText, RotatingText, ShinyText, BorderGlow, GlassSurface, MagicBento, CircularGallery, StarBorder, SplashCursor, GooeyNav
