---
id: T-201
title: Add the scheduling schema (catalog, series, sessions, calendar_holds with exclusion constraint, instructor availability)
milestone: M2
status: todo
depends_on: [T-117b]
spec_refs: [DOM-10, SCH-15, MOD-02]
estimated_size: S
---

## Goal
The tables behind the "zero double booking" guarantee exist, with the exclusion constraint as the single authority.

## Context
- `migrations/0010_scheduling.sql`: copy **verbatim** from `spec/06-domain-model.md`:
  - §3 `calendar_holds` (with `CREATE EXTENSION IF NOT EXISTS btree_gist;`, which T-003 may already create; keep `IF NOT EXISTS`).
  - §4.3: `level_tracks`, `levels`, `offerings`, `price_plans`, `series`, `series_members`, `sessions`, `session_instructors`, `session_participants`, plus their indexes (`sessions_series_occurrence`, `sessions_time`).
  - §4.11: `instructor_availability`.
- The key constraint, which must be present exactly:
```sql
CONSTRAINT calendar_holds_no_overlap
  EXCLUDE USING gist (tenant_id WITH =, subject_kind WITH =, subject_id WITH =, hold_range WITH &&)
```
- RLS (`eq_enable_rls`) on every table above.
- Rust row structs in `eq-scheduling::repo::rows` (`OfferingRow`, `PricePlanRow`, `SeriesRow`, `SessionRow`, `SessionParticipantRow`, `SessionInstructorRow`, `LevelTrackRow`, `LevelRow`), plus enums `SessionKind`, `SessionStatus`, `ParticipantStatus`, `InstructorRole`, `PricePlanKind`, `SeriesStatus`.
- The calendar row type lives in `eq-core::calendar` (T-203), because rental shares it.

## Scope
### In
- The migration, row structs and enums.

### Out
- Services and APIs (T-203+).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-scheduling/tests/schema.rs` | test | Task-listed verification file. |
| `migrations/0010_scheduling.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `exclusion_semantics` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Migrations apply. Two overlapping holds for the same subject → SQLSTATE `23P01`. Adjacent (`[10:00,11:00)` and `[11:00,12:00)`) → OK. Different tenants, same subject id → OK. → `eq-scheduling/tests/schema.rs::exclusion_semantics`
- AC2. A `sessions` row with an inclusive upper bound is rejected by the CHECK. → `half_open_enforced`
- AC3. A duplicate `(series_id, occurrence_date)` for non-cancelled sessions → unique violation. A cancelled one plus a new one → OK. → `series_occurrence_unique`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-scheduling/tests/schema.rs` | `exclusion_semantics`, `half_open_enforced`, `series_occurrence_unique` |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; the isolation suite (T-116) still passes with the new tables
- [ ] Commit `feat(scheduling): schema and conflict ledger [T-201]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
