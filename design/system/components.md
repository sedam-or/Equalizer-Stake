# Komponen React Bits — BMD Music School

Daftar komponen React Bits yang digunakan + konfigurasi untuk BMD Music.

---

## Daftar Komponen

| # | Komponen | Kegunaan | Dependencies |
|---|----------|----------|-------------|
| 1 | FloatingLines | Background garis-garis animasi | three |
| 2 | BlurText | Headline blur-in saat scroll | motion |
| 3 | RotatingText | Sub-headline berganti otomatis | motion |
| 4 | ShinyText | Accent brand name | motion |
| 5 | BorderGlow | Border glow saat hover | — |
| 6 | GlassSurface | Card background transparan | — |
| 7 | MagicBento | Bento grid + spotlight + particles | gsap |
| 8 | CircularGallery | Gallery melingkar + scroll | ogl |
| 9 | StarBorder | Tombol dengan border animasi | — |
| 10 | SplashCursor | Efek fluid cursor interaktif | — |
| 11 | GooeyNav | Navigasi dengan efek gooey | — |

---

## 1. FloatingLines

Background garis-garis animasi menggunakan Three.js.

### Hero Configuration

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

### CTA Section Configuration

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

### Props

| Prop | Type | Default | Hero | CTA |
|------|------|---------|------|-----|
| enabledWaves | array | ['top','middle','bottom'] | ['top','middle','bottom'] | ['top','middle','bottom'] |
| lineCount | array | [6] | [8, 12, 16] | [6, 10, 14] |
| lineDistance | array | [5] | [8, 6, 4] | [10, 8, 5] |
| bendRadius | number | 10.0 | 5.0 | 4.0 |
| bendStrength | number | -5.0 | -0.5 | -0.3 |
| interactive | boolean | true | true | true |
| parallax | boolean | true | true | true |
| gradientStart | string | — | #08CB00 | #253900 |
| gradientEnd | string | — | #000000 | #000000 |
| animationSpeed | number | 1 | 0.8 | 0.6 |
| mixBlendMode | string | 'screen' | 'screen' | 'screen' |

---

## 2. BlurText

Teks dengan efek blur-in saat scroll masuk viewport.

### Configuration

```jsx
<BlurText
  text="Sistem Penjadwalan Kursus Musik & Reservasi Studio"
  delay={150}
  animateBy="words"
  direction="top"
  className="text-4xl md:text-5xl font-bold text-white"
/>
```

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| text | string | "" | Teks headline |
| delay | number | 200 | 150ms antar word |
| animateBy | string | "words" | "words" |
| direction | string | "top" | "top" |
| className | string | "" | text-4xl md:text-5xl font-bold text-white |

---

## 3. RotatingText

Teks yang berganti otomatis dengan animasi.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| texts | array | [] | ["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"] |
| rotationInterval | number | 2000 | 2000ms |
| staggerFrom | string | "first" | "last" |
| staggerDuration | number | 0 | 0.025 |
| transition | object | — | spring, damping: 30, stiffness: 400 |

---

## 4. ShinyText

Teks dengan efek kilau/glow.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| text | string | — | "BMD MUSIC SCHOOL" |
| speed | number | 2 | 2 seconds |
| color | string | "#b5b5b5" | "#EEEEEE" |
| shineColor | string | "#ffffff" | "#08CB00" |
| spread | number | 120 | 120 degrees |
| direction | string | "left" | "left" |

---

## 5. BorderGlow

Card dengan border glow saat hover.

### Configuration

```jsx
<BorderGlow
  edgeSensitivity={30}
  glowColor="120 100 50"
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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| edgeSensitivity | number | 30 | 30 |
| glowColor | string | "40 80 80" | "120 100 50" (hijau) |
| backgroundColor | string | "#120F17" | "#120F17" |
| borderRadius | number | 28 | 20 |
| glowRadius | number | 40 | 30 |
| glowIntensity | number | 1.0 | 0.8 |
| colors | array | [...] | ['#08CB00', '#253900', '#08CB00'] |

---

## 6. GlassSurface

Card dengan efek glassmorphism.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| width | number/string | 200 | "100%" |
| height | number/string | 80 | "auto" |
| borderRadius | number | 20 | 20 |
| brightness | number | 50 | 30 |
| opacity | number | 0.93 | 0.15 |
| blur | number | 11 | 11 |
| backgroundOpacity | number | 0 | 0.1 |
| saturation | number | 1 | 1.2 |

---

## 7. MagicBento

Bento grid dengan spotlight, particles, dan border glow.

### Configuration

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
  glowColor="8, 203, 0"
/>
```

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| textAutoHide | boolean | true | true |
| enableStars | boolean | true | true |
| enableSpotlight | boolean | true | true |
| enableBorderGlow | boolean | true | true |
| enableTilt | boolean | false | false |
| enableMagnetism | boolean | true | true |
| clickEffect | boolean | true | true |
| spotlightRadius | number | 300 | 300 |
| particleCount | number | 12 | 12 |
| glowColor | string | "132, 0, 255" | "8, 203, 0" (hijau) |

---

## 8. CircularGallery

Gallery melingkar dengan scroll interaktif.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| items | array | — | 6 studio items |
| bend | number | 3 | 3 |
| textColor | string | "#ffffff" | "#EEEEEE" |
| borderRadius | number | 0.05 | 0.05 |
| font | string | "bold 30px Figtree" | "bold 24px Poppins" |
| scrollEase | number | 0.05 | 0.02 |

---

## 9. StarBorder

Tombol dengan border animasi bintang.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| as | string | "button" | "button" |
| color | string | "white" | "#08CB00" |
| speed | string | "6s" | "5s" |
| thickness | number | 1 | 1 |

---

## 10. SplashCursor

Efek fluid cursor interaktif.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| SIM_RESOLUTION | number | 128 | 128 |
| DYE_RESOLUTION | number | 1440 | 1024 |
| DENSITY_DISSIPATION | number | 3.5 | 3.5 |
| VELOCITY_DISSIPATION | number | 2 | 2 |
| SPLAT_RADIUS | number | 0.2 | 0.15 |
| SPLAT_FORCE | number | 6000 | 4000 |
| RAINBOW_MODE | boolean | true | false |
| COLOR | string | "#ff0000" | "#08CB00" |
| TRANSPARENT | boolean | — | true |

---

## 11. GooeyNav

Navigasi dengan efek gooey/bubble.

### Configuration

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

### Props

| Prop | Type | Default | Nilai |
|------|------|---------|-------|
| items | array | [] | 4 nav items |
| particleCount | number | 15 | 15 |
| particleDistances | array | [90, 10] | [90, 10] |
| particleR | number | 100 | 100 |
| initialActiveIndex | number | 0 | 0 |
| animationTime | number | 600 | 600ms |
| timeVariance | number | 300 | 300ms |
| colors | array | [1,2,3,1,2,3,1,4] | [1,2,3,1,2,3,1,4] |
