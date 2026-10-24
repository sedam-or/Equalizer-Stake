# Module: notify (optional)

Purpose: deliver messages to people over in-app, email and WhatsApp, with templates per locale, routing by category, retries, fallback, circuit breaker, quiet hours and per-payer aggregation.
Policy: `NotifyPolicy` (04 §6.8). Tables: 06 §4.8.
Legacy reference: `v0-capstone:srs/05-notifications.md`, `backend/app/services/notification.py`, `whatsapp.py`, `email.py`.

## 1. Architecture

- **NTF-01. Request API (internal).** Other modules never send directly. They call `eq_core::ports::Notifier::request(tx, NotificationRequest)`. When notify is disabled, the no-op implementation writes nothing and logs `suppressed` at debug level.

```rust
pub struct NotificationRequest {
    pub recipient: PersonId,             // resolved to payer/guardian by caller when relevant
    pub category: Category,              // Critical | Standard
    pub template: &'static str,          // e.g. "billing.invoice_issued"
    pub params: serde_json::Value,       // template variables (money as {"$money":..}, dates ISO)
    pub dedupe_key: Option<String>,      // prevents duplicates (e.g. "reminder:{session}:60")
    pub aggregate_key: Option<String>,   // NTF-06 grouping, e.g. "session_reminder:{payer}:{date}"
    pub send_after: Option<Timestamp>,
}
```

- **NTF-02. Pipeline.**
  1. `request` inserts into `notifications` (pending), inside the caller's tx, with `ON CONFLICT (dedupe_key) DO NOTHING`.
  2. The dispatcher job `notify.dispatch` runs every 5 s. It picks pending rows with `send_after ≤ now` (`SKIP LOCKED`), renders them and delivers them per routing.
  3. Status ends as `sent` (≥ 1 channel delivered), `failed` (standard, all channels failed) or `critical_failure` (critical, all failed). A critical failure emits `notify.delivery_failed.v1` and creates an admin attention item (v0 rule).
- **NTF-03. Routing.** The channel order comes from `routing.critical` / `routing.standard`. For each channel in order:
  - Skip it (and record a `deliveries` row) if it is disabled or has no config, the recipient has no address on it, the breaker is open, or the recipient opted out (standard messages only).
  - Otherwise try `attempts_per_channel` times with `backoff_seconds[i]` between attempts. The waits are scheduled by re-enqueueing the job with `run_at`, never by sleeping in a worker.
  - Stop at the first success.
  - **In-app is always written** in addition to the routed channels, and needs no channel config.
- **NTF-04. Circuit breaker** (per tenant × channel, `breaker_states`):
  - closed → open after `failure_threshold` consecutive failures.
  - open → half_open after `cooldown_minutes`. One trial message: success → closed, failure → open again.
  - Admins can reset it from Settings → Channels.
  - The **platform default** SMTP (cloud only) has its own global breaker.
- **NTF-05. Quiet hours.** Standard messages whose local send time falls inside quiet hours are deferred to the quiet-hours end (`status=deferred`, `send_after` updated). Critical messages are never deferred.
- **NTF-06. Aggregation.** With `aggregate_per_payer`, requests with the same `aggregate_key` whose `send_after` falls within 2 minutes of each other are merged at dispatch into one message. They use the `*.digest` variant of the template, with a `items` list. Example: "Reminder: Ayu (Piano, 15:00) and Bima (Voice, 15:45) have lessons today."
- **NTF-07. Templates.**
  - Templates live in `crates/eq-notify/templates/{locale}/{template}.{channel}.{ext}`: `email.subject.txt`, `email.html` (MJML-compiled at build time into HTML, with an inline-styled layout), `email.txt`, `whatsapp.txt` and `inapp.txt`.
  - Rendering uses MiniJinja with custom filters `money`, `date`, `time`, `datetime` and `term(key)`. The `term` filter applies tenant terminology.
  - Missing locale → `en`. CI checks that every template exists in `en` and `id` for every channel it routes to (TST-09).
  - Tenant overrides: v1 has none. Planned for v1.x with `template_overrides` and a safe editor.
- **NTF-08. Channels.**
  - **Email:** SMTP via `lettre` (STARTTLS or TLS). The config is `{host, port, username, password, from_name, from_email, security}`. "Send test" verifies it.
  - **WhatsApp**, with two providers:
    - (a) `whatsapp_cloud`: the official Meta Cloud API. Config `{phone_number_id, access_token, template_namespace?}`. Business-initiated messages require pre-approved templates, so each WhatsApp template file has a header `# wa_template: <name>, lang: <code>, params: [...]`. Docs explain the template approval.
    - (b) `whatsapp_http`: a generic webhook for local gateways (Fonnte, Wablas and similar). Config `{url, method, headers, body_template}`, where `body_template` is a MiniJinja JSON with `{{to}}` and `{{text}}`, and a success status range. This keeps the v0 "unofficial gateway" use case vendor-neutral.
  - **In-app:** rows in `notifications` read through `/me/notifications`. A badge count is polled every 60 s, or on SPA focus.
- **NTF-09. Addresses.** Email comes from `persons.email` and phone from `persons.phone` (E.164), both decrypted at send time inside the job, never logged. Messages for minors go to their **primary guardian** (the caller passes the guardian as recipient; notify also defends by redirecting any student under the guardian age requirement).
- **NTF-10. Opt-out.** `notification_prefs(person_id, channel, standard_opt_out bool)` is managed in the profile. Critical messages ignore opt-out but still respect channel availability. Every email footer has an unsubscribe link for standard messages (signed token).

## 2. Template catalog (v1)

| Template | Category | Triggered by | Params |
|---|---|---|---|
| `identity.invite` | critical | user_invited | tenant_name, role, url, expires_at |
| `identity.password_reset` | critical | reset requested | url, expires_minutes |
| `identity.account_locked` (to admins) | critical | account_locked | person_name, until |
| `scheduling.session_scheduled` (to instructor) | standard | session_scheduled | offering, starts_at, resource |
| `scheduling.session_rescheduled` | critical if < 24 h, else standard | session_rescheduled | student/offering, old, new, reason |
| `scheduling.session_cancelled` | critical if < 24 h | session_cancelled | offering, starts_at, reason, credit |
| `scheduling.session_reminder` (+ `.digest`) | standard | reminders job | items[{student, offering, starts_at, resource}] |
| `enrollment.registration_received` (to applicant) | standard | registration_submitted | applicant_name, students |
| `enrollment.registration_new` (to admins) | standard | registration_submitted | applicant_name, count, url |
| `enrollment.registration_approved` / `_rejected` | critical | approve/reject | students, start dates / reason |
| `enrollment.leave_decided` | standard | leave approve/reject | student, dates, decision |
| `billing.invoice_issued` | critical | invoice_issued | number, total, due_at, reference, instructions, url |
| `billing.reminder` | critical | reminders job | number, total_due, due_at, days |
| `billing.invoice_overdue` | critical | invoice_overdue | number, total_due, late_fee |
| `billing.payer_frozen` / `_unfrozen` | critical | freeze events | invoices, total_due |
| `billing.payment_verified` / `_rejected` | critical | verify/reject | amount, invoice, reason |
| `attendance.checkin_pending` (to admins) | standard | checkin_recorded(pending) | instructor, session, reasons, url |
| `attendance.review_overdue` (to owner) | critical | review_overdue | count, oldest_minutes |
| `attendance.checkin_reviewed` (to instructor) | standard | checkin_reviewed | session, decision, note |
| `rental.reservation_*` (requested, confirmed, rejected, expired, cancelled) | standard / critical (expired) | rental events | resource, range, price, status |
| `feedback.progress_note` | standard | note published | student, instructor, url |
| `notify.critical_failure` (to admins, in-app only) | critical | delivery_failed | recipient, template |

## 3. API

| Method | Path | Perm |
|---|---|---|
| GET | `/me/notifications?cursor&unread` / POST `/me/notifications/read {ids \| all}` | any |
| GET/PUT | `/me/notification-prefs` | any |
| GET | `/settings/channels` (secrets redacted: `"password": "••••"`) | `notify.channels.manage` |
| PUT | `/settings/channels/{email\|whatsapp}` | `notify.channels.manage` |
| POST | `/settings/channels/{channel}/test {to}` | `notify.channels.manage` |
| POST | `/settings/channels/{channel}/reset-breaker` | `notify.channels.manage` |
| GET | `/notifications?status&category&recipient_id&from&to` (delivery log) | `notify.log.read` |
| POST | `/notifications/{id}/resend` | `notify.log.read` + admin |
| GET | `/public/unsubscribe?token=` | none |

Permissions: `notify.channels.manage` goes to owner and admin. `notify.log.read` goes to owner and admin.

## 4. Acceptance criteria

1. **Fallback.** Given critical routing [email, whatsapp] and email failing (mock SMTP 5xx), Then 3 email attempts at +10/+20/+40 s, then WhatsApp is attempted. On success the status is `sent` and there are 4 delivery rows.
2. **Critical failure.** Given both channels fail, Then `critical_failure`, an event is emitted, and an admin in-app notification exists.
3. **Breaker.** Given 5 consecutive WhatsApp failures, Then the breaker is open, and the next message skips WhatsApp with `skipped_breaker`. After 30 min, one trial message is sent.
4. **Quiet hours.** Given quiet hours 21:00–07:00, When a standard message is due at 22:00 local, Then it is deferred to 07:00. A critical message is sent immediately.
5. **Aggregation.** Given 2 reminders for siblings with the same aggregate key, Then 1 WhatsApp message listing both.
6. **Dedupe.** The same `dedupe_key` requested twice → 1 notification row.
7. **No secrets leak.** GET /settings/channels never returns a password or token. Logs never contain an email address or phone number (a log-capture test).
