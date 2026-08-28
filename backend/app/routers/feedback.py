"""Feedback router — Instructor → Guardian feedback."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from pydantic import BaseModel, Field

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import create_audit_log

router = APIRouter(prefix="/api/v1/feedback", tags=["Feedback"])


class FeedbackCreate(BaseModel):
    session_id: str
    rating: int = Field(ge=1, le=5)
    comment: str | None = None
    category: str = "general"


@router.post("/")
async def create_feedback(
    data: FeedbackCreate,
    user: User = Depends(require_role("teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Instructor kirim feedback ke guardian."""
    from app.models.feedback import Feedback
    from app.models.session import Session
    from app.models.teacher import Teacher
    from app.models.student import Student

    # Verify session exists
    result = await db.execute(select(Session).where(Session.id_session == UUID(data.session_id)))
    session = result.scalar_one_or_none()
    if not session:
        raise HTTPException(status_code=404, detail="Sesi tidak ditemukan")

    # Verify teacher owns this session
    teacher_result = await db.execute(
        select(Teacher).where(Teacher.id_user == user.id_user)
    )
    teacher = teacher_result.scalar_one_or_none()
    if not teacher or session.id_teacher != teacher.id_teacher:
        raise HTTPException(status_code=403, detail="Anda bukan pemilik sesi ini")

    # Get student → guardian
    student_result = await db.execute(
        select(Student).where(Student.id_student == session.id_student)
    )
    student = student_result.scalar_one_or_none()
    if not student:
        raise HTTPException(status_code=404, detail="Siswa tidak ditemukan")

    # Create feedback
    feedback = Feedback(
        id_session=UUID(data.session_id),
        id_teacher=teacher.id_teacher,
        id_guardian=student.id_guardian,
        rating=data.rating,
        comment=data.comment,
        category=data.category,
    )
    db.add(feedback)
    await db.flush()  # get UUID before audit log

    await create_audit_log(
        db,
        table_name="feedbacks",
        record_id=feedback.id_feedback,
        action="INSERT",
        changed_by=user.id_user,
        after_state={"rating": data.rating, "category": data.category},
    )

    await db.commit()
    await db.refresh(feedback)

    return {
        "id_feedback": str(feedback.id_feedback),
        "rating": feedback.rating,
        "category": feedback.category,
        "message": "Feedback berhasil dikirim",
    }


@router.get("/")
async def list_my_feedback(
    user: User = Depends(require_role("guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Guardian lihat feedback dari instructor."""
    from app.models.feedback import Feedback
    from app.models.guardian import Guardian
    from sqlalchemy import select

    # Get guardian
    result = await db.execute(
        select(Guardian).where(Guardian.id_user == user.id_user)
    )
    guardian = result.scalar_one_or_none()
    if not guardian:
        return {"data": [], "total": 0}

    # Get feedback for this guardian
    feedback_q = await db.execute(
        select(Feedback)
        .where(Feedback.id_guardian == guardian.id_guardian)
        .order_by(Feedback.created_at.desc())
    )
    feedbacks = feedback_q.scalars().all()

    return {
        "data": [
            {
                "id_feedback": str(f.id_feedback),
                "id_session": str(f.id_session),
                "rating": f.rating,
                "comment": f.comment,
                "category": f.category,
                "created_at": f.created_at.isoformat() if f.created_at else None,
            }
            for f in feedbacks
        ],
        "total": len(feedbacks),
    }
