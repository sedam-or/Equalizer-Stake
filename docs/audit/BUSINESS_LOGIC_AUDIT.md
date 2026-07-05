# Business Logic Audit — SRS Rules Implementation

**Date:** 2026-07-05
**Source:** SRS Addendum (99-addendum-business-rules.md)

---

## BUSINESS RULES STATUS

### ✅ IMPLEMENTED (3 rules)

#### 1. Jatuh Tempo Pembayaran — Tanggal 10
**SRS:** Invoice due date = tanggal 10 setiap bulan
**Code:** `billing.py:41` — `due_date = date.today().replace(day=10)`
**Status:** ✅ Implemented

#### 2. Denda Rp 100.000
**SRS:** Denda Rp 100.000 untuk invoice terlambat
**Code:** `billing.py:120` — `invoice.total_amount += Decimal("100000")`
**Status:** ✅ Implemented

#### 3. Freeze Account — Tanggal 11
**SRS:** Freeze akun guardian jika invoice belum dibayar setelah tanggal 11
**Code:** `scheduler.py:113` — `job_fine_freeze()` runs on 11th
**Status:** ✅ Implemented

---

### ❌ NOT IMPLEMENTED (4 rules)

#### 4. Cuti Max 2 Bulan Per Tahun
**SRS:** Guardian bisa freeze anak max 2 bulan per tahun
**Current:** enrollment_status="cuti" exists, no max validation
**Gap:** No check for cumulative cuti days per year
**Fix:** Add validation in student update endpoint
**Priority:** 🟡 IMPORTANT

#### 5. Durasi Sesi (30/45 Menit)
**SRS:** Sesi kursus 30 atau 45 menit
**Current:** duration_minutes in Pricing model, no validation on Session
**Gap:** Session model doesn't validate duration
**Fix:** Add check constraint or validation
**Priority:** 🟡 IMPORTANT

#### 6. Syarat Usia Tunas (Min 3.5 Tahun)
**SRS:** Kelas tunas min usia 3.5 tahun
**Current:** No age validation
**Gap:** Student.date_of_birth exists but not validated
**Fix:** Add validation when class_type="tunas"
**Priority:** 🟢 NICE TO HAVE

#### 7. Waktu Operasional (Senin-Sabtu, 08:00-20:00)
**SRS:** BMD buka Senin-Sabtu, 08:00-20:00
**Current:** No operational hours validation
**Gap:** Session can be created outside operational hours
**Fix:** Add validation in scheduling
**Priority:** 🟢 NICE TO HAVE

---

## AUTOMATED JOBS STATUS

| Job | Schedule | Business Rule | Status |
|-----|----------|---------------|--------|
| job_auto_invoice | 5th of month | Auto-generate invoices | ✅ |
| job_fine_freeze | 11th of month | Apply fine + freeze | ✅ |
| job_draft_cleanup | Daily 3 AM | Delete old drafts | ✅ |
| job_session_reminder | Every hour | Remind upcoming sessions | ✅ |
| job_database_backup | Daily 2 AM | PostgreSQL dump | ✅ |

---

## PRICING LOGIC

### Registration Fee — ✅ CORRECT
- All 9 instruments match SRS
- Paket Liburan: Rp 60.000

### SPP (Monthly Fee) — ✅ CORRECT
- Per instrument × class_type × level
- 67 pricing records seeded

### Fine System — ✅ CORRECT
- Rp 100.000 flat fine
- Applied on 11th if unpaid
- Guardian account frozen

---

## INVOICE FLOW

```
Day 1-4:   Guardian pays previous month invoice
Day 5:     Auto-invoice generated (job_auto_invoice)
Day 6-10:  Guardian has time to pay
Day 11:    Fine applied + account frozen (job_fine_freeze)
Day 12+:   Guardian must pay fine + original to unfreeze
```

**Status:** ✅ Fully implemented

---

## ATTENDANCE FLOW

```
1. Student arrives at BMD
2. Opens app → submit attendance
3. GPS validated (150m radius)
4. Photo uploaded
5. Teacher reviews
6. Verified/rejected
```

**Status:** ✅ GPS + photo implemented
**Gap:** ⚠️ Face-API.js not implemented (client-side face recognition)

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Cuti max 2 bulan validation:** Add check in student update endpoint
2. **Durasi sesi validation:** Add check in session creation

### P2 — MEDIUM
3. **Syarat usia tunas:** Add validation when class_type="tunas"
4. **Waktu operasional:** Add validation in scheduling

### P3 — LOW
5. **Face-API.js:** Client-side face recognition for attendance
