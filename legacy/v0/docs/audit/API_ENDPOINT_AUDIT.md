# API Endpoint Audit — Frontend vs Backend

**Date:** 2026-07-05
**Scope:** All frontend fetch calls vs backend router endpoints

---

## CRITICAL MISMATCHES

### 1. Override Endpoint — WRONG PATH

**Frontend:** `/api/v1/attendances/override`
**Backend:** `/api/v1/override` (override.py prefix)
**File:** `frontend/src/app/dashboard/override/page.tsx:68`
**Fix:** Change to `/api/v1/override/`

---

### 2. Sessions vs Scheduling — CONFUSING NAMING

**Frontend:** `/api/v1/sessions/`, `/api/v1/sessions/rooms`, `/api/v1/sessions/teachers`
**Backend:** `/api/v1/sessions` (scheduling.py prefix)
**Status:** ✅ CORRECT (scheduling.py uses sessions prefix)
**Note:** File named scheduling.py but prefix is sessions — confusing but works

---

### 3. My Children Endpoint — NEEDS VERIFICATION

**Frontend:** `/api/v1/my/children?limit=50`
**Backend:** `/api/v1/my` prefix (my_portal.py)
**Status:** ⚠️ NEED TO VERIFY endpoint exists in my_portal.py

---

## COMPLETE ENDPOINT MAPPING

### Auth (auth.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| POST /api/v1/auth/login | ✅ | OK |
| POST /api/v1/auth/change-password | ✅ | OK |
| POST /api/v1/auth/forgot-password | ✅ | OK |
| GET /api/v1/auth/me/profile | ✅ | OK |

### Admin Management (admin_mgmt.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/admin-mgmt/admins | ✅ | OK |
| POST /api/v1/admin-mgmt/admins | ✅ | OK |
| DELETE /api/v1/admin-mgmt/admins/{id} | ✅ | OK |

### Students (students.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/students/ | ✅ | OK |
| POST /api/v1/students/ | ✅ | OK |
| PATCH /api/v1/students/{id} | ✅ | OK |
| DELETE /api/v1/students/{id} | ✅ | OK |

### Teachers (teachers.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/teachers | ✅ | OK |
| POST /api/v1/teachers | ✅ | OK |
| PUT /api/v1/teachers/{id} | ✅ | OK |
| DELETE /api/v1/teachers/{id} | ✅ | OK |

### Sessions/Scheduling (scheduling.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/sessions/ | ✅ | OK |
| POST /api/v1/sessions/ | ✅ | OK |
| GET /api/v1/sessions/rooms | ✅ | OK |
| GET /api/v1/sessions/teachers | ✅ | OK |

### Attendance (attendance.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/attendance/ | ✅ | OK |
| POST /api/v1/attendance/ | ✅ | OK |
| POST /api/v1/attendance/{id}/review | ✅ | OK |

### Override (override.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| ❌ /api/v1/attendances/override | /api/v1/override | **WRONG PATH** |

### Reports (reports.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/reports/admin-stats | ✅ | OK |
| GET /api/v1/reports/guardian-stats | ✅ | OK |
| GET /api/v1/reports/owner-dashboard | ✅ | OK |
| GET /api/v1/reports/teacher-stats | ✅ | OK |

### Registrations (registrations.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/registrations/ | ✅ | OK |
| POST /api/v1/registrations/ | ✅ | OK |
| PUT /api/v1/registrations/{id}/approve | ✅ | OK |

### Pricing (pricing.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/pricing | ✅ | OK |
| POST /api/v1/pricing | ✅ | OK |
| PUT /api/v1/pricing/{id} | ✅ | OK |

### Holidays (holidays.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/holidays/ | ✅ | OK |
| POST /api/v1/holidays/ | ✅ | OK |
| DELETE /api/v1/holidays/{date} | ✅ | OK |

### Notifications (notification.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/notifications/ | ✅ | OK |
| GET /api/v1/notifications/unread-count | ✅ | OK |
| PUT /api/v1/notifications/toggle | ✅ | OK |

### Feedback (feedback.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/feedback/ | ✅ | OK |
| POST /api/v1/feedback/ | ✅ | OK |

### Ratings (ratings.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/ratings/ | ✅ | OK |
| POST /api/v1/ratings/ | ✅ | OK |

### Photos (photos.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| POST /api/v1/photos/upload | ✅ | OK |
| GET /api/v1/photos/profile/photo | ✅ | OK |

### Privacy (privacy.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| POST /api/v1/privacy/delete-request | ✅ | OK |

### My Portal (my_portal.py)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/my/schedule | ✅ | OK |
| GET /api/v1/my/children | ⚠️ | NEED VERIFY |

---

## SUMMARY

| Category | Total | Correct | Wrong | Need Verify |
|----------|-------|---------|-------|-------------|
| Auth | 4 | 4 | 0 | 0 |
| Admin Mgmt | 3 | 3 | 0 | 0 |
| Students | 4 | 4 | 0 | 0 |
| Teachers | 4 | 4 | 0 | 0 |
| Sessions | 4 | 4 | 0 | 0 |
| Attendance | 3 | 3 | 0 | 0 |
| Override | 1 | 0 | **1** | 0 |
| Reports | 4 | 4 | 0 | 0 |
| Registrations | 3 | 3 | 0 | 0 |
| Pricing | 3 | 3 | 0 | 0 |
| Holidays | 3 | 3 | 0 | 0 |
| Notifications | 3 | 3 | 0 | 0 |
| Feedback | 2 | 2 | 0 | 0 |
| Ratings | 2 | 2 | 0 | 0 |
| Photos | 2 | 2 | 0 | 0 |
| Privacy | 1 | 1 | 0 | 0 |
| My Portal | 2 | 1 | 0 | **1** |

**Total:** 48 endpoints → 46 correct, 1 wrong, 1 need verify

---

## RECOMMENDED FIXES

### P0 — CRITICAL
1. **Override endpoint:** `/api/v1/attendances/override` → `/api/v1/override/`

### P1 — HIGH
2. **Verify my/children endpoint** exists in my_portal.py
3. **Rename scheduling.py** → sessions.py (match prefix)
