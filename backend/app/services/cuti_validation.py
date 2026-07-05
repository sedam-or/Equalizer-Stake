"""Cuti validation — max 2 months (60 days) per year.

SRS Addendum Section 3: Kebijakan Cuti
"""

from datetime import date, timedelta
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.student import Student

MAX_CUTI_DAYS_PER_YEAR = 60  # 2 months


async def validate_cuti_limit(
    db: AsyncSession,
    student_id: str,
    cuti_start: date,
    cuti_end: date,
) -> tuple[bool, int, int]:
    """Validate cuti doesn't exceed 2 months per year.
    
    Returns: (is_valid, requested_days, remaining_days)
    """
    # Calculate requested cuti days
    requested_days = (cuti_end - cuti_start).days + 1
    
    # Get student's current cuti usage this year
    year_start = date(cuti_start.year, 1, 1)
    year_end = date(cuti_start.year, 12, 31)
    
    result = await db.execute(
        select(Student.cuti_days_used).where(Student.id_student == student_id)
    )
    current_used = result.scalar() or 0
    
    # Calculate remaining days
    remaining = MAX_CUTI_DAYS_PER_YEAR - current_used
    
    is_valid = requested_days <= remaining
    
    return is_valid, requested_days, remaining


async def apply_cuti(
    db: AsyncSession,
    student_id: str,
    cuti_start: date,
    cuti_end: date,
) -> Student:
    """Apply cuti to student."""
    result = await db.execute(
        select(Student).where(Student.id_student == student_id)
    )
    student = result.scalar_one()
    
    requested_days = (cuti_end - cuti_start).days + 1
    
    student.enrollment_status = "cuti"
    student.cuti_start_date = cuti_start
    student.cuti_end_date = cuti_end
    student.cuti_days_used += requested_days
    
    await db.flush()
    return student


async def reactivate_expired_cuti(db: AsyncSession) -> int:
    """Reactivate students whose cuti period has ended.
    
    Called by scheduler job_daily_cuti_check.
    """
    today = date.today()
    
    result = await db.execute(
        select(Student).where(
            Student.enrollment_status == "cuti",
            Student.cuti_end_date < today,
            Student.deleted_at.is_(None),
        )
    )
    students = result.scalars().all()
    
    for student in students:
        student.enrollment_status = "active"
        student.cuti_start_date = None
        student.cuti_end_date = None
    
    await db.flush()
    return len(students)
