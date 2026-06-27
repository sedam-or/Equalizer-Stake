# Landing Page Design — BMD Music School

Dokumen referensi pembuatan frontend landing page BMD Music School.
Untuk detail komponen React Bits, lihat `FLOATING_LINE_BG.md` dan file komponen lainnya di `design/`.

---

## Design System

### Palette

| Token | Hex | Kegunaan |
|-------|-----|----------|
| Primary | `#000000` | Background utama, text gelap |
| Secondary | `#08CB00` | Aksen hijau, tombol CTA, glow |
| Tertiary | `#253900` | Background section gelap |
| Neutral | `#EEEEEE` | Background section terang |
| White | `#FFFFFF` | Text di atas background gelap |

### Typography

| Elemen | Font | Weight | Ukuran |
|--------|------|--------|--------|
| Heading 1 | Poppins | Bold (700) | 48px / 36px mobile |
| Heading 2 | Poppins | SemiBold (600) | 32px / 24px mobile |
| Heading 3 | Poppins | SemiBold (600) | 24px / 20px mobile |
| Body | Inter | Regular (400) | 16px / 14px mobile |
| Body Small | Inter | Regular (400) | 14px / 12px mobile |
| Label | Inter | Medium (500) | 12px |
| Button | Inter | SemiBold (600) | 16px |

### Spacing Scale

| Token | Nilai | Kegunaan |
|-------|-------|----------|
| xs | 4px | Gap kecil |
| sm | 8px | Padding ikon |
| md | 16px | Padding card |
| lg | 24px | Section padding |
| xl | 32px | Section gap |
| 2xl | 48px | Section margin |
| 3xl | 64px | Hero padding |

### Breakpoints

| Breakpoint | Lebar | Layout |
|------------|-------|--------|
| Mobile | 320px - 768px | Single column, stack |
| Tablet | 768px - 1024px | 2 columns |
| Desktop | 1024px+ | Full layout |

---

## Komponen React Bits

### Komponen yang Digunakan

| Komponen | Lokasi | Kegunaan | Dependencies |
|----------|--------|----------|-------------|
| FloatingLines | Hero + CTA | Background garis-garis animasi | three |
| BlurText | Hero, Layanan, Cara Kerja, Studio, Testimonial, CTA | Headline blur-in saat scroll | motion |
| RotatingText | Hero | Sub-headline berganti otomatis | motion |
| ShinyText | Hero, Cara Kerja, CTA | Accent brand name | motion |
| BorderGlow | Layanan cards | Border glow saat hover | — |
| GlassSurface | Layanan, Cara Kerja, Testimonial | Card background transparan | — |
| MagicBento | Layanan | Bento grid dengan spotlight + particles | gsap |
| CircularGallery | Studio | Gallery melingkar + scroll | ogl |
| StarBorder | Hero CTA, CTA section | Tombol dengan border animasi | — |
| SplashCursor | Hero | Efek fluid cursor interaktif | — |
| GooeyNav | Navigation | Navigasi dengan efek gooey | — |

### Konfigurasi Per Komponen

#### FloatingLines (Hero)

```jsx
<FloatingLines
  enabledWaves={['top', 'middle', 'bottom']}
  lineCount={[8, 12, 16]}
  lineDistance={[8, 6, 4]}
  bendRadius={5.0}
  bendStrength={-0.5}
  interactive={true}
  parallax={true}
  gradientStart="#08CB00"
  gradientEnd="#000000"
  animationSpeed={0.8}
  mixBlendMode="screen"
/>
```

#### FloatingLines (CTA Section)

```jsx
<FloatingLines
  enabledWaves={['top', 'middle', 'bottom']}
  lineCount={[6, 10, 14]}
  lineDistance={[10, 8, 5]}
  bendRadius={4.0}
  bendStrength={-0.3}
  interactive={true}
  parallax={true}
  gradientStart="#253900"
  gradientEnd="#000000"
  animationSpeed={0.6}
  mixBlendMode="screen"
/>
```

#### BlurText (Headlines)

```jsx
<BlurText
  text="Sistem Penjadwalan Kursus Musik & Reservasi Studio"
  delay={150}
  animateBy="words"
  direction="top"
  className="text-4xl md:text-5xl font-bold text-white"
/>
```

#### RotatingText (Hero Sub-headline)

```jsx
<RotatingText
  texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]}
  mainClassName="text-2xl md:text-3xl text-[#08CB00] font-semibold"
  staggerFrom="last"
  initial={{ y: "100%" }}
  animate={{ y: 0 }}
  exit={{ y: "-120%" }}
  staggerDuration={0.025}
  splitLevelClassName="overflow-hidden pb-0.5"
  transition={{ type: "spring", damping: 30, stiffness: 400 }}
  rotationInterval={2000}
/>
```

#### ShinyText (Brand Name)

```jsx
<ShinyText
  text="BMD MUSIC SCHOOL"
  speed={2}
  delay={0}
  color="#EEEEEE"
  shineColor="#08CB00"
  spread={120}
  direction="left"
/>
```

#### BorderGlow (Feature Cards)

```jsx
<BorderGlow
  edgeSensitivity={30}
  glowColor="120 100 50"  /* hijau */
  backgroundColor="#120F17"
  borderRadius={20}
  glowRadius={30}
  glowIntensity={0.8}
  coneSpread={25}
  animated={false}
  colors={['#08CB00', '#253900', '#08CB00']}
>
  {/* Card content */}
</BorderGlow>
```

#### GlassSurface (Cards)

```jsx
<GlassSurface
  width="100%"
  height="auto"
  borderRadius={20}
  brightness={30}
  opacity={0.15}
  blur={11}
  displace={0}
  backgroundOpacity={0.1}
  saturation={1.2}
  className="p-6"
>
  {/* Card content */}
</GlassSurface>
```

#### MagicBento (Features Grid)

```jsx
<MagicBento
  textAutoHide={true}
  enableStars={true}
  enableSpotlight={true}
  enableBorderGlow={true}
  enableTilt={false}
  enableMagnetism={true}
  clickEffect={true}
  spotlightRadius={300}
  particleCount={12}
  glowColor="8, 203, 0"  /* #08CB00 RGB */
/>
```

#### CircularGallery (Studio)

```jsx
<CircularGallery
  bend={3}
  textColor="#EEEEEE"
  borderRadius={0.05}
  scrollEase={0.02}
  font="bold 24px Poppins"
  items={[
    { image: "", text: "Studio Gitar" },
    { image: "", text: "Studio Piano" },
    { image: "", text: "Studio Vokal" },
    { image: "", text: "Studio Drum" },
    { image: "", text: "Ruang Rekaman" },
    { image: "", text: "Ruang Latihan" },
  ]}
/>
```

#### StarBorder (CTA Button)

```jsx
<StarBorder
  as="button"
  className="px-8 py-3 text-lg font-semibold"
  color="#08CB00"
  speed="5s"
>
  Jadwalkan Sekarang
</StarBorder>
```

#### SplashCursor (Hero Background)

```jsx
<SplashCursor
  SIM_RESOLUTION={128}
  DYE_RESOLUTION={1024}
  DENSITY_DISSIPATION={3.5}
  VELOCITY_DISSIPATION={2}
  SPLAT_RADIUS={0.15}
  SPLAT_FORCE={4000}
  SHADING={true}
  RAINBOW_MODE={false}
  COLOR="#08CB00"
  BACK_COLOR={{ r: 0, g: 0, b: 0 }}
  TRANSPARENT={true}
/>
```

#### GooeyNav (Navigation)

```jsx
<GooeyNav
  items={[
    { label: "Beranda", href: "#" },
    { label: "Layanan", href: "#layanan" },
    { label: "Studio", href: "#studio" },
    { label: "Hubungi", href: "#hubungi" },
  ]}
  particleCount={15}
  particleDistances={[90, 10]}
  particleR={100}
  initialActiveIndex={0}
  animationTime={600}
  timeVariance={300}
  colors={[1, 2, 3, 1, 2, 3, 1, 4]}
/>
```

---

## Struktur Section

### Section 1: Hero (Full-Screen)

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | FloatingLines | Full-screen, z-index 0 |
| Cursor effect | SplashCursor | Full-screen, z-index 10 |
| Brand name | ShinyText | Top center |
| Headline | BlurText | Center |
| Sub-headline | RotatingText | Center, bawah headline |
| CTA button | StarBorder | Center, bawah sub-headline |
| Scroll indicator | Animated chevron | Bottom center |

### Section 2: Layanan Kami (Bento Grid)

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | #000000 | Full width |
| Heading | BlurText | Top center |
| Grid | MagicBento (6 cards) | Center |
| Card effect | BorderGlow + GlassSurface | Per card |

### Section 3: Cara Kerja (3 Langkah)

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | #253900 | Full width |
| Heading | BlurText | Top center |
| 3 cards | GlassSurface | Horizontal (desktop), stack (mobile) |
| Angka | ShinyText | Top card |

### Section 4: Studio Showcase (Gallery)

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | #000000 | Full width |
| Heading | BlurText | Top center |
| Gallery | CircularGallery | Center |

### Section 5: Testimonial

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | #EEEEEE | Full width |
| Heading | BlurText (warna #000000) | Top center |
| 3 cards | GlassSurface (terang) | Horizontal (desktop), scroll (mobile) |

### Section 6: CTA

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | FloatingLines (variant 2) | Full width |
| Heading | ShinyText | Center |
| Deskripsi | Inter regular, putih | Center |
| CTA button | StarBorder | Center |

### Section 7: Footer

| Elemen | Komponen | Posisi |
|--------|----------|--------|
| Background | #000000 | Full width |
| Logo + nama | — | Left |
| Info kontak | — | Left |
| Navigation links | — | Center |
| Copyright | — | Bottom |

---

## Scroll Animations

| Section | Trigger | Efek |
|---------|---------|------|
| Hero | Page load | FloatingLines start, BlurText animate, SplashCursor activate |
| Layanan | Scroll masuk 20% | MagicBento cards stagger 50ms |
| Cara Kerja | Scroll masuk 20% | 3 cards slide up 200ms delay |
| Studio | Scroll masuk 20% | CircularGallery mulai interaktif |
| Testimonial | Scroll masuk 20% | Cards fade in 300ms |
| CTA | Scroll masuk 20% | FloatingLines baru muncul |

---

## Mobile Rules (320px - 768px)

| Element | Perubahan |
|---------|-----------|
| GooeyNav | Hamburger menu (slide-in) |
| Hero text | H1: 36px, Sub: 24px |
| Hero CTA | Full-width button |
| MagicBento | 1 kolom (stack) |
| Cara Kerja | 1 kolom (stack) |
| CircularGallery | bend: 2 (lebih pipih) |
| Testimonial | Horizontal scroll (1 card visible) |
| CTA | Full-width button |
| SplashCursor | Disabled (performa) |
| FloatingLines | lineCount dikurangi [4, 6, 8] |
| Padding | Section: 24px horizontal |

---

## Accessibility

- [ ] Contrast ratio 4.5:1 untuk text
- [ ] Touch targets min 44x44px
- [ ] Alt text untuk semua gambar
- [ ] Keyboard navigation support
- [ ] prefers-reduced-motion respect
- [ ] Skip to main content link
- [ ] Heading hierarchy h1 → h2 → h3
- [ ] aria-labels untuk icon-only buttons

---

## File Structure

```
frontend/src/
├── app/
│   ├── page.tsx              # Landing page
│   ├── layout.tsx            # Root layout (Poppins + Inter fonts)
│   ├── globals.css           # Global styles
│   └── login/page.tsx        # Login page
├── components/
│   ├── landing/
│   │   ├── Hero.tsx          # Hero section
│   │   ├── Layanan.tsx       # Features section
│   │   ├── CaraKerja.tsx     # How it works section
│   │   ├── Studio.tsx        # Studio showcase
│   │   ├── Testimonial.tsx   # Testimonials
│   │   ├── CTA.tsx           # Call to action
│   │   └── Footer.tsx        # Footer
│   ├── nav/
│   │   └── GooeyNav.tsx      # Navigation bar
│   └── ui/
│       ├── FloatingLines.tsx  # Background effect
│       ├── BlurText.tsx       # Text animation
│       ├── RotatingText.tsx   # Rotating text
│       ├── ShinyText.tsx      # Shiny text
│       ├── BorderGlow.tsx     # Border glow card
│       ├── GlassSurface.tsx   # Glass card
│       ├── MagicBento.tsx     # Bento grid
│       ├── CircularGallery.tsx # Gallery
│       ├── StarBorder.tsx     # Star border button
│       └── SplashCursor.tsx   # Fluid cursor
└── lib/
    └── utils.ts              # cn() utility
```

---

## Referensi

- React Bits: https://www.reactbits.app/
- BMD Music Instagram: https://www.instagram.com/bmdmusic/
- FloatingLines source: `design/FLOATING_LINE_BG.md`
- UI/UX guidelines: `design/` folder
