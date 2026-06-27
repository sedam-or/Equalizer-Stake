# ADR-002: Next.js PWA over Native App

## Status

Accepted

## Context

Equalizer-Stake needs a frontend for three user roles (admin, teacher, guardian). Key requirements: face detection for attendance, GPS geofencing (150m), offline capability for attendance submission, camera access for face verification. Target users are in Indonesia with varying device quality and network conditions.

## Decision

Build a **Next.js 15 Progressive Web App** (React 18, Tailwind CSS) instead of native Android/iOS apps.

## Rationale

| Factor | PWA (Next.js) | Native (React Native / Flutter) |
|--------|---------------|--------------------------------|
| Development cost | Single codebase, web team | Need iOS + Android or cross-platform |
| Distribution | URL — no app store review | App store (7-14 day review, rejection risk) |
| Updates | Instant on next visit | App store review + user must update |
| Camera/face-api.js | WebRTC + face-api.js works in browser | Native ML Kit / ARKit more powerful |
| GPS | Geolocation API sufficient for 150m radius | Native GPS slightly more accurate |
| Offline | Service Worker + IndexedDB | Built-in SQLite, more robust |
| Install size | ~2MB cached assets | 20-50MB minimum |
| Device reach | Any device with Chrome/Safari | Need to build for each platform |
| Push notifications | Web Push (limited on iOS) | Full native push |

### Key factors:

1. **No app store friction**: Indonesia has high WhatsApp adoption. Distributing a PWA link via WhatsApp is faster than asking guardians to install from Play Store. No developer account fees ($25 Play, $99/year App Store).

2. **face-api.js runs in browser**: ~550KB model weights, runs inference in WebAssembly/WebGL. For attendance verification (face detected: yes/no), accuracy is sufficient. No biometric data leaves the device — aligns with UU PDP compliance.

3. **GPS 150m radius**: Web Geolocation API accuracy is ±10-50m on mobile. Sufficient for 150m geofencing. Native GPS would be ±5m — overkill for this use case.

4. **Offline attendance**: Service Worker caches the attendance page + face-api.js models. IndexedDB queues the submission. Syncs when online. This handles spotty school Wi-Fi.

5. **Guardian simplicity**: Guardians only view invoices/payments. PWA installed to home screen feels native. No complex features requiring native APIs.

### iOS limitations (accepted):

- Web Push only works on iOS 16.4+ (PWA must be installed to home screen)
- face-api.js performance slower on older iPhones — acceptable since attendance is infrequent
- No background sync on iOS — queued submissions sync when app is opened

## Consequences

- Accept slightly lower GPS accuracy (10-50m vs 5m native) — 150m radius compensates
- iOS push notifications require iOS 16.4+ and PWA installation — fallback to WhatsApp notification
- face-api.js models load ~550KB on first attendance page visit — service worker caches after
- No access to native biometric auth (Face ID) — use JWT + face-api.js for attendance only
- Must handle PWA install prompt UX (Add to Home Screen banner)

## Alternatives Considered

- **React Native (Expo)**: Cross-platform native, but adds build complexity, app store dependency, separate deployment pipeline
- **Flutter**: Good performance, but team has React/TypeScript experience, not Dart
- **Capacitor/Cordova**: Hybrid wrapper around web app — adds complexity without native benefits beyond PWA
