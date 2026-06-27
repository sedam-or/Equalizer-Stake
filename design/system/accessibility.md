# Accessibility — Landing Page BMD Music School

Checklist WCAG 2.1 Level AA untuk landing page.

---

## Color Contrast

| Elemen | Foreground | Background | Ratio | Status |
|--------|------------|------------|-------|--------|
| Body text (dark bg) | #FFFFFF | #000000 | 21:1 | ✅ AAA |
| Body text (light bg) | #000000 | #EEEEEE | 18.1:1 | ✅ AAA |
| Green accent | #08CB00 | #000000 | 5.9:1 | ✅ AA |
| Green on white | #08CB00 | #FFFFFF | 3.6:1 | ⚠️ Large text only |
| Muted text | #888888 | #000000 | 3.5:1 | ⚠️ Large text only |
| Link text | #08CB00 | #FFFFFF | 3.6:1 | ⚠️ Add underline |

---

## Keyboard Navigation

| Rule | Status |
|------|--------|
| Tab order matches visual order | Required |
| All interactive elements focusable | Required |
| Focus rings visible (2-4px, #08CB00) | Required |
| Skip to main content link | Required |
| Escape closes modals/menus | Required |
| No keyboard traps | Required |

---

## Screen Reader

| Rule | Status |
|------|--------|
| Semantic HTML (h1→h2→h3) | Required |
| aria-labels for icon-only buttons | Required |
| alt text for all images | Required |
| aria-live for dynamic content | Required |
| Logical reading order | Required |
| Form labels with for attribute | Required |

---

## Touch & Interaction

| Rule | Standard | Status |
|------|----------|--------|
| Min touch target | 44x44px (Apple) / 48x48dp (MD) | Required |
| Min spacing between targets | 8px | Required |
| Tap feedback | Visual within 100ms | Required |
| No hover-only interactions | Required |
| Disabled state visible | Required |

---

## Motion

| Rule | Status |
|------|--------|
| prefers-reduced-motion respected | Required |
| Animations < 500ms | Required |
| No flashing content (3 per second) | Required |
| Parallax optional | Required |

---

## Forms

| Rule | Status |
|------|--------|
| Visible labels (not placeholder-only) | Required |
| Error messages near fields | Required |
| Required fields marked | Required |
| Auto-complete attributes | Required |

---

## Responsive

| Rule | Status |
|------|--------|
| Viewport meta: width=device-width | Required |
| No horizontal scroll | Required |
| Readable at 200% zoom | Required |
| Text reflows at 320px | Required |
| No text images | Required |

---

## Focus Management

| Element | Focus Style |
|---------|-------------|
| Links | 2px solid #08CB00, 2px offset |
| Buttons | 2px solid #08CB00, 2px offset |
| Form inputs | 2px solid #08CB00, 2px offset |
| GooeyNav items | 2px solid #08CB00, 2px offset |
| StarBorder buttons | 2px solid #08CB00, 2px offset |
