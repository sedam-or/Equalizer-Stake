"""Reports service — owner dashboard KPI aggregation."""

from datetime import datetime, timezone

from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.attendance import Attendance
from app.models.audit_log import AuditLog
from app.models.invoice import Invoice
from app.models.payment import Payment
from app.models.room import Room
from app.models.session import Session
from app.models.student import Student
from app.models.user import User


async def get_owner_dashboard(db: AsyncSession) -> dict:
    """Aggregate KPIs for owner read-only dashboard.

    Returns: revenue, attendance_rate, frozen_accounts, studio_utilization,
    audit_summary, total_students, total_invoices.
    """
    now = datetime.now(timezone.utc)
    current_period = now.strftime("%Y-%m")

    # Revenue: sum of verified payments for current billing period
    revenue_q = await db.execute(
        select(func.coalesce(func.sum(Payment.amount), 0)).where(
            Payment.is_verified == True,  # noqa: E712
        )
    )
    total_revenue = float(revenue_q.scalar() or 0)

    # Revenue this month
    revenue_month_q = await db.execute(
        select(func.coalesce(func.sum(Payment.amount), 0)).join(
            Invoice, Payment.id_invoice == Invoice.id_invoice
        ).where(
            Payment.is_verified == True,  # noqa: E712
            Invoice.billing_period == current_period,
        )
    )
    revenue_this_month = float(revenue_month_q.scalar() or 0)

    # Attendance rate: verified / total submitted
    total_att_q = await db.execute(select(func.count(Attendance.id_attendance)))
    total_attendance = int(total_att_q.scalar() or 0)

    verified_att_q = await db.execute(
        select(func.count(Attendance.id_attendance)).where(
            Attendance.verification_status == "verified"
        )
    )
    verified_attendance = int(verified_att_q.scalar() or 0)

    attendance_rate = (
        round(verified_attendance / total_attendance * 100, 1)
        if total_attendance > 0
        else 0.0
    )

    # Frozen accounts: users with locked_until still in future
    frozen_q = await db.execute(
        select(func.count(User.id_user)).where(
            User.locked_until.isnot(None),
            User.locked_until > now,
        )
    )
    frozen_accounts = int(frozen_q.scalar() or 0)

    # Inactive students
    inactive_q = await db.execute(
        select(func.count(Student.id_student)).where(
            Student.enrollment_status == "inactive"
        )
    )
    inactive_students = int(inactive_q.scalar() or 0)

    # Studio utilization: sessions this month / total room capacity slots
    rooms_q = await db.execute(select(func.count(Room.id_room)))
    total_rooms = int(rooms_q.scalar() or 0)

    sessions_month_q = await db.execute(
        select(func.count(Session.id_session)).where(
            func.to_char(Session.session_date, "YYYY-MM") == current_period
        )
    )
    sessions_this_month = int(sessions_month_q.scalar() or 0)

    # Utilization = sessions / (rooms * 30 days * 8 slots/day) * 100
    max_slots = total_rooms * 30 * 8 if total_rooms > 0 else 1
    studio_utilization = round(sessions_this_month / max_slots * 100, 1)

    # Audit summary: last 7 days activity count
    total_students_q = await db.execute(
        select(func.count(Student.id_student)).where(
            Student.deleted_at.is_(None)
        )
    )
    total_students = int(total_students_q.scalar() or 0)

    # Invoice stats
    unpaid_q = await db.execute(
        select(func.count(Invoice.id_invoice)).where(
            Invoice.status.in_(["issued", "overdue"])
        )
    )
    unpaid_invoices = int(unpaid_q.scalar() or 0)

    overdue_q = await db.execute(
        select(func.count(Invoice.id_invoice)).where(
            Invoice.status == "overdue"
        )
    )
    overdue_invoices = int(overdue_q.scalar() or 0)

    # Recent audit log (last 10)
    audit_q = await db.execute(
        select(AuditLog).order_by(AuditLog.created_at.desc()).limit(10)
    )
    recent_audit = [
        {
            "id_audit": str(log.id_audit),
            "table_name": log.table_name,
            "action": log.action,
            "created_at": log.created_at.isoformat(),
        }
        for log in audit_q.scalars().all()
    ]

    return {
        "total_revenue": total_revenue,
        "revenue_this_month": revenue_this_month,
        "attendance_rate": attendance_rate,
        "verified_attendance": verified_attendance,
        "total_attendance": total_attendance,
        "frozen_accounts": frozen_accounts,
        "inactive_students": inactive_students,
        "studio_utilization": studio_utilization,
        "sessions_this_month": sessions_this_month,
        "total_rooms": total_rooms,
        "total_students": total_students,
        "unpaid_invoices": unpaid_invoices,
        "overdue_invoices": overdue_invoices,
        "recent_audit": recent_audit,
        "billing_period": current_period,
    }
