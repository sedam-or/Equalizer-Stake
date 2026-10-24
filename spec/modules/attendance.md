# Module: attendance (optional; requires scheduling)

Purpose: (a) **instructor check-in**, proving the teacher was present (GPS/time/photo/face, with admin review); (b) **student attendance marks** (present/late/absent/excused).
Policy: `AttendancePolicy` (04 §6.6). Tables: 06 §4.6.
Legacy reference: `v0-capstone:srs/04-attendance.md`, `backend/app/services/attendance.py`, `services/photo_storage.py`.

## 1. Rules — instructor check-in

- **ATT-01. Eligibility to check in.** The actor is an instructor of the session, the session is `scheduled`, `instructor_checkin.enabled`, and there is no existing check-in for `(session, instructor)` (DB unique). A second submission returns 409 `already_checked_in`.
- **ATT-02. Server-side checks** (pure function `domain::checkin::evaluate(input, policy, location, session, server_now) -> Checks`):
  - **time**: `server_now ∈ [start - window_before, start + window_after]` gives `pass`. Before that it's `early`, after it's `late`. The server clock is authoritative. `client_at` is stored, and a skew greater than 5 min is logged as a warning.
  - **gps**:
    - `gps.enabled=false` → `disabled`.
    - Location has no `geo` → `not_configured`.
    - No coordinates sent → `unavailable`.
    - Otherwise haversine distance d. If `d <= radius_m + min(jitter_allowance_m, reported_accuracy_m)` → `pass`, else `fail`.
  - **photo**:
    - Not required → `disabled`.
    - Missing → `missing`.
    - Otherwise decode with `image` and check: decodes as JPEG/PNG/WebP, ≥ min_width×min_height, ≤ max_bytes. Fail → `invalid`.
    - On success, re-encode as JPEG q=85 with EXIF stripped, encrypt and store (SEC-03, purpose `checkin_photo`).
  - **face**:
    - `face.enabled=false` → `disabled`.
    - The client sends `face_ratio_bps`, computed on device (ATT-06). `>= min` → `pass`, `< min` → `fail`, absent → `not_run`.
- **ATT-03. Outcome.**
  - `verified` if every check is in {pass, disabled, not_configured}. **A `not_configured` result never fails a check-in**, but it is flagged in the admin UI.
  - Otherwise the outcome follows `on_failed_check`: `pending_review` (default) or `rejected`.
  - `unavailable` GPS counts as failed.
  - Emits `checkin_recorded`. A pending result notifies admins.
- **ATT-04. Immutability.** There is no update or delete route. Only the review fields are writable, once, by `POST /checkins/{id}/review {decision: approve|reject, note}`.
- **ATT-05. Review SLA.**
  - The job `attendance.review_watch` runs every 10 min.
  - Pending checks older than `remind_after_minutes` → reminder to admins (dedupe per check-in).
  - Older than `sla_minutes` with `escalate_to_owner` → `review_overdue` event → owner notification.
- **ATT-06. Client-side face detection.**
  - The SPA uses **MediaPipe Tasks Vision `FaceDetector`** (WASM, model `blaze_face_short_range.tflite`, self-hosted under `/static/models/`, about 230 KB), lazy-loaded only on the check-in page.
  - `face_ratio_bps = round(10000 × bbox_area / image_area)` of the largest face.
  - If detection fails to load or takes longer than 3 s, the client submits without `face_ratio_bps` (→ `not_run` → review).
  - **The server doesn't re-run face detection in v1.** This is an explicit trade-off: there's no ML runtime in the binary. The photo is kept for human review instead.
  - An optional Cargo feature `face-recheck` (ONNX via `ort`) is reserved for v1.x.
- **ATT-07. Offline queue.**
  - If the network is down, the SPA stores the payload (GPS, client_at, photo blob, face ratio) in IndexedDB, encrypted with a per-device AES-GCM key (WebCrypto, non-extractable, stored in IndexedDB).
  - It shows "Saved offline. It will be sent automatically" and retries with Background Sync (or retries on app focus).
  - The server marks `offline=true` when `server_now - client_at > 2 min`. The time check uses **client_at** for offline submissions, only if the skew between client_at and the request's first-attempt header `X-EQ-Queued-At` is consistent. Otherwise the result is `late` → review.
  - Photos are deleted from IndexedDB after a 2xx response.
- **ATT-08. Photo retention.** The job `attendance.purge_photos` runs daily and deletes check-in photo files older than `core.privacy.photo_retention_days`: the physical file plus `files.deleted_at`. The check-in row stays.

## 2. Rules — student marks

- **ATT-10. Marking.**
  - `PUT /sessions/{id}/marks [{person_id, status, note?}]` by the instructor (if `marked_by ∈ {Instructor, Either}`) or an admin (`Admin`, `Either`).
  - Allowed from session start until `end + lock_after_hours`. After that, only admins with `attendance.marks.override` can change marks (audited).
  - `late` is suggested automatically if the instructor marks after `start + late_after_minutes`; the instructor can override.
  - Blocked (frozen) participants can be marked; the UI shows the badge.
- **ATT-11. No-show consequence.** On transition to `absent`, emit `attendance.no_show_recorded.v1 {session_id, person_id}`. Enrollment applies `scheduling.cancellation.student_no_show`. Changing `absent` to another status within the lock window emits `attendance.no_show_reverted.v1`, and enrollment voids the credit if one was created. *(These two events are added to the catalog in 03 §6.)*
- **ATT-12. Session completion.** If `instructor_checkin.enabled`, a session completed with no check-in appears in the "Needs attention" list ("No check-in for Lesson X").

## 3. API

| Method | Path | Perm |
|---|---|---|
| GET | `/me/checkin/today` → today's sessions for the instructor with check-in state and window | instructor |
| POST | `/sessions/{id}/checkin` (multipart: `photo`, `json={client_at, gps:{lat,lng,accuracy_m}?, face_ratio_bps?}`), header `X-EQ-Queued-At?` → Checkin | instructor of the session |
| GET | `/checkins?status=pending_review&from&to&instructor_id` | `attendance.review` |
| GET | `/checkins/{id}` (includes a photo URL signed for 5 min, map coords, distance, checks) | `attendance.review` or own |
| POST | `/checkins/{id}/review` | `attendance.review` |
| GET | `/files/{id}?sig=&exp=` | signed URL (SEC-15) |
| GET/PUT | `/sessions/{id}/marks` | instructor of the session / admin |
| GET | `/attendance/summary?person_id&from&to` → `{sessions, present, late, absent, excused, rate_bps}` | scoped |

## 4. Permissions

| Permission | owner | admin | instructor | guardian | student |
|---|---|---|---|---|---|
| attendance.checkin.submit | — | — | own sessions | — | — |
| attendance.review | read | ✅ | — | — | — |
| attendance.marks.write | — | ✅ | own sessions (policy) | — | — |
| attendance.marks.override | — | ✅ | — | — | — |
| attendance.marks.read | ✅ | ✅ | own sessions | linked | own |

## 5. Ports

- `AttendanceStats` (for enrollment eligibility and reports): `marks(person, range) -> Vec<Mark>` and `has_pending(person, range) -> bool`.

## 6. Acceptance criteria

1. **All pass.** Given radius 150 m, jitter 15 m, accuracy 10 m, a location at (-6.2000, 106.8000) (neutral test coordinates) and the instructor at 120 m, within the window, with a valid photo and face 3500 bps, Then `verified`.
2. **GPS fail.** The same at 200 m → `pending_review`, `checks.gps=fail`, and an admin notification event.
3. **Not configured.** Given the location has no geo and GPS is enabled, Then `checks.gps=not_configured` and the status is `verified`.
4. **Duplicate.** A second check-in for the same session → 409.
5. **Photo too small.** 320×240 → `checks.photo=invalid` → pending_review.
6. **Escalation.** A pending check-in older than 120 min → owner notified once.
7. **Lock window.** An instructor marking 49 h after session end with `lock_after_hours=48` → 403 `marks_locked`. An admin with override → 200 and audited.
8. **Photo purge.** A photo older than `photo_retention_days` is gone from disk, and its file row has `deleted_at`.
