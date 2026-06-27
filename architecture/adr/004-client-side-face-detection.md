# ADR-004: Client-Side Face Detection

## Status

Accepted

## Context

Equalizer-Stake requires face verification for teacher attendance check-in. The face check confirms a real person is present (not a photo of someone else). The system must comply with Indonesia's UU PDP (Personal Data Protection Law) which classifies biometric data as "special category personal data" with strict consent and storage requirements.

## Decision

Run face detection **entirely client-side** using face-api.js in the browser. Server receives only a boolean `{ face_verified: true, confidence: 0.87 }` — never the image or face descriptor.

## Rationale

### Privacy / Legal

1. **UU PDP special category**: Biometric data (face descriptors, photos) requires explicit consent, purpose limitation, and security measures equivalent to sensitive personal data. Storing face data server-side triggers full compliance burden.

2. **No biometric data storage**: By never transmitting face images or descriptors to the server, the system avoids storing biometric data entirely. This eliminates data breach risk for face data — there's nothing to breach.

3. **Consent simplification**: Teachers consent to "face is verified during check-in" not "your face data is stored on our servers." Easier to explain, easier to obtain.

### Technical

4. **face-api.js is sufficient**: The use case is binary — face detected or not, with confidence score. No need for face recognition (matching against stored embeddings). face-api.js TinyFaceDetector runs at ~200ms on mid-range Android.

5. **Bandwidth**: Sending a selfie image (~200KB) per attendance check vs sending a boolean (~50 bytes). On Indonesian mobile networks (variable 3G/4G), this matters.

6. **Model size**: TinyFaceDetector + Landmark68 = ~550KB. Cached by service worker after first load. Subsequent visits: instant.

### Architecture

```
Browser                              Server
┌─────────────┐                    ┌──────────────┐
│ Webcam      │                    │              │
│ face-api.js │                    │ POST /attendance │
│ detect()    │───{face_verified:──│ {session_id, │
│             │    true,           │  gps_lat,    │
│ confidence: │    0.87}           │  gps_lng,    │
│    0.87     │                    │  face_verified:│
│             │                    │   true}      │
│ Image NOT   │                    │              │
│ transmitted │                    │ No image     │
│             │                    │ stored       │
└─────────────┘                    └──────────────┘
```

## Consequences

- **No face recognition**: System verifies "a face is present" not "this specific teacher's face." Acceptable for attendance — the combination of face-present + GPS-in-geofence + session-assigned is sufficient verification.
- **Spoofing risk**: A photo held up to camera could pass. Mitigation: face-api.js landmark detection (68 points) makes static photo spoofing harder. Not foolproof — accepted tradeoff for privacy benefit.
- **Offline capability**: Face detection works fully offline (model cached by service worker). Attendance submission queues in IndexedDB if offline.
- **Device dependency**: Requires camera access. If camera unavailable, fallback to GPS-only check with admin review flag.

## Alternatives Considered

- **Server-side face recognition (AWS Rekognition / Azure Face)**: More accurate, enables face matching, but transmits biometric data to cloud — major UU PDP compliance burden, ongoing cost, latency on Indonesian networks
- **Server-side face-api.js**: Same library but run on server — requires transmitting image, stores biometric data, defeats privacy purpose
- **Native ML Kit / ARKit**: Better accuracy, but requires native app (ADR-002 decided PWA)
- **Liveness detection (blink/move)**: Stronger anti-spoofing but significantly worse UX — teachers check in multiple times daily, friction unacceptable
