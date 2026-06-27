"""Exam eligibility service — FR-07 auto-calculate eligibility."""

from uuid import UUID
from datetime import datetime, timezone, timedelta

from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.student import Student
from app.models.attendance import Attendance
from app.models.session import Session


# Business rules
MIN_ATTENDANCE_RATE = 0.90  # ≥90%
MAX_NO_SHOW_3MONTHS = 1
MIN_ENROLLMENT_MONTHS = 3


async def check_eligibility(
    db: AsyncSession,
    student_id: UUID,
) -> dict:
    """Check exam eligibility for a student.

    Criteria:
    1. Active account (enrollment_status == "active")
    2. Attendance rate ≥90% (last 3 months)
    3. Max 1 no-show in last 3 months
    4. Enrolled ≥3 months (new students shown with incomplete data)
    """
    # Get student
    result = await db.execute(
        select(Student).where(Student.id_student == student_id)
    )
    student = result.scalar_one_or_none()

    if not student:
        return {"eligible": False, "reason": "Siswa tidak ditemukan"}

    if student.enrollment_status != "active":
        return {
            "eligible": False,
            "reason": "Akun tidak aktif",
            "status": student.enrollment_status,
        }

    # Check enrollment duration
    three_months_ago = datetime.now(timezone.utc) - timedelta(days=90)
    enrollment_new = student.created_at > three_months_ago if hasattr(student, "created_at") else False

    # Get attendance stats (last 3 months)
    attendance_result = await db.execute(
        select(
            func.count(Attendance.id_attendance).label("total"),
            func.sum(func.cast(Attendance.verification_status == "verified", type_=func.count().type)).label("present"),
            func.sum(func.cast(Attendance.verification_status == "absent_no_show", type_=func.count().type)).label("no_show"),
        )
        .join(Session, Attendance.id_session == Session.id_session)
        .where(
            Session.id_student == student_id,
            Session.session_date >= three_months_ago.date(),
        )
    )
    stats = attendance_result.one_or_none()

    total = stats.total if stats and stats.total else 0
    present = stats.present if stats and stats.present else 0
    no_show = stats.no_show if stats and stats.no_show else 0

    # Calculate attendance rate
    attendance_rate = present / total if total > 0 else 0

    # Check eligibility
    eligible = True
    reasons = []

    if attendance_rate < MIN_ATTENDANCE_RATE:
        eligible = False
        reasons.append(f"Tingkat kehadiran {attendance_rate*100:.0f}% (minimal 90%)")

    if no_show > MAX_NO_SHOW_3MONTHS:
        eligible = False
        reasons.append(f"Alpha {no_show} kali (maksimal 1 dalam 3 bulan)")

    return {
        "eligible": eligible,
        "student_id": str(student_id),
        "student_name": student.full_name,
        "instrument": student.instrument,
        "attendance_rate": round(attendance_rate * 100, 1),
        "total_sessions": total,
        "present": present,
        "no_show": no_show,
        "enrollment_new": enrollment_new,
        "reasons": reasons,
        "note": "Data belum lengkap (kurang dari 3 bulan)" if enrollment_new else None,
    }
