---
id: T-203
title: Add core calendar holds (write, rewrite, release, conflict mapping, overlap queries)
milestone: M2
status: todo
depends_on: [T-201]
spec_refs: [DOM-10, DOM-11, SCH-03]
estimated_size: S
---

## Goal
One shared, small API in `eq-core::calendar` that sessions (scheduling) and reservations (rental) use to claim time. A conflict becomes a structured `slot_taken` error naming the subject.

## Context
- Hold rules (DOM-10):

| Owner | Holds | hold_range |
|---|---|---|
| Session | resource | `[start, end + buffer)` |
| | each instructor (person) | `[start, end + buffer)` |
| | each participant (person) | `[start, end)` |
| Reservation | resource | `[start, end + buffer)` |
| | booker (person) | `[start, end)` |

- `buffer` is passed in by the caller (snapshotted on the owner row). Calendar never reads policy.
- Cancelling, expiring or rejecting deletes the owner's holds. Completed sessions keep them. Reschedule = delete + insert in one tx. A blocked participant keeps the hold.
- `23P01` maps to `CalendarError::Conflict`. Then (SCH-03) the caller rolls back and calls `find_overlaps` in a **new** read-only tx to name the subjects.

## Interfaces
```rust
pub enum SubjectKind { Resource, Person }
pub enum OwnerKind { Session, Reservation }
pub struct HoldSpec { pub subject_kind: SubjectKind, pub subject_id: Uuid, pub range: TsRange }
pub struct TsRange { pub start: Timestamp, pub end: Timestamp } // half-open; ctor validates start < end; maps to PgRange<DateTime<Utc>>
pub async fn claim(tx: &mut Tx, tenant: TenantId, owner: (OwnerKind, Uuid), holds: &[HoldSpec]) -> Result<(), CalendarError>;
pub async fn release(tx: &mut Tx, owner: (OwnerKind, Uuid)) -> Result<u64, CalendarError>;
pub async fn release_subject(tx: &mut Tx, owner: (OwnerKind, Uuid), subject: (SubjectKind, Uuid)) -> Result<u64, CalendarError>;
pub async fn rewrite(tx: &mut Tx, tenant: TenantId, owner: (OwnerKind, Uuid), holds: &[HoldSpec]) -> Result<(), CalendarError>; // release + claim
pub async fn find_overlaps(tx: &mut Tx, subjects: &[(SubjectKind, Uuid)], range: TsRange, exclude_owner: Option<(OwnerKind, Uuid)>) -> Result<Vec<HoldRow>, CalendarError>;
pub async fn holds_in_window(tx: &mut Tx, subjects: &[(SubjectKind, Uuid)], window: TsRange) -> Result<Vec<HoldRow>, CalendarError>; // for availability/alternatives
pub fn session_holds(resource: Uuid, instructors: &[Uuid], participants: &[Uuid], start: Timestamp, end: Timestamp, buffer_min: u16) -> Vec<HoldSpec>;
pub fn reservation_holds(resource: Uuid, booker: Uuid, start: Timestamp, end: Timestamp, buffer_min: u16) -> Vec<HoldSpec>;
#[derive(thiserror::Error, Debug)] pub enum CalendarError { #[error("conflict")] Conflict, #[error(transparent)] Db(#[from] sqlx::Error) }
```
- `claim` inserts all holds in **one** statement (`INSERT … SELECT FROM unnest(...)`) so a conflict on any subject aborts all of them. Each hold `id` is a UUIDv7.
- **Important:** after a `23P01`, the Postgres tx is aborted. `claim` returns `Conflict`, and the caller must not reuse the tx.
- `slot_taken` conflict DTO (shared): `ConflictDto { subject_kind, subject_id, subject_name, owner_kind, range: {start, end} }`. Name resolution (resource name, person display_name) is done by the caller (scheduling/rental) via a small helper `calendar::describe(tx, holds) -> Vec<ConflictDto>` that joins `resources` and `persons`.

## Scope
### In
- `eq_core::calendar` as above, plus `From<CalendarError> for AppError` (Conflict → 409 `slot_taken` with an empty conflicts list; callers enrich it).

### Out
- Session/reservation logic.

## Acceptance criteria
- AC1. `session_holds` with buffer 10: the resource and instructor ranges end at end+10, participants at end. → `calendar::tests::hold_shapes`
- AC2. Claiming two sessions back-to-back in the same room with buffer 10 → the second at end+5 conflicts, at end+10 succeeds. → `buffer_enforced_by_db`
- AC3. `claim` is all-or-nothing when the participant conflicts but the resource is free. → `claim_atomic`
- AC4. `find_overlaps` after a conflict identifies the instructor as the colliding subject. → `overlap_identifies_subject`
- AC5. `rewrite` to an overlapping range of its **own** previous holds succeeds (release happens first). → `rewrite_self_overlap_ok`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/calendar.rs` | AC names above |

## Definition of done
- [ ] Rust checks pass
- [ ] Commit `feat(core): calendar holds API [T-203]`, signed off

## Report

## Questions
