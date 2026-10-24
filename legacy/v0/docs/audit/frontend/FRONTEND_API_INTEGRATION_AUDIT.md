# Frontend API Integration Audit — T-183

**Date:** 2026-07-05
**Scope:** All frontend fetch calls vs backend endpoints

---

## API CALL INVENTORY

### admins (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/admin-mgmt/admins | ✅ admin_mgmt.py | OK |

### assign-session (4 calls)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/students/ | ✅ students.py | OK |
| GET /api/v1/sessions/teachers | ✅ scheduling.py | OK |
| GET /api/v1/sessions/rooms | ✅ scheduling.py | OK |
| POST /api/v1/sessions/ | ✅ scheduling.py | OK |

### feedback (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/feedback/ | ✅ feedback.py | OK |

### holidays (2 calls)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/holidays/ | ✅ holidays.py | OK |
| POST /api/v1/holidays/ | ✅ holidays.py | OK |

### my-schedule (2 calls)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/auth/me/profile | ✅ auth.py | OK |
| GET /api/v1/my/children | ✅ my_portal.py | OK |

### my-students (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/students/ | ✅ students.py | OK |

### notification-settings (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| PUT /api/v1/notifications/toggle | ✅ notification.py | OK |

### override (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| POST /api/v1/override/ | ✅ override.py | OK |

### profile (6 calls)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/auth/me/profile | ✅ auth.py | OK |
| PATCH /api/v1/auth/me/profile | ✅ auth.py | OK |
| POST /api/v1/auth/change-password | ✅ auth.py | OK |
| POST /api/v1/photos/upload | ✅ photos.py | OK |
| POST /api/v1/privacy/delete-request | ✅ privacy.py | OK |

### ratings (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/ratings/ | ✅ ratings.py | OK |

### reports (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/reports/owner-dashboard | ✅ reports.py | OK |

### schedule (4 calls)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/sessions/rooms | ✅ scheduling.py | OK |
| GET /api/v1/sessions/teachers | ✅ scheduling.py | OK |
| GET /api/v1/students/ | ✅ students.py | OK |
| GET /api/v1/sessions/ | ✅ scheduling.py | OK |

### teachers (1 call)
| Frontend Call | Backend Endpoint | Status |
|--------------|------------------|--------|
| GET /api/v1/teachers | ✅ teachers.py | OK |

---

## RESPONSE FORMAT ANALYSIS

### Consistent Format (data/total/limit/offset):
- students ✅
- teachers ✅
- sessions/rooms ✅
- sessions/teachers ✅
- billing/invoices ✅
- pricing ✅
- holidays ✅
- notifications ✅
- admin-mgmt ✅

### Inconsistent Format:
- auth/me — returns single object (not paginated)
- auth/me/profile — returns single object
- reports/* — returns custom objects
- override/* — returns custom objects

---

## ERROR RESPONSE HANDLING

### Pages using parseApiError():
- admins ✅
- assign-session ✅
- holidays ✅
- my-students ✅
- override ✅
- pricing ✅
- profile ✅
- teachers ✅

### Pages NOT using parseApiError:
- feedback ⚠️
- ratings ⚠️
- reports ⚠️
- schedule ⚠️
- All pages without error handling ❌

---

## CRITICAL FINDINGS

### 1. Scheduling Prefix Mismatch — ✅ FIXED

**Was:** Frontend called `/api/v1/scheduling/` but backend is `/api/v1/sessions/`
**Fix:** Changed to `/api/v1/sessions/` in assign-session page

### 2. Billing Path Mismatch — ✅ FIXED (T-178)

**Was:** Frontend called `/api/billing/` but backend is `/api/v1/billing/invoices`
**Fix:** Updated in billing page

### 3. Override Path Mismatch — ✅ FIXED (T-178)

**Was:** Frontend called `/api/v1/attendances/override` but backend is `/api/v1/override/`
**Fix:** Updated in override page

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Add parseApiError to remaining pages:** feedback, ratings, reports, schedule
2. **Standardize response format:** All list endpoints should use {data, total, limit, offset}

### P2 — MEDIUM
3. **Add pagination to my_portal endpoints:** schedule, children, payments, students, attendance, invoices
4. **Document API response format:** Create API reference for frontend team
