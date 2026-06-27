# File Structure — Frontend BMD Music School

Struktur file untuk landing page dan seluruh frontend.

---

## Current Structure

```
frontend/src/
├── app/
│   ├── page.tsx                  # Landing page (TODO)
│   ├── layout.tsx                # Root layout (fonts: Poppins + Inter)
│   ├── globals.css               # Global styles + CSS variables
│   ├── login/
│   │   └── page.tsx              # Login form + role redirect ✅
│   └── dashboard/
│       ├── layout.tsx            # Dashboard layout (sidebar + bottom nav) ✅
│       ├── page.tsx              # 4 role-based dashboards ✅
│       ├── schedule/
│       │   └── page.tsx          # Scheduling UI + new session form ✅
│       ├── attendance/
│       │   └── page.tsx          # Camera + GPS + face detection ✅
│       └── billing/
│           └── page.tsx          # Invoice list + payment info ✅
├── components/
│   ├── landing/                  # TODO: Landing page sections
│   │   ├── Hero.tsx
│   │   ├── Layanan.tsx
│   │   ├── CaraKerja.tsx
│   │   ├── Studio.tsx
│   │   ├── Testimonial.tsx
│   │   ├── CTA.tsx
│   │   └── Footer.tsx
│   ├── nav/
│   │   ├── GooeyNav.tsx          # TODO
│   │   ├── MobileMenu.tsx        # TODO
│   │   ├── Sidebar.tsx           # Dashboard sidebar ✅
│   │   └── BottomNav.tsx         # Dashboard bottom nav ✅
│   └── ui/                       # TODO: React Bits components
│       ├── FloatingLines.tsx
│       ├── BlurText.tsx
│       ├── RotatingText.tsx
│       ├── ShinyText.tsx
│       ├── BorderGlow.tsx
│       ├── GlassSurface.tsx
│       ├── MagicBento.tsx
│       ├── CircularGallery.tsx
│       ├── StarBorder.tsx
│       └── SplashCursor.tsx
├── hooks/                        # TODO
│   ├── useScrollAnimation.ts
│   └── useMediaQuery.ts
├── lib/
│   └── utils.ts                  # cn() utility ✅
└── types/
    └── index.ts                  # TODO
```

## Status

| Page | Status | File |
|------|--------|------|
| Login | ✅ Done | app/login/page.tsx |
| Dashboard (Owner) | ✅ Done | app/dashboard/page.tsx |
| Dashboard (Admin) | ✅ Done | app/dashboard/page.tsx |
| Dashboard (Guru) | ✅ Done | app/dashboard/page.tsx |
| Dashboard (Wali Murid) | ✅ Done | app/dashboard/page.tsx |
| Schedule | ✅ Done | app/dashboard/schedule/page.tsx |
| Attendance | ✅ Done | app/dashboard/attendance/page.tsx |
| Billing | ✅ Done | app/dashboard/billing/page.tsx |
| Landing Page | TODO | app/page.tsx |
| Notifications | TODO | app/dashboard/notifications/page.tsx |
| Audit Log | TODO | app/dashboard/audit/page.tsx |
| Profile | TODO | app/dashboard/profile/page.tsx |

---

## Public Assets

```
frontend/public/
├── manifest.json                 # PWA manifest ✅
├── favicon.ico                   # TODO
├── icons/                        # TODO
│   ├── icon-192x192.png
│   └── icon-512x512.png
└── images/                       # TODO
    ├── logo.png
    ├── studio-*.jpg
    └── avatar-*.jpg
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
