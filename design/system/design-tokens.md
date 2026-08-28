# Design Tokens — Equalizer-Stake

Warna, tipografi, spacing, dan breakpoints untuk seluruh website.

---

## Palette

| Token | Hex | RGB | Kegunaan |
|-------|-----|-----|----------|
| Primary | `#000000` | 0, 0, 0 | Background utama, text gelap |
| Secondary | `#08CB00` | 8, 203, 0 | Aksen hijau, tombol CTA, glow |
| Tertiary | `#253900` | 37, 57, 0 | Background section gelap |
| Neutral | `#EEEEEE` | 238, 238, 238 | Background section terang |
| White | `#FFFFFF` | 255, 255, 255 | Text di atas background gelap |

### Gradient

| Nama | Dari | Ke | Kegunaan |
|------|------|----|----------|
| Hero | `#08CB00` | `#000000` | FloatingLines hero |
| CTA | `#253900` | `#000000` | FloatingLines CTA section |

---

## Typography

### Font Family

| Elemen | Font | Source |
|--------|------|--------|
| Headings | Poppins | Google Fonts |
| Body | Inter | Google Fonts |

### Type Scale

| Elemen | Font | Weight | Desktop | Mobile |
|--------|------|--------|---------|--------|
| H1 | Poppins | Bold (700) | 48px | 36px |
| H2 | Poppins | SemiBold (600) | 32px | 24px |
| H3 | Poppins | SemiBold (600) | 24px | 20px |
| Body | Inter | Regular (400) | 16px | 14px |
| Body Small | Inter | Regular (400) | 14px | 12px |
| Label | Inter | Medium (500) | 12px | 12px |
| Button | Inter | SemiBold (600) | 16px | 16px |

### Line Height

| Elemen | Line Height |
|--------|-------------|
| H1 | 1.2 |
| H2 | 1.3 |
| H3 | 1.4 |
| Body | 1.5 |
| Body Small | 1.4 |

---

## Spacing Scale

| Token | Nilai | Kegunaan |
|-------|-------|----------|
| xs | 4px | Gap kecil, ikon inline |
| sm | 8px | Padding ikon, gap item |
| md | 16px | Padding card, gap card |
| lg | 24px | Section padding horizontal |
| xl | 32px | Section gap antar elemen |
| 2xl | 48px | Section margin vertical |
| 3xl | 64px | Hero padding |

---

## Breakpoints

| Name | Min Width | Max Width | Layout |
|------|-----------|-----------|--------|
| Mobile | 320px | 767px | Single column, stack |
| Tablet | 768px | 1023px | 2 columns |
| Desktop | 1024px | — | Full layout, sidebar optional |

---

## Border Radius

| Token | Nilai | Kegunaan |
|-------|-------|----------|
| sm | 8px | Small buttons, badges |
| md | 12px | Input fields |
| lg | 20px | Cards (GlassSurface, BorderGlow) |
| xl | 28px | Large cards, modals |
| full | 9999px | Pills, avatars |

---

## Shadows

| Token | Value | Kegunaan |
|-------|-------|----------|
| sm | `0 1px 2px rgba(0,0,0,0.05)` | Subtle elevation |
| md | `0 4px 6px rgba(0,0,0,0.1)` | Cards |
| lg | `0 10px 15px rgba(0,0,0,0.1)` | Elevated cards |
| glow | `0 0 20px rgba(8,203,0,0.3)` | Green glow effect |

---

## Z-Index Scale

| Token | Nilai | Kegunaan |
|-------|-------|----------|
| base | 0 | Default |
| dropdown | 10 | Dropdowns, menus |
| sticky | 20 | Sticky nav |
| overlay | 40 | Modals, overlays |
| cursor | 50 | SplashCursor |
| max | 100 | Maximum z-index |
