"""Audit log router — query audit logs."""

from uuid import UUID

from fastapi import APIRouter, Depends, Query
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import get_audit_logs

router = APIRouter(prefix="/api/audit", tags=["Audit Log"])


@router.get("/")
async def list_audit_logs(
    table_name: str | None = Query(None),
    record_id: UUID | None = Query(None),
    changed_by: UUID | None = Query(None),
    limit: int = Query(100, le=500),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """List audit logs (Owner: all, Admin: operational only)."""
    logs = await get_audit_logs(db, table_name, record_id, changed_by, limit, offset)
    return [
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
    ]
