# ADR-0008: Effective-dated policies + term snapshots on commitments

- Status: Accepted (2026-10-25)

## Decision
- Policies have `effective_from`.
- Long-lived commitments (invoice, reservation, enrollment leave terms) **copy the relevant terms** when they are created.
- Checks without commitments (opening hours, buffer) use the policy in force at the time being checked.

## Consequences
- ✅ Changing the late fee never alters already-issued invoices, which avoids disputes.
- ❌ Some duplication of data in `terms` jsonb columns (small).
