# URGENT TASKS (U-xxx) — Critical Fixes
**Created:** 2026-07-02
**Priority:** HIGHEST — blokir semua task lain sampai selesai
**Rule:** Setiap page HARUS fetch dari API, TIDAK ADA dummy data, TIDAK ADA local state untuk data

---

## U-001: Hapus Semua Dummy Data — Semua Dashboard Page
- **Status**: ✅ Selesai
- **Priority**: 🔴 URGENT
- **Deskripsi**: Hapus SEMUA data dummy dari SEMUA dashboard page. Ganti dengan fetch dari API.
- **Rule**: `const data = [...]` = DILARANG. `fetch('/api/...')` = WAJIB.
- **Tanggal**: 2026-07-02

### Pages yang sudah dibersihkan:

| # | Page | Status | File |
|---|------|--------|------|
| 1 | /dashboard/attendance | ✅ | API: GET /api/attendance/ |
| 2 | /dashboard/billing | ✅ | API: GET /api/billing/ |
| 3 | /dashboard/schedule | ✅ | API: GET /api/sessions/ |
| 4 | /dashboard/my-schedule | ✅ | API: GET /api/my/schedule |
| 5 | /dashboard/my-children | ✅ | API: GET /api/my/children |
| 6 | /dashboard/my-payments | ✅ | API: GET /api/my/payments |
| 7 | /dashboard/payments | ✅ | API: GET /api/payments/ |
| 8 | /dashboard/students | ✅ | API: GET /api/students/ |
| 9 | /dashboard/holidays | ✅ | API: GET /api/holidays/ |
| 10 | /dashboard/studio | ✅ | API: GET /api/sessions/rooms |
| 11 | /dashboard/notifications | ✅ | API: GET /api/notifications/ |
| 12 | /dashboard/profile | ✅ | API: GET /api/auth/me/profile |
| 13 | /dashboard/admins | ✅ | API: GET /api/admin-mgmt/admins |
| 14 | /dashboard/override | ✅ | API: GET /api/sessions/ |
| 15 | /dashboard/settings | ✅ | API: GET /api/settings |
| 16 | /dashboard/pricing | ✅ | API: GET /api/pricing/ |
| 17 | /dashboard/reports | ✅ | API: GET /api/reports/owner-dashboard |
| 18 | /dashboard/audit-log | ✅ | API: GET /api/audit?limit=100&offset=0 |
| 19 | /dashboard/registrations | ✅ | API: GET /api/registrations/ |
| 20 | /dashboard/notification-settings | ✅ | API: GET /api/notifications/toggle |

### Pattern untuk setiap page:
```tsx
// BENAR ✅
const [data, setData] = useState([]);
useEffect(() => {
 fetch('/api/endpoint', { headers: { Authorization: `Bearer ${token}` } })
 .then(r => r.json())
 .then(setData);
}, []);

// SALAH ❌
const data = [{ id: 1, name: "dummy" }];
```

---

## U-002: Studio Coordinates Fix
- **Status**: ✅ Selesai
- **Deskripsi**: Ganti koordinat studio dari -7.7470, 110.3554 ke -7.737428, 110.429669

---

## U-003: Rules Enforcement
- **Status**: ✅ Selesai
- **Deskripsi**: Tambahkan aturan "NO dummy data, NO local state, ALWAYS API" ke TASKS.md
