# Equalizer-Stake — Database Architecture

## 11 Tables with Relationships

```
┌──────────────┐ ┌──────────────┐
│ users │ │ guardians │
├──────────────┤ ├──────────────┤
│ id (PK) │◄──┐ │ id (PK) │
│ email │ │ │ user_id (FK) │──► users.id
│ password_hash│ │ │ full_name │
│ role │ │ │ phone (enc) │
│ full_name │ │ │ nik (enc) │
│ is_active │ │ │ address (enc)│
│ created_at │ │ │ created_at │
│ updated_at │ │ │ updated_at │
│ deleted_at │ │ │ deleted_at │
└──────────────┘ │ └──────┬───────┘
 │ │
 │ │ 1:N
 │ ▼
 │ ┌──────────────┐ ┌──────────────┐
 │ │ students │ │ rooms │
 │ ├──────────────┤ ├──────────────┤
 │ │ id (PK) │ │ id (PK) │
 │ │ guardian_id │──┐ │ name │
 │ │ full_name │ │ │ capacity │
 │ │ date_of_birth│ │ │ location │
 │ │ rate_per_sess│ │ │ gps_lat │
 │ │ is_active │ │ │ gps_lng │
 │ │ created_at │ │ │ created_at │
 │ │ updated_at │ │ │ updated_at │
 │ │ deleted_at │ │ │ deleted_at │
 │ └──────┬───────┘ │ └──────┬───────┘
 │ │ │ │
 │ │ N:M │ │ 1:N
 │ ▼ │ ▼
 │ ┌──────────────┐ │ ┌──────────────┐
 │ │ sessions │ │ │ sessions │
 │ ├──────────────┤ │ ├──────────────┤
 │ │ id (PK) │ │ │ id (PK) │
 │ │ student_id │──┘ │ room_id (FK) │──► rooms.id
 │ │ teacher_id │──┐ │ day_of_week │
 │ │ room_id (FK) │──┼───►│ start_time │
 │ │ date │ │ │ end_time │
 │ │ start_time │ │ │ is_active │
 │ │ end_time │ │ │ created_at │
 │ │ status │ │ │ updated_at │
 │ │ notes │ │ │ deleted_at │
 │ │ created_at │ │ └──────────────┘
 │ │ updated_at │ │
 │ │ deleted_at │ │
 │ └──────┬───────┘ │
 │ │ │
 │ │ 1:1 │
 │ ▼ │
 │ ┌──────────────┐ │ ┌──────────────┐
 │ │ attendance │ │ │ teachers │
 │ ├──────────────┤ │ ├──────────────┤
 │ │ id (PK) │ │ │ id (PK) │
 │ │ session_id │──┘ │ user_id (FK) │──► users.id
 │ │ check_in_time│ │ full_name │
 │ │ gps_lat │ │ phone (enc) │
 │ │ gps_lng │ │ specialization│
 │ │ face_verified│ │ created_at │
 │ │ photo_url │ │ updated_at │
 │ │ status │ │ deleted_at │
 │ │ created_at │ └──────────────┘
 │ │ updated_at │
 │ │ deleted_at │
 │ └──────────────┘
 │
 │ ┌──────────────┐ ┌──────────────┐
 │ │ invoices │ │ payments │
 │ ├──────────────┤ ├──────────────┤
 │ │ id (PK) │ │ id (PK) │
 │ │ student_id │──┐ │ invoice_id │──► invoices.id
 │ │ guardian_id │──┼───►│ amount │
 │ │ amount │ │ │ payment_date │
 │ │ period │ │ │ method │
 │ │ status │ │ │ proof_url │
 │ │ due_date │ │ │ status │
 │ │ paid_at │ │ │ verified_by │──► users.id
 │ │ created_at │ │ │ created_at │
 │ │ updated_at │ │ │ updated_at │
 │ │ deleted_at │ │ │ deleted_at │
 │ └──────────────┘ │ └──────────────┘
 │ │
 │ ┌──────────────┐ │ ┌──────────────┐
 │ │notifications │ │ │ audit_logs │
 │ ├──────────────┤ │ ├──────────────┤
 │ │ id (PK) │ │ │ id (PK) │
 │ │ user_id (FK) │──┘ │ user_id (FK) │──► users.id
 │ │ type │ │ action │
 │ │ channel │ │ table_name │
 │ │ subject │ │ record_id │
 │ │ body │ │ before_state │ (JSONB)
 │ │ status │ │ after_state │ (JSONB)
 │ │ sent_at │ │ ip_address │
 │ │ retry_count │ │ user_agent │
 │ │ created_at │ │ created_at │
 │ └──────────────┘ └──────────────┘
```

## Table Definitions (SQLAlchemy)

```python
# app/models/base.py
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy import func
from datetime import datetime

class TimestampMixin:
 created_at: Mapped[datetime] = mapped_column(server_default=func.now())
 updated_at: Mapped[datetime] = mapped_column(server_default=func.now(), onupdate=func.now())

class SoftDeleteMixin:
 deleted_at: Mapped[datetime | None] = mapped_column(default=None, nullable=True)
```

### Guardian

```python
class Guardian(Base, TimestampMixin, SoftDeleteMixin):
 __tablename__ = "guardians"
 id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
 user_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("users.id"), unique=True)
 full_name: Mapped[str] = mapped_column(String(200))
 phone: Mapped[str] = mapped_column(String(500)) # AES-256-GCM encrypted
 nik: Mapped[str] = mapped_column(String(500)) # AES-256-GCM encrypted
 address: Mapped[str] = mapped_column(Text(1000)) # AES-256-GCM encrypted
 students: Mapped[list["Student"]] = relationship(back_populates="guardian")
```

### Invoice

```python
class Invoice(Base, TimestampMixin, SoftDeleteMixin):
 __tablename__ = "invoices"
 id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
 student_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("students.id"))
 guardian_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("guardians.id"))
 amount: Mapped[Decimal] = mapped_column(Numeric(12, 2))
 period: Mapped[date] # first day of billing month
 status: Mapped[str] = mapped_column(String(20), default="pending") # pending|paid|overdue|cancelled
 due_date: Mapped[date]
 paid_at: Mapped[datetime | None]
 payments: Mapped[list["Payment"]] = relationship(back_populates="invoice")
```

### AuditLog

```python
class AuditLog(Base):
 __tablename__ = "audit_logs"
 id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
 user_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("users.id"))
 action: Mapped[str] = mapped_column(String(20)) # CREATE|UPDATE|DELETE|LOGIN|LOGOUT
 table_name: Mapped[str] = mapped_column(String(50))
 record_id: Mapped[uuid.UUID | None]
 before_state: Mapped[dict | None] = mapped_column(JSONB)
 after_state: Mapped[dict | None] = mapped_column(JSONB)
 ip_address: Mapped[str | None] = mapped_column(String(45))
 user_agent: Mapped[str | None] = mapped_column(String(500))
 created_at: Mapped[datetime] = mapped_column(server_default=func.now())
 # No SoftDeleteMixin — audit logs never soft-deleted
```

## Indexing Strategy

```sql
-- Users
CREATE UNIQUE INDEX idx_users_email ON users (email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role ON users (role) WHERE deleted_at IS NULL;

-- Students
CREATE INDEX idx_students_guardian_id ON students (guardian_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_students_active ON students (is_active) WHERE deleted_at IS NULL AND is_active = true;

-- Sessions
CREATE INDEX idx_sessions_student_date ON sessions (student_id, date) WHERE deleted_at IS NULL;
CREATE INDEX idx_sessions_teacher_date ON sessions (teacher_id, date) WHERE deleted_at IS NULL;
CREATE INDEX idx_sessions_room_date ON sessions (room_id, date) WHERE deleted_at IS NULL;

-- Attendance
CREATE UNIQUE INDEX idx_attendance_session ON attendance (session_id) WHERE deleted_at IS NULL;

-- Invoices
CREATE INDEX idx_invoices_guardian_status ON invoices (guardian_id, status) WHERE deleted_at IS NULL;
CREATE INDEX idx_invoices_period ON invoices (period) WHERE deleted_at IS NULL;
CREATE INDEX idx_invoices_due_date ON invoices (due_date) WHERE status = 'pending' AND deleted_at IS NULL;

-- Payments
CREATE INDEX idx_payments_invoice ON payments (invoice_id) WHERE deleted_at IS NULL;

-- Notifications
CREATE INDEX idx_notifications_user_status ON notifications (user_id, status);

-- Audit logs (time-series, no soft delete)
CREATE INDEX idx_audit_created_at ON audit_logs (created_at);
CREATE INDEX idx_audit_table_record ON audit_logs (table_name, record_id);
CREATE INDEX idx_audit_user ON audit_logs (user_id, created_at);
```

All partial indexes use `WHERE deleted_at IS NULL` to exclude soft-deleted rows from query results and index size.

## Soft Delete Pattern

```python
# Every query filters soft-deleted rows
async def get_active_students(db: AsyncSession) -> list[Student]:
 result = await db.execute(
 select(Student).where(Student.deleted_at.is_(None))
 )
 return list(result.scalars().all())

# Soft delete sets timestamp, never removes row
async def soft_delete_student(db: AsyncSession, student_id: uuid.UUID):
 await db.execute(
 update(Student)
 .where(Student.id == student_id)
 .values(deleted_at=func.now())
 )
```

**Why soft delete**: PII data retention for audit trail, FK integrity for invoices/attendance, UU PDP compliance (data must be retrievable for legal requests).

## JSONB for Audit Before/After State

```python
# app/middleware/audit.py — captures state diff
async def capture_audit(
 db: AsyncSession,
 user_id: uuid.UUID,
 action: str,
 table_name: str,
 record_id: uuid.UUID,
 before: dict | None, # SQLAlchemy model → dict before change
 after: dict | None, # dict after change
):
 audit = AuditLog(
 user_id=user_id,
 action=action,
 table_name=table_name,
 record_id=record_id,
 before_state=before,
 after_state=after,
 ip_address=request.client.host,
 user_agent=request.headers.get("user-agent"),
 )
 db.add(audit)

# Serialization helper
def model_to_dict(instance) -> dict:
 return {
 c.name: str(getattr(instance, c.name)) # str() for UUID, Decimal, datetime
 for c in instance.__table__.columns
 if c.name not in ("password_hash",) # exclude sensitive
 }
```

Querying JSONB audit data:

```sql
-- Find all changes to a specific student record
SELECT action, before_state, after_state, created_at
FROM audit_logs
WHERE table_name = 'students' AND record_id = '<uuid>'
ORDER BY created_at DESC;

-- Find who changed a specific field
SELECT user_id, created_at, before_state->>'phone' as old_phone, after_state->>'phone' as new_phone
FROM audit_logs
WHERE table_name = 'guardians'
 AND before_state->>'phone' != after_state->>'phone';
```

## Partitioning Strategy for AuditLog

AuditLog grows unbounded. Partition by month on `created_at`:

```sql
-- Create parent table
CREATE TABLE audit_logs (
 id UUID DEFAULT gen_random_uuid(),
 user_id UUID REFERENCES users(id),
 action VARCHAR(20) NOT NULL,
 table_name VARCHAR(50) NOT NULL,
 record_id UUID,
 before_state JSONB,
 after_state JSONB,
 ip_address VARCHAR(45),
 user_agent VARCHAR(500),
 created_at TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (created_at);

-- Monthly partitions (auto-generated by cron or pg_partman)
CREATE TABLE audit_logs_2026_01 PARTITION OF audit_logs
 FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE audit_logs_2026_02 PARTITION OF audit_logs
 FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
-- ... continues monthly

-- Cleanup: drop partitions older than 2 years (compliance retention)
-- DROP TABLE audit_logs_2024_01;
```

Index on each partition automatically inherits. Query planner prunes irrelevant partitions.

## Migration Workflow (Alembic)

```bash
# Generate migration after model changes
alembic revision --autogenerate -m "add_invoice_status_enum"

# Apply migrations
alembic upgrade head

# Rollback one step
alembic downgrade -1

# Check current version
alembic current
```

```ini
# alembic.ini
[alembic]
script_location = alembic
sqlalchemy.url = postgresql+asyncpg://user:pass@localhost:5432/equalizer_stake

# alembic/env.py — async support
from sqlalchemy.ext.asyncio import async_engine_from_config

def run_async_migrations():
 connectable = async_engine_from_config(config.get_section(config.config_ini_section), ...)
 async with connectable.connect() as connection:
 await connection.run_sync(do_run_migrations)
```

Migrations run in CI before deployment. Docker entrypoint runs `alembic upgrade head` before starting Uvicorn.
