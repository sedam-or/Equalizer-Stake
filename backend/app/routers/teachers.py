"""Teacher router — admin CRUD for teacher profiles."""

import uuid
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.teacher import Teacher
from app.models.user import User
from app.services.audit import create_audit_log
from app.schemas.teacher import TeacherCreate, TeacherUpdate, TeacherResponse

router = APIRouter(prefix="/api/v1/teachers", tags=["Teachers"])


@router.get("", response_model=list[TeacherResponse])
async def list_teachers(
    is_active: bool | None = None,
    db: AsyncSession = Depends(get_db),
    user: User = Depends(require_role("admin", "owner")),
):
    """Admin/Owner: list all teachers."""
    base = select(Teacher, User.username).join(User, Teacher.id_user == User.id_user)
    if is_active is not None:
        base = base.where(Teacher.is_active == is_active)
    base = base.order_by(Teacher.full_name)
    result = await db.execute(base)
    rows = result.all()
    return [
        TeacherResponse(
            id_teacher=str(t.id_teacher),
            id_user=str(t.id_user),
            full_name=t.full_name,
            instruments_specialization=list(t.instruments_specialization),
            employment_type=t.employment_type,
            max_students_per_session=t.max_students_per_session,
            is_active=t.is_active,
            username=username,
        )
        for t, username in rows
    ]


@router.post("", response_model=TeacherResponse, status_code=status.HTTP_201_CREATED)
async def create_teacher(
    data: TeacherCreate,
    db: AsyncSession = Depends(get_db),
    user: User = Depends(require_role("admin")),
):
    """Admin: create teacher profile (User must already exist with role=teacher)."""
    # Verify user exists and has role=teacher
    target_user = await db.get(User, data.id_user)
    if not target_user:
        raise HTTPException(status_code=404, detail="User tidak ditemukan")
    if target_user.role != "teacher":
        raise HTTPException(status_code=400, detail="User bukan role teacher")
    # Check no existing teacher profile
    existing = await db.execute(select(Teacher).where(Teacher.id_user == data.id_user))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=400, detail="Teacher profile sudah ada")

    teacher = Teacher(
        id_user=data.id_user,
        full_name=data.full_name,
        instruments_specialization=data.instruments_specialization,
        employment_type=data.employment_type,
        max_students_per_session=data.max_students_per_session,
    )
    db.add(teacher)
    await db.flush()
    return TeacherResponse(
        id_teacher=str(teacher.id_teacher),
        id_user=str(teacher.id_user),
        full_name=teacher.full_name,
        instruments_specialization=teacher.instruments_specialization,
        employment_type=teacher.employment_type,
        max_students_per_session=teacher.max_students_per_session,
        is_active=teacher.is_active,
        username=target_user.username,
    )


@router.get("/{teacher_id}", response_model=TeacherResponse)
async def get_teacher(
    teacher_id: uuid.UUID,
    db: AsyncSession = Depends(get_db),
    user: User = Depends(require_role("admin", "owner")),
):
    """Admin/Owner: get teacher profile."""
    teacher = await db.get(Teacher, teacher_id)
    if not teacher:
        raise HTTPException(status_code=404, detail="Teacher tidak ditemukan")
    target_user = await db.get(User, teacher.id_user)
    return TeacherResponse(
        id_teacher=str(teacher.id_teacher),
        id_user=str(teacher.id_user),
        full_name=teacher.full_name,
        instruments_specialization=teacher.instruments_specialization,
        employment_type=teacher.employment_type,
        max_students_per_session=teacher.max_students_per_session,
        is_active=teacher.is_active,
        username=target_user.username if target_user else None,
    )


@router.put("/{teacher_id}", response_model=TeacherResponse)
async def update_teacher(
    teacher_id: uuid.UUID,
    data: TeacherUpdate,
    db: AsyncSession = Depends(get_db),
    user: User = Depends(require_role("admin")),
):
    """Admin: update teacher profile."""
    teacher = await db.get(Teacher, teacher_id)
    if not teacher:
        raise HTTPException(status_code=404, detail="Teacher tidak ditemukan")

    update_data = data.model_dump(exclude_unset=True)
    for key, value in update_data.items():
        setattr(teacher, key, value)
    await db.flush()

    target_user = await db.get(User, teacher.id_user)
    return TeacherResponse(
        id_teacher=str(teacher.id_teacher),
        id_user=str(teacher.id_user),
        full_name=teacher.full_name,
        instruments_specialization=teacher.instruments_specialization,
        employment_type=teacher.employment_type,
        max_students_per_session=teacher.max_students_per_session,
        is_active=teacher.is_active,
        username=target_user.username if target_user else None,
    )


@router.delete("/{teacher_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_teacher(
    teacher_id: uuid.UUID,
    db: AsyncSession = Depends(get_db),
    user: User = Depends(require_role("admin")),
):
    """Admin: soft delete teacher."""
    teacher = await db.get(Teacher, teacher_id)
    if not teacher:
        raise HTTPException(status_code=404, detail="Teacher tidak ditemukan")
    from datetime import datetime, timezone
    teacher.deleted_at = datetime.now(timezone.utc)
    teacher.is_active = False
    await db.flush()
