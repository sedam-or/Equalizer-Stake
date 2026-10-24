# 27 — UX States and Feedback

This document says **what the person sees** in every state and after every action: which feedback pattern is used (inline, toast, banner, dialog, page state), the exact catalogue of toasts, dialogs and banners, and the state matrix of every screen. It builds on the tokens and components in spec 09, the client state machines STM-90..96 in spec 20, the error UX in ERR-61/62, and the routes in spec 08 §6. It adds no business rules.

ID prefix: **UX-**. Copy follows BRAND §6 and spec 09 §6. Visual reference for every pattern: the `/__ds` demo route (spec 09 §4).

## 1. Principles

- **UX-01. Feedback matches the consequence.** The bigger the effect, the more prominent the pattern: a field mistake is inline, a saved change is a toast, an ongoing condition is a banner, a decision that can't be undone is a dialog.
- **UX-02. One message per event.** An event produces exactly one pattern. A failed form submit never shows both a toast and inline errors for the same problem.
- **UX-03. Say what happened and what to do next.** Every error names the next step ("Try 15:50 or Room 3"). Every empty state has one primary action.
- **UX-04. Never colour alone.** Every tone pairs colour with an icon and text (WCAG 1.4.1). Status badges, toasts and banners follow the tone table in §3.
- **UX-05. The server decides, the client explains.** The UI never claims success before the server confirms, except the two optimistic cases in FE-33 (marks, notification reads).
- **UX-06. Confirm what can't be undone; don't confirm what can.** Ritma has no generic undo, so irreversible or costly actions get a confirm dialog (§6). Reversible, low-cost actions (mark read, toggle a filter, save a draft) never ask.
- **UX-07. Nothing disappears on its own if it needs action.** A toast with an action button stays until dismissed (WCAG 2.2.1). Errors that block work are never transient.

## 2. Which pattern to use

| Situation | Pattern | Example |
|---|---|---|
| A field value is wrong or missing | **Inline field error** under the field, focus moved to the first invalid field | "Due day must be on or after the issue day" |
| The whole form failed for a reason not tied to one field | **Form alert** (danger `Alert` at the top of the form) | `policy_violation` without a pointer, `idempotency_mismatch` |
| A booking conflicts | **Conflict panel** in the form (FE-31) | `slot_taken` with alternative chips |
| A mutation succeeded | **Toast** (success) | "Invoice sent to Mrs. Sari" |
| A background or transient problem | **Toast** (error or info) | `network`, `rate_limited`, `unavailable` |
| An ongoing condition that affects the whole app or page | **Banner** (app-level under the top bar, or page-level under the page header) | Payer frozen, offline, insecure URL |
| A decision with irreversible or costly effects | **Dialog** (confirm or destructive confirm) | Cancel a session, reset setup |
| The page can't show its content | **Full-page state** (`ErrorState`, `EmptyState`) | `not_found`, `module_disabled`, empty list |
| A long task in context | **Sheet** (bottom on mobile, right on desktop) | New session, record payment |
| Something needs staff attention later | **Attention item** (dashboard list, no toast) | Materialisation conflict, pending check-in |
| Something happened for this person while away | **In-app notification** (inbox + bell badge) | "Your leave was approved" |

Never use a toast for validation errors, a dialog for success, or a banner for one-off events.

## 3. Patterns (anatomy and behaviour)

### 3.1 Tones

| Tone | Icon (lucide) | Surface / text / border tokens | Used for |
|---|---|---|---|
| `success` | `circle-check` | `--color-success-subtle` / `--color-text` / `--color-success` | Completed actions |
| `info` | `info` | `--color-info-subtle` / `--color-text` / `--color-info` | Neutral news, tips |
| `warning` | `triangle-alert` | `--color-warning-subtle` / `--color-text` / `--color-warning` | Conditions that limit actions, soon-to-expire items |
| `danger` | `octagon-alert` | `--color-danger-subtle` / `--color-text` / `--color-danger` | Failures, blocked states, destructive actions |
| `neutral` | context icon | `--color-surface-sunken` / `--color-text` / `--color-border` | Offline, read-only |

### 3.2 Toast

- **Placement:** bottom-centre on mobile (above the bottom nav), bottom-right on ≥ md. `--z-toast`.
- **Anatomy:** tone icon · title (one line, ≤ 60 characters) · optional description (≤ 2 lines) · optional single action button · close button (`aria-label` "Dismiss").
- **Duration:** success and info 5 s; error 10 s; **any toast with an action stays until dismissed** (UX-07). The timer pauses on hover, focus and while the page is hidden.
- **Stacking:** at most 3 visible, newest at the bottom. A toast with the same `id` (§5) replaces the existing one instead of stacking (for example repeated `network`).
- **Error toasts** include the `request_id` as a "Copy error ID" action when the code is `internal` or `unavailable` (ERR-61).
- **Accessibility:** success and info use `role="status"` (`aria-live="polite"`); error uses `role="alert"`. Toasts never take focus. Everything a toast offers is also reachable elsewhere (UX-07).
- **Library:** the shadcn `Sonner` toaster, wrapped by `useToast()` in `web/src/lib/toast.ts`, which only accepts ids from the catalogue.

### 3.3 Dialog

- **Types:** `confirm` (neutral), `destructive` (danger primary button), `type-to-confirm` (the person types a word shown in the dialog before the danger button is enabled), `alert` (one button), `form` (short input such as a reason).
- **Anatomy:** title (a question or a statement) · body (consequence in plain words, with the affected item named) · optional input (reason, note) · buttons. The primary button names the action ("Cancel lesson"), never "OK" or "Yes".
- **Buttons:** on ≥ sm right-aligned, secondary then primary. On mobile, full-width and stacked with the primary at the bottom, closest to the thumb.
- **Focus:** opens with focus on the first input, else on the **secondary** (safe) button for `destructive` and `type-to-confirm`, else on the primary. Focus is trapped; on close, it returns to the element that opened the dialog.
- **Closing:** Esc and the scrim close the dialog unless an input is dirty (then a "Discard changes?" confirm appears). `type-to-confirm` dialogs can't be closed by the scrim.
- **Pending:** while the request runs, the primary shows a spinner, both buttons are disabled, and Esc is ignored. On error, the dialog stays open and shows the error inline in the body (never a toast behind the scrim).
- **Library:** shadcn `AlertDialog` (confirm types) and `Dialog` (form), Radix underneath.

### 3.4 Banner

- **App-level** banners sit under the top bar on every page, full width, max 2 at once, ordered by severity. **Page-level** banners sit under the page header.
- **Anatomy:** tone icon · one sentence · optional link or action · close button only when the condition is informational (§7 says which).
- **Accessibility:** `role="status"` for warning/info, `role="alert"` only when it appears as the result of the person's own action. Banners never move focus.

### 3.5 Inline field error and form alert

- The field gets `aria-invalid="true"` and `--color-danger` border; the message sits below the field with an icon and is linked by `aria-describedby` (FE-36).
- Messages come from `errors.field.<code>` with params (ERR-31), for example "Must be 28 or less".
- On submit with errors: all errors appear at once (ERR-31), focus moves to the first invalid field, and a screen-reader announcement says "2 fields need attention".
- Client-side validation (VAL-30) runs on blur and on submit, not on every keystroke, except for live formats (money, time) that show inline hints.

### 3.6 Full-page states

| State | Component | Content |
|---|---|---|
| `pending` | `Skeleton` in the final layout | No spinner-only pages. Skeletons appear after 150 ms to avoid flashing |
| `empty` | `EmptyState` | Line illustration, title, one sentence on why it's empty, primary action |
| `no_results` | `EmptyState` (compact) | "No results for these filters" + "Clear filters" (D-54) |
| `error` | `ErrorState` | Tone danger, what happened, "Try again" (refetch), "Copy error ID" for `internal` |
| `forbidden` | `ErrorState` | "You don't have access to this page" + "Go to dashboard". Shows the role switcher hint if another role has access |
| `not_found` | `ErrorState` | "We couldn't find this" + back link. Used for out-of-scope items too (AUTH-04) |
| `module_disabled` | `ErrorState` | "{Module} is turned off" + owner: "Turn it on in Settings → Features"; others: "Ask the owner" |
| `offline` | page content + offline banner | Read-only cached data where available (FE-35), else `ErrorState` "You're offline" |

### 3.7 Loading and progress inside a page

- Background refetch: a 2 px indeterminate bar at the top of the content area (STM-91 `success`), never a skeleton.
- Buttons: spinner replaces the icon, width locked, `aria-busy="true"` (spec 09 §4).
- Long jobs started by the person (import commit, cycle run, export): a progress card in the page with status text, never a modal spinner.

## 4. Copy and keys

- Key families: `toast.<module>.<event>`, `dialog.<module>.<name>.{title,body,confirm,cancel}`, `banner.<name>`, `empty.<screen>.{title,body,action}`, `state.<name>.{title,body,action}`, `errors.<code>`, `errors.field.<code>`.
- Each catalogue entry below gives the `en` copy as the source. `id` copy follows BRAND §6 (polite *Anda*; *Bapak/Ibu* only in parent messages). `pnpm i18n:check` fails on a missing key in either locale (TST-09).
- Copy uses terminology keys for role and entity names (`{term.instructor}`, FE-37) and formatters for money, dates and times.
- Titles are sentence case, no trailing period; bodies are full sentences. No exclamation marks in errors (BRAND §6).

## 5. Toast catalogue

Every toast the app can show. A toast not in this table is a review blocker; add a row in the same PR. "—" in Action means no action button.

| ID | Trigger | Tone | `en` title (description) | Action |
|---|---|---|---|---|
| **Generic** | | | | |
| `toast.generic.saved` | Any successful save without a more specific toast | success | Changes saved | — |
| `toast.generic.network` | `network` on a mutation | error | You're offline or the server can't be reached (Your changes weren't saved.) | Try again |
| `toast.generic.network_query` | `network` on a query after retries | error | Can't load the latest data | Try again |
| `toast.generic.rate_limited` | `rate_limited` | error | Too many requests (Wait {seconds} seconds and try again.) | — |
| `toast.generic.unavailable` | `unavailable` | error | The service is busy (Please try again in a moment.) | Copy error ID |
| `toast.generic.internal` | `internal` on a mutation | error | Something went wrong (Nothing was changed.) | Copy error ID |
| `toast.generic.copied` | Copy button | info | Copied | — |
| `toast.generic.back_online` | Connection restored after the offline banner | success | You're back online | — |
| **Auth and profile** | | | | |
| `toast.identity.signed_out` | Logout (shown on the login page) | info | You've been signed out | — |
| `toast.identity.session_expired` | Failed refresh → login page | info | Your session ended. Please sign in again | — |
| `toast.identity.reset_sent` | `POST /auth/forgot` 202 | info | If that email is registered, a reset link is on its way | — |
| `toast.identity.password_changed` | Password change | success | Password changed (You've been signed out on other devices.) | — |
| `toast.identity.role_switched` | Role switch | info | You're now acting as {role} | — |
| `toast.identity.totp_enabled` / `_disabled` | TOTP change | success | Two-step sign-in turned on / off | — |
| `toast.identity.device_revoked` | `DELETE /me/sessions/{id}` | success | Device signed out | — |
| `toast.identity.invite_sent` | Invite by email/WhatsApp | success | Invitation sent to {name} | — |
| `toast.identity.invite_link_copied` | Invite by link | success | Invitation link copied (It works for {days} days.) | — |
| `toast.identity.person_created` | `POST /people` | success | {name} added | View |
| `toast.identity.person_deactivated` / `_reactivated` | | success | {name} deactivated / reactivated | — |
| **Setup and settings** | | | | |
| `toast.core.rules_saved` | Policy saved, effective now | success | Rules saved | — |
| `toast.core.rules_scheduled` | Policy saved with a future date | success | Rules scheduled for {date} | — |
| `toast.core.module_on` / `_off` | Module toggled | success | {module} turned on / off | — |
| `toast.core.branding_saved` | Branding saved | success | Branding updated | — |
| `toast.core.branding_adjusted` | DS-10 darkened the colour | info | We darkened your colour slightly for readability | — |
| `toast.core.draft_restored` | Draft restored | info | Unsaved changes restored | Discard |
| `toast.core.attention_resolved` | Attention item resolved | success | Marked as resolved | — |
| **Scheduling** | | | | |
| `toast.scheduling.session_created` | `POST /sessions` 201 | success | Lesson booked ({offering}, {date} {time}, {resource}) | View |
| `toast.scheduling.session_created_below_min` | 201 with `below_min_capacity` | warning | Lesson booked, but below the minimum group size | View |
| `toast.scheduling.session_rescheduled` | Reschedule | success | Lesson moved to {date} {time} | — |
| `toast.scheduling.session_cancelled` | Cancel | success | Lesson cancelled ({credit} make-up credit) | — |
| `toast.scheduling.participant_added` / `_removed` | | success | {name} added to / removed from the lesson | — |
| `toast.scheduling.series_created` | Series created | success | Weekly slot created ({count} lessons scheduled) | View |
| `toast.scheduling.series_ended` | Series ended | success | Weekly slot ends on {date} | — |
| `toast.scheduling.closure_added` | Closure saved | success | Closed on {range} ({count} lessons affected) | Review |
| `toast.scheduling.closures_imported` | CSV import | success | {count} closures imported | — |
| `toast.scheduling.session_reverted` | Owner revert | success | Lesson restored | — |
| **Enrollment** | | | | |
| `toast.enrollment.registration_sent` | `/join` submit 202 | success | Registration sent (We'll contact you at {contact}.) | — |
| `toast.enrollment.registration_approved` | Approve | success | Registration approved ({count} students enrolled) | View |
| `toast.enrollment.registration_rejected` | Reject | success | Registration declined | — |
| `toast.enrollment.trial_booked` | Trial booked | success | Trial lesson booked for {name} | — |
| `toast.enrollment.enrollment_changed` | Change level/series/plan | success | Changes apply from {date} | — |
| `toast.enrollment.enrollment_ended` | End | success | Enrollment ends on {date} | — |
| `toast.enrollment.leave_requested` | Leave request | success | Leave requested ({days} days) | — |
| `toast.enrollment.leave_approved` / `_rejected` / `_cancelled` | Decision | success | Leave approved / declined / cancelled | — |
| `toast.enrollment.import_committed` | Import commit | success | {count} students imported | View |
| **Billing** | | | | |
| `toast.billing.invoice_issued` | Issue | success | Invoice {number} sent to {payer} | View |
| `toast.billing.invoice_cancelled` | Cancel | success | Invoice {number} cancelled | — |
| `toast.billing.line_added` | Adjustment/credit line | success | Invoice updated (New total {total}.) | — |
| `toast.billing.reference_set` | Manual reference | success | Payment code set | — |
| `toast.billing.payment_recorded` | Staff records payment | success | Payment of {amount} recorded | — |
| `toast.billing.payment_submitted` | Payer "I've paid" | success | Thanks, we'll confirm your payment soon | — |
| `toast.billing.payment_verified` / `_rejected` | Verify queue | success | Payment verified / rejected | — |
| `toast.billing.payment_duplicate_warning` | Verify with a possible duplicate | warning | This may be a duplicate payment (Check the reference and date.) | — |
| `toast.billing.payer_unfrozen` | Unfreeze | success | {payer} can book again | — |
| `toast.billing.cycle_started` | Manual cycle run | info | Creating invoices for {period} | — |
| `toast.billing.cycle_done` | Cycle finished | success | {issued} invoices sent ({skipped} skipped) | Review |
| **Attendance** | | | | |
| `toast.attendance.checked_in` | Check-in `verified` | success | Checked in for {offering} at {time} | — |
| `toast.attendance.checkin_pending` | Check-in `pending_review` | info | Check-in sent for review (An admin will confirm it.) | — |
| `toast.attendance.checkin_queued` | Offline queue | info | Saved offline. It will be sent automatically | — |
| `toast.attendance.checkin_synced` | Queue sent | success | Offline check-in sent | — |
| `toast.attendance.checkin_reviewed` | Review | success | Check-in approved / rejected | — |
| `toast.attendance.marks_saved` | Marks | success | Attendance saved | — |
| **Rental** | | | | |
| `toast.rental.reserved` | Confirmed | success | {resource} booked for {range} | View |
| `toast.rental.requested` | Needs approval | info | Request sent (We'll hold the slot until {deadline}.) | — |
| `toast.rental.pending_payment` | Deposit needed | warning | Pay {amount} by {deadline} to confirm | Pay |
| `toast.rental.approved` / `_rejected` | Admin decision | success | Reservation approved / declined | — |
| `toast.rental.cancelled` | Cancel | success | Reservation cancelled ({refund}% credit) | — |
| `toast.rental.guest_check_email` | Guest booking 202 | info | Check your email to confirm the booking within 15 minutes | — |
| `toast.rental.rates_saved` | Rate card | success | Prices saved, effective {date} | — |
| **Notify** | | | | |
| `toast.notify.channel_saved` | Channel config | success | {channel} settings saved | — |
| `toast.notify.test_sent` / `_failed` | Send test | success / error | Test message sent / failed ({reason}) | — |
| `toast.notify.breaker_reset` | Reset breaker | success | {channel} sending resumed | — |
| `toast.notify.resent` | Resend | success | Message queued again | — |
| `toast.notify.prefs_saved` | Preferences | success | Notification preferences saved | — |
| **Feedback, reports, privacy, files** | | | | |
| `toast.feedback.note_published` / `_submitted` / `_withdrawn` | Note | success | Note published / sent for review / withdrawn | — |
| `toast.feedback.rating_sent` | Rating | success | Thanks for your rating | — |
| `toast.feedback.comment_hidden` | Moderation | success | Comment hidden | — |
| `toast.reports.export_started` | CSV export | info | Preparing your export | — |
| `toast.privacy.export_requested` | Data export | info | We're preparing your data (You'll get a download link.) | — |
| `toast.privacy.deletion_requested` | Deletion request | success | Deletion request received (We'll respond by {date}.) | — |
| `toast.files.upload_failed` | Upload rejected (type/size) | error | This file can't be used ({reason}) | — |
| `toast.audit.chain_ok` / `_broken` | Verify | success / error | Audit log intact / Audit log mismatch at entry {seq} | — / View |

## 6. Dialog catalogue

| ID | Opened by | Type | `en` title / body | Primary · Secondary | Notes |
|---|---|---|---|---|---|
| `dialog.generic.discard_changes` | Closing a dirty form, sheet or dialog | confirm | Discard changes? / Your edits will be lost. | Discard · Keep editing | |
| `dialog.generic.version_conflict` | `version_conflict` (ERR-61) | alert | Someone changed this / Reload to see the latest version. Your edits are kept as a draft. | Reload | Draft kept (FE-30) |
| `dialog.generic.draft_restore` | Opening a form with a newer draft (FE-30) | confirm | Restore unsaved changes? / You have changes from {time}. | Restore · Discard | |
| `dialog.identity.logout_all` | Profile → Sign out everywhere | confirm | Sign out of all devices? / You'll need to sign in again here too. | Sign out everywhere · Cancel | |
| `dialog.identity.deactivate_person` | Person → Deactivate | destructive | Deactivate {name}? / They lose access right away. Their history is kept. | Deactivate · Cancel | Reason optional |
| `dialog.identity.remove_role` | Remove role | destructive | Remove the {role} role from {name}? | Remove role · Cancel | `last_owner` shown inline |
| `dialog.identity.disable_totp` | Profile → Turn off two-step | form | Turn off two-step sign-in? / Enter your password and a current code. | Turn off · Cancel | |
| `dialog.identity.recovery_codes` | TOTP confirmed | alert | Save your recovery codes / Each code works once. We won't show them again. | I've saved them | Download/copy buttons in the body |
| `dialog.core.module_enable_deps` | STM-95 | confirm | {module} needs {deps} / Turn on {deps} too? | Turn on both · Cancel | |
| `dialog.core.module_disable_dependents` | STM-95 | destructive | Turning off {module} also turns off {dependents} | Turn off all · Cancel | |
| `dialog.core.module_disable_with_data` | STM-95 | confirm | Turn off {module}? / Your data is kept. Turn it back on any time. | Turn off · Cancel | MOD-07 copy |
| `dialog.core.rules_effective_date` | Save rules | form | When should these rules apply? / Now · Start of next month · Pick a date | Save rules · Cancel | POL-05 |
| `dialog.core.reset_setup` | Setup → Reset | type-to-confirm (`RESET`) | Start setup again? / This deletes the school, rooms and rules you created. | Reset setup · Cancel | Allowed only without operational data (SET-12) |
| `dialog.core.currency_locked` | Changing currency after invoices exist | alert | Currency can't change / Invoices already use {currency}. | OK, got it | `core.currency.locked` |
| `dialog.scheduling.cancel_session` | Session → Cancel | form (destructive) | Cancel this lesson? / {offering}, {date} {time}. {participants} will be told. Credit: {credit}. | Cancel lesson · Keep lesson | Reason required per policy; scope session/participant |
| `dialog.scheduling.reschedule_confirm` | Drag-and-drop drop (spec 09 §4.1) | confirm | Move this lesson? / From {old} to {new}. People will be notified. | Move lesson · Cancel | |
| `dialog.scheduling.override_hours` | Booking outside opening hours (admin) | confirm | Book outside opening hours? / {time} is outside {hours}. This will be recorded. | Book anyway · Cancel | SCH-02 audit |
| `dialog.scheduling.end_series` | Series → End | form | End this weekly slot? / Lessons after {date} will be removed. | End slot · Cancel | |
| `dialog.scheduling.revert_session` | Owner revert | confirm | Restore this lesson? | Restore · Cancel | 409 inline if holds can't be re-acquired |
| `dialog.enrollment.reject_registration` | Registration → Decline | form (destructive) | Decline this registration? / Reason (sent to the applicant if "notify" is on). | Decline · Cancel | |
| `dialog.enrollment.end_enrollment` | End | form (destructive) | End {student}'s enrollment? / From {date}. Future lessons are removed. | End enrollment · Cancel | |
| `dialog.enrollment.leave_over_limit` | Leave over the limit with `EndAndRequireReregistration` (ENR-07) | destructive | This leave ends the enrollment / {student} has {remaining} leave days left. Continuing ends the enrollment on {date}, and they'll need to register again. | End and continue · Change dates | |
| `dialog.enrollment.import_commit` | Import → Commit | confirm | Import {count} students? / This creates people, enrollments and weekly slots in one go. | Import · Cancel | Disabled while errors > 0 |
| `dialog.billing.issue_invoice` | Draft → Issue | confirm | Send invoice {number}? / {payer} will get it by {channels}. | Send invoice · Cancel | |
| `dialog.billing.cancel_invoice` | Cancel | form (destructive) | Cancel invoice {number}? / Reason is required. | Cancel invoice · Keep invoice | Owner override variant says "This invoice has payments" |
| `dialog.billing.reject_payment` | Verify queue → Reject | form | Reject this payment? / Tell the payer why. | Reject payment · Cancel | |
| `dialog.billing.unfreeze` | Payer → Unfreeze | form | Let {payer} book again? | Unfreeze · Cancel | `unpaid_overdue` inline; owner override needs a reason |
| `dialog.billing.run_cycle` | Billing → Run cycle | confirm | Create invoices for {period}? / Preview: {count} invoices, total {total}. | Create invoices · Cancel | Shows dry-run result first |
| `dialog.attendance.review_checkin` | Review → Approve/Reject | form | Approve / Reject this check-in? / Note (optional). | Approve / Reject · Cancel | |
| `dialog.attendance.camera_denied` | STM-96 `camera_denied` | alert | Camera access is off / Allow the camera in your browser settings to check in. | OK, got it | Link to browser help |
| `dialog.rental.cancel_reservation` | Cancel | destructive | Cancel this booking? / You'll get {refund}% back as credit. | Cancel booking · Keep booking | After the deadline: staff-only, refund input |
| `dialog.rental.reject_reservation` | Admin reject | form | Decline this request? | Decline · Cancel | |
| `dialog.notify.channel_secret` | Changing channel credentials (SEC-23) | form | Confirm it's you / Enter your password to change these credentials. | Confirm · Cancel | Step-up |
| `dialog.feedback.withdraw_note` | Withdraw note | confirm | Withdraw this note? / Parents won't see it any more. | Withdraw · Cancel | |
| `dialog.privacy.request_deletion` | My data → Delete | type-to-confirm (`DELETE`) | Delete {name}'s personal data? / Invoices are kept without the name. This can't be undone once completed. | Request deletion · Cancel | `open_invoices` inline |
| `dialog.audit.export` | Audit → Export | confirm | Export audit log? / The export itself is recorded. | Export · Cancel | |

## 7. Banner catalogue

| ID | Level | Condition | Tone | `en` text | Action | Dismissible |
|---|---|---|---|---|---|---|
| `banner.offline` | app | Browser offline | neutral | You're offline. Showing data from {time} | — | No (clears when back online) |
| `banner.payer_frozen` | app | `/me.payer_state = frozen` (FE-32) | danger | Booking is paused until {invoices} are paid | View invoices | No |
| `banner.payer_restricted` | app | `restricted` | warning | Some actions are limited until {invoices} are paid | View invoices | No |
| `banner.insecure_url` | app | `meta/app.insecure_public_url` (owner/admin) | danger | This address isn't secure. Camera, location and app install won't work | How to fix | No |
| `banner.update_available` | app | Owner, new version (OPS-10) | info | Ritma {version} is available | What's new | Yes (per version) |
| `banner.tenant_suspended` | app | `tenant_suspended` (cloud) | danger | This school's account is paused. Contact support | Contact | No |
| `banner.plan_limit` | app | Cloud plan limit reached (spec 15) | warning | You've reached {limit} students on your plan | See plans | Yes (per day) |
| `banner.notify_in_app_only` | page (Settings, dashboard) | No email/WhatsApp configured | info | Messages only appear in the app. Connect email or WhatsApp | Connect | Yes |
| `banner.rules_read_only` | page (Rules) | Admin viewing rules | neutral | Only the owner can change rules | — | No |
| `banner.rules_scheduled` | page (Rules) | Future version pending | info | New rules start on {date} | View | No |
| `banner.module_disabled_data` | page (module screens reachable read-only) | Module off but data kept (MOD-08) | neutral | {module} is off. This data is read-only | Turn on (owner) | No |
| `banner.enrollment_disabled_sessions` | page (Schedule) | MOD-08 enrollment off | warning | Weekly slots are paused. Review future lessons | Review | Yes |
| `banner.setup_checklist` | page (dashboard) | Checklist items left (05 §2.5) | info | {count} steps left to finish setting up | Show | Collapsible |
| `banner.location_pin_missing` | page (attendance settings) | GPS on, no pin | warning | Set your studio location so location checks work | Set location | No |
| `banner.checkin_window` | page (`/checkin`) | Outside the window | info | Check-in opens at {time} | — | No |
| `banner.registration_closed` | page (`/join`) | Public form off | neutral | Online registration is closed. Contact the school | — | No |

## 8. Screen state matrix

Rows are the routes in spec 08 §6. Columns are the states every screen must handle. "✓" means the generic pattern from §3.6; text means a screen-specific variant (copy key under `empty.<screen>` or `state.<screen>`).

| Route | Empty | Error / not found | Forbidden / module off | Offline | Special states |
|---|---|---|---|---|---|
| `/setup` | — | ✓ step error inline, "Try again" | setup token missing → "Open the link printed by the installer" | ✓ disabled Next | resume at step (STM-94), preset locked after step 1 |
| `/login` etc. | — | inline errors; `locked` shows "Try again after {time}" | — | ✓ | `needs_totp` step, invite expired → "Ask for a new invitation" |
| `/join` | — | inline; 429 → "Too many attempts, try later" | registration closed banner | ✓ | submitted confirmation page |
| `/dashboard` | per widget: "Nothing scheduled today" | widget-level `ErrorState` (others still load) | widget hidden when its module is off | cached schedule | welcome (`?welcome=1`), setup checklist |
| `/schedule` | "No rooms yet" → Add room | ✓ | ✓ | read-only cached week | conflict panel, closure shading, now line, blocked participants lock |
| `/schedule/series` | "No weekly slots" → Create slot | ✓ | ✓ | ✓ | materialisation conflicts list |
| `/catalog` | "No courses yet" → Add course | ✓ | ✓ | ✓ | `price_needs_review` chip |
| `/resources`, `/closures` | "No rooms" / "No holidays" → Add / Import CSV | ✓ | ✓ | ✓ | import preview errors |
| `/people`, `/people/:id` | "No {term.students} yet" → Add / Import / Share registration link | ✓ (`not_found` for out of scope) | ✓ | ✓ | deactivated badge, PII hidden without `read_pii` |
| `/registrations` | "No registrations waiting" | ✓ | ✓ | ✓ | duplicate-match suggestion |
| `/students/:id` | per tab empty copy | ✓ | tabs hidden for disabled modules | ✓ | frozen badge, on-leave badge |
| `/import` | upload step | row errors table | ✓ | ✓ | commit disabled until 0 errors, progress card |
| `/billing/invoices` | "No invoices yet" (admin) / "You're all paid up" (payer) | ✓ | ✓ | ✓ | overdue (partially paid) badge, needs_manual_reference chip |
| `/billing/payments` | "No payments to check" | ✓ | ✓ | ✓ | duplicate warning, reference match highlight |
| `/billing/payers` | "No paused payers" | ✓ | ✓ | ✓ | — |
| `/checkin` | "No lessons today" | ✓ | ✓ | offline queue card with pending count | camera denied, GPS unavailable, window not open, already checked in |
| `/attendance/review` | "No check-ins to review" | ✓ | ✓ | ✓ | `not_configured` GPS flag, SLA age |
| `/sessions/:id` | — | ✓ | ✓ | read-only | marks locked (`marks_locked`), cancelled hatch |
| `/rental`, `/rental/reservations` | "No rooms for rent" (admin: set rate cards) / "No bookings" | ✓ | ✓ | ✓ | hold countdown, pending payment, frozen booker |
| `/feedback` | "No notes yet" | ✓ | ✓ | ✓ | pending review, anonymous ratings |
| `/reports` | per widget `insufficient` marker | widget-level | revenue hidden without `reports.revenue.read` | ✓ | print view |
| `/audit` | "No entries for this range" | ✓ | admin sees allowed categories only | ✓ | verify result |
| `/privacy` | "No requests" | ✓ | — | ✓ | request status timeline, `open_invoices` block |
| `/notifications` | "You're all caught up" | ✓ | ✓ | cached | unread/read (optimistic) |
| `/settings/*` | — | ✓ | admin read-only banner on rules | ✓ | effective-date dialog, history, branding preview |

## 9. Accessibility checklist for feedback

- Toast, banner and inline messages are announced (§3.2/3.4/3.5); nothing relies on colour alone (UX-04).
- Dialogs: `role="alertdialog"` for confirm types with `aria-labelledby` (title) and `aria-describedby` (body); focus order per §3.3; Esc behaviour per §3.3.
- Every action in a toast also exists in the page (UX-07); toasts never steal focus.
- Motion: toasts slide 8 px and fade over `--duration-base`; dialogs scale from 0.98 and fade; with `prefers-reduced-motion`, only the 100 ms opacity fade remains (spec 09 §2.4).
- Touch targets ≥ 44 px for toast actions, dialog buttons and banner links.
- Tested by Playwright + axe on the `/__ds` route for every pattern and tone, light and dark (TST-12).

## 10. Decisions taken in this document

| ID | Decision | Why |
|---|---|---|
| D-72 | Light `--color-warning-fg` is white, not gray-950 | gray-950 on amber-600 is 3.08:1 (fails AA); white is 6.4:1 (spec 09 §2.6) |
| D-73 | New token `--color-border-input` (gray-500 light and dark) for input borders | gray-300 on white is 1.48:1; WCAG 1.4.11 needs 3:1 for control boundaries |
| D-74 | The brand hex reference is `#005EB9` (computed from OKLCH 0.49 0.20 255) | BRAND §7 said ≈ `#2456E0`, which is a different hue |
| D-75 | Error toasts last 10 s; toasts with an action stay until dismissed | WCAG 2.2.1; spec 09 said 5 s for all |
| D-76 | Destructive dialogs focus the safe button; type-to-confirm only for reset setup and data deletion | Prevents accidental confirmation; keeps friction rare |
| D-77 | Skeletons appear after 150 ms | Avoids flashing on fast responses |
