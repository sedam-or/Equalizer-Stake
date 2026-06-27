# Animations — Landing Page BMD Music School

Scroll animations, transitions, dan micro-interactions.

---

## Scroll Animations

| Section | Trigger | Efek | Duration | Easing |
|---------|---------|------|----------|--------|
| Hero | Page load | FloatingLines start | — | — |
| Hero | Page load | BlurText animate | 150ms/word | linear |
| Hero | Page load | SplashCursor activate | — | — |
| Layanan | Scroll masuk 20% | MagicBento cards stagger | 50ms/card | ease-out |
| Cara Kerja | Scroll masuk 20% | 3 cards slide up | 200ms delay | spring |
| Studio | Scroll masuk 20% | CircularGallery mulai interaktif | — | — |
| Testimonial | Scroll masuk 20% | Cards fade in | 300ms | ease-out |
| CTA | Scroll masuk 20% | FloatingLines baru muncul | 500ms | ease-in |

---

## Micro-Interactions

| Elemen | Trigger | Efek | Duration |
|--------|---------|------|----------|
| StarBorder button | Hover | Border glow bergerak | 5s loop |
| BorderGlow card | Hover | Border glow + mesh gradient | 0.3s |
| MagicBento card | Hover | Spotlight + particles + tilt | 0.3s |
| MagicBento card | Click | Ripple effect | 0.8s |
| GooeyNav item | Click | Bubble particles + pill animation | 600ms |
| CircularGallery | Scroll | Gallery bergerak + bend | continuous |
| SplashCursor | Mouse move | Fluid simulation | continuous |
| FloatingLines | Mouse move | Lines bergerak mengikuti cursor | continuous |

---

## Transition Tokens

| Token | Duration | Easing | Kegunaan |
|-------|----------|--------|----------|
| fast | 150ms | ease-out | Micro-interactions |
| normal | 300ms | ease-out | State changes |
| slow | 500ms | ease-in-out | Page transitions |
| spring | 300ms | spring(damping:30, stiffness:400) | Bouncy animations |

---

## Reduced Motion

Semua animasi harus respect `prefers-reduced-motion`:

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

- FloatingLines: disable interactive + parallax
- SplashCursor: disable entirely
- BlurText: show text immediately (no blur)
- RotatingText: show first text only (no rotation)
- ShinyText: disable shine animation
- MagicBento: disable particles + spotlight + tilt
- CircularGallery: static display (no scroll bend)
