"""Audit log router — query audit logs."""

from uuid import UUID

from fastapi import APIRouter, Depends, Query
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.models.audit_log import AuditLog

router = APIRouter(prefix="/api/audit", tags=["Audit Log"])


@router.get("")
async def list_audit_logs(
    table_name: str | None = Query(None),
    record_id: UUID | None = Query(None),
    changed_by: UUID | None = Query(None),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """List audit logs (Owner: all, Admin: operational only)."""
    base = select(AuditLog)
    if table_name:
        base = base.where(AuditLog.table_name == table_name)
    if record_id:
        base = base.where(AuditLog.record_id == record_id)
    if changed_by:
        base = base.where(AuditLog.changed_by == changed_by)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(AuditLog.created_at.desc()).offset(offset).limit(limit))
    logs = result.scalars().all()
    return {
        "data": [
            {
                "id_audit": str(log.id_audit),
                "table_name": log.table_name,
                "record_id": str(log.record_id),
                "action": log.action,
                "changed_by": str(log.changed_by),
                "created_at": log.created_at.isoformat(),
                "ip_address": log.ip_address,
            }
            for log in logs
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }
