# Module: rental (optional; requires scheduling)

Purpose: hourly booking of rentable resources (rehearsal or recording studios, practice rooms) by members or guests, with optional admin approval, deposits and expiring holds.
Policy: `RentalPolicy` (04 §6.7). Tables: `rate_cards`, `reservations` (06 §4.7), plus calendar holds.
Legacy reference: `v0-capstone:srs/08-studio-reservation.md`.

## 1. Rules

- **RNT-01. Rentable resources.** Only resources with `rentable=true` appear. Rental shares the conflict ledger with lessons, so **a room can't be both rented and taught in at the same time** (DOM-10).
- **RNT-02. Booking validation** (pure `domain::booking::validate`), for booking type `t`:
  - `min_minutes ≤ duration ≤ max_minutes`
  - `duration % step_minutes == 0`
  - `start ≥ now + min_notice_hours`
  - `start ≤ now + max_days_ahead days`
  - `[start, end)` inside the rental hours (`SameAsScheduling` → scheduling opening hours) and not in a closure
  - the booker is not `restricted`/`frozen` (PayerState port, if billing is enabled)
- **RNT-03. Price** = Σ over 15-minute slices of the applicable `hourly_minor / 4`. A slice uses the peak rate if it falls in a peak window. The rate card is the latest one with `effective_from ≤ start local date`. If no rate card exists, the response is 422 `rule=rental.rate_card_missing`, and the admin sees a setup checklist item. The price is snapshotted on the reservation.
- **RNT-04. Creating a reservation** (one tx, holds written **at creation**, so pending requests also block the slot):
  - `approval=AdminApproval`: status `requested`; `hold_expires_at = now + 48 h` (technical constant: approvals must not hold slots forever, the admin sees a countdown).
  - `approval=Auto`, payment `PayLater`: status `confirmed`.
  - `approval=Auto`, payment `DepositToConfirm`/`FullToConfirm`: status `pending_payment`; `hold_expires_at = now + hold_minutes`; billing issues the deposit/full invoice (via `reservation_requested` with `needs_payment`, see BIL-10).
  - On a conflict the response is 409 with alternatives (same algorithm as SCH-05, restricted to rentable resources).
- **RNT-05. Approval.** An admin approves with `POST /reservations/{id}/approve` → `confirmed` (PayLater) or `pending_payment`, with a new hold expiry. Rejecting with `/reject {reason}` deletes the holds.
- **RNT-06. Payment confirmation.** On `billing.invoice_paid.v1` for an invoice linked to a reservation (`reservations.invoice_id`), status `pending_payment` → `confirmed` and `hold_expires_at = NULL`.
- **RNT-07. Expiry.** The job `rental.expire_holds` runs every minute. `requested`/`pending_payment` reservations past `hold_expires_at` → `expired`, holds deleted, the linked unpaid invoice is cancelled, and `reservation_expired` is emitted.
- **RNT-08. Cancellation by the booker.** Allowed until `start - cancel_until_hours_before` (from the snapshotted terms); `refund_bps` comes from the terms. After the deadline only staff can cancel, with a refund chosen by staff (0–10000). Cancelling deletes the holds and emits `reservation_cancelled {refund_bps}`.
- **RNT-09. Completion.** The job `rental.autocomplete` (every 15 min) sets `confirmed` → `completed` after the end.
- **RNT-10. Guests (`MembersAndGuests`).** The public booking page `/book` lets a visitor pick a slot, then enter name, email and phone. The server creates a person with the `student` role and status `invited`, sends a magic link, and the reservation is created once the email is verified by the link. The slot is held for 15 minutes during verification with status `requested` and a 15-minute expiry.

## 2. API

| Method | Path | Perm |
|---|---|---|
| GET | `/rental/resources` (rentable, with booking types and current rates) | any authenticated; `/public/rental/resources` if guests are allowed |
| GET | `/rental/availability?date&resource_id&booking_type` → free starts per allowed duration | any / public |
| POST | `/rental/quote {resource_id, booking_type, starts_at, duration_minutes}` → `{price_minor, deposit_minor, breakdown}` | any / public |
| POST | `/reservations {…, purpose?}` → Reservation \| 409 \| 422 | `rental.reserve` |
| POST | `/public/reservations {…, guest:{name,email,phone}}` → 202 (check your email) | public (guests) |
| GET | `/reservations?status&from&to&resource_id&booker_id` | scoped |
| POST | `/reservations/{id}/approve` \| `/reject` \| `/cancel` | staff / booker (cancel) |
| GET/POST | `/rental/rate-cards` | `rental.rates.write` |

## 3. Permissions

| Permission | owner | admin | instructor | guardian | student |
|---|---|---|---|---|---|
| rental.reserve | — | ✅ (on behalf of anyone) | ✅ | ✅ | ✅ |
| rental.read | ✅ all | ✅ all | own | own | own |
| rental.decide | — | ✅ | — | — | — |
| rental.rates.write | — | ✅ | — | — | — |

## 4. Acceptance criteria

1. **Shared ledger.** Given a lesson in Studio A 15:00–16:00 with buffer 10, When renting Studio A 16:00–17:00, Then 409. When renting 16:10–17:10 with step 60, Then 422 for step misalignment, if the step requires on-the-hour starts.
2. **Hold expiry.** Given DepositToConfirm and `hold_minutes=60`, When nothing is paid within 60 min, Then expired, holds released, the invoice cancelled, and the slot bookable by others.
3. **Peak pricing.** Given a 100k/h base, a 150k/h peak from 18:00–21:00 and a 17:00–19:00 booking, Then the price is 250k.
4. **Cancellation window.** Given `cancel_until=24` and 100%, When cancelling 30 h before, Then refund_bps=10000. When the booker tries 10 h before, Then 403 `cancel_window_passed`.
5. **Frozen booker.** Given a frozen booker, When reserving, Then 422 `rule=billing.overdue.action`.
