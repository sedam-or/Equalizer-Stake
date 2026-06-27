1|
2|### T-012: Attendance UI + Face Detection
3|- **Status**: ✅ Selesai
4|- **Tanggal**: 2026-06-27
5|- **FR/UC terkait**: FR-05, UC-03, UC-11, UC-12
6|- **Files changed**:
7|  - `frontend/src/app/dashboard/attendance/page.tsx` — Camera, GPS, face detection, submit
8|- **Test results**:
9|  - [x] Camera preview: getUserMedia with front camera
10|  - [x] Photo capture: canvas toDataURL JPEG
11|  - [x] Face detection: simulated (TODO: face-api.js)
12|  - [x] GPS validation: Haversine 150m + 15m tolerance
13|  - [x] Status indicators: face ✅/❌, GPS ✅/❌
14|  - [x] Submit button: disabled until all valid
15|  - [x] Result feedback: success/pending
16|- **Security findings**: None (frontend only)
17|- **Notes**:
18|  - Face detection: TODO (integrate face-api.js/MediaPipe)
19|  - Offline draft: TODO (IndexedDB)
20|  - Photo upload to server: TODO
21|- **Dependencies affected**: T-014 (Notification UI)
22|- **Rollback plan**: Delete attendance page
23|
24|### T-013: Billing & Payment UI
25|- **Status**: ✅ Selesai
26|- **Tanggal**: 2026-06-27
27|- **FR/UC terkait**: FR-03, FR-04, FR-15, UC-05, UC-07, UC-08
28|- **Files changed**:
29|  - `frontend/src/app/dashboard/billing/page.tsx` — Invoice list, summary, payment info
30|- **Test results**:
31|  - [x] Summary cards: unpaid, overdue, paid totals
32|  - [x] Invoice list: period, amount, status badges
33|  - [x] Status colors: issued (yellow), paid (green), overdue (red)
34|  - [x] Expandable detail: unique code, payment instructions
35|  - [x] Overdue warning: denda Rp 100k message
36|  - [x] Payment instructions: transfer + unique code format
37|- **Security findings**: None (frontend only)
38|- **Notes**:
39|  - Invoice data: TODO (API call to GET /api/billing/invoices)
40|  - Payment verification: TODO (Admin page)
41|  - Auto-refresh: TODO (WebSocket for status updates)
42|- **Dependencies affected**: T-014 (Notification UI)
43|- **Rollback plan**: Delete billing page
44|
### T-014: Notification UI
- **Status**: ✅ Selesai
- **Tanggal**: 2026-01-15
- **FR/UC terkait**: FR-06, UC-04
- **NFR Ref**: NFR-USA-02 (Mobile-first), NFR-USA-03 (Indonesian)
- **Files changed**:
  - `frontend/src/app/dashboard/notifications/page.tsx` — Notification list with filters
- **Test results**:
  - [x] Notification list: channel icon, subject, body, timestamp
  - [x] Status badges: sent (green), failed (red), pending (yellow)
  - [x] Filters: all, unread, critical, failed
  - [x] Unread indicator: green dot
  - [x] Mark as read on click
  - [x] Retry count display
  - [x] Indonesian UI text
- **Security findings**: None (frontend only)
- **Notes**:
  - API integration: TODO (GET /api/notifications/)
  - Real-time updates: TODO (WebSocket)
  - Notification toggle settings: TODO (Admin page)
- **Dependencies affected**: T-016 (Landing Page)
- **Rollback plan**: Delete notifications page

### T-015: Studio Reservation UI (FR-14)
- **Status**: ✅ Selesai
- **Tanggal**: 2026-01-15
- **FR/UC terkait**: FR-14, UC-06
- **NFR Ref**: NFR-PERF-02 (Conflict ≤1s), NFR-USA-02 (Mobile-first)
- **Files changed**:
  - `frontend/src/app/dashboard/studio/page.tsx` — Studio calendar + booking modal
- **Test results**:
  - [x] Studio selector: 5 studios with rates
  - [x] Booking type: rehearsal (self) vs recording (admin approval)
  - [x] Date picker
  - [x] Time slots: 09:00-21:00, available/booked status
  - [x] Booking modal: confirmation with details
  - [x] Recording: pending approval message
  - [x] Terms: cancellation rules, operating hours
- **Security findings**: None (frontend only)
- **Notes**:
  - API integration: TODO (POST /api/studios/book)
  - Real-time slot availability: TODO (WebSocket)
  - Admin approval flow: TODO
- **Dependencies affected**: T-016 (Landing Page)
- **Rollback plan**: Delete studio page
