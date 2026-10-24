# 21 — Data Flows

This document shows **how data moves** through Ritma for every important use case: which layer touches it, in which transaction, what is written, what is emitted, and where sensitive data is encrypted, decrypted or dropped.

- The generic request lifecycle is ARC-04; the flows below are its concrete instances.
- States and guards are `20-state-machines.md` (STM-\*). Inputs are checked per `22-validation-and-sanitization.md` (VAL-07 stages V0–V8). Formats are `23-contracts-and-canonical-forms.md`.
- Diagrams use the layer names of `17-clean-architecture.md`: **HTTP** (`http.rs` adapter + `eq-api` middleware), **SVC** (service), **DOM** (pure domain), **REPO** (`repo.rs`), **DB** (PostgreSQL), **OBX** (outbox dispatcher), **JOB** (job runner), **NTF** (notify), **EXT** (external provider).

Tags: **[D-nn]** a decision, listed in §8.

## 1. Generic flows

### FLW-01. Write request (every mutation)

```mermaid
sequenceDiagram
  autonumber
  participant C as Client
  participant H as HTTP
  participant S as SVC
  participant D as DOM
  participant R as REPO
  participant DB as DB
  C->>H: POST /x (JSON, Bearer, If-Match?, Idempotency-Key?)
  H->>H: V0 limits, rate limit · request_id · tenant · auth · module gate
  H->>H: V1–V4 ValidatedJson → Collector → Command
  H->>S: command + Actor
  S->>DB: BEGIN · SET LOCAL app.tenant_id, app.actor_id
  S->>R: idempotency key insert (if header) · load aggregate FOR UPDATE / version
  S->>D: transition(state, command, policy@at, now)
  D-->>S: new state + effects | typed error
  S->>R: writes (rows, holds)
  S->>R: audit::record (same tx)
  S->>R: outbox::emit (same tx) · Notifier::request (same tx)
  S->>DB: COMMIT
  DB-->>S: ok | constraint error (mapped per ERR-43)
  S-->>H: result
  H-->>C: 2xx DTO + ETag | problem+json
```

- Steps 6–13 are **one transaction**. A failure anywhere rolls back the business write, the audit entry, the events and the notifications together (ARC-05).
- Policy is read with `PolicyStore::get(at)` where `at` is the time of the thing checked (POL-05), or from the record's snapshot.
- The response is built from the committed state, not from the request.

### FLW-02. Read request (lists and details)

`HTTP` (V0–V2 query parsing, `Page` limits) → `SVC` (`Scope` from the `Actor`, IDN-10) → `REPO` (one SQL query with the scope predicate and keyset pagination on `(sort_key, id)`, SBD-41) → DTO mapping (S2 fields only with `read_pii`, decrypted in SVC through `eq_core::crypto`, each decryption of another person's PII audited per AUD-01, throttled per day) → `200 Page<T>`. Reads open a read-only transaction with the tenant set, so RLS applies.

### FLW-03. Event dispatch (outbox → handlers)

```mermaid
sequenceDiagram
  participant DB as DB (outbox)
  participant O as OBX
  participant Hn as Handler (module)
  DB-->>O: NOTIFY eq_outbox (or 2 s poll)
  O->>DB: claim ≤100 rows FOR UPDATE SKIP LOCKED (pending, due)
  loop each event in id order, per aggregate sequential
    O->>Hn: handle(envelope) in its own tx (tenant set)
    Hn->>DB: INSERT event_receipts(handler, event) — skip if exists
    Hn->>DB: state changes · audit · own outbox events
    Hn-->>O: Ok | Retryable | Permanent
  end
  O->>DB: dispatched_at | attempts+1, next_attempt_at | dead_letter_at
```

Handlers of disabled modules return `Ok` without work (ERR-52). A handler never calls another module directly; it re-reads through ports when it needs more than the payload (CAN-60).

### FLW-04. Jobs and cron

The cron planner (1/min) evaluates each `CronSpec` in the tenant TZ and inserts due jobs with a `dedupe_key` (`ON CONFLICT DO NOTHING`). Workers claim with `SKIP LOCKED`, set `app.tenant_id`, run the handler as `ActorCtx::system(kind)` through the **same services** as HTTP (so audit and events are identical), and record `done`/backoff/`dead` (STM-81). Policy-driven days ("issue on the 5th") are checked by a daily job against the policy in force, never hard-coded (ARC-09).

### FLW-05. Cache invalidation

Writes to policies and modules `NOTIFY eq_policy_changed` / `eq_modules_changed` with `{tenant_id, module}` **after commit** (a `NOTIFY` inside a tx is delivered at commit). Every process listens and evicts its `moka` entries; the 30 s TTL is the safety net (MOD-05, POL-04). Report widgets cache 60 s per tenant/range and are evicted by `core.module_toggled.v1` (RPT-03).

## 2. Identity flows

### FLW-10. Login

```mermaid
sequenceDiagram
  participant C as SPA
  participant H as HTTP
  participant S as Auth SVC
  participant DB as DB
  C->>H: POST /auth/login {email, password, totp?}
  H->>S: canonical Email (VAL-11), password as SecretString
  S->>DB: bidx = HMAC(email) → person + credentials FOR UPDATE
  alt unknown email
    S->>S: argon2 verify against a dummy hash (timing)
    S-->>C: 401 invalid_credentials
  else known
    S->>S: STM-23 lock check · argon2id verify
    S->>DB: counters (STM-23) · audit security.login.*
    S->>DB: refresh_sessions insert (family) · access JWT (sub, tid, role, sid, ver)
    S-->>C: 200 {access_token, person, roles} + Set-Cookie eq_rt
  end
```

The access token stays in memory (STM-90). `ver` binds it to `password_changed_at`.

### FLW-11. Refresh and rotation

`POST /auth/refresh` (cookie + `X-EQ-CSRF: 1` + Origin check) → hash the cookie → load the row `FOR UPDATE` → STM-22: active → rotate (new row, same family, new cookie, new JWT); rotated → revoke the family, audit `security.refresh_reuse`, 401; expired/revoked → 401 `session_expired`. The SPA runs this single-flight and broadcasts the new token to other tabs (FE-11).

### FLW-12. Invite and password reset

- **Invite:** staff `POST /people/{id}/invite` → invite row (token hash) + `identity.user_invited.v1` → notify renders `identity.invite` with a URL built from `RITMA_PUBLIC_URL` + `/invite/{token}` (CAN-80) → recipient `GET /auth/invites/{token}` (shows tenant, role) → `POST …/accept {password}` → one tx: password hash, invite accepted, membership active, refresh family → logged in.
- **Reset:** `POST /auth/forgot {email}` → always 202 after constant work; if the person exists and is under `reset_max_per_hour`, a reset row + `password_reset_requested.v1` → notify sends the link → `POST /auth/reset` → one tx: password, token used, **all** refresh sessions revoked, audit.

## 3. Setup and settings

### FLW-20. Setup wizard (self-host first run)

```mermaid
sequenceDiagram
  participant O as Operator
  participant Srv as ritma serve
  participant SPA as SPA /setup
  participant API as /setup API
  participant DB as DB
  Srv->>DB: no tenants → generate setup token
  Srv-->>O: log + RITMA_DATA_DIR/setup-token.txt (URL with token)
  O->>SPA: open /setup?token=…
  SPA->>API: GET /setup/status, /setup/presets, /setup/countries
  SPA->>API: POST /setup/school {…, owner, setup_token} (step 1)
  API->>DB: ONE tx: tenant · owner · location · resources · catalog · tenant_modules · policies v1 · audit
  API-->>SPA: tokens + next step 2
  SPA->>API: PUT /setup/modules (step 2, MOD-06 semantics)
  SPA->>API: POST /settings/policies/{m}/preview (debounced, step 3)
  SPA->>API: POST /setup/complete {policies}
  API->>DB: policies v2 for changed modules · setup_completed_at · audit
  API-->>SPA: done → /dashboard?welcome=1
```

### FLW-21. Policy change

Settings → Rules form (`PolicyForm`, ajv on change) → `POST /settings/policies/{m}/preview` (V0–V7, no write, returns errors + summary) → owner picks "Apply from" → `PUT /settings/policies/{m}` → insert version, audit before/after, `core.policy_changed.v1`, `NOTIFY` → caches evicted (FLW-05) → modules read the new version for future checks; **existing commitments keep their snapshot** (POL-05).

### FLW-22. Module toggle

Settings → Features (STM-95 dialog) → `PUT /settings/modules {enabled}` → closure checks → per change `on_enable`/`on_disable` + audit + `core.module_toggled.v1` in one tx → `NOTIFY eq_modules_changed` → `ModuleGate` cache evicted → the SPA refetches `/meta/modules` and rebuilds navigation (FE-15).

## 4. Scheduling flows

### FLW-30. Create a session (with conflict)

```mermaid
sequenceDiagram
  participant UI as ScheduleGrid / form
  participant H as HTTP
  participant S as Scheduling SVC
  participant D as DOM
  participant DB as DB
  UI->>H: POST /sessions/check (debounced dry run)
  H->>S: same pipeline, read-only
  S-->>UI: {ok, conflicts, alternatives, warnings}
  UI->>H: POST /sessions + Idempotency-Key
  H->>S: SessionCreate command
  S->>D: slot rules (duration, step, opening hours, closure, capacity) with policy@starts_at
  S->>DB: INSERT session · instructors · participants · calendar_holds
  alt exclusion_violation 23P01
    DB-->>S: error → ROLLBACK
    S->>DB: new read-only tx: overlapping holds for the window
    S->>D: alternatives::find (SCH-05)
    S-->>UI: 409 slot_taken {conflicts, alternatives}
    UI->>UI: conflict panel, click alternative → resubmit (FE-31)
  else ok
    S->>DB: audit · session_scheduled.v1 · COMMIT
    S-->>UI: 201 Session + ETag
  end
```

### FLW-31. Materialisation

Daily `scheduling.materialize` (and on demand after series/member changes) → per active series, per date up to `today + materialize_weeks_ahead`: skip if a session exists for `(series, date)`; apply closures (`on_closure`), the cycle cap (`sessions_per_cycle`, `extra_occurrence`), leave (EnrollmentStatus port) → insert session + holds in a **savepoint per occurrence**, so one conflict skips that date only and raises a `materialize_conflict` attention item with alternatives; `occurrence_skipped.v1` when a credit applies → `materialized_until` advanced.

### FLW-32. Cancellation → make-up credit

```mermaid
sequenceDiagram
  participant G as Guardian (portal)
  participant SCH as Scheduling
  participant OBX as Outbox
  participant ENR as Enrollment
  participant NTF as Notify
  G->>SCH: POST /sessions/{id}/cancel {scope: participant, participant_id}
  SCH->>SCH: scope check (own student) · credit_bps = tier(hours_before) (pure)
  SCH->>SCH: participant cancelled · hold deleted · audit · session_cancelled.v1
  OBX->>ENR: session_cancelled.v1
  ENR->>ENR: credit > 0 and enabled and < max_outstanding → makeup_credits (unique per source)
  ENR->>OBX: makeup_credit_granted.v1
  OBX->>NTF: session_cancelled.v1 / makeup_credit_granted.v1 → notifications (critical if < 24 h)
```

## 5. Enrollment and billing flows

### FLW-40. Public registration → approval

1. `/join` loads `GET /public/catalog` (S0) and the form rules.
2. `POST /public/registrations` (multipart): V0 (5/h per IP, captcha if configured) → honeypot → V3 kinds → payment proof via `files::accept` (sniff, re-encode, **encrypt**) → applicant JSON **encrypted** into `applicant_enc` (AAD = tenant|registrations|applicant_enc|id) → only `applicant_display` stays plaintext → `registration_submitted.v1` → confirmation message to the applicant's address (taken from the request, never logged).
3. Admin opens `/registrations/{id}`: SVC decrypts `applicant_enc` (permission `enrollment.registration.read`, audited) and looks up existing persons by blind index (email/phone) to suggest links.
4. `POST /registrations/{id}/approve`: **one tx** creates persons (PII encrypted per field, blind indexes), links, memberships, enrollments (with `leave_terms` snapshot), series members (capacity checked under lock), optional invites → `enrollment_activated.v1` per active enrollment → billing turns registration fees into `pending_charges` (unique `source_ref`) → scheduling materialises the new members on demand.

### FLW-41. Monthly cycle issuance

```mermaid
sequenceDiagram
  participant Cron as Cron planner
  participant J as billing.issue_cycle
  participant P as BillableSource ports (enrollment, …)
  participant B as Billing SVC
  participant DB as DB
  participant N as Notify
  Cron->>DB: daily 00:05 local: is today issue_day? → job dedupe billing.issue:{tenant}:{YYYY-MM}
  J->>P: billable_lines(period) (read-only)
  loop each payer (one tx per payer)
    J->>B: skip frozen (policy) · build draft from lines + pending_charges
    B->>B: proration (pure, BIL-03) · totals (Money, half-up once per line)
    B->>DB: invoice (unique payer+period) · number (gap-free sequence) · reference (≤3 attempts) · terms snapshot
    B->>DB: audit · invoice_issued.v1 · Notifier (critical)
  end
  J->>DB: run summary {issued, skipped_frozen, failed[]} · attention if failures
  N->>N: dispatch billing.invoice_issued to the payer
```

A failure for one payer rolls back only that payer's tx; re-running the job is safe because of the unique index (BIL-02).

### FLW-42. Payment → verification → paid → unfreeze

Payer `POST /invoices/{id}/payments` (proof upload encrypted, `Idempotency-Key`) → `submitted` + attention `payment_to_verify` → admin queue shows matches (`reference_entered = reference_code`, or amount = total for UniqueAmount) → `POST /payments/{id}/verify` → one tx: payment verified, `paid_minor += amount`, invoice status (STM-50), `payment_recorded.v1`, `invoice_paid.v1` → billing handler re-evaluates payer state (STM-52 t3) → `payer_unfrozen.v1` → scheduling sets `blocked` participants back to `expected` (STM-32 t3); rental lifts the restriction; notify sends `payment_verified` and `payer_unfrozen`.

### FLW-43. Overdue → late fee → freeze

Daily `billing.overdue` (00:10 local) → for each open invoice past `due_at`: lock it, `overdue`, late fee per **snapshotted** terms (once) → `invoice_overdue.v1` → recompute the payer state from all overdue invoices (strictest snapshotted action) → `payer_frozen.v1 {state}` → scheduling blocks future participations (frozen only) → rental blocks new bookings → notify critical message → every API response for that payer's session carries `X-EQ-Payer-State` and `/me.payer_state` drives the banner (FE-32).

## 6. Attendance, rental, notify, privacy

### FLW-50. Instructor check-in (online and offline)

```mermaid
sequenceDiagram
  participant I as Instructor SPA (STM-96)
  participant Q as IndexedDB queue (STM-97)
  participant H as HTTP
  participant S as Attendance SVC
  participant F as eq_core::files
  participant DB as DB
  I->>I: camera photo · GPS fix · MediaPipe face ratio (on device)
  alt online
    I->>H: POST /sessions/{id}/checkin (multipart photo + json)
  else offline
    I->>Q: encrypt payload with device key · queue
    Q->>H: replay on reconnect with X-EQ-Queued-At
  end
  H->>F: sniff · decode with limits · size check · re-encode JPEG · strip EXIF · encrypt
  H->>S: CheckinCommand (client_at, gps, face_ratio, file_id)
  S->>S: evaluate (pure): time vs server clock / queued-at rule, haversine vs radius, photo, face
  S->>DB: checkin (unique per instructor) · audit · checkin_recorded.v1
  S-->>I: 201 {status, checks}
  Q->>Q: delete the queued item and photo blob after 2xx
```

Raw GPS coordinates are S1 for the instructor's own check-in and shown only to reviewers; the photo is S2 and only reachable through a 5-minute signed URL (SEC-15) until the retention job deletes it (ATT-08).

### FLW-51. Reservation with deposit

`POST /rental/quote` (dry run: price breakdown from the rate card in force, peak slices) → `POST /reservations` + `Idempotency-Key` → one tx: RNT-02 checks, PayerState port, price/terms snapshot, holds, `pending_payment` with `hold_expires_at` → `reservation_requested.v1 {needs_payment}` → billing issues a `rental` invoice (`deposit` line) due at `hold_expires_at` → payment verified → `invoice_paid.v1` → rental confirms (STM-60 t4) → or `rental.expire_holds` expires it, deletes holds and cancels the unpaid invoice.

### FLW-52. Notification pipeline

```mermaid
sequenceDiagram
  participant M as Any module (in its tx)
  participant DB as notifications
  participant J as notify.dispatch (every 5 s)
  participant R as Renderer (MiniJinja)
  participant Ch as Channel adapter
  participant X as SMTP / WhatsApp
  M->>DB: Notifier::request (dedupe_key ON CONFLICT DO NOTHING)
  J->>DB: claim pending due rows SKIP LOCKED · aggregate by aggregate_key (2 min)
  J->>J: quiet hours (standard) → deferred
  J->>R: template(locale, channel) + params (S0/S1) + term filter
  J->>J: decrypt recipient email/phone in memory (never logged)
  loop routing order
    J->>Ch: breaker closed? opted in? address present?
    Ch->>X: send via eq_core::outbound (timeouts, allow-listed host)
    X-->>Ch: ok | retryable | permanent
    Ch->>DB: deliveries row · retry → re-enqueue with run_at (no sleeping)
  end
  J->>DB: status sent | failed | critical_failure (+ event, attention)
```

### FLW-53. Privacy export and erasure

- **Export:** `POST /privacy/export {subject_id}` (scope check) → job `privacy.export` → collect through ports (each module returns its rows for the subject) → ZIP (`person.json` with decrypted PII, CSVs with the formula guard) → file `export` **encrypted** → signed URL valid 24 h → audit `privacy.export.created` → the file is deleted after 24 h (STM-72).
- **Erasure:** STM-71 → job `privacy.erase` → one tx: end enrollments, delete future participations and holds, deactivate memberships, revoke sessions, anonymise the person (null `*_enc` and `*_bidx`), anonymise audit entries for the subject, keep invoices with the anonymised name → after commit: physical deletion of avatar, check-in photos and payment proofs (STM-15) → confirmation to the contact captured before erasure.

### FLW-54. File upload and download

Upload: multipart part → V0 body limit (route-specific) → stream to a temp file under `RITMA_DATA_DIR/tmp` with a size cap → `files::accept(purpose)` (VAL-15) → storage via `object_store` (local or S3) at `files/{tenant}/{yyyy}/{mm}/{file_id}` → `files` row with sha256 → `{file_id}`. Download: `GET /api/v1/files/{id}?exp&sig` → HMAC check (constant time) bound to the actor → RLS-scoped row lookup → stream (decrypting in 1 MiB STREAM chunks if encrypted) with `nosniff` and `Content-Disposition` (CAN-81).

### FLW-55. CSV/XLSX import

`POST /import/students/preview` (file) → VAL-17 limits → parse in `spawn_blocking` → per row: field kinds (VAL-21) + reference lookups by key/email → preview stored as a short-lived `upload_id` (the import file, 24 h) → UI shows row errors → fix and re-upload, or `POST /import/students/commit {upload_id}` → **one tx** for all rows (refused unless 0 errors) → the same services as manual creation (audit and events per entity) → error-report CSV if anything fails.

## 7. Data lineage (sensitive data)

**FLW-60.** Where S2/S3 data exists in plaintext. Everywhere else it is ciphertext, a hash, or absent.

| Data (class) | Enters | Plaintext exists only in | At rest | Leaves the server as | Never in |
|---|---|---|---|---|---|
| Password (S3) | login, accept, reset, change | request memory (`SecretString`, zeroized) | argon2id hash | — | logs, audit, events, responses, drafts |
| Refresh / invite / reset tokens (S3) | generated by the server | the response cookie or the outgoing message | SHA-256 | cookie / link once | logs, audit, events |
| TOTP secret (S3) | setup | setup response (QR) and verification memory | AES-GCM (`totp_secret_enc`) | QR once | logs, audit |
| Channel credentials (S3) | Settings → Channels | request memory, adapter memory at send time | AES-GCM (`config_enc`) | redacted `••••` | logs, audit (keys only), events |
| Email, phone (S2) | people forms, registration, guest booking | request memory, SVC memory for `read_pii` responses, notify job memory at send time | AES-GCM + HMAC blind index | `read_pii` responses, outgoing messages, privacy export | logs, audit (ids only), events, notification params, URLs |
| Legal name, birth date, address (S2) | people forms, registration | same as above | AES-GCM | `read_pii` responses, export | logs, audit, events |
| Registration applicant data (S2) | `/join` | request memory, review screen | AES-GCM (`applicant_enc`) | review screen (admin) | logs, events |
| Check-in photo, payment proof, registration docs (S2) | uploads | upload memory/temp file (deleted after encryption) | encrypted file (STREAM) | signed URL, 5 min | logs, static paths |
| Offline check-in payload (S2) | instructor device | device memory | IndexedDB, WebCrypto AES-GCM (non-extractable key) | replay request | localStorage, logs |
| Drafts (may contain S2 typed by the user) | forms | browser + `drafts` row | plaintext JSON in `drafts` (RLS, owner only), cleared on logout and after 7 days | the same person only | logs, audit [D-55] |
| Display name (S1) | everywhere | plaintext by design (SEC-04) | plaintext | role-scoped responses, messages, audit | logs (ids only) |

## 8. Decisions taken in this document

| ID | Decision |
|---|---|
| D-55 | Server-side drafts are stored as plaintext JSON under RLS, readable only by their author, and purged after 7 days; forms that collect S2 values (registration review, people PII fields) exclude those fields from drafts |
