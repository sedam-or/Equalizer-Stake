# ADR-003: Soft Delete over Hard Delete for PII

## Status

Accepted

## Context

Equalizer-Stake stores personal data (NIK, phone, address) for guardians and students. The system generates invoices linked to students, attendance records linked to sessions, and audit logs tracking all changes. Users may request data deletion under Indonesia's UU PDP (Personal Data Protection Law). The question: hard-delete rows or soft-delete (mark deleted, keep data)?

## Decision

Use **soft delete** pattern: add `deleted_at TIMESTAMPTZ NULL` column to all domain tables. Filter `WHERE deleted_at IS NULL` on all queries. For PII erasure requests, anonymize fields (replace with "ANONYMIZED") rather than deleting rows.

## Rationale

### Why not hard delete

1. **Foreign key integrity**: Invoices reference `student_id`. Deleting a student cascades to invoices or leaves orphans. Financial records must be retained for accounting.

2. **Audit trail**: AuditLog references `record_id`. If the record is hard-deleted, audit entries reference nothing. Compliance requires knowing *who changed what* even after the entity is "gone."

3. **UU PDP "right to erasure" ≠ delete rows**: The law allows anonymization as a form of erasure. Replacing PII with "ANONYMIZED" satisfies the requirement while preserving structural data.

4. **Accidental deletion recovery**: Admin deletes wrong student → soft delete allows restore. Hard delete requires backup restore (data loss window).

### Soft delete implementation

```sql
-- All queries filter soft-deleted
SELECT * FROM students WHERE deleted_at IS NULL;

-- Partial indexes exclude soft-deleted rows
CREATE INDEX idx_students_guardian ON students (guardian_id) WHERE deleted_at IS NULL;
```

### PII anonymization (UU PDP compliance)

```python
# When user requests deletion
guardian.phone = enc.encrypt("ANONYMIZED")
guardian.nik = enc.encrypt("ANONYMIZED")
guardian.address = enc.encrypt("ANONYMIZED")
guardian.full_name = "ANONYMIZED"
guardian.deleted_at = datetime.utcnow()
# Row preserved for FK integrity, PII gone
```

## Consequences

- All queries must include `WHERE deleted_at IS NULL` — enforced via base query helper
- Partial indexes keep index size small (soft-deleted rows excluded)
- Database grows unboundedly — need periodic purge job for records older than retention period (2 years)
- Audit log preserves full history including deleted entities
- Anonymized records remain queryable for aggregate reporting (revenue by month)

## Tradeoffs Accepted

| Concern | Mitigation |
|---------|------------|
| Query complexity | Base query filter in service layer, always applied |
| Storage growth | Cron job purges soft-deleted records after 2-year retention |
| "Unique" constraints | Partial unique indexes: `UNIQUE (email) WHERE deleted_at IS NULL` |
| Confusion about data state | Clear `deleted_at` field, API excludes deleted by default |

## Alternatives Considered

- **Hard delete + cascade**: Breaks audit trail, loses financial record integrity
- **Separate "archived" table**: More complex, same result, harder to query across active+archived
- **Event sourcing**: Overengineered for this scope (11 entities, single-tenant school)
