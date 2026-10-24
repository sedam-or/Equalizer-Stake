# Pricing & Billing Accuracy Audit

**Date:** 2026-07-05
**Scope:** Pricing consistency across landing page, register-course, dashboard, database

---

## PRICING COMPARISON

### 1. Registration Fee

| Instrument | SRS | Landing Page | DB | Status |
|------------|-----|--------------|-----|--------|
| Piano Klasik | 420.000 | 420.000 | ✅ | Match |
| Piano Pop | 420.000 | 420.000 | ✅ | Match |
| Vokal | 420.000 | 420.000 | ✅ | Match |
| Biola | 540.000 | 540.000 | ✅ | Match |
| Gitar Akustik | 420.000 | 420.000 | ✅ | Match |
| Keyboard | 480.000 | 480.000 | ✅ | Match |
| Drum | 420.000 | 420.000 | ✅ | Match |
| Flute | 420.000 | 420.000 | ✅ | Match |
| Saxophone | 420.000 | 420.000 | ✅ | Match |

**Status:** ✅ All registration fees match SRS

---

### 2. SPP (Monthly Fee)

#### Piano Klasik
| Level | SRS | Landing Page | Status |
|-------|-----|--------------|--------|
| Beginner | 420.000 | 420.000 | ✅ |
| Grade 1-2 | 480.000 | 480.000 | ✅ |
| Grade 3-4 | 540.000 | 540.000 | ✅ |
| Grade 5-6 | 630.000 | 630.000 | ✅ |

#### Piano Pop
| Level | SRS | Landing Page | Status |
|-------|-----|--------------|--------|
| Umum | 600.000 | 600.000 | ✅ |

#### Vokal
| Tipe | SRS | Landing Page | Status |
|------|-----|--------------|--------|
| Private (1 student) | 600.000 | 600.000 | ✅ |
| Intensif (2 student) | 390.000 | 390.000 | ✅ |
| Intensif (3 student) | 330.000 | ❌ MISSING | GAP |
| Semi Intensif (4 student) | 300.000 | 300.000 | ✅ |
| Expatriate Private | 840.000 | ❌ MISSING | GAP |
| Expatriate 2 student | 660.000 | ❌ MISSING | GAP |
| Expatriate 3 student | 600.000 | ❌ MISSING | GAP |

#### Other Instruments (Biola, Gitar, Keyboard, Drum, Flute, Saxophone)
| Tipe | SRS | Landing Page | Status |
|------|-----|--------------|--------|
| Private | 420.000 | 420.000 | ✅ |

---

## BILLING LOGIC

### 1. Invoice Generation — ✅ IMPLEMENTED
- Cron job: `job_auto_invoice` runs on 5th of each month
- Creates invoice per guardian (aggregates all active students)
- Generates unique_code (3 digit) for transfer verification

### 2. Invoice Fields — ✅ CORRECT
- id_guardian (per guardian, not per student)
- unique_code (3 digit, unique per billing period)
- billing_period (YYYY-MM format)
- total_amount (sum of all student SPP)
- due_date (10th of month)
- status (draft, issued, paid, overdue, cancelled)

### 3. Fine System — ✅ IMPLEMENTED
- apply_fine endpoint exists
- Fine amount: Rp 100.000 (from SRS)
- Applied when invoice overdue

### 4. Payment Verification — ✅ IMPLEMENTED
- verify_payment endpoint
- Matches unique_code
- Updates invoice status to "paid"
- Creates payment record

---

## CRITICAL GAPS

### 1. Missing Pricing Tiers — ⚠️
**SRS has but landing page missing:**
- Vokal Intensif 3 student: Rp 330.000
- Vokal Expatriate Private: Rp 840.000
- Vokal Expatriate 2 student: Rp 660.000
- Vokal Expatriate 3 student: Rp 600.000

**Impact:** Users can't see all pricing options
**Fix:** Add missing tiers to HargaSection.tsx

### 2. Register Course Pricing — ⚠️ NEED VERIFY
**Question:** Does register-course page show correct pricing from SRS?
**Status:** Need to verify

### 3. Dashboard Pricing vs Landing Page — ⚠️ NEED VERIFY
**Question:** Are pricing in dashboard (admin) and landing page consistent?
**Status:** Dashboard fetches from API, landing page is hardcoded

---

## SUMMARY

| Source | Registration Fee | SPP | Status |
|--------|-----------------|-----|--------|
| SRS | 9 instruments | Multiple tiers | Source of truth |
| Landing Page | ✅ Match | ⚠️ Missing 4 tiers | Partial |
| Database | ✅ Seeded | ✅ Seeded | OK |
| Dashboard | ✅ From API | ✅ From API | OK |

---

## RECOMMENDED FIXES

### P1 — HIGH
1. **Add missing Vokal tiers to landing page:** Intensif 3, Expatriate Private, Expatriate 2, Expatriate 3
2. **Verify register-course pricing:** Ensure it matches SRS

### P2 — MEDIUM
3. **Sync landing page with API:** Consider fetching pricing from API instead of hardcoding
4. **Add Paket Liburan pricing:** SRS has Rp 60.000 but not in landing page
