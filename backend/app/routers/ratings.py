"""Ratings router — Wali Murid → Guru rating."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from pydantic import BaseModel, Field

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import create_audit_log

router = APIRouter(prefix="/api/v1/ratings", tags=["Ratings"])


class RatingCreate(BaseModel):
    teacher_id: str
    rating: int = Field(ge=1, le=5)
    comment: str | None = None


@router.post("/")
async def create_rating(
    data: RatingCreate,
    user: User = Depends(require_role("guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Wali murid rate guru."""
    from app.models.rating import Rating
    from app.models.guardian import Guardian
    from app.models.teacher import Teacher

    # Get guardian
    result = await db.execute(
        select(Guardian).where(Guardian.id_user == user.id_user)
    )
    guardian = result.scalar_one_or_none()
    if not guardian:
        raise HTTPException(status_code=404, detail="Profil wali murid tidak ditemukan")

    # Verify teacher exists
    teacher_result = await db.execute(
        select(Teacher).where(Teacher.id_teacher == UUID(data.teacher_id))
    )
    teacher = teacher_result.scalar_one_or_none()
    if not teacher:
        raise HTTPException(status_code=404, detail="Guru tidak ditemukan")

    # Create rating
    rating = Rating(
        id_teacher=UUID(data.teacher_id),
        id_guardian=guardian.id_guardian,
        rating=data.rating,
        comment=data.comment,
    )
    db.add(rating)
    await db.flush()

    await create_audit_log(
        db,
        table_name="ratings",
        record_id=rating.id_rating,
        action="INSERT",
        changed_by=user.id_user,
        after_state={"rating": data.rating, "teacher_id": data.teacher_id},
    )

    await db.commit()
    await db.refresh(rating)

    return {
        "id_rating": str(rating.id_rating),
        "rating": rating.rating,
        "message": "Rating berhasil dikirim",
    }


@router.get("/{teacher_id}")
async def get_teacher_ratings(
    teacher_id: str,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Admin lihat rating guru + rata-rata."""
    from app.models.rating import Rating

    result = await db.execute(
        select(Rating).where(Rating.id_teacher == UUID(teacher_id))
        .order_by(Rating.created_at.desc())
    )
    ratings = result.scalars().all()

    # Calculate average
    avg_result = await db.execute(
        select(func.avg(Rating.rating), func.count(Rating.id_rating))
        .where(Rating.id_teacher == UUID(teacher_id))
    )
    avg_data = avg_result.one()
    avg_rating = float(avg_data[0]) if avg_data[0] else 0
    total_count = avg_data[1] or 0

    return {
        "teacher_id": teacher_id,
        "average_rating": round(avg_rating, 2),
        "total_ratings": total_count,
        "ratings": [
            {
                "id_rating": str(r.id_rating),
                "rating": r.rating,
                "comment": r.comment,
                "created_at": r.created_at.isoformat() if r.created_at else None,
            }
            for r in ratings
        ],
    }
