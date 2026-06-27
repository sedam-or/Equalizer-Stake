# Equalizer-Stake — Frontend Architecture (Next.js PWA)

## App Router Structure

```
frontend/
├── public/
│   ├── manifest.json           # PWA manifest
│   ├── sw.js                   # Service worker (Workbox)
│   ├── icons/
│   │   ├── icon-192.png
│   │   └── icon-512.png
│   └── models/                 # face-api.js model weights
│       ├── tiny_face_detector_model-weights_manifest.json
│       ├── tiny_face_detector_model-shard1
│       └── face_landmark_68_model-*
├── src/
│   ├── app/
│   │   ├── layout.tsx          # Root layout: <html lang="id">, fonts, metadata
│   │   ├── page.tsx            # Landing / redirect to dashboard
│   │   ├── (auth)/
│   │   │   ├── login/page.tsx
│   │   │   └── layout.tsx      # No sidebar, centered form
│   │   ├── (dashboard)/
│   │   │   ├── layout.tsx      # Sidebar + top nav + role guard
│   │   │   ├── page.tsx        # Dashboard home (role-specific)
│   │   │   ├── students/
│   │   │   │   ├── page.tsx    # List students
│   │   │   │   ├── [id]/page.tsx
│   │   │   │   └── new/page.tsx
│   │   │   ├── teachers/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [id]/page.tsx
│   │   │   ├── sessions/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [id]/page.tsx
│   │   │   ├── attendance/
│   │   │   │   └── page.tsx    # GPS + face check-in page
│   │   │   ├── invoices/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [id]/page.tsx
│   │   │   ├── payments/
│   │   │   │   └── page.tsx
│   │   │   ├── notifications/
│   │   │   │   └── page.tsx
│   │   │   └── admin/
│   │   │       ├── page.tsx    # Admin dashboard
│   │   │       ├── audit-log/page.tsx
│   │   │       └── settings/page.tsx
│   │   └── guardian/
│   │       ├── layout.tsx      # Guardian-specific layout
│   │       ├── page.tsx        # Guardian dashboard
│   │       ├── invoices/page.tsx
│   │       └── payments/page.tsx
│   ├── components/
│   │   ├── layouts/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── TopNav.tsx
│   │   │   └── RoleGuard.tsx   # Client-side role check + redirect
│   │   ├── features/
│   │   │   ├── attendance/
│   │   │   │   ├── GpsCheck.tsx        # Geofence 150m validation
│   │   │   │   ├── FaceCapture.tsx     # Webcam + face-api.js detection
│   │   │   │   └── AttendanceForm.tsx  # Combined GPS + face submit
│   │   │   ├── billing/
│   │   │   │   ├── InvoiceTable.tsx
│   │   │   │   ├── PaymentForm.tsx
│   │   │   │   └── PaymentProofUpload.tsx
│   │   │   ├── students/
│   │   │   │   ├── StudentForm.tsx
│   │   │   │   └── StudentTable.tsx
│   │   │   └── auth/
│   │   │       └── LoginForm.tsx
│   │   └── shared/
│   │       ├── DataTable.tsx   # Sortable, paginated table
│   │       ├── Modal.tsx
│   │       ├── Toast.tsx
│   │       ├── LoadingSpinner.tsx
│   │       └── OfflineBanner.tsx  # Shows when navigator.onLine === false
│   ├── lib/
│   │   ├── api.ts              # Axios instance with interceptors
│   │   ├── auth.ts             # Token storage, refresh logic
│   │   ├── face-detect.ts      # face-api.js wrapper: loadModels(), detectFace()
│   │   ├── gps.ts              # Geolocation + haversine distance calc
│   │   ├── crypto.ts           # SubtleCrypto for client-side draft encryption
│   │   └── db.ts               # IndexedDB wrapper (idb-keyval)
│   ├── hooks/
│   │   ├── useAuth.ts          # Auth context consumer
│   │   ├── useGeolocation.ts   # WatchPosition wrapper
│   │   └── useOfflineSync.ts   # Queue mutations when offline
│   ├── stores/
│   │   ├── authStore.ts        # Zustand: user, token, role
│   │   └── offlineQueue.ts     # Zustand: queued mutations for sync
│   └── types/
│       └── index.ts            # Shared TypeScript types (mirror Pydantic schemas)
├── next.config.ts              # PWA config, env vars
├── tailwind.config.ts
├── tsconfig.json
├── package.json
└── Dockerfile
```

## Component Hierarchy

```
RootLayout (html, body, fonts)
├── AuthLayout (no sidebar)
│   └── LoginPage
│       └── LoginForm
├── DashboardLayout (sidebar + topnav)
│   ├── RoleGuard(allowedRoles=["admin","teacher"])
│   │   ├── DashboardPage
│   │   ├── StudentsPage → StudentTable → DataTable
│   │   ├── StudentDetailPage → StudentForm
│   │   ├── TeachersPage
│   │   ├── SessionsPage
│   │   ├── AttendancePage → GpsCheck + FaceCapture → AttendanceForm
│   │   ├── InvoicesPage → InvoiceTable
│   │   └── AdminPages (audit log, settings)
│   └── RoleGuard(allowedRoles=["guardian"])
│       ├── GuardianDashboard
│       ├── GuardianInvoices
│       └── GuardianPayments
└── OfflineBanner (global, conditional)
```

## State Management

**Zustand** for client state (lightweight, no boilerplate):

```typescript
// stores/authStore.ts
import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface AuthState {
  user: User | null
  token: string | null
  role: 'admin' | 'teacher' | 'guardian' | null
  setAuth: (user: User, token: string) => void
  logout: () => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      role: null,
      setAuth: (user, token) => set({ user, token, role: user.role }),
      logout: () => set({ user: null, token: null, role: null }),
    }),
    { name: 'auth-storage' }
  )
)
```

**Server state**: React Query (TanStack Query) for API data — caching, refetch, optimistic updates.

## PWA Configuration

### manifest.json

```json
{
  "name": "Equalizer-Stake",
  "short_name": "EQ-Stake",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#1e40af",
  "icons": [
    { "src": "/icons/icon-192.png", "sizes": "192x192", "type": "image/png" },
    { "src": "/icons/icon-512.png", "sizes": "512x512", "type": "image/png" }
  ]
}
```

### Service Worker (Workbox via next-pwa)

```typescript
// next.config.ts
import withPWAInit from 'next-pwa'

const withPWA = withPWAInit({
  dest: 'public',
  register: true,
  skipWaiting: true,
  disable: process.env.NODE_ENV === 'development',
  runtimeCaching: [
    {
      urlPattern: /^https:\/\/api\.equalizer-stake\.com\/.*/i,
      handler: 'NetworkFirst',
      options: {
        cacheName: 'api-cache',
        expiration: { maxEntries: 50, maxAgeSeconds: 300 },
        networkTimeoutSeconds: 5,
      },
    },
    {
      urlPattern: /\.(png|jpg|svg|woff2)$/,
      handler: 'CacheFirst',
      options: {
        cacheName: 'static-assets',
        expiration: { maxEntries: 100, maxAgeSeconds: 30 * 24 * 60 * 60 },
      },
    },
  ],
})

export default withPWA({
  // next config
})
```

### Offline Strategy

- **API calls**: NetworkFirst — try network, fall back to cache (5min TTL)
- **Static assets**: CacheFirst — serve from cache, update in background
- **Attendance submission**: Queue in IndexedDB if offline, sync via `useOfflineSync` hook when online
- **Offline banner**: `navigator.onLine` listener, shows `<OfflineBanner />` globally

## face-api.js Integration

```typescript
// lib/face-detect.ts
import * as faceapi from 'face-api.js'

let modelsLoaded = false

export async function loadModels() {
  if (modelsLoaded) return
  const MODEL_URL = '/models'
  await Promise.all([
    faceapi.nets.tinyFaceDetector.loadFromUri(MODEL_URL),
    faceapi.nets.faceLandmark68Net.loadFromUri(MODEL_URL),
  ])
  modelsLoaded = true
}

export async function detectFace(
  video: HTMLVideoElement
): Promise<{ detected: boolean; confidence: number }> {
  await loadModels()
  const detection = await faceapi
    .detectSingleFace(video, new faceapi.TinyFaceDetectorOptions({
      inputSize: 224,
      scoreThreshold: 0.5,
    }))
    .withFaceLandmarks()

  if (!detection) return { detected: false, confidence: 0 }
  return { detected: true, confidence: detection.detection.score }
}
```

Component usage:

```tsx
// components/features/attendance/FaceCapture.tsx
'use client'
import { useRef, useEffect, useState } from 'react'
import { detectFace } from '@/lib/face-detect'

export function FaceCapture({ onVerified }: { onVerified: () => void }) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const [status, setStatus] = useState<'idle' | 'detecting' | 'verified' | 'failed'>('idle')

  useEffect(() => {
    navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user' } })
      .then(stream => { if (videoRef.current) videoRef.current.srcObject = stream })
  }, [])

  const handleDetect = async () => {
    setStatus('detecting')
    const result = await detectFace(videoRef.current!)
    if (result.detected && result.confidence > 0.6) {
      setStatus('verified')
      onVerified()
    } else {
      setStatus('failed')
    }
  }

  return (
    <div>
      <video ref={videoRef} autoPlay muted className="w-full max-w-sm rounded" />
      <button onClick={handleDetect} disabled={status === 'detecting'}>
        {status === 'detecting' ? 'Detecting...' : 'Verify Face'}
      </button>
      {status === 'failed' && <p className="text-red-500">Face not detected. Try again.</p>}
      {status === 'verified' && <p className="text-green-500">✓ Face verified</p>}
    </div>
  )
}
```

## IndexedDB for Drafts

```typescript
// lib/db.ts
import { get, set, del, keys } from 'idb-keyval'

const DRAFT_PREFIX = 'draft:'

export async function saveDraft(formId: string, data: unknown) {
  await set(`${DRAFT_PREFIX}${formId}`, {
    data,
    savedAt: Date.now(),
  })
}

export async function loadDraft<T>(formId: string): Promise<T | null> {
  const draft = await get(`${DRAFT_PREFIX}${formId}`)
  return draft?.data ?? null
}

export async function clearDraft(formId: string) {
  await del(`${DRAFT_PREFIX}${formId}`)
}

export async function getAllDraftKeys(): Promise<string[]> {
  const allKeys = await keys()
  return allKeys.filter(k => typeof k === 'string' && k.startsWith(DRAFT_PREFIX))
}
```

Drafts auto-save every 30s via `useEffect` interval in forms. Cleared on successful submit.
