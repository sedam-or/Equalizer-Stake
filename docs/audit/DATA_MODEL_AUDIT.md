# Data Model Audit — SRS vs Implementation

**Date:** 2026-07-05
**Scope:** All entities in 99-entities.md + 99-addendum-business-rules.md vs SQLAlchemy models

---

## CRITICAL FINDINGS

### 1. Student Level System — WRONG

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

**Current Model (WRONG):**
```python
level: Mapped[str] = mapped_column(
    Enum("pemula", "dasar", "menengah", "lanjut", name="student_level"),
    nullable=False, default="pemula"
)
```

**Gap:** Model uses flat universal levels. SRS requires per-instrument levels.
**Impact:** Student level data is meaningless — "pemula" doesn't tell which grade.
**Fix:** Remove enum constraint, use String(50) with validation at application level.

---

### 2. Student Instrument Enum — MISSING bass

**SRS Addendum (Section 5):**
Instruments: biola, piano, gitar, drum, keyboard, flute, saxophone, vokal, ensemble, **bass**

**Current Model:**
```python
instrument: Mapped[str] = mapped_column(
    Enum("biola", "piano", "gitar", "drum", "keyboard", "flute", "saxophone", "vokal", "ensemble", name="instrument_type"),
    nullable=False
)
```

**Gap:** "bass" not in enum.
**Impact:** Can't register bass students.
**Fix:** Add "bass" to instrument_type enum.

---

### 3. Pricing Model — MISSING class_type from SRS Addendum

**SRS Addendum (Section 7):**
Class types: private, intensif_2, intensif_3, semi_intensif, expatriate_private, expatriate_2, expatriate_3, tunas, paket_liburan

**Current Model:**
```python
class_type: Mapped[str] = mapped_column(String(50), nullable=False)
```

**Gap:** String(50) — no enum validation. Accepts any value.
**Impact:** Invalid class types can be inserted.
**Fix:** Add enum or application-level validation.

---

### 4. Guardian Account Status — CORRECT

**SRS:** active, frozen, inactive, pending_deletion
**Model:** Enum("active", "frozen", "inactive", "pending_deletion")
**Status:** ✅ Match

---

### 5. Session Status — CORRECT

**SRS:** draft, pending, locked, completed, cancelled, rescheduled
**Model:** Enum("draft", "pending", "locked", "completed", "cancelled", "rescheduled", "pending_approval", "approved")
**Status:** ✅ Match (model has extra statuses for approval workflow)

---

### 6. Invoice — CORRECT

**SRS:** id_guardian, unique_code, billing_period, total_amount, due_date, status
**Model:** All fields present
**Status:** ✅ Match

---

### 7. Teacher — CORRECT

**SRS:** id_user, full_name, instruments_specialization (JSONB), employment_type, max_students_per_session
**Model:** All fields present
**Status:** ✅ Match

---

### 8. Registration — PARTIAL

**SRS:** No registration entity in original SRS (added in implementation)
**Model:** Has full_name, phone_whatsapp, email, address, instrument, level, class_type, schedule_id, payment_proof_path, children_data
**Gap:** instrument/level/class_type should match SRS Addendum enums
**Status:** ⚠️ Partial

---

### 9. Room — CORRECT

**SRS:** room_name, room_type (kursus, rehearsal, recording), capacity, is_available
**Model:** All fields present
**Status:** ✅ Match

---

### 10. Attendance — CORRECT

**SRS:** id_session, id_student, id_teacher, status, photo_url, gps_lat, gps_lng, client_timestamp, verification_status
**Model:** All fields present
**Status:** ✅ Match

---

## SUMMARY

| Entity | Status | Critical Issues |
|--------|--------|-----------------|
| User | ✅ | — |
| Guardian | ✅ | — |
| Student | ❌ CRITICAL | Level system wrong, missing bass |
| Teacher | ✅ | — |
| Room | ✅ | — |
| Session | ✅ | — |
| Invoice | ✅ | — |
| Payment | ✅ | — |
| Attendance | ✅ | — |
| Notification | ✅ | — |
| Pricing | ⚠️ | No enum validation on class_type |
| Registration | ⚠️ | instrument/level need SRS Addendum values |

---

## RECOMMENDED FIXES (Priority Order)

### P0 — CRITICAL
1. **Student level system:** Remove flat enum, use String(50) with per-instrument validation
2. **Add bass to instrument enum:** ALTER TYPE instrument_type ADD VALUE 'bass'

### P1 — HIGH
3. **Pricing class_type:** Add validation for SRS Addendum class types
4. **Registration instrument/level:** Update to match SRS Addendum values

### P2 — MEDIUM
5. **Holidays table:** Create DB model (currently in-memory)
6. **Draft auto-save:** Add frontend 30s timer + localStorage sync
