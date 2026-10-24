# Backend Model Audit — T-179

**Date:** 2026-07-05
**Scope:** All 18 model files in backend/app/models/

---

## MODEL SUMMARY

| Model | Table | Fields | Encrypted | Enums | FK | Index | Soft Delete | Timestamps |
|-------|-------|--------|-----------|-------|-----|-------|-------------|------------|
| attendance | attendances | 12 | 1 | 2 | 2 | 0 | ✅ | ✅ |
| audit_log | audit_logs | 10 | 0 | 1 | 0 | 0 | ❌ | ✅ |
| deletion_request | deletion_requests | 6 | 0 | 1 | 1 | 1 | ❌ | ✅ |
| draft | drafts | 6 | 0 | 0 | 1 | 2 | ❌ | ✅ |
| feedback | feedbacks | 8 | 0 | 0 | 3 | 3 | ❌ | ✅ |
| guardian | guardians | 13 | 4 | 1 | 1 | 1 | ✅ | ✅ |
| invoice | invoices | 11 | 0 | 1 | 1 | 0 | ✅ | ✅ |
| notification | notifications | 10 | 0 | 3 | 1 | 0 | ❌ | ✅ |
| payment | payments | 9 | 0 | 1 | 2 | 0 | ✅ | ✅ |
| pricing | pricings | 12 | 0 | 0 | 0 | 0 | ❌ | ✅ |
| rating | ratings | 7 | 0 | 0 | 2 | 2 | ❌ | ✅ |
| registration | registrations | 12 | 2 | 1 | 0 | 0 | ❌ | ✅ |
| room | rooms | 7 | 0 | 1 | 0 | 0 | ❌ | ✅ |
| session | sessions | 11 | 0 | 2 | 4 | 0 | ❌ | ✅ |
| student | students | 11 | 1 | 3 | 1 | 0 | ✅ | ✅ |
| system_setting | system_settings | 6 | 0 | 0 | 0 | 1 | ❌ | ✅ |
| teacher | teachers | 8 | 0 | 1 | 1 | 0 | ❌ | ✅ |
| user | users | 18 | 1 | 1 | 0 | 0 | ✅ | ✅ |

**TOTALS:** 18 models, 176 fields, 9 encrypted, 18 enums, 20 FK, 10 indexes, 6 soft delete

---

## CRITICAL FINDINGS

### 1. Student Level System — ❌ WRONG

**Current Model:**
```python
level: Mapped[str] = mapped_column(
 Enum("pemula", "dasar", "menengah", "lanjut", name="student_level"),
 nullable=False, default="pemula"
)
```

**SRS Addendum (Section 6):**
Level HARUS per instrumen, bukan universal.

| Instrumen | Levels |
|-----------|--------|
| Piano Klasik | pra_elementary, elementary, grade_1, grade_2, grade_3, grade_4, grade_5, grade_6 |
| Piano Pop | beginner, intermediate, intermediate_2 |
| Vokal | basic, intermediate, advance |
| Biola | basic, intermediate, advance |
| Gitar | basic, intermediate, advance |
| Keyboard | basic, intermediate, advance |
| Drum | basic, intermediate, advance |
| Flute | basic, intermediate, advance |
| Saxophone | basic, intermediate, advance |

**Impact:** Student level data is meaningless — "pemula" doesn't tell which grade.
**Fix:** Remove enum constraint, use String(50) with validation at application level.

---

### 2. Student Instrument Enum — ⚠️ MISSING bass

**Current Model:**
```python
instrument: Mapped[str] = mapped_column(
 Enum("biola", "piano", "gitar", "drum", "keyboard", "flute", "saxophone", "vokal", "ensemble", name="instrument_type"),
 nullable=False
)
```

**SRS Addendum (Section 5):**
Instruments: biola, piano, gitar, drum, keyboard, flute, saxophone, vokal, ensemble, **bass**

**Impact:** Can't register bass students.
**Fix:** `ALTER TYPE instrument_type ADD VALUE 'bass'`

---

### 3. Missing Indexes — ⚠️ PERFORMANCE

**Models with no indexes on foreign keys:**
- attendance (id_session, id_student)
- invoice (id_guardian)
- payment (id_invoice)
- session (id_teacher, id_student, id_room)
- student (id_guardian)

**Impact:** Slow queries on large datasets
**Fix:** Add index=True to all foreign key fields

---

### 4. Missing Soft Delete — ⚠️ INCONSISTENT

**Models WITH soft delete (deleted_at):**
- attendance, guardian, invoice, payment, student, user

**Models WITHOUT soft delete:**
- audit_log (correct — immutable)
- deletion_request, draft, feedback, notification, pricing, rating, registration, room, session, system_setting, teacher

**Impact:** Can't soft-delete records for recovery
**Fix:** Add deleted_at to: teacher, room, session, feedback, rating

---

### 5. Registration Model — ⚠️ NO ENUM VALIDATION

**Current:**
```python
instrument: Mapped[str] = mapped_column(String(50), nullable=False)
level: Mapped[str] = mapped_column(String(50), nullable=False)
class_type: Mapped[str] = mapped_column(String(50), nullable=False)
```

**Issue:** String(50) — accepts any value, no validation
**Fix:** Add enum or application-level validation

---

## ENUM VALUES AUDIT

### ✅ CORRECT Enums:
- guardian.account_status: active, frozen, inactive, pending_deletion
- session.session_type: reguler, teori, rehearsal, recording, kompensasi
- session.session_status: draft, pending, locked, completed, cancelled, rescheduled, pending_approval, approved
- attendance.status: hadir, izin, sakit, alpha
- attendance.verification_status: pending_review, verified, rejected
- invoice.status: draft, issued, paid, overdue, cancelled
- payment.payment_method: transfer, cash, qris, ewallet
- user.role: owner, admin, teacher, guardian

### ❌ WRONG Enums:
- student.level: pemula, dasar, menengah, lanjut (should be per-instrument)

### ⚠️ NEED VALIDATION:
- student.instrument: missing bass
- registration.instrument/level/class_type: no enum

---

## RECOMMENDED FIXES

### P0 — CRITICAL
1. **Student level system:** Remove flat enum, use String(50) with per-instrument validation
2. **Add bass to instrument enum:** ALTER TYPE instrument_type ADD VALUE 'bass'

### P1 — HIGH
3. **Add indexes to foreign keys:** All FK fields should have index=True
4. **Add soft delete to:** teacher, room, session, feedback, rating
5. **Add enum validation to registration:** instrument, level, class_type

### P2 — MEDIUM
6. **Add blind index to:** student.full_name_hash (for search)
7. **Add unique constraints:** pricing (instrument + class_type + level + effective_date)
