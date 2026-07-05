# Landing Page Audit — Impeccable Audit

**Date:** 2026-07-05
**Target:** `/` (Landing Page)
**Register:** Brand (design IS the product)

---

## Audit Health Score

| # | Dimension | Score | Key Finding |
|---|-----------|-------|-------------|
| 1 | Accessibility | 2/4 | 26 gray-400/500 text instances (contrast risk) |
| 2 | Performance | 3/4 | Good, minor optimization possible |
| 3 | Responsive | 3/4 | 24 breakpoint instances, works on mobile |
| 4 | Theming | 4/4 | Consistent tokens, no dark mode issues |
| 5 | Anti-Patterns | 1/4 | **CRITICAL: 6 AI tells found** |
| **Total** | | **13/20** | **Acceptable (significant work needed)** |

---

## Anti-Patterns Verdict (CRITICAL)

### Does this look AI-generated?

**YES.** Multiple AI tells detected:

#### 1. Glassmorphism (Navbar)
**File:** `Navbar.tsx:25`
```
bg-white/90 backdrop-blur-md shadow-sm border-b border-gray-100
```
**Ban:** Glassmorphism as default. Remove blur, use solid bg.

#### 2. Hero-Metric Template
**File:** `HeroSection.tsx:9`
```
text-4xl font-bold tracking-tight text-white sm:text-5xl lg:text-6xl
```
**Ban:** Hero-metric template (big number + small label).

#### 3. Identical Card Grids (2x)
**Files:** `ProgramKursusSection.tsx:20`, `HargaSection.tsx:107`
```
grid-cols-1 sm:grid-cols-2 lg:grid-cols-3
```
**Ban:** Identical card grids repeated endlessly.

#### 4. Eyebrow Sections (3x)
**File:** `Footer.tsx:16,27,37`
```
text-sm font-bold uppercase tracking-wider text-white
```
**Ban:** Tiny uppercase tracked eyebrow above every section.

#### 5. Reflex-Reject Fonts
**Fonts:** Plus Jakarta Sans + Inter
**Status:** On the brand register's reflex-reject list:
> "Plus Jakarta Sans · Inter · DM Sans · Outfit..."

**Impact:** Creates monoculture with 55-95% of AI-generated surfaces.

#### 6. Gray Text Contrast
**Count:** 26 instances of gray-400/500 text
**Risk:** Many may fail WCAG AA (4.5:1 contrast ratio)

---

## Detailed Findings

### P0 (Critical) — Must Fix

| # | Issue | File | Line |
|---|-------|------|------|
| 1 | Glassmorphism navbar | Navbar.tsx | 25 |
| 2 | Reflex-reject fonts (Plus Jakarta Sans + Inter) | globals.css | — |
| 3 | Gray-400 text on white (contrast failure) | Multiple | — |

### P1 (High) — Should Fix

| # | Issue | File | Line |
|---|-------|------|------|
| 4 | Identical card grids | ProgramKursusSection.tsx | 20 |
| 5 | Identical card grids | HargaSection.tsx | 107 |
| 6 | Eyebrow sections in footer | Footer.tsx | 16,27,37 |

### P2 (Medium) — Could Fix

| # | Issue | File | Line |
|---|-------|------|------|
| 7 | Hero heading could be tighter | HeroSection.tsx | 9 |
| 8 | Missing text-wrap: balance on h1-h3 | Multiple | — |
| 9 | Missing prefers-reduced-motion | Multiple | — |

### P3 (Low) — Nice to Have

| # | Issue | File | Line |
|---|-------|------|------|
| 10 | No aria-label on social icons | Footer.tsx | — |
| 11 | Missing semantic landmarks | page.tsx | — |

---

## Recommendations

### 1. Replace Fonts (P0)
Replace Plus Jakarta Sans + Inter with distinctive fonts.
Suggestion: **Geist Sans** (Vercel) + **Geist Mono** for a tech-brand feel, or **Space Grotesk** + **Source Serif 4** for warmth + technicality.

### 2. Fix Navbar (P0)
Replace glassmorphism with solid bg:
```tsx
// Before
className="bg-white/90 backdrop-blur-md shadow-sm border-b border-gray-100"

// After
className="bg-white shadow-sm border-b border-gray-100"
```

### 3. Fix Contrast (P0)
Replace gray-400 with gray-600 for body text:
```tsx
// Before
className="text-gray-400"

// After
className="text-gray-600"
```

### 4. Break Card Grids (P1)
Use varied layouts:
- ProgramKursusSection: Featured card + 2 smaller
- HargaSection: Asymmetric grid or stacked list

### 5. Remove Eyebrows (P1)
Replace uppercase tracked headings with plain h4:
```tsx
// Before
<h4 className="text-sm font-bold uppercase tracking-wider text-white">

// After
<h4 className="text-sm font-semibold text-gray-300">
```

---

## Executive Summary

- **Audit Health Score:** 13/20 (Acceptable)
- **Total issues:** 11 (3 P0, 3 P1, 3 P2, 2 P3)
- **Rating:** Significant anti-pattern work needed

**Top 3 Critical Issues:**
1. Glassmorphism navbar (AI tell)
2. Reflex-reject fonts (Plus Jakarta Sans + Inter)
3. Gray-400 contrast failures (26 instances)

**Priority Actions:**
1. Replace fonts → eliminates biggest AI tell
2. Fix navbar → removes glassmorphism
3. Fix contrast → improves accessibility
