"""Scheduling service — session CRUD with conflict validation."""

from datetime import date, time, timedelta
from uuid import UUID

from sqlalchemy import and_, or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.guardian import Guardian
from app.models.room import Room
from app.models.session import Session
from app.models.student import Student
from app.models.teacher import Teacher
from app.schemas.scheduling import SessionCreate, SlotAvailability

# Operational hours
STUDIO_OPEN = time(8, 0)   # 08:00 WIB
STUDIO_CLOSE = time(21, 0)  # 21:00 WIB
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

    # Query existing sessions for same date (SELECT FOR UPDATE to prevent race conditions)
    query = select(Session).where(
        and_(
            Session.session_date == session_date,
            Session.status.notin_(["cancelled"]),
            Session.id_session != exclude_session_id if exclude_session_id else True,
        )
    ).with_for_update()
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
    # Operational hours validation (08:00-21:00 WIB)
    if data.start_time < STUDIO_OPEN or data.end_time > STUDIO_CLOSE:
        raise ValueError(
            f"Sesi di luar jam operasional studio (08:00-21:00 WIB). "
            f"Jam yang diminta: {data.start_time.strftime('%H:%M')}-{data.end_time.strftime('%H:%M')}"
        )
    if data.start_time >= data.end_time:
        raise ValueError("Jam mulai harus sebelum jam selesai")

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


async def validate_session_ownership(
    db: AsyncSession, session_id: UUID, user_id: UUID
) -> bool:
    """Verify teacher owns the session (via teacher.id_user link)."""
    result = await db.execute(
        select(Session)
        .join(Teacher, Session.id_teacher == Teacher.id_teacher)
        .where(Session.id_session == session_id, Teacher.id_user == user_id)
    )
    return result.scalar_one_or_none() is not None


async def suggest_alternative_slots(
    db: AsyncSession,
    room_id: UUID,
    session_date: date,
    start_time: time,
    end_time: time,
    max_suggestions: int = 3,
) -> list[dict]:
    """Suggest up to 3 alternative slots per SRS 4.2 priority rules.

    Priority 1: Different room, same time
    Priority 2: Same room, nearby time (±1 hour)
    Priority 3: Different room, nearby time (±1 hour)
    """
    from datetime import datetime as dt

    duration = dt.combine(session_date, end_time) - dt.combine(session_date, start_time)
    suggestions: list[dict] = []

    # Get all rooms
    rooms_result = await db.execute(select(Room).where(Room.is_available == True))
    all_rooms = rooms_result.scalars().all()
    other_rooms = [r for r in all_rooms if r.id_room != room_id]

    # Get existing sessions for that date
    existing_result = await db.execute(
        select(Session).where(
            Session.session_date == session_date,
            Session.status.notin_(["cancelled"]),
        )
    )
    existing = existing_result.scalars().all()

    def is_slot_free(rid: UUID, s_time: time, e_time: time) -> bool:
        s_dt = dt.combine(session_date, s_time)
        e_dt = dt.combine(session_date, e_time)
        brk = e_dt + timedelta(minutes=BREAK_MINUTES)
        for s in existing:
            if s.id_room != rid:
                continue
            ss = dt.combine(session_date, s.start_time)
            se = dt.combine(session_date, s.end_time)
            sb = se + timedelta(minutes=BREAK_MINUTES)
            if not (e_dt <= ss or s_dt >= sb):
                return False
        return True

    # Priority 1: different room, same time
    for r in other_rooms:
        if len(suggestions) >= max_suggestions:
            break
        if is_slot_free(r.id_room, start_time, end_time):
            suggestions.append({
                "room_id": str(r.id_room),
                "room_name": r.room_name,
                "start_time": start_time.strftime("%H:%M"),
                "end_time": end_time.strftime("%H:%M"),
                "priority": 1,
            })

    # Priority 2: same room, nearby time (±1 hour)
    for delta_hours in [1, -1, 2, -2]:
        if len(suggestions) >= max_suggestions:
            break
        new_start_dt = dt.combine(session_date, start_time) + timedelta(hours=delta_hours)
        new_start = new_start_dt.time()
        new_end_dt = new_start_dt + duration
        new_end = new_end_dt.time()
        if new_start < STUDIO_OPEN or new_end > STUDIO_CLOSE:
            continue
        if is_slot_free(room_id, new_start, new_end):
            # Avoid duplicate
            dup = any(
                s["room_id"] == str(room_id)
                and s["start_time"] == new_start.strftime("%H:%M")
                for s in suggestions
            )
            if not dup:
                suggestions.append({
                    "room_id": str(room_id),
                    "start_time": new_start.strftime("%H:%M"),
                    "end_time": new_end.strftime("%H:%M"),
                    "priority": 2,
                })

    # Priority 3: different room, nearby time (±1 hour)
    for r in other_rooms:
        if len(suggestions) >= max_suggestions:
            break
        for delta_hours in [1, -1]:
            if len(suggestions) >= max_suggestions:
                break
            new_start_dt = dt.combine(session_date, start_time) + timedelta(hours=delta_hours)
            new_start = new_start_dt.time()
            new_end_dt = new_start_dt + duration
            new_end = new_end_dt.time()
            if new_start < STUDIO_OPEN or new_end > STUDIO_CLOSE:
                continue
            if is_slot_free(r.id_room, new_start, new_end):
                dup = any(
                    s["room_id"] == str(r.id_room)
                    and s["start_time"] == new_start.strftime("%H:%M")
                    for s in suggestions
                )
                if not dup:
                    suggestions.append({
                        "room_id": str(r.id_room),
                        "room_name": r.room_name,
                        "start_time": new_start.strftime("%H:%M"),
                        "end_time": new_end.strftime("%H:%M"),
                        "priority": 3,
                    })

    return suggestions[:max_suggestions]


async def _validate_room_slot_no_teacher(
    db: AsyncSession,
    room_id: UUID,
    student_id: UUID,
    session_date: date,
    start_time: time,
    end_time: time,
) -> list[dict]:
    """Validate room + student conflicts (no teacher check for guardian booking)."""
    from datetime import datetime as dt

    conflicts: list[dict] = []

    weekday = session_date.weekday()
    if weekday == 6:
        return [{"conflict_type": "general", "message": "Hari Minggu studio tutup"}]

    if start_time < STUDIO_OPEN or end_time > STUDIO_CLOSE:
        return [{"conflict_type": "general", "message": f"Jam di luar operasional (09:00-21:00 WIB)"}]

    if start_time >= end_time:
        return [{"conflict_type": "general", "message": "Jam mulai harus sebelum jam selesai"}]

    duration_dt = dt.combine(session_date, end_time) - dt.combine(session_date, start_time)
    if duration_dt < timedelta(hours=1):
        conflicts.append({"conflict_type": "general", "message": "Durasi minimum 1 jam"})
    if duration_dt > timedelta(hours=4):
        conflicts.append({"conflict_type": "general", "message": "Durasi maksimum 4 jam"})
    if conflicts:
        return conflicts

    start_dt = dt.combine(session_date, start_time)
    end_dt = dt.combine(session_date, end_time)

    query = select(Session).where(
        and_(
            Session.session_date == session_date,
            Session.status.notin_(["cancelled"]),
        )
    ).with_for_update()
    result = await db.execute(query)
    existing = result.scalars().all()

    for s in existing:
        s_start = dt.combine(session_date, s.start_time)
        s_end = dt.combine(session_date, s.end_time)
        s_break_end = s_end + timedelta(minutes=BREAK_MINUTES)
        if not (end_dt <= s_start or start_dt >= s_break_end):
            if s.id_room == room_id:
                conflicts.append({"conflict_type": "room", "message": "Ruangan sudah digunakan"})
            if s.id_student == student_id:
                conflicts.append({"conflict_type": "student", "message": "Siswa sudah ada sesi lain"})

    return conflicts


async def _validate_guardian_owns_student(
    db: AsyncSession, guardian_user_id: UUID, student_id: UUID
) -> bool:
    """Verify guardian owns the student."""
    result = await db.execute(
        select(Student)
        .join(Guardian, Student.id_guardian == Guardian.id_guardian)
        .where(Student.id_student == student_id, Guardian.id_user == guardian_user_id)
    )
    return result.scalar_one_or_none() is not None


async def book_studio_rehearsal(
    db: AsyncSession,
    user_id: UUID,
    student_id: UUID,
    room_id: UUID,
    session_date: date,
    start_time: time,
    end_time: time,
) -> Session:
    """Guardian self-booking for rehearsal studio session."""
    # Validate guardian owns student
    if not await _validate_guardian_owns_student(db, user_id, student_id):
        raise PermissionError("Anda tidak memiliki akses ke data siswa ini")

    # Booking window: max 14 days ahead
    if session_date > date.today() + timedelta(days=14):
        raise ValueError("Booking maksimal 14 hari ke depan")

    if session_date < date.today():
        raise ValueError("Tidak bisa memesan tanggal yang sudah lewat")

    # Validate conflicts
    conflicts = await _validate_room_slot_no_teacher(db, room_id, student_id, session_date, start_time, end_time)
    if conflicts:
        alternatives = await suggest_alternative_slots(db, room_id, session_date, start_time, end_time)
        raise ConflictError(conflicts=conflicts, alternatives=alternatives)

    session = Session(
        id_student=student_id,
        # ponytail: no teacher assigned at booking; admin assigns later if needed
        id_teacher=student_id,  # placeholder — real flow assigns teacher separately
        id_room=room_id,
        session_date=session_date,
        start_time=start_time,
        end_time=end_time,
        session_type="rehearsal",
        status="pending",
    )
    db.add(session)
    await db.flush()
    return session


async def request_recording_session(
    db: AsyncSession,
    user_id: UUID,
    student_id: UUID,
    room_id: UUID,
    session_date: date,
    start_time: time,
    end_time: time,
) -> Session:
    """Guardian requests a recording session (pending admin approval)."""
    if not await _validate_guardian_owns_student(db, user_id, student_id):
        raise PermissionError("Anda tidak memiliki akses ke data siswa ini")

    if session_date > date.today() + timedelta(days=30):
        raise ValueError("Booking recording maksimal 30 hari ke depan")

    if session_date < date.today():
        raise ValueError("Tidak bisa memesan tanggal yang sudah lewat")

    conflicts = await _validate_room_slot_no_teacher(db, room_id, student_id, session_date, start_time, end_time)
    if conflicts:
        alternatives = await suggest_alternative_slots(db, room_id, session_date, start_time, end_time)
        raise ConflictError(conflicts=conflicts, alternatives=alternatives)

    session = Session(
        id_student=student_id,
        id_teacher=student_id,  # placeholder — assigned on approval
        id_room=room_id,
        session_date=session_date,
        start_time=start_time,
        end_time=end_time,
        session_type="recording",
        status="pending_approval",
    )
    db.add(session)
    await db.flush()
    return session


async def approve_recording(
    db: AsyncSession,
    session_id: UUID,
    teacher_id: UUID,
) -> Session:
    """Admin approves a pending recording session."""
    result = await db.execute(
        select(Session).where(Session.id_session == session_id)
    )
    session = result.scalar_one_or_none()
    if not session:
        raise LookupError("Sesi tidak ditemukan")
    if session.status != "pending_approval":
        raise ValueError(f"Sesi tidak dalam status pending_approval. Status saat ini: {session.status}")

    # Re-validate slot now that teacher is assigned
    conflicts = await validate_slot(
        db,
        teacher_id=teacher_id,
        room_id=session.id_room,
        student_id=session.id_student,
        session_date=session.session_date,
        start_time=session.start_time,
        end_time=session.end_time,
        exclude_session_id=session.id_session,
    )
    if not conflicts.available:
        raise ConflictError(conflicts=conflicts.conflicts, alternatives=[])

    session.id_teacher = teacher_id
    session.status = "approved"
    await db.flush()
    return session


class ConflictError(Exception):
    """Raised when slot conflict detected."""
    def __init__(self, conflicts: list[dict], alternatives: list[dict]):
        self.conflicts = conflicts
        self.alternatives = alternatives
        super().__init__("Konflik slot terdeteksi")
