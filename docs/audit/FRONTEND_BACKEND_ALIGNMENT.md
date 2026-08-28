# Frontend-Backend Alignment Audit

**Date:** 2026-07-05
**Scope:** Request/response format comparison between frontend interfaces and backend serializers

---

## ALIGNMENT STATUS

### 1. Student — ✅ ALIGNED

**Backend Response:**
```json
{
 "data": [{
 "id_student": "uuid",
 "id_guardian": "uuid",
 "full_name": "encrypted",
 "instrument": "piano",
 "level": "pemula",
 "enrollment_status": "active",
 "enrollment_date": "2026-01-01",
 "eligible_for_exam": false
 }],
 "total": 10,
 "limit": 20,
 "offset": 0
}
```

**Frontend Interface:**
```typescript
interface Student {
 id_student: string;
 id_guardian: string;
 full_name: string;
 instrument: string;
 level: string;
 enrollment_status: string;
 enrollment_date: string;
}
```

**Status:** ✅ Fields match

---

### 2. Teacher — ✅ ALIGNED

**Backend Response (TeacherResponse):**
```json
{
 "id_teacher": "uuid",
 "id_user": "uuid",
 "full_name": "string",
 "instruments_specialization": ["piano", "gitar"],
 "employment_type": "tetap",
 "max_students_per_session": 3,
 "is_active": true,
 "username": "string"
}
```

**Frontend Interface:**
```typescript
interface Teacher {
 id_teacher: string;
 id_user: string;
 full_name: string;
 instruments_specialization: string[];
 employment_type: string;
 max_students_per_session: number;
 is_active: boolean;
 username: string | null;
}
```

**Status:** ✅ Fields match

---

### 3. Session — ⚠️ PARTIAL

**Backend Response:**
```json
{
 "data": [{
 "id_session": "uuid",
 "session_date": "2026-07-05",
 "start_time": "09:00",
 "end_time": "10:00",
 "session_type": "reguler",
 "status": "pending",
 "teacher_name": "Pak Dimas",
 "student_name": "Andi",
 "room_name": "Studio A"
 }]
}
```

**Frontend Interface:**
```typescript
interface Session {
 id_session: string;
 session_date: string;
 start_time: string;
 end_time: string;
 session_type: string;
 status: string;
 teacher_name?: string;
 student_name?: string;
 room_name?: string;
}
```

**Status:** ⚠️ Need to verify backend actually returns teacher_name, student_name, room_name (joins required)

---

### 4. Pricing — ⚠️ NEED VERIFY

**Backend Response:**
```json
{
 "id_pricing": "uuid",
 "instrument": "biola",
 "level": "advance",
 "price": 420000.0,
 "effective_date": "2024-01-01",
 "description": null,
 "is_active": true,
 "created_at": "timestamp"
}
```

**Frontend Interface:**
```typescript
interface PricingItem {
 id_pricing: string;
 instrument: string;
 level: string;
 price: number;
 effective_date: string;
 description: string | null;
 is_active: boolean;
 created_at: string | null;
}
```

**Status:** ✅ Fields match
**Note:** Backend uses `monthly_fee` in model but serializes as `price` — correct

---

### 5. Invoice — ⚠️ NEED VERIFY

**Frontend Interface:**
```typescript
interface Invoice {
 id_invoice: string;
 billing_period?: string;
 period?: string;
 amount_total?: number;
 amount?: number;
 due_date: string;
 status: string;
 unique_code?: string;
}
```

**Issue:** Frontend has `billing_period` OR `period`, `amount_total` OR `amount` — suggests backend response inconsistent

**Status:** ⚠️ Need to verify actual backend response

---

### 6. Notification — ⚠️ NEED VERIFY

**Frontend Interface:**
```typescript
// From notifications page
{item.message}
```

**Issue:** Frontend renders `item.message` but Notification model has `body` not `message`

**Status:** ⚠️ LIKELY MISMATCH — model uses `body`, frontend expects `message`

---

### 7. Registration — ✅ ALIGNED

**Backend Response:**
```json
{
 "id_registration": "uuid",
 "status": "pending",
 "message": "Pendaftaran berhasil"
}
```

**Status:** ✅ Simple response, aligned

---

## CRITICAL FINDINGS

### 1. Notification Field Name — MISMATCH

**Model:** `body` (Text field)
**Frontend:** `message` (rendered in JSX)
**Impact:** Notification text won't render
**Fix:** Change frontend to use `body` or add `message` alias in serializer

### 2. Invoice Field Names — INCONSISTENT

**Frontend:** Uses both `billing_period`/`period` and `amount_total`/`amount`
**Impact:** May render undefined values
**Fix:** Standardize backend response to use consistent field names

### 3. Session Joins — NEED VERIFY

**Frontend expects:** `teacher_name`, `student_name`, `room_name`
**Backend:** Need to verify if JOINs are performed to get these names
**Impact:** May show undefined names

---

## RECOMMENDED FIXES

### P0 — CRITICAL
1. **Notification:** Verify `body` vs `message` field name
2. **Invoice:** Standardize field names

### P1 — HIGH
3. **Session:** Verify JOINs for teacher_name, student_name, room_name
4. **Pricing:** Verify `price` serialization (model has `monthly_fee`)

### P2 — MEDIUM
5. **Add response validation:** Use Pydantic response_model on all endpoints
6. **Document API:** Create OpenAPI/Swagger docs for frontend team
