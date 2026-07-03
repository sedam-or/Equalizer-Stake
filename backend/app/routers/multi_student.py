"""Multi-student router — guardian:student management."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.models.student import Student
from app.services.multi_student import (
    get_guardian_students,
    add_student_to_guardian,
    freeze_guardian_household,
)

router = APIRouter(prefix="/api/v1/guardian", tags=["Multi-Student"])


class AddStudentRequest(BaseModel):
    full_name: str
    instrument: str
    level: str = "pemula"


@router.get("/{guardian_id}/students")
async def list_students(
    guardian_id: str,
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner", "guardian")),
    db: AsyncSession = Depends(get_db),
):
    """List all students under guardian."""
    base = select(Student).where(Student.id_guardian == UUID(guardian_id), Student.deleted_at.is_(None))
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Student.created_at.desc()).offset(offset).limit(limit))
    students = result.scalars().all()
    return {
        "data": [
            {
                "id_student": str(s.id_student),
                "full_name": s.full_name,
                "instrument": s.instrument,
                "level": s.level,
                "enrollment_status": s.enrollment_status,
            }
            for s in students
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.post("/{guardian_id}/students", status_code=status.HTTP_201_CREATED)
async def add_student(
    guardian_id: str,
    data: AddStudentRequest,
    user: User = Depends(require_role("admin")),
    db: AsyncSession = Depends(get_db),
):
    """Add student to guardian (Admin only)."""
    try:
        student = await add_student_to_guardian(
            db=db,
            guardian_id=UUID(guardian_id),
            student_data=data.model_dump(),
            created_by=user.id_user,
        )
        return {
            "id_student": str(student.id_student),
            "full_name": student.full_name,
            "message": "Siswa berhasil ditambahkan",
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/{guardian_id}/freeze")
async def freeze_household(
    guardian_id: str,
    reason: str = "Tagihan belum lunas",
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Freeze all students under guardian."""
    count = await freeze_guardian_household(db, UUID(guardian_id), reason)
    return {
        "message": f"{count} siswa dibekukan",
        "frozen_count": count,
    }
