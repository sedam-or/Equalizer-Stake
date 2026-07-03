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
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Get schedule for current user (teacher or guardian)."""
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
    for inv in invoices_q.scalars().all():
        pay_q = await db.execute(select(Payment).where(Payment.id_invoice == inv.id_invoice))
        payments = pay_q.scalars().all()
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
