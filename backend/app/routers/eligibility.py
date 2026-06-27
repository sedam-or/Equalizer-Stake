"""Eligibility router — exam eligibility check."""

from uuid import UUID

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.eligibility import check_eligibility

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
