"""Reports router — eligibility + owner dashboard KPIs."""

from uuid import UUID

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.eligibility import check_eligibility
from app.services.reports import get_owner_dashboard

router = APIRouter(prefix="/api/reports", tags=["Reports"])


@router.get("/eligibility/{student_id}")
async def get_eligibility(
    student_id: str,
    user: User = Depends(require_role("admin", "owner", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Check exam eligibility for a student."""
    result = await check_eligibility(db, UUID(student_id))
    return result


@router.get("/owner-dashboard")
async def owner_dashboard(
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Aggregate KPI dashboard (Owner only). Read-only."""
    data = await get_owner_dashboard(db)
    return data
