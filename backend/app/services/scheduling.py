"""Scheduling service — session CRUD with conflict validation."""

from datetime import date, time, timedelta
from uuid import UUID

from sqlalchemy import and_, or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.session import Session
from app.schemas.scheduling import SessionCreate, SlotAvailability

# Operational hours
OPERATIONAL_HOURS = {
    "weekday": (time(14, 30), time(17, 0)),  # Mon-Fri
    "saturday": (time(8, 0), time(12, 0)),   # Sat
}
BREAK_MINUTES = 10  # mandatory break between sessions


async def validate_slot(
    db: AsyncSession,
    teacher_id: UUID,
    room_id: UUID,
    student_id: UUID,
    session_date: date,
    start_time: time,
    end_time: time,
    exclude_session_id: UUID | None = None,
) -> SlotAvailability:
    """Validate slot availability (teacher, room, student)."""
    conflicts = []

    # Check operational hours
    weekday = session_date.weekday()
    if weekday == 6:  # Sunday
        return SlotAvailability(available=False, conflicts=[], message="Hari Minggu tutup")

    # Build time range with break
    from datetime import datetime as dt
    start_dt = dt.combine(session_date, start_time)
    end_dt = dt.combine(session_date, end_time)
    break_start = end_dt
    break_end = end_dt + timedelta(minutes=BREAK_MINUTES)

    # Query existing sessions for same date
    query = select(Session).where(
        and_(
            Session.session_date == session_date,
            Session.status.notin_(["cancelled"]),
            Session.id_session != exclude_session_id if exclude_session_id else True,
        )
    )
    result = await db.execute(query)
    existing = result.scalars().all()

    for s in existing:
        s_start = dt.combine(session_date, s.start_time)
        s_end = dt.combine(session_date, s.end_time)
        s_break_end = s_end + timedelta(minutes=BREAK_MINUTES)

        # Check overlap (including break)
        if not (end_dt <= s_start or start_dt >= s_break_end):
            if s.id_teacher == teacher_id:
                conflicts.append({"conflict_type": "teacher", "message": f"Guru sudah ada sesi lain"})
            if s.id_room == room_id:
                conflicts.append({"conflict_type": "room", "message": f"Ruangan sudah digunakan"})
            if s.id_student == student_id:
                conflicts.append({"conflict_type": "student", "message": f"Siswa sudah ada sesi lain"})

    return SlotAvailability(available=len(conflicts) == 0, conflicts=conflicts)


async def create_session(db: AsyncSession, data: SessionCreate) -> Session:
    """Create new session after conflict validation."""
    session = Session(
        id_student=data.id_student,
        id_teacher=data.id_teacher,
        id_room=data.id_room,
        session_date=data.session_date,
        start_time=data.start_time,
        end_time=data.end_time,
        session_type=data.session_type,
        status="pending",
    )
    db.add(session)
    await db.commit()
    await db.refresh(session)
    return session
