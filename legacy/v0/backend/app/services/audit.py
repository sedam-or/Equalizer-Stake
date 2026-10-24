"""Audit log service — immutable audit trail for all critical data changes."""

import uuid
from uuid import UUID
from datetime import datetime, timezone
from typing import Any

from fastapi import Request
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.audit_log import AuditLog


async def create_audit_log(
    db: AsyncSession,
    table_name: str,
    record_id: UUID,
    action: str,
    changed_by: UUID,
    before_state: dict[str, Any] | None = None,
    after_state: dict[str, Any] | None = None,
    request: Request | None = None,
    category: str | None = "operasional",
) -> AuditLog | None:
    """Create immutable audit log entry.
    
    If log creation fails, the caller should rollback the data change.
    """
    try:
        ip_address = None
        user_agent = None

        if request:
            ip_address = request.client.host if request.client else None
            user_agent = request.headers.get("user-agent")

        log_entry = AuditLog(
            table_name=table_name,
            record_id=record_id,
            action=action,
            before_state=before_state,
            after_state=after_state,
            changed_by=changed_by,
            ip_address=ip_address,
            user_agent=user_agent,
            category=category,
        )
        db.add(log_entry)
        # Do NOT commit here — caller commits atomically with data change
        return log_entry
    except Exception:
        # If audit log creation fails, caller must rollback
        return None


async def log_auth_event(
    db: AsyncSession,
    action: str,
    user_id: UUID | None = None,
    details: str | None = None,
    request: Request | None = None,
) -> AuditLog | None:
    """Log authentication events (LOGIN_SUCCESS, LOGIN_FAILED, REGISTER, PASSWORD_CHANGE, ACCOUNT_LOCKED)."""
    ip_address = None
    user_agent = None
    if request:
        ip_address = request.client.host if request.client else None
        user_agent = request.headers.get("user-agent")

    log_entry = AuditLog(
        table_name="auth",
        record_id=user_id or uuid.uuid4(),
        action=action,
        changed_by=user_id,
        details=details,
        ip_address=ip_address,
        user_agent=user_agent,
    )
    db.add(log_entry)
    return log_entry


async def get_audit_logs(
    db: AsyncSession,
    table_name: str | None = None,
    record_id: UUID | None = None,
    changed_by: UUID | None = None,
    limit: int = 100,
    offset: int = 0,
) -> list[AuditLog]:
    """Query audit logs with filters."""
    query = select(AuditLog)

    if table_name:
        query = query.where(AuditLog.table_name == table_name)
    if record_id:
        query = query.where(AuditLog.record_id == record_id)
    if changed_by:
        query = query.where(AuditLog.changed_by == changed_by)

    query = query.order_by(AuditLog.created_at.desc()).offset(offset).limit(limit)
    result = await db.execute(query)
    return list(result.scalars().all())


def model_to_dict(instance, exclude: set[str] | None = None) -> dict[str, Any]:
    """Convert SQLAlchemy model instance to dict for audit logging."""
    exclude = exclude or {"created_at", "updated_at", "deleted_at"}
    return {
        c.key: str(getattr(instance, c.key)) if isinstance(getattr(instance, c.key), UUID) else getattr(instance, c.key)
        for c in instance.__table__.columns
        if c.key not in exclude
    }
