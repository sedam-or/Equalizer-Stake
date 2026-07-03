"""Student router — CRUD for admin. T-093: audit logs on all mutations."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from pydantic import BaseModel, Field
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.student import Student
from app.models.user import User
from app.services.audit import create_audit_log

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


def _student_dict(s: Student) -> dict:
    """Serialize student for audit before/after state."""
    return {
        "id_student": str(s.id_student),
        "id_guardian": str(s.id_guardian),
        "full_name": s.full_name,
        "instrument": s.instrument,
        "level": s.level,
        "enrollment_status": s.enrollment_status,
        "enrollment_date": str(s.enrollment_date) if s.enrollment_date else None,
        "eligible_for_exam": s.eligible_for_exam,
    }


@router.get("")
async def list_students(
    guardian_id: UUID | None = Query(None),
    status_filter: str | None = Query(None, alias="status"),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """List all students (admin/owner)."""
    base = select(Student).where(Student.deleted_at.is_(None))
    if guardian_id:
        base = base.where(Student.id_guardian == guardian_id)
    if status_filter:
        base = base.where(Student.enrollment_status == status_filter)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Student.created_at.desc()).offset(offset).limit(limit))
    students = result.scalars().all()
    return {
        "data": [
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
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.post("/", status_code=201)
async def create_student(
    body: StudentCreate,
    request: Request,
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
    await db.flush()
    await create_audit_log(
        db, table_name="students", record_id=student.id_student,
        action="INSERT", changed_by=user.id_user,
        after_state=_student_dict(student), request=request,
    )
    await db.commit()
    return {"id_student": str(student.id_student), "detail": "Siswa berhasil ditambahkan"}


@router.patch("/{student_id}")
async def update_student(
    student_id: UUID,
    body: StudentUpdate,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Update student."""
    result = await db.execute(select(Student).where(Student.id_student == student_id, Student.deleted_at.is_(None)))
    student = result.scalar_one_or_none()
    if not student:
        raise HTTPException(status_code=404, detail="Siswa tidak ditemukan")
    before = _student_dict(student)
    if body.full_name is not None: student.full_name = body.full_name
    if body.instrument is not None: student.instrument = body.instrument
    if body.level is not None: student.level = body.level
    if body.enrollment_status is not None: student.enrollment_status = body.enrollment_status
    after = _student_dict(student)
    await create_audit_log(
        db, table_name="students", record_id=student_id,
        action="UPDATE", changed_by=user.id_user,
        before_state=before, after_state=after, request=request,
    )
    await db.commit()
    return {"detail": "Siswa berhasil diperbarui"}


@router.delete("/{student_id}")
async def delete_student(
    student_id: UUID,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Soft-delete student."""
    from datetime import datetime, timezone
    result = await db.execute(select(Student).where(Student.id_student == student_id, Student.deleted_at.is_(None)))
    student = result.scalar_one_or_none()
    if not student:
        raise HTTPException(status_code=404, detail="Siswa tidak ditemukan")
    before = _student_dict(student)
    student.deleted_at = datetime.now(timezone.utc)
    await create_audit_log(
        db, table_name="students", record_id=student_id,
        action="DELETE", changed_by=user.id_user,
        before_state=before, request=request,
    )
    await db.commit()
    return {"detail": "Siswa berhasil dihapus"}


@router.post("/{student_id}/restore")
async def restore_student(
    student_id: UUID,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Restore soft-deleted student."""
    result = await db.execute(select(Student).where(Student.id_student == student_id, Student.deleted_at.is_not(None)))
    student = result.scalar_one_or_none()
    if not student:
        raise HTTPException(status_code=404, detail="Siswa tidak ditemukan atau belum dihapus")
    student.deleted_at = None
    after = _student_dict(student)
    await create_audit_log(
        db, table_name="students", record_id=student_id,
        action="UPDATE", changed_by=user.id_user,
        after_state=after, request=request,
    )
    await db.commit()
    return {"detail": "Siswa berhasil dipulihkan"}
