"""Student router — CRUD for admin."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.student import Student
from app.models.user import User

router = APIRouter(prefix="/api/students", tags=["Students"])


class StudentCreate(BaseModel):
    id_guardian: UUID
    full_name: str = Field(min_length=1, max_length=255)
    instrument: str
    level: str = "pemula"
    enrollment_date: str  # YYYY-MM-DD


class StudentUpdate(BaseModel):
    full_name: str | None = None
    instrument: str | None = None
    level: str | None = None
    enrollment_status: str | None = None


@router.get("")
async def list_students(
    guardian_id: UUID | None = Query(None),
    status_filter: str | None = Query(None, alias="status"),
    limit: int = Query(100, le=500),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """List all students (admin/owner)."""
    query = select(Student).where(Student.deleted_at.is_(None))
    if guardian_id:
        query = query.where(Student.id_guardian == guardian_id)
    if status_filter:
        query = query.where(Student.enrollment_status == status_filter)
    query = query.order_by(Student.created_at.desc()).offset(offset).limit(limit)
    result = await db.execute(query)
    students = result.scalars().all()
    return [
        {
            "id_student": str(s.id_student),
            "id_guardian": str(s.id_guardian),
            "full_name": s.full_name,
            "instrument": s.instrument,
            "level": s.level,
            "enrollment_status": s.enrollment_status,
            "enrollment_date": str(s.enrollment_date),
            "eligible_for_exam": s.eligible_for_exam,
        }
        for s in students
    ]


@router.post("/", status_code=201)
async def create_student(
    body: StudentCreate,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Create new student."""
    from datetime import date as dt_date
    student = Student(
        id_guardian=body.id_guardian,
        full_name=body.full_name,
        instrument=body.instrument,
        level=body.level,
        enrollment_status="active",
        enrollment_date=dt_date.fromisoformat(body.enrollment_date),
    )
    db.add(student)
    await db.commit()
    return {"id_student": str(student.id_student), "detail": "Siswa berhasil ditambahkan"}


@router.patch("/{student_id}")
async def update_student(
    student_id: UUID,
    body: StudentUpdate,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Update student."""
    result = await db.execute(select(Student).where(Student.id_student == student_id, Student.deleted_at.is_(None)))
    student = result.scalar_one_or_none()
    if not student:
        raise HTTPException(status_code=404, detail="Siswa tidak ditemukan")
    if body.full_name is not None: student.full_name = body.full_name
    if body.instrument is not None: student.instrument = body.instrument
    if body.level is not None: student.level = body.level
    if body.enrollment_status is not None: student.enrollment_status = body.enrollment_status
    await db.commit()
    return {"detail": "Siswa berhasil diperbarui"}


@router.delete("/{student_id}")
async def delete_student(
    student_id: UUID,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Soft-delete student."""
    from datetime import datetime, timezone
    result = await db.execute(select(Student).where(Student.id_student == student_id, Student.deleted_at.is_(None)))
    student = result.scalar_one_or_none()
    if not student:
        raise HTTPException(status_code=404, detail="Siswa tidak ditemukan")
    student.deleted_at = datetime.now(timezone.utc)
    await db.commit()
    return {"detail": "Siswa berhasil dihapus"}
