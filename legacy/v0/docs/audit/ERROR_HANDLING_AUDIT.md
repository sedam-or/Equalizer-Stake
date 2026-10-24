# Error Handling & Edge Cases Audit

**Date:** 2026-07-05
**Scope:** All dashboard pages — error handling, loading states, edge cases

---

## ERROR HANDLING STATUS

### Pages with PROPER error handling (try-catch + error state + loading):
- admins ✅
- assign-session ✅
- my-students ✅
- override ✅
- pricing ✅
- profile ✅
- teachers ✅

### Pages with PARTIAL error handling:
- attendance ⚠️ (has catch but no try, no error state)
- audit-log ⚠️ (has try-catch but no error state)
- feedback ⚠️ (has try-catch but no loading state)
- holidays ⚠️ (has catch but no try, no error state)
- ratings ⚠️ (has try-catch but no loading state)
- registrations ⚠️ (has try-catch but no error state)
- schedule ⚠️ (has try-catch + error state)
- settings ⚠️ (has try-catch but no error state)

### Pages with NO error handling:
- billing ❌ (no try-catch, no error state)
- my-payments ❌ (no try-catch, no error state)
- my-schedule ❌ (no try-catch, no error state)
- notification-settings ❌ (no try-catch, no error state)
- notifications ❌ (no try-catch, no error state)
- payments ❌ (no try-catch, no error state)
- reports ❌ (no try-catch, no error state)
- students ❌ (no try-catch, no error state)
- studio ❌ (no try-catch, no error state)

---

## EDGE CASES NOT HANDLED

### 1. Empty Data States
**Status:** ⚠️ PARTIAL
- Some pages show "Tidak ada data" message
- Some pages show nothing (blank page)
- No consistent empty state component usage

### 2. Network Errors
**Status:** ⚠️ PARTIAL
- Pages with try-catch handle network errors
- Pages without try-catch will crash on network error
- No offline detection or retry mechanism

### 3. Unauthorized (401) Handling
**Status:** ❌ MISSING
- No automatic redirect to login on 401
- Token expired → user stays on page with no feedback
- Need: global 401 handler → redirect to /login

### 4. Forbidden (403) Handling
**Status:** ❌ MISSING
- No "Access Denied" page or message
- User sees blank page or error
- Need: 403 page or toast notification

### 5. Server Error (500) Handling
**Status:** ⚠️ PARTIAL
- Pages with error state show error message
- Pages without error state crash
- Need: consistent error display

### 6. Loading States
**Status:** ⚠️ PARTIAL
- 23/25 pages have loading state
- feedback, ratings missing loading state
- No skeleton loading on most pages

---

## SUMMARY

| Category | Total | Proper | Partial | None |
|----------|-------|--------|---------|------|
| Error Handling | 25 | 7 | 8 | 10 |
| Loading States | 25 | 23 | 0 | 2 |
| Empty States | 25 | ~10 | ~10 | ~5 |
| 401 Handling | 1 | 0 | 0 | 1 |
| 403 Handling | 1 | 0 | 0 | 1 |

---

## RECOMMENDED FIXES

### P0 — CRITICAL
1. **Global 401 handler:** Add axios/fetch interceptor → redirect to /login on 401
2. **Fix 10 pages without error handling:** Add try-catch + error state to billing, my-payments, my-schedule, notification-settings, notifications, payments, reports, students, studio

### P1 — HIGH
3. **Consistent empty state:** Use EmptyState component on all pages
4. **Loading skeletons:** Add SkeletonCard/SkeletonTable to all pages
5. **403 page:** Create access denied page or toast

### P2 — MEDIUM
6. **Retry mechanism:** Add retry button on network errors
7. **Offline detection:** Show "Offline" banner when network unavailable
8. **Error boundary:** Already exists, verify it catches all errors
