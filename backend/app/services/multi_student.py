"""Guardian multi-student service — 1:N guardian:student."""

from uuid import UUID
from datetime import datetime, timezone

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.guardian import Guardian
from app.models.student import Student


async def get_guardian_students(
    db: AsyncSession,
    guardian_id: UUID,
) -> list[Student]:
    """Get all students under a guardian."""
    result = await db.execute(
        select(Student).where(Student.id_guardian == guardian_id)
    )
    return list(result.scalars().all())


async def add_student_to_guardian(
    db: AsyncSession,
    guardian_id: UUID,
    student_data: dict,
    created_by: UUID,
) -> Student:
    """Add student to guardian (Admin only).

    Business rules:
    - Only Admin can add student
    - Student must complete registration + audition first
    - Guardian account must be active
    """
    # Check guardian exists and active
    result = await db.execute(
        select(Guardian).where(Guardian.id_guardian == guardian_id)
    )
    guardian = result.scalar_one_or_none()
    if not guardian or guardian.account_status == "inactive":
        raise ValueError("Akun wali murid tidak aktif")

    # Create student
    student = Student(
        id_guardian=guardian_id,
        full_name=student_data["full_name"],
        instrument=student_data["instrument"],
        level=student_data.get("level", "pemula"),
        enrollment_status="active",
    )
    db.add(student)
    await db.commit()
    await db.refresh(student)
    return student


async def freeze_guardian_household(
    db: AsyncSession,
    guardian_id: UUID,
    reason: str,
) -> int:
    """Freeze ALL students under guardian.

    Business rule: Freeze applies to household, not individual.
    """
    students = await get_guardian_students(db, guardian_id)
    for student in students:
        student.enrollment_status = "frozen"

    # Update guardian status
    result = await db.execute(
        select(Guardian).where(Guardian.id_guardian == guardian_id)
    )
    guardian = result.scalar_one_or_none()
    if guardian:
        guardian.account_status = "frozen"
        guardian.frozen_at = datetime.now(timezone.utc)
        guardian.frozen_reason = reason

    await db.commit()
    return len(students)


async def get_guardian_invoice_total(
    db: AsyncSession,
    guardian_id: UUID,
    billing_period: str,
) -> int:
    """Get total student count for invoice (1 invoice per guardian)."""
    students = await get_guardian_students(db, guardian_id)
    return len([s for s in students if s.enrollment_status == "active"])
