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
- `eq_core::calendar` as above, plus `From<CalendarError> for AppError` (Conflict → `AppError::Conflict { code: &codes::SLOT_TAKEN, … }`, 409, with an empty conflicts list; callers enrich it).
- Register the shared code `pub static SLOT_TAKEN: ErrorCodeDef = ErrorCodeDef { code: "slot_taken", status: 409, title: "Slot already taken" }` in `eq_core::error::codes` and add it to `CODES` (ERR-21, ERR-30; scheduling and rental both use it, and `eq-core` cannot depend on them).

### Out
- Session/reservation logic.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/tests/calendar.rs` | test | Task-listed verification file. |

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T7 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- T/R: duplicate or unauthorized state transitions → use transactional state checks, idempotency/If-Match where specified, and audit committed mutations (SBD-53/57, STM rules in the task).
- Abuse-case acceptance criterion: `claim_atomic` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(core): calendar holds API [T-203]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
