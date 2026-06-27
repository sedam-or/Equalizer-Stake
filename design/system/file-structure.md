# File Structure — Frontend BMD Music School

Struktur file untuk landing page dan seluruh frontend.

---

## Landing Page

```
frontend/src/
├── app/
│   ├── page.tsx                  # Landing page (7 sections)
│   ├── layout.tsx                # Root layout (fonts: Poppins + Inter)
│   ├── globals.css               # Global styles + CSS variables
│   ├── login/
│   │   └── page.tsx              # Login page
│   └── dashboard/
│       ├── layout.tsx            # Dashboard layout (sidebar + bottom nav)
│       └── page.tsx              # Dashboard page (KPI cards)
├── components/
│   ├── landing/
│   │   ├── Hero.tsx              # Hero section
│   │   ├── Layanan.tsx           # Features section (MagicBento)
│   │   ├── CaraKerja.tsx         # How it works (3 steps)
│   │   ├── Studio.tsx            # Studio showcase (CircularGallery)
│   │   ├── Testimonial.tsx       # Testimonials (3 cards)
│   │   ├── CTA.tsx               # Call to action
│   │   └── Footer.tsx            # Footer
│   ├── nav/
│   │   ├── GooeyNav.tsx          # Desktop navigation
│   │   ├── MobileMenu.tsx        # Mobile hamburger menu
│   │   ├── Sidebar.tsx           # Dashboard sidebar
│   │   └── BottomNav.tsx         # Dashboard bottom nav
│   └── ui/
│       ├── FloatingLines.tsx     # Background effect (Three.js)
│       ├── BlurText.tsx          # Text blur animation (motion)
│       ├── RotatingText.tsx      # Rotating text (motion)
│       ├── ShinyText.tsx         # Shiny text (motion)
│       ├── BorderGlow.tsx        # Border glow card
│       ├── GlassSurface.tsx      # Glass card
│       ├── MagicBento.tsx        # Bento grid (gsap)
│       ├── CircularGallery.tsx   # Gallery (ogl)
│       ├── StarBorder.tsx        # Star border button
│       └── SplashCursor.tsx      # Fluid cursor
├── hooks/
│   ├── useScrollAnimation.ts     # IntersectionObserver hook
│   └── useMediaQuery.ts          # Responsive breakpoint hook
├── lib/
│   └── utils.ts                  # cn() utility (clsx + twMerge)
└── types/
    └── index.ts                  # TypeScript types
```

---

## Public Assets

```
frontend/public/
├── manifest.json                 # PWA manifest
├── favicon.ico                   # Favicon
├── icons/                        # PWA icons
│   ├── icon-192x192.png
│   └── icon-512x512.png
└── images/                       # Landing page images
    ├── logo.png                  # BMD Music logo
    ├── studio-*.jpg              # Studio photos (untuk CircularGallery)
    └── avatar-*.jpg              # Testimonial avatars
```

---

## Dependencies

### Package.json

```json
{
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "three": "^0.170.0",
    "ogl": "^1.0.0",
    "gsap": "^3.12.0",
    "motion": "^12.0.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "@types/react": "^19.0.0",
    "@types/node": "^22.0.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0"
  }
}
```

### Install

```bash
npm install --legacy-peer-deps
```
