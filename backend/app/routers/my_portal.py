"""My router — role-scoped endpoints for teacher + guardian portal."""

from fastapi import APIRouter, Depends
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import get_current_user
from app.models.user import User

router = APIRouter(prefix="/api/my", tags=["My Portal"])


@router.get("/schedule")
async def my_schedule(user: User = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    """Get schedule for current user (teacher or guardian)."""
    from app.models.session import Session
    from app.models.teacher import Teacher
    from app.models.guardian import Guardian
    from app.models.student import Student

    if user.role == "teacher":
        result = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
        teacher = result.scalar_one_or_none()
        if not teacher:
            return []
        q = await db.execute(
            select(Session).where(Session.id_teacher == teacher.id_teacher)
            .order_by(Session.session_date.desc()).limit(50)
        )
        return [
            {
                "id_session": str(s.id_session),
                "session_date": str(s.session_date),
                "start_time": s.start_time.strftime("%H:%M") if s.start_time else None,
                "end_time": s.end_time.strftime("%H:%M") if s.end_time else None,
                "status": s.status,
                "session_type": s.session_type,
            }
            for s in q.scalars().all()
        ]

    elif user.role == "guardian":
        result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
        guardian = result.scalar_one_or_none()
        if not guardian:
            return []
        students_q = await db.execute(
            select(Student.id_student).where(Student.id_guardian == guardian.id_guardian, Student.deleted_at.is_(None))
        )
        student_ids = [row[0] for row in students_q.fetchall()]
        if not student_ids:
            return []
        q = await db.execute(
            select(Session).where(Session.id_student.in_(student_ids))
            .order_by(Session.session_date.desc()).limit(50)
        )
        return [
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
        ]
    return []


@router.get("/children")
async def my_children(user: User = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    """Get children (students) for guardian."""
    from app.models.guardian import Guardian
    from app.models.student import Student

    if user.role != "guardian":
        return []
    result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
    guardian = result.scalar_one_or_none()
    if not guardian:
        return []
    q = await db.execute(
        select(Student).where(Student.id_guardian == guardian.id_guardian, Student.deleted_at.is_(None))
        .order_by(Student.created_at.desc())
    )
    return [
        {
            "id_student": str(s.id_student),
            "full_name": s.full_name,
            "instrument": s.instrument,
            "level": s.level,
            "enrollment_status": s.enrollment_status,
            "eligible_for_exam": s.eligible_for_exam,
        }
        for s in q.scalars().all()
    ]


@router.get("/payments")
async def my_payments(user: User = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    """Get payments for guardian's invoices."""
    from app.models.guardian import Guardian
    from app.models.invoice import Invoice
    from app.models.payment import Payment

    if user.role != "guardian":
        return []
    result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
    guardian = result.scalar_one_or_none()
    if not guardian:
        return []

    invoices_q = await db.execute(
        select(Invoice).where(Invoice.id_guardian == guardian.id_guardian).order_by(Invoice.created_at.desc())
    )
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
    return results
