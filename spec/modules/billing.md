# Module: billing (optional; requires enrollment OR rental)

Purpose: invoices (monthly cycle, rental, manual), payment references, manual payment verification, late fees, restrict/freeze, and receipts.
Policy: `BillingPolicy` (04 §6.5). Tables: 06 §4.5.
Legacy reference: `v0-capstone:srs/03-billing.md`, `srs/09-multi-student.md`, addendum §1, `backend/app/services/billing.py`, `scheduler.py`.

## 1. Rules

- **BIL-01. Amounts.** All math uses `core::money` (ARC-11): integer minor units in the tenant currency, with percentages in bps rounded half-up once per line. `total = subtotal + tax + late_fee + unique_amount` (DB CHECK).
- **BIL-02. Cycle issuance.**
  - The daily cron `billing.plan_cycle` runs at 00:05 tenant TZ. If today's local day == `cycle.issue_day`, it enqueues `billing.issue_cycle` with dedupe key `billing.issue:{tenant}:{YYYY-MM}`.
  - The job:
    1. Gets payers from the `BillableSource` ports (enrollment, and any other registered sources) for period `[first day, last day]` of the month.
    2. For each payer with ≥ 1 line, one transaction per payer creates the invoice (`source=cycle`, `period_key=YYYY-MM`, grouping per policy) with lines, then issues it (BIL-04).
    3. Payers in state `frozen` are skipped when `skip_issuing_while_frozen`.
    4. A failure for one payer doesn't stop the others. Failures are retried by re-running the job; the unique index makes this idempotent. The run summary `{issued, skipped_frozen, failed:[payer_id]}` is written to the job result and the admin "Needs attention" list.
  - Performance target: 500 payers in ≤ 60 s.
- **BIL-03. Proration (`first_cycle`).**
  - The first-cycle rule applies when an enrollment starts after the period start.
  - `Full`: full monthly fee.
  - `StartNextCycle`: no tuition line this period.
  - `ProratedBySessions`: `fee × remaining_scheduled_sessions / total_scheduled_sessions_in_period`. Counts come from the series pattern via the scheduling port. If total = 0, the fee is 0. Rounding is half-up.
  - Registration fees are always charged in full on the next invoice after activation, or on an immediate `source=enrollment` invoice if the admin selects "Invoice now".
- **BIL-04. Issuing.** A draft invoice becomes issued:
  1. `number` is assigned from `invoice_sequences` (scope by `numbering.reset`: `INV-2026-00042` or `INV-202611-0042`), with gap-free `UPDATE … RETURNING` inside the tx.
  2. The payment reference is assigned per `payment_reference`:
     - `UniqueCode{digits}`: a random code in `[0, 10^digits)`, zero-padded, unique among the tenant's **open** invoices (partial unique index). Up to **3 attempts**; if all collide, the status becomes `needs_manual_reference` and an admin "Needs attention" item is created.
     - `UniqueAmount{max_minor}`: a random `unique_amount_minor` in `[1, max_minor]` in whole major units (for IDR, multiples of 100 minor = Rp1). The resulting `total_minor` is unique among open invoices. Same 3-attempt rule.
  3. `issued_at = now`, and `due_at` is the end of `cycle.due_day` in tenant TZ (23:59:59.999 local). Non-cycle invoices use `issued_at + (due_day - issue_day) days`, minimum 1 day.
  4. `terms` snapshot: `{late_fee, overdue, partial_payment, payment_reference, payment_methods}`.
  5. Emits `invoice_issued` → notify (critical) to the payer, with a link, the amount, the due date, the reference and the payment instructions.
- **BIL-05. Reminders.** The daily cron at 09:00 tenant TZ, for each open invoice and each offset in `reminders_days_from_due` (negative = before), sends a `billing.reminder` notification on local date `due_date + offset`. The dedupe key is `reminder:{invoice}:{offset}`.
- **BIL-06. Overdue and late fee.**
  - The daily cron `billing.overdue` runs at 00:10 tenant TZ. For each `issued`/`partially_paid` invoice with `now > due_at`, it sets status `overdue` and emits `invoice_overdue`.
  - Using the invoice's **terms snapshot**:
    - If `late_fee.enabled`, `late_fee_applied_at IS NULL`, and local date ≥ due date + `days_after_due`:
      - it adds a `late_fee` line: Flat amount, or Percent of `subtotal + tax` rounded;
      - it updates `late_fee_minor` and `total_minor`;
      - it sets `late_fee_applied_at`.
      - If `UniqueAmount` is in use, uniqueness of the new total is re-checked; on collision, the new total is allowed (the reference match uses the code, not the amount, for overdue invoices). This edge case is documented.
    - If `overdue.action != None` and local date ≥ due date + `overdue.days_after_due`, it sets the payer state (BIL-08).
- **BIL-07. Payments.**
  - **Recording** `POST /invoices/{id}/payments {amount_minor, method, received_on, reference_entered?, proof_file_id?, idempotency_key}`:
    - By **staff**, `status=verified` immediately.
    - By the **payer**, from the portal "I've paid" + proof upload: `status=submitted`. It appears in the verification queue.
  - **Verification** `POST /payments/{id}/verify` or `/reject {reason}`:
    - Matching help: the queue shows reference matches. The rule is `reference_entered == invoice.reference_code`, or the amount equals `total_minor` when UniqueAmount is used.
    - Verifying a second payment with the same `(invoice, reference_entered, amount, received_on)` warns "Possible duplicate" (v0 FR-15 §4.3). The `idempotency_key` uniqueness prevents API double-submits.
  - **Applying** (in the verifying tx): `paid_minor += amount`. The status becomes `paid` if `paid_minor >= total_minor`, otherwise `partially_paid` (or stays `overdue`). Overpayment is recorded and flagged in "Needs attention" (no automatic credit in v1).
  - **`partial_payment.allowed = false`:** payer-submitted amounts < remaining get 422 `rule=billing.partial_payment.allowed`. Staff may still record them, with a warning.
  - **`min_first_payment_bps`:** the first payment must be ≥ bps × total, otherwise 422, for payer submissions.
  - Emits `payment_recorded`, and `invoice_paid` when it reaches paid.
- **BIL-08. Payer state (restrict/freeze).**
  - The state is computed from **all** of the payer's overdue invoices, using the strictest snapshotted `overdue.action`.
  - Entering `restricted`/`frozen`: upsert `payer_states`, emit `payer_frozen.v1` (payload has `state`) → scheduling blocks participants (only for `frozen`), rental blocks new bookings (both states), and notify sends a critical message.
  - Exit:
    - `AutoOnFullPayment`: when the last overdue invoice becomes paid, the state goes back to normal and `payer_unfrozen` is emitted.
    - `AdminManual`: an admin calls `POST /payers/{id}/unfreeze {note}`. It is **refused** with 409 `unpaid_overdue` unless all overdue invoices are paid. The **owner override** (`billing.override.unfreeze`) can force it with a mandatory reason; it is audited as category `compliance`.
  - Every request by a frozen payer's session gets the header `X-EQ-Payer-State: frozen`. The SPA shows the banner and blocks actions per 04 §6.5.
- **BIL-09. Cancelling and adjusting.**
  - `POST /invoices/{id}/cancel {reason}` (admin): only with `paid_minor = 0`. Otherwise the admin must issue a credit line.
  - `POST /invoices/{id}/lines` (admin, invoice not paid/cancelled): adds an `adjustment` or `credit` line with a reason and recomputes totals. Audited.
  - Owner override: cancel any non-paid invoice with a reason.
- **BIL-10. Manual and rental invoices.**
  - Admins can create `source=manual` invoices with free lines.
  - Rental: `rental.reservation_confirmed.v1` (or a deposit requirement) creates and issues an invoice `source=rental` with a `rental` or `deposit` line. Its due time is the reservation's `hold_expires_at` for deposits, or the reservation start for PayLater.
  - `rental.reservation_cancelled.v1` with `refund_bps > 0` on a paid invoice creates a `credit` line on the payer's **next** invoice (v1 has no cash refunds through the system). The admin sees a "Refund due" item if the payer has no next invoice.
- **BIL-11. Receipts and PDFs.** `GET /invoices/{id}/document?format=html|pdf` renders a printable invoice or receipt using the tenant logo and name, the localised labels and the payment instructions. HTML is primary. PDF comes from the browser print stylesheet in v1, so there is no server-side PDF engine.
- **BIL-12. Payment gateway seam (v1.x).** Trait `PaymentProvider { create_checkout(invoice) -> Url; verify_webhook(req) -> PaymentEvent }`. v1 ships **no** provider, only the trait, the `method=gateway` enum value and the webhook route stub returning 501. Planned providers: Xendit, Midtrans, Stripe.

## 2. API

| Method | Path | Perm |
|---|---|---|
| GET | `/invoices?status&payer_id&period_key&q&cursor` | scoped (payer: own) |
| GET | `/invoices/{id}` (includes lines, payments, terms, payment instructions) | scoped |
| POST | `/invoices` (manual, draft) / POST `/invoices/{id}/issue` | `billing.invoice.write` |
| POST | `/invoices/{id}/lines` / `/cancel` | `billing.invoice.write` |
| PATCH | `/invoices/{id}/reference {reference_code}` (needs_manual_reference) | `billing.invoice.write` |
| POST | `/invoices/{id}/payments` | staff `billing.payment.record`, payer own |
| GET | `/payments?status=submitted` (verification queue) | `billing.payment.verify` |
| POST | `/payments/{id}/verify` / `/reject` | `billing.payment.verify` |
| GET | `/payers/{id}/state` / POST `/payers/{id}/unfreeze` | read scoped / `billing.payer.unfreeze` |
| POST | `/billing/run-cycle {period_key, dry_run}` | `billing.invoice.write` (manual trigger; dry run returns the preview) |
| GET | `/invoices/{id}/document` | scoped |
| GET | `/me/invoices` | payer |
| POST | `/billing/webhooks/{provider}` | provider signature (501 in v1) |

## 3. Permissions

| Permission | owner | admin | instructor | guardian | student |
|---|---|---|---|---|---|
| billing.invoice.read | ✅ | ✅ | — | own (payer) | own (if payer) |
| billing.invoice.write | — | ✅ | — | — | — |
| billing.payment.record | — | ✅ | — | own submit | own submit |
| billing.payment.verify | — | ✅ | — | — | — |
| billing.payer.unfreeze | — | ✅ (BIL-08 rule) | — | — | — |
| billing.override.unfreeze / override.cancel | ✅ | — | — | — | — |

## 4. Events, jobs, ports

- **Emits:** `invoice_issued`, `payment_recorded`, `invoice_paid`, `invoice_overdue`, `payer_frozen`, `payer_unfrozen`.
- **Consumes:**
  - `enrollment_activated` (registration fee pending line, stored in `pending_charges` until the next invoice: add the table `pending_charges(id, tenant_id, payer_id, kind, description jsonb, amount_minor, student_id, enrollment_id, created_at, invoiced_in uuid)`)
  - leave fee events
  - `rental.reservation_confirmed/cancelled`
  - `enrollment_ended` (no proration refund in v1)
- **Jobs:** `billing.plan_cycle` (daily 00:05), `billing.issue_cycle`, `billing.overdue` (daily 00:10), `billing.reminders` (daily 09:00).
- **Ports implemented:** `PayerState` (`state(payer) -> normal|restricted|frozen`), used by scheduling, rental and the SPA via `/me`; `RegistrationFeePaid(student, offering) -> bool`.

## 5. Acceptance criteria

1. **Cycle issuance.** Given `issue_day=5`, 2 guardians (one with 2 students), When the clock is the 5th at 00:05, Then exactly 2 invoices exist, and the 2-student guardian's invoice has 2 tuition lines. Running the job again creates 0 more.
2. **Due time.** Given `due_day=10` and TZ Asia/Jakarta, Then `due_at` = the 10th 23:59:59.999 +07:00.
3. **Late fee and freeze.** Given terms Flat Rp100.000, `days_after_due=1`, Freeze, AdminManual and an unpaid invoice, When the clock is the 11th at 00:10, Then the invoice is overdue, `late_fee_minor=10_000_000`, the total increases accordingly, the payer state is frozen, the participants of their students' future sessions are `blocked`, and a critical notification is queued.
4. **Policy change doesn't affect old invoices.** Given the owner changes the late fee to Rp50.000 on the 8th, Then the invoice issued on the 5th still gets Rp100.000 (snapshot).
5. **Unique code collisions.** Given codes are forced to collide (test RNG), Then after 3 attempts the status is `needs_manual_reference`.
6. **Unfreeze rule.** Given AdminManual and a frozen payer with an unpaid overdue invoice, When the admin unfreezes, Then 409. After the full payment is verified, unfreeze succeeds and the participants return to `expected`.
7. **Partial payment.** Given `min_first_payment_bps=7000` and total 600.000, When the payer submits 300.000, Then 422. When they submit 420.000, Then 201 submitted. When it's verified, Then partially_paid.
8. **Proration.** Given an enrollment starting on the 15th of a month with 4 series occurrences of which 2 remain, Then the tuition line is 50%.
9. **Money safety.** A property test: for random line sets, `total == Σ lines + tax + unique_amount` and there's no overflow below 10^15 minor.
