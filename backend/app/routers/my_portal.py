"""My router — role-scoped endpoints for teacher + guardian portal."""

from fastapi import APIRouter, Depends, Query
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import get_current_user
from app.models.user import User

router = APIRouter(prefix="/api/my", tags=["My Portal"])


@router.get("/schedule")
async def my_schedule(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    instrument: str | None = Query(None),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Get schedule for current user (teacher or guardian).
    Teacher mode: filter by ?instrument=piano (optional, backward compatible).
    """
    from app.models.session import Session
    from app.models.teacher import Teacher
    from app.models.guardian import Guardian
    from app.models.student import Student

    if user.role == "teacher":
        result = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
        teacher = result.scalar_one_or_none()
        if not teacher:
            return {"data": [], "total": 0, "limit": limit, "offset": offset}
        from app.models.room import Room
        base = select(Session).where(Session.id_teacher == teacher.id_teacher, Session.status.notin_(["cancelled"]))

        # Filter by instrument (SRS FR-02): join student → filter by instrument
        if instrument:
            instrument_students = await db.execute(
                select(Student.id_student).where(Student.instrument == instrument, Student.deleted_at.is_(None))
            )
            instrument_ids = [row[0] for row in instrument_students.fetchall()]
            if not instrument_ids:
                return {"data": [], "total": 0, "limit": limit, "offset": offset}
            base = base.where(Session.id_student.in_(instrument_ids))
        count_q = await db.execute(select(func.count()).select_from(base.subquery()))
        total = count_q.scalar()
        q = await db.execute(base.order_by(Session.session_date.desc()).offset(offset).limit(limit))
        sessions = q.scalars().all()
        student_ids = list({s.id_student for s in sessions})
        room_ids = list({s.id_room for s in sessions})
        students_q = await db.execute(select(Student).where(Student.id_student.in_(student_ids))) if student_ids else None
        rooms_q = await db.execute(select(Room).where(Room.id_room.in_(room_ids))) if room_ids else None
        students_map = {s.id_student: s.full_name for s in (students_q.scalars().all() if students_q else [])}
        rooms_map = {r.id_room: r.room_name for r in (rooms_q.scalars().all() if rooms_q else [])}
        return {
            "data": [
                {
                    "id_session": str(s.id_session),
                    "session_date": str(s.session_date),
                    "start_time": s.start_time.strftime("%H:%M") if s.start_time else None,
                    "end_time": s.end_time.strftime("%H:%M") if s.end_time else None,
                    "status": s.status,
                    "session_type": s.session_type,
                    "student_name": students_map.get(s.id_student, ""),
                    "room_name": rooms_map.get(s.id_room, ""),
                }
                for s in sessions
            ],
            "total": total,
            "limit": limit,
            "offset": offset,
        }

    elif user.role == "guardian":
        result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
        guardian = result.scalar_one_or_none()
        if not guardian:
            return {"data": [], "total": 0, "limit": limit, "offset": offset}
        students_q = await db.execute(
            select(Student.id_student).where(Student.id_guardian == guardian.id_guardian, Student.deleted_at.is_(None))
        )
        student_ids = [row[0] for row in students_q.fetchall()]
        if not student_ids:
            return {"data": [], "total": 0, "limit": limit, "offset": offset}
        base = select(Session).where(Session.id_student.in_(student_ids))
        count_q = await db.execute(select(func.count()).select_from(base.subquery()))
        total = count_q.scalar()
        q = await db.execute(base.order_by(Session.session_date.desc()).offset(offset).limit(limit))
        return {
            "data": [
                {
                    "id_session": str(s.id_session),
                    "session_date": str(s.session_date),
                    "start_time": s.start_time.strftime("%H:%M") if s.start_time else None,
                    "end_time": s.end_time.strftime("%H:%M") if s.end_time else None,
                    "status": s.status,
                    "session_type": s.session_type,
                    "student_id": str(s.id_student),
                }
                for s in q.scalars().all()
            ],
            "total": total,
            "limit": limit,
            "offset": offset,
        }
    return {"data": [], "total": 0, "limit": limit, "offset": offset}


@router.get("/children")
async def my_children(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Get children (students) for guardian."""
    from app.models.guardian import Guardian
    from app.models.student import Student

    if user.role != "guardian":
        return {"data": [], "total": 0, "limit": limit, "offset": offset}
    result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
    guardian = result.scalar_one_or_none()
    if not guardian:
        return {"data": [], "total": 0, "limit": limit, "offset": offset}
    base = select(Student).where(Student.id_guardian == guardian.id_guardian, Student.deleted_at.is_(None))
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    q = await db.execute(base.order_by(Student.created_at.desc()).offset(offset).limit(limit))
    return {
        "data": [
            {
                "id_student": str(s.id_student),
                "full_name": s.full_name,
                "instrument": s.instrument,
                "level": s.level,
                "enrollment_status": s.enrollment_status,
                "eligible_for_exam": s.eligible_for_exam,
            }
            for s in q.scalars().all()
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.get("/payments")
async def my_payments(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Get payments for guardian's invoices."""
    from app.models.guardian import Guardian
    from app.models.invoice import Invoice
    from app.models.payment import Payment

    if user.role != "guardian":
        return {"data": [], "total": 0, "limit": limit, "offset": offset}
    result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
    guardian = result.scalar_one_or_none()
    if not guardian:
        return {"data": [], "total": 0, "limit": limit, "offset": offset}

    base = select(Invoice).where(Invoice.id_guardian == guardian.id_guardian)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    invoices_q = await db.execute(base.order_by(Invoice.created_at.desc()).offset(offset).limit(limit))
    results = []
    # Batch load all payments for these invoices (N+1 fix)
    invoices_list = invoices_q.scalars().all()
    invoice_ids = [inv.id_invoice for inv in invoices_list]
    invoices_map = {inv.id_invoice: inv for inv in invoices_list}
    all_payments = []
    if invoice_ids:
        pay_q = await db.execute(select(Payment).where(Payment.id_invoice.in_(invoice_ids)))
        all_payments = pay_q.scalars().all()
    payments_by_invoice = {}
    for p in all_payments:
        payments_by_invoice.setdefault(p.id_invoice, []).append(p)

    for inv_id in invoice_ids:
        inv = invoices_map[inv_id]
        payments = payments_by_invoice.get(inv_id, [])
        results.append({
            "id_invoice": str(inv.id_invoice),
            "status": inv.status,
            "amount_total": float(inv.total_amount),
            "billing_period": inv.billing_period,
            "due_date": str(inv.due_date),
            "unique_code": inv.unique_code,
            "payments": [
                {
                    "id_payment": str(p.id_payment),
                    "amount": float(p.amount),
                    "method": p.method,
                    "is_verified": p.is_verified,
                    "payment_date": p.payment_date.isoformat(),
                }
                for p in payments
            ],
        })
    return {"data": results, "total": total, "limit": limit, "offset": offset}


@router.get("/students")
async def my_students(
    instrument: str | None = Query(None),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """List students that this teacher teaches.
    Teacher mode only. Filter by ?instrument=piano (optional).
    """
    from app.models.teacher import Teacher
    from app.models.student import Student
    from app.models.session import Session

    if user.role != "teacher":
        return {"data": [], "total": 0, "limit": limit, "offset": offset}

    result = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
    teacher = result.scalar_one_or_none()
    if not teacher:
        return {"data": [], "total": 0, "limit": limit, "offset": offset}

    # Get student IDs from sessions this teacher teaches
    sessions_q = await db.execute(
        select(Session.id_student).where(
            Session.id_teacher == teacher.id_teacher,
            Session.status.notin_(["cancelled"]),
        ).distinct()
    )
    student_ids = [row[0] for row in sessions_q.fetchall()]
    if not student_ids:
        return {"data": [], "total": 0, "limit": limit, "offset": offset}

    # Build query with optional instrument filter
    base = select(Student).where(
        Student.id_student.in_(student_ids),
        Student.deleted_at.is_(None),
    )
    if instrument:
        base = base.where(Student.instrument == instrument)

    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    q = await db.execute(base.order_by(Student.created_at.desc()).offset(offset).limit(limit))
    students = q.scalars().all()

    return {
        "data": [
            {
                "id_student": str(s.id_student),
                "full_name": s.full_name,
                "instrument": s.instrument,
                "level": s.level,
                "enrollment_status": s.enrollment_status,
                "eligible_for_exam": s.eligible_for_exam,
            }
            for s in students
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.get("/attendance")
async def my_attendance(
    instrument: str | None = Query(None),
    status_filter: str | None = Query(None, alias="status"),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """List attendance records for this teacher.
    Teacher mode only. Filter by ?instrument=piano (optional).
    """
    from app.models.teacher import Teacher
    from app.models.student import Student
    from app.models.session import Session
    from app.models.attendance import Attendance

    if user.role != "teacher":
        return {"data": [], "total": 0, "limit": limit, "offset": offset}

    result = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
    teacher = result.scalar_one_or_none()
    if not teacher:
        return {"data": [], "total": 0, "limit": limit, "offset": offset}

    # Build base query: attendance → session (teacher's sessions)
    base = (
        select(Attendance)
        .join(Session, Attendance.id_session == Session.id_session)
        .where(Session.id_teacher == teacher.id_teacher)
    )

    # Filter by instrument via student
    if instrument:
        instrument_students = await db.execute(
            select(Student.id_student).where(
                Student.instrument == instrument, Student.deleted_at.is_(None)
            )
        )
        instrument_ids = [row[0] for row in instrument_students.fetchall()]
        if not instrument_ids:
            return {"data": [], "total": 0, "limit": limit, "offset": offset}
        base = base.where(Attendance.id_session.in_(
            select(Session.id_session).where(Session.id_student.in_(instrument_ids))
        ))

    # Filter by verification status
    if status_filter:
        base = base.where(Attendance.verification_status == status_filter)

    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    q = await db.execute(base.order_by(Attendance.created_at.desc()).offset(offset).limit(limit))
    records = q.scalars().all()

    return {
        "data": [
            {
                "id_attendance": str(a.id_attendance),
                "id_session": str(a.id_session),
                "gps_lat": float(a.gps_lat),
                "gps_lng": float(a.gps_lng),
                "face_valid": a.face_valid,
                "verification_status": a.verification_status,
                "created_at": a.created_at.isoformat() if a.created_at else None,
            }
            for a in records
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }
