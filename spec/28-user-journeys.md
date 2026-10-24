# 28 — User Journeys

End-to-end journeys for every persona in spec 01 §3, plus the self-hoster, the data subject and the contributor. Each journey ties the person's goal to the screens (spec 08 §6), the flows (FLW-\*, spec 21), the states (STM-\*, spec 20) and the feedback (UX-\*, spec 27) that serve it, names the moments where things can go wrong and how the person recovers, and ends with the metric that proves the journey works. Journeys add no business rules; when a step needs one, it cites the rule.

ID prefix: **JRN-**. Each journey is also the script for one Playwright journey test (TST-07) and one usability check in the private beta (brand/GTM.md §2).

## How to read a journey

| Column | Meaning |
|---|---|
| Step | What the person does, in their words |
| Screen | Route or channel (WhatsApp, email) |
| System | What Ritma does, with the spec reference |
| Can go wrong → recovery | The failure moment, what the person sees (UX id) and how they continue |

"Feels" lines record the emotional peaks and pits from the beta interviews' point of view: the places where the product must be at its best.

## JRN-01. Owner: from nothing to the first lesson (self-host)

- **Persona:** Ibu Rina, owns a music school with 4 rooms and ~120 students, runs it on WhatsApp and Excel today. Her nephew does "computer things".
- **Goal:** a working schedule she trusts, today.
- **Trigger:** read about Ritma in a music-teacher group.
- **Success metric:** median time from first page load to first scheduled session < 10 min (PRD-07, GTM); setup completion ≥ 70%.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Nephew runs the one-liner on a cheap VPS | terminal | `install.sh` asks domain + email, writes compose and `.env`, starts app/Postgres/Caddy, prints the setup URL with token (DST-30) | No Docker → script offers to install it. No domain → http with a loud warning (`banner.insecure_url`) |
| 2 | Opens the link | `/setup?token=…` | Setup token checked (SET-11), step 1 form | Token missing or used → "Open the link printed by the installer" (spec 27 §8) |
| 3 | Types the school name, picks Indonesia, chooses "Music school", 4 rooms, creates her owner account | `/setup` step 1 | One tx: tenant, owner, location, rooms, catalog, modules, policies v1 (FLW-20, STM-11) | Weak password → inline `password_weak`. Network drop → step not committed, Next stays enabled |
| 4 | Leaves the suggested features on | step 2 | `PUT /setup/modules`, dependency dialogs (STM-95) | Turns off Enrollment → `dialog.core.module_disable_dependents` explains Billing goes too |
| 5 | Reads the rule summaries, changes the late fee, presses Finish | step 3 | Live preview + summary (POL-07), only changed modules saved as v2 | Due day before issue day → inline error with the fix |
| 6 | Lands on the dashboard with a welcome and a checklist | `/dashboard?welcome=1` | Setup checklist banner (05 §2.5) | — |
| 7 | Adds a teacher and a student, books the first lesson | `/people`, `/schedule` | Session created with holds (FLW-30, STM-31); `toast.scheduling.session_created` | Room taken → conflict panel with alternatives (FE-31); one tap books an alternative |

**Feels:** anxious at step 1 (will this break?), relieved at step 5 (the summary reads like her own rules), proud at step 7.

## JRN-02. Owner: change a rule later in three steps

- **Goal:** "From next month the late fee is Rp50.000."
- **Success metric:** rule change done in ≤ 3 screens (PRD-05); no complaint that old invoices changed (POL-05).

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Settings → Rules → Billing | `/settings/rules/billing` | Form from schema, current summary (FE-20/21) | Admin opens it → read-only banner `banner.rules_read_only` |
| 2 | Edits the fee, reads the new summary | same | Debounced preview (FLW-21) | Invalid value → inline error, Save disabled |
| 3 | Save → "Start of next month" | `dialog.core.rules_effective_date` | New version with `effective_from`; audit; `core.policy_changed.v1`; caches evicted (CACHE-12) | Someone saved meanwhile → `dialog.generic.version_conflict`, edits kept as a draft |

Already-issued invoices keep their snapshotted terms (POL-05). The page shows `banner.rules_scheduled` until the date.

## JRN-03. Admin: a normal morning at the front desk

- **Persona:** Mas Dimas, front desk, desktop, compact density.
- **Goal:** clear today's surprises before the 14:30 rush.
- **Success metric:** attention list emptied by noon; schedule save p95 ≤ 1 s (PRD-10).

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Opens the dashboard | `/dashboard` | Widgets load in parallel with skeletons (RPT-03, UX §3.6) | One widget fails → widget-level `ErrorState`, others still show |
| 2 | Works the "Needs attention" list | dashboard → item | Materialisation conflicts, pending check-ins, payment checks, manual references (RPT-01, STM-14) | — |
| 3 | Resolves a materialisation conflict by moving a lesson | `/schedule` | Reschedule with `If-Match` (SCH-12); `toast.scheduling.session_rescheduled` | Concurrent edit → version conflict dialog |
| 4 | Verifies three transfer payments | `/billing/payments` | Reference match help; verify → invoice paid (FLW-42, STM-51/50) | Possible duplicate → `toast.billing.payment_duplicate_warning`; wrong amount → reject with reason |
| 5 | Approves a registration for two siblings | `/registrations/:id` | One tx: guardian, students, enrollments, series members, invites (FLW-40, STM-40/41) | Slot full for child 2 → 409, nothing created; picks another slot |
| 6 | A teacher calls in sick: cancels her lessons with credit | `/schedule` → `dialog.scheduling.cancel_session` | Credit by policy (SCH-10), parents notified (FLW-32) | — |

**Feels:** the morning used to start with 40 WhatsApp messages; now with one list.

## JRN-04. Admin: month-end billing

- **Goal:** every family gets the right invoice on the 5th, without typing.
- **Success metric:** 500 payers issued in ≤ 60 s (BIL-02); 0 duplicate invoices.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | (Optional) previews the cycle | `dialog.billing.run_cycle` | Dry run shows count and total (`/billing/run-cycle`) | Price missing → checklist item "Set your prices" |
| 2 | On the 5th at 00:05 the cycle runs by itself | — | Per-payer tx, dedupe key, invoices issued, WhatsApp/email sent (FLW-41, STM-50, FLW-52) | One payer fails → others continue; failure in the attention list; rerun is idempotent |
| 3 | Reads the summary | `toast.billing.cycle_done` / attention | Run summary `{issued, skipped_frozen, failed}` | Reference collisions → `needs_manual_reference` items to fill |
| 4 | On due day + 1, overdue invoices get the late fee and payers freeze | — | Snapshotted terms (FLW-43, STM-52) | Freeze applied by mistake → owner override unfreeze with reason (BIL-08) |
| 5 | A frozen parent pays; the admin verifies; booking reopens | `/billing/payments` | Paid → auto unfreeze when all overdue paid (STM-52) | `AdminManual` → `dialog.billing.unfreeze`, refused with `unpaid_overdue` until paid |

## JRN-05. Instructor: a teaching day

- **Persona:** Kak Sinta, piano teacher, phone only, comfortable density.
- **Goal:** know where to be, prove she was there, record who came.
- **Success metric:** check-in in ≤ 30 s from opening the page; ≥ 95% check-ins `verified` without review once GPS is set.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Gets a reminder on WhatsApp | WhatsApp | `scheduling.reminders` → notify (NTF-02) | Channel down → fallback to email, then in-app (NTF-03) |
| 2 | Opens "Today" and taps Check in | `/checkin` | Window check, camera, GPS, on-device face ratio (STM-96, ATT-02/06) | Camera blocked → `dialog.attendance.camera_denied`; outside window → `banner.checkin_window` |
| 3 | Submits | same | Evaluation → verified or pending_review (FLW-50, STM-55) | No signal → `toast.attendance.checkin_queued`, sent later (ATT-07), `toast.attendance.checkin_synced` |
| 4 | After the lesson marks attendance | `/sessions/:id` | Marks saved optimistically (FE-33); absent → no-show event (ATT-11) | After the lock window → `marks_locked`, asks the admin |
| 5 | Writes a progress note | `/feedback` | Published or sent for review per policy (FBK-01) | — |

## JRN-06. Guardian: register two children

- **Persona:** Ibu Sari, two children (8 and 11), phone, comfortable density.
- **Goal:** sign both up without visiting the school.
- **Success metric:** registration form completion ≥ 80% of starts; approval within 2 working days.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Opens the registration link the school shared | `/join` | Public catalog, policy-driven fields (ENR-03) | Form closed → `banner.registration_closed` |
| 2 | Adds two students, preferred slots, consent, optional proof | same | Honeypot, rate limits, encrypted applicant data | 429 → "Too many attempts, try later"; upload too big → inline file error |
| 3 | Submits | same | `registration_submitted`, confirmation message to her contact (FLW-40) | Network drop → nothing sent, form data kept in the page |
| 4 | Gets approval and a portal invite | WhatsApp / email → `/invite/:token` | Sets a password, logs in (FLW-12, STM-21) | Link expired → "Ask for a new invitation" (`invite_expired`) |
| 5 | Sees both children with a switcher | `/dashboard`, `/students/:id` | Guardian scope (AUTH §7, ENR-10) | — |

## JRN-07. Guardian: pay the monthly invoice

- **Goal:** pay the right amount, once, and be sure the school knows.
- **Success metric:** ≥ 70% of invoices paid by due date in beta schools; payment verification median < 1 working day.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Receives the invoice message with amount, code and due date | WhatsApp / email | `invoice_issued` → notify critical (BIL-04) | — |
| 2 | Transfers from her bank app using the code | bank | — | — |
| 3 | Taps "I've paid" and uploads the receipt | `/billing/invoices/:id` | Payment `submitted` with `Idempotency-Key` (BIL-07, STM-51) | Below the minimum first payment → inline 422; double tap → replayed response, one payment |
| 4 | Gets "Payment verified" | WhatsApp / in-app | Verified → paid → receipt (BIL-11) | Rejected → message with the reason and a way to resubmit |
| — | If she pays late | app | Frozen → `banner.payer_frozen`, booking actions disabled with a reason (FE-32) | Pays → auto unfreeze (or admin) |

## JRN-08. Guardian: leave and cancellations

- **Goal:** pause during a family trip; cancel one lesson without losing it.
- **Success metric:** leave requests decided within 2 working days; make-up credits used before expiry ≥ 60%.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Requests leave for 2 weeks | `/students/:id` → leave | Validated with snapshotted leave terms (ENR-07, STM-42) | Over the limit → 422 with remaining days, or `dialog.enrollment.leave_over_limit` |
| 2 | Leave approved | notification | Sessions in range cancelled with credit 0, billing per `billing_during_leave` | — |
| 3 | Cancels one lesson 30 h ahead | `/sessions/:id` → `dialog.scheduling.cancel_session` (participant scope) | Credit tier by hours before (SCH-10, FLW-32), make-up credit (STM-43) | Policy forbids payer cancel → action hidden; too late → credit 0 explained in the dialog |
| 4 | The admin books the make-up | admin | Credits consumed (ENR-08) | Credit expired → shown as expired with date |

## JRN-09. Member or guest: rent a studio

- **Persona:** a band of adult students, or a walk-in guest.
- **Goal:** book Studio A for two hours on Friday evening.
- **Success metric:** booking completed in ≤ 2 min; expired holds < 20% of deposit bookings.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Picks a room, date and duration; sees free starts and price | `/rental` or `/book` | Availability (SCH-14), quote with peak slices (RNT-03) | No rate card → admin checklist; guest sees "not available" |
| 2 | Books | same | Holds written at creation; status by approval/payment mode (RNT-04, STM-60) | Taken meanwhile → 409 with alternatives; frozen booker → 422 explained |
| 3 | Pays the deposit before the hold expires | invoice | `invoice_paid` → confirmed (RNT-06, FLW-51) | Not paid in time → expired, slot released, invoice cancelled (`rental.reservation_expired`) |
| 3g | Guest: confirms the email link within 15 min | email | Magic link verifies and creates the reservation (RNT-10) | Link late → hold gone, start again |
| 4 | Cancels if plans change | `/rental/reservations` | Refund by snapshotted terms (RNT-08) | After the deadline → `cancel_window_passed`, contact staff |

## JRN-10. Self-hoster or IT partner: run it for years

- **Persona:** the nephew, or a local IT shop serving several schools.
- **Goal:** install once, keep it safe, update without fear, move to the cloud if wanted.
- **Success metric:** zero manual migration steps per release (OPS-10); restore tested in CI (TST-13).

| # | Step | Channel | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Installs (JRN-01 step 1) or deploys a 1-click template | terminal / hosting UI | Same image everywhere (CTR-01) | — |
| 2 | Checks health | `ritma doctor`, `/readyz` | DB, RLS, secrets, clock, disk (OPS-02) | ⚠️/❌ lines say how to fix |
| 3 | Backups run daily; downloads one | Settings → Backups | Encrypted backup (OPS-05) | Lost secret key → backups unreadable; the UI warned at setup (SEC-31) |
| 4 | Updates when the banner says so | `docker compose pull && up -d` | Migrations on start, downgrade guard (OPS-04/10) | Bad release → roll back the image tag; forward-only migrations are compatible for one release |
| 5 | Verifies the image | `cosign verify` | Keyless signature, provenance (CTR-20) | Verification fails → don't run it; report via SECURITY.md |
| 6 | Moves the school to the cloud | Export / import | Export everything + import (DST, T-702/T-906) | — |

## JRN-11. Data subject: see or delete my data

- **Goal:** a parent wants a copy of their data, then asks for deletion when leaving the school.
- **Success metric:** export ready ≤ 1 h; deletion completed within `deletion_sla_business_days`.

| # | Step | Screen | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Requests an export | `/privacy` | Export job, signed link for 24 h (AUD-10, STM-72) | Link expired → request again |
| 2 | Requests deletion | `/privacy` → `dialog.privacy.request_deletion` | Request with due date (AUD-11, STM-71) | Open invoices → blocked with the reason |
| 3 | Admin verifies identity; erasure runs | admin | PII nulled, files deleted, invoices kept anonymised (FLW-53) | Missed SLA → attention item 3 business days before due |

## JRN-12. Contributor: add a template for my kind of school

- **Persona:** a swimming coach who knows YAML a little.
- **Goal:** "Ritma should work for swimming schools."
- **Success metric:** ≥ 5 merged presets and translations by external contributors 6 months after v1.0 (GTM).

| # | Step | Where | System | Can go wrong → recovery |
|---|---|---|---|---|
| 1 | Finds "Add a preset" in CONTRIBUTING | GitHub | — | — |
| 2 | Copies `presets/blank.yaml`, fills modules, terminology, rules and catalog | editor | Preset schema (SET-01, POL-09) | Unknown key → `cargo test -p eq-core presets` explains it |
| 3 | Opens a PR from a branch `docs/<issue>-swimming-preset` | GitHub | CI validates the preset (TST-08), PR title check (COMMIT_CONVENTION), DCO | Title rejected → the CI message shows the expected format |
| 4 | Maintainer reviews and squash-merges | GitHub | BRANCHING.md | — |

## Coverage

| Persona (spec 01 §3) | Journeys |
|---|---|
| owner | JRN-01, JRN-02, JRN-04 (overrides) |
| admin | JRN-03, JRN-04 |
| instructor | JRN-05 |
| guardian | JRN-06, JRN-07, JRN-08, JRN-11 |
| student | JRN-07 (if payer), JRN-09 |
| guest | JRN-09 |
| self-hoster / IT partner | JRN-10 |
| contributor | JRN-12 |
