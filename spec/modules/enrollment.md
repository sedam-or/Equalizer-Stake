# Module: enrollment (optional; requires scheduling)

Purpose: students and guardians (1:N), public registration (with placement), enrollments with lifecycle, leave, make-up credits and eligibility.
Policy: `EnrollmentPolicy` (04 §6.4). Tables: 06 §4.4 (+ `persons`, `person_links`, `series_members`).
Legacy reference: `v0-capstone:srs/09-multi-student.md`, the addendum §3, §6, §7 and §9, `backend/app/routers/registrations.py`, `services/eligibility.py`, `cuti_validation.py`, `age_validation.py`.

## 1. Rules

- **ENR-01. Payer resolution.** An enrollment's `payer_id` is:
  - the student's **primary guardian**, if the student's age at `starts_on` < `guardians.required_under_age_years`, or the student has a primary guardian anyway;
  - otherwise the student themself.
  - A student under the age limit with no guardian is rejected with 422 `rule=enrollment.guardians.required_under_age_years`.
- **ENR-02. Age.** If the offering has `min_age_months` / `max_age_months` and the student's birth date is known, the age at `starts_on` is checked. With `age.enforcement`: Block gives 422; Warn gives a warning; Off does nothing. If the birth date is unknown and `collect_birth_date=Required`, the request is rejected with 422.
- **ENR-03. Public registration (`/join`).**
  - The public form is available when `registration.public_form` is on.
  - The applicant gives contact details (name, email/phone; address per policy) and 1..10 students, each with name, birth date (per policy), offering, level (optional), preferred slots, and an optional note. They then accept the consent text.
  - Optional upload: a payment proof (`payment_proof` policy; image/pdf ≤ 5 MB).
  - Anti-abuse: a honeypot field, rate limits of 5/hour per IP and 3/day per email, and an optional Turnstile/hCaptcha if `EQ_CAPTCHA_*` is configured.
  - The data is stored **encrypted** in `registrations.applicant_enc`. No person is created until approval.
  - Emits `registration_submitted`, and sends the applicant a confirmation message through notify, using the email/phone from the form.
- **ENR-04. Approval.**
  - `approval = Auto`: registrations are approved on submit, but only if every student passes the age and capacity checks and `placement = None`. Otherwise they fall back to admin approval.
  - Admin approval opens a review screen:
    - It matches existing persons by email/phone blind index and suggests "Link to existing guardian ‘Ibu Sari’".
    - For each student the admin chooses the offering, level, series (an existing series with free capacity, or "create new slot" through an inline series form) and start date.
    - Approving runs one transaction: create/link persons, guardian links, memberships, enrollments (`active` if `starts_on <= today`, else `pending`), series members, invites (optional checkbox "Send portal invite").
  - `placement = Audition | Trial`: before approval the admin can schedule a **trial session** (kind=trial, consumes_credit=false) for each student. The registration shows the trial result field (free text plus a level pick).
- **ENR-05. Enrollment lifecycle.** Follows 06 §5.2. Activation snapshots `leave_terms`. Activation emits `enrollment_activated`, and billing adds the registration fee line if the offering has a `registration_fee` price plan and the student has never paid one for that offering. The "never paid" check goes through the billing port and is answered by billing, not enrollment.
- **ENR-06. Changing level / series / price plan.** `POST /enrollments/{id}/change {effective_on, level_id?, series_id?, price_plan_id?}`:
  - Membership in the old series ends on `effective_on - 1` and starts in the new one on `effective_on`.
  - Price plan changes apply from the next billing cycle that starts on or after `effective_on`.
- **ENR-07. Leave (cuti).**
  - Requested by the payer (portal) or staff: `{enrollment_id, from, until, reason}`.
  - Validation, using the **snapshotted** `leave_terms`:
    - `enabled`
    - `from >= today + min_notice_days` (staff may override)
    - no overlap with other active leaves (DB exclusion)
    - **days used this enrollment-year** + requested days ≤ `max_days_per_year`. The enrollment-year is a rolling 365 days ending at `until`.
  - If over the limit and `over_limit = Block`: 422 `rule=enrollment.leave.max_days_per_year`, with `{used, remaining}`.
  - If over the limit and `EndAndRequireReregistration`: the UI warns "This will end the enrollment; the student must register again". On confirmation the enrollment ends with `ended_reason=leave_limit` on `from`.
  - Approval (staff, or automatically if the request came from staff) emits `enrollment_on_leave`, which scheduling uses to cancel those sessions with credit 0 and billing uses per `billing_during_leave`.
  - `fee > 0` adds a `leave_fee` line to the next invoice through an event.
  - The job `enrollment.leave_transitions` (daily 00:15) moves enrollments `active → on_leave` on `from` and `on_leave → active` on `until + 1`.
- **ENR-08. Make-up credits.**
  - Created from the `scheduling.session_cancelled.v1` participant credit (bps > 0), from no-show if `student_no_show = MakeupCredit`, and from closures if `MakeupCredit`.
  - `expires_on = source session date + expires_after_days`.
  - New credits beyond `max_outstanding` are not created; the event is logged with an `info` audit.
  - **Redeeming:** staff book a session of kind `makeup` with the participant and select credits totalling ≥ 10000 bps. They are marked `used` with `used_session_id`. Guardians can't self-book make-ups in v1.
  - The job `enrollment.expire_credits` runs daily.
- **ENR-09. Eligibility (optional).**
  - `GET /enrollments/{id}/eligibility` → `{status: eligible|not_eligible|insufficient_data|pending_verification, label, details: {tenure_months, attendance_bps, no_shows_in_window, sessions_in_window}}`.
  - Inputs come from the attendance port (marks). If attendance is disabled, the result is `insufficient_data`.
  - Rules (pure function):
    - `insufficient_data` if tenure < `min_tenure_months` or sessions in window < `min_sessions_for_data`.
    - `pending_verification` if any mark in the window is unconfirmed (attendance review pending).
    - `eligible` iff `attendance_bps >= min` and `no_shows <= max`.
- **ENR-10. Guardian portal.** Guardians see a **student switcher** (tabs on mobile) listing all linked students with status badges ("Frozen" from the billing port). Each student view shows the schedule, attendance marks, credits, leave and feedback. The guardian can't add students; they use "Register another child", which pre-fills the `/join` form with their contact and links on approval (v0 rule SRS-09 §4).
- **ENR-11. Import.** `POST /import/students` takes CSV/XLSX columns `student_name, birth_date, guardian_name, guardian_email, guardian_phone, offering_key, level_key, weekday, start_time, instructor_email, resource_name, starts_on`.
  - Step 1 uploads and returns a preview with per-row validation.
  - Step 2 commits in one transaction. It is all-or-nothing, with an error report CSV.
  - This is the main adoption path (DST-20).

## 2. API

| Method | Path | Perm |
|---|---|---|
| POST | `/public/registrations` (multipart) | none (ENR-03) |
| GET | `/registrations?status=` / GET `/registrations/{id}` | `enrollment.registration.read` |
| POST | `/registrations/{id}/approve` `{students:[{index, offering_id, level_id?, series_id \| new_series, starts_on}], link_guardian_id?, send_invite}` | `enrollment.registration.write` |
| POST | `/registrations/{id}/reject` `{reason, notify}` | same |
| POST | `/registrations/{id}/trial` `{student_index, starts_at, resource_id, instructor_id}` | same |
| GET | `/students` `?q&status&offering_id&payer_id` / GET `/students/{id}` | scoped |
| GET/POST | `/enrollments`, GET `/enrollments/{id}` | scoped / `enrollment.write` |
| POST | `/enrollments/{id}/activate` \| `/change` \| `/end {reason, on}` | `enrollment.write` |
| GET/POST | `/enrollments/{id}/leaves`, POST `/leaves/{id}/approve` \| `/reject` \| `/cancel` | payer (request own), staff |
| GET | `/enrollments/{id}/credits` | scoped |
| GET | `/enrollments/{id}/eligibility` | scoped |
| GET | `/me/students` | guardian |
| POST | `/import/students/preview` → `{rows:[{index, ok, errors}], summary}`; POST `/import/students/commit {upload_id}` | `enrollment.write` |

## 3. Permissions

| Permission | owner | admin | instructor | guardian | student |
|---|---|---|---|---|---|
| enrollment.registration.read/write | read | ✅ | — | — | — |
| enrollment.read | ✅ | ✅ | own students | linked | own |
| enrollment.write | — | ✅ | — | — | — |
| enrollment.leave.request | — | ✅ | — | linked | own (if payer) |
| enrollment.leave.decide | — | ✅ | — | — | — |

## 4. Ports implemented

- `EnrollmentStatus` (for scheduling): `is_on_leave(person, date)`.
- `BillableSource` (for billing): for a payer and period, returns tuition lines. For each active or on-leave enrollment of the payer's students in the period, it applies `billing_during_leave`, `first_cycle` proration and the price plan effective at the period start.
- `StudentDirectory` (for notify, reports): students and their payer.

## 5. Acceptance criteria

1. **Guardian required.** Given `required_under_age_years=18` and a 10-year-old student without a guardian, When enrolling, Then 422.
2. **Age block.** Given min age 84 months and a 6-year-old, Then 422 with `enforcement=Block`, and a warning with `Warn`.
3. **Approval transaction.** Given a registration with 2 children, When approved, Then 1 guardian, 2 students, 2 enrollments, 2 series memberships and an audit entry are created in one tx. If series capacity is full for child 2, Then the whole approval fails with 409, and nothing is created.
4. **Leave limit.** Given `max_days_per_year=30` with 20 days used, When requesting 15 days with Block, Then 422 with remaining=10. With `EndAndRequireReregistration` and confirmation, Then the enrollment ends with `leave_limit`.
5. **Credit from cancellation.** Given a cancellation event with credit 10000, Then 1 available credit expiring per policy. Duplicate events create no duplicate (unique key).
6. **Eligibility.** Given 3 months tenure, 20 sessions, 18 present, 1 absent and 1 excused (attendance 90%), and no_shows=1, Then eligible.
7. **Import.** Given a 100-row CSV with 1 invalid row, When previewing, Then 99 ok and 1 error. Commit is refused until there are 0 errors (all-or-nothing).
