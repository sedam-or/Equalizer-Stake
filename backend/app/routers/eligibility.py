"""Reports router — eligibility + owner dashboard KPIs."""

from uuid import UUID

from fastapi import APIRouter, Depends, Query
from fastapi.responses import HTMLResponse
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from sqlalchemy import select, func
from app.services.audit import create_audit_log
from app.services.eligibility import check_eligibility
from app.services.reports import get_owner_dashboard, render_dashboard_html

router = APIRouter(prefix="/api/v1/reports", tags=["Reports"])


@router.get("/eligibility/{student_id}")
async def get_eligibility(
    student_id: str,
    user: User = Depends(require_role("admin", "owner", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Check exam eligibility for a student."""
    result = await check_eligibility(db, UUID(student_id))
    return result


@router.get("/owner-dashboard")
async def owner_dashboard(
    start_date: str | None = Query(
        None,
        description="Tanggal mulai (YYYY-MM-DD). Default awal bulan ini.",
        pattern=r"^\d{4}-\d{2}-\d{2}$",
    ),
    end_date: str | None = Query(
        None,
        description="Tanggal akhir (YYYY-MM-DD). Default akhir bulan ini.",
        pattern=r"^\d{4}-\d{2}-\d{2}$",
    ),
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Aggregate KPI dashboard (Owner only). Read-only.

    Query params:
    - start_date: filter start (YYYY-MM-DD), default awal bulan ini
    - end_date: filter end (YYYY-MM-DD), default akhir bulan ini
    """
    # Validate: both or neither
    if bool(start_date) != bool(end_date):
        return {
            "detail": "Parameter start_date dan end_date harus diisi bersamaan."
        }

    data = await get_owner_dashboard(db, start_date=start_date, end_date=end_date)

    await create_audit_log(
        db,
        table_name="owner_dashboard",
        record_id=user.id_user,
        action="INSERT",
        changed_by=user.id_user,
        after_state={"action": "view_owner_dashboard"},
        category="keamanan",
    )
    await db.commit()

    return data


@router.get("/export/pdf")
async def export_pdf(
    start_date: str | None = Query(
        None,
        description="Tanggal mulai (YYYY-MM-DD).",
        pattern=r"^\d{4}-\d{2}-\d{2}$",
    ),
    end_date: str | None = Query(
        None,
        description="Tanggal akhir (YYYY-MM-DD).",
        pattern=r"^\d{4}-\d{2}-\d{2}$",
    ),
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Export owner dashboard sebagai HTML printable (PDF via browser Print).

    Kirim header Content-Disposition agar browser menyarankan download.
    """
    if bool(start_date) != bool(end_date):
        return {
            "detail": "Parameter start_date dan end_date harus diisi bersamaan."
        }

    data = await get_owner_dashboard(db, start_date=start_date, end_date=end_date)
    html = render_dashboard_html(data)

    await create_audit_log(
        db,
        table_name="owner_dashboard",
        record_id=user.id_user,
        action="INSERT",
        changed_by=user.id_user,
        after_state={"action": "export_dashboard_pdf"},
        category="keamanan",
    )
    await db.commit()

    return HTMLResponse(
        content=html,
        headers={
            "Content-Disposition": 'attachment; filename="laporan-dashboard.html"',
        },
    )


# ═══════════════════════════════════════════════════════════════
# DASHBOARD ENDPOINTS (T-121) — Admin, Teacher, Guardian
# ═══════════════════════════════════════════════════════════════

@router.get("/admin-stats")
async def admin_dashboard_stats(
    user: User = Depends(require_role("admin")),
    db: AsyncSession = Depends(get_db),
):
    """Admin dashboard — real-time KPI + pending actions."""
    from datetime import date, datetime, timedelta
    from sqlalchemy import and_
    from app.models.session import Session
    from app.models.attendance import Attendance
    from app.models.invoice import Invoice
    from app.models.notification import Notification

    today = date.today()

    # Sessions today
    sessions_today_q = await db.execute(
        select(func.count()).select_from(Session).where(Session.session_date == today)
    )
    sessions_today = sessions_today_q.scalar() or 0

    # Pending attendance reviews
    pending_att_q = await db.execute(
        select(func.count()).select_from(Attendance).where(
            Attendance.verification_status == "pending_review"
        )
    )
    pending_attendance = pending_att_q.scalar() or 0

    # Overdue invoices
    overdue_q = await db.execute(
        select(func.count()).select_from(Invoice).where(
            and_(Invoice.status == "issued", Invoice.due_date < today)
        )
    )
    overdue_invoices = overdue_q.scalar() or 0

    # Failed notifications today
    failed_notif_q = await db.execute(
        select(func.count()).select_from(Notification).where(
            and_(Notification.status == "failed", Notification.created_at >= datetime.combine(today, datetime.min.time()))
        )
    )
    failed_notifications = failed_notif_q.scalar() or 0

    # Recent sessions (today)
    recent_sessions_q = await db.execute(
        select(Session).where(Session.session_date == today).order_by(Session.start_time).limit(10)
    )
    recent_sessions = []
    for s in recent_sessions_q.scalars().all():
        recent_sessions.append({
            "id_session": str(s.id_session),
            "start_time": s.start_time.strftime("%H:%M") if s.start_time else None,
            "session_type": s.session_type,
            "status": s.status,
        })

    return {
        "greeting": f"Selamat datang, {user.username}",
        "date": today.isoformat(),
        "kpi": {
            "sessions_today": sessions_today,
            "pending_attendance": pending_attendance,
            "overdue_invoices": overdue_invoices,
            "failed_notifications": failed_notifications,
        },
        "recent_sessions": recent_sessions,
    }


@router.get("/teacher-stats")
async def teacher_dashboard_stats(
    user: User = Depends(require_role("teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Teacher dashboard — today's schedule + weekly summary."""
    from datetime import date, datetime, timedelta
    from app.models.teacher import Teacher
    from app.models.session import Session
    from app.models.attendance import Attendance

    today = date.today()
    week_start = today - timedelta(days=today.weekday())

    result = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
    teacher = result.scalar_one_or_none()
    if not teacher:
        return {"greeting": f"Selamat datang, {user.username}", "kpi": {}, "today_sessions": []}

    # Sessions today
    today_sessions_q = await db.execute(
        select(Session).where(
            Session.id_teacher == teacher.id_teacher,
            Session.session_date == today,
        ).order_by(Session.start_time)
    )
    today_sessions = []
    for s in today_sessions_q.scalars().all():
        today_sessions.append({
            "id_session": str(s.id_session),
            "start_time": s.start_time.strftime("%H:%M") if s.start_time else None,
            "end_time": s.end_time.strftime("%H:%M") if s.end_time else None,
            "session_type": s.session_type,
            "status": s.status,
        })

    # Weekly summary
    week_total_q = await db.execute(
        select(func.count()).select_from(Session).where(
            Session.id_teacher == teacher.id_teacher,
            Session.session_date.between(week_start, today),
        )
    )
    week_total = week_total_q.scalar() or 0

    week_verified_q = await db.execute(
        select(func.count()).select_from(Attendance).join(
            Session, Attendance.id_session == Session.id_session
        ).where(
            Session.id_teacher == teacher.id_teacher,
            Attendance.verification_status == "verified",
            Attendance.created_at >= datetime.combine(week_start, datetime.min.time()),
        )
    )
    week_verified = week_verified_q.scalar() or 0

    return {
        "greeting": f"Selamat pagi, {user.username}",
        "date": today.isoformat(),
        "kpi": {
            "sessions_today": len(today_sessions),
            "week_total": week_total,
            "week_verified": week_verified,
            "week_pending": max(0, week_total - week_verified),
        },
        "today_sessions": today_sessions,
    }


@router.get("/guardian-stats")
async def guardian_dashboard_stats(
    user: User = Depends(require_role("guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Guardian dashboard — child schedule, invoices, attendance."""
    from datetime import date, datetime, timedelta
    from app.models.guardian import Guardian
    from app.models.student import Student
    from app.models.session import Session
    from app.models.attendance import Attendance
    from app.models.invoice import Invoice

    today = date.today()
    week_end = today + timedelta(days=7)

    result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
    guardian = result.scalar_one_or_none()
    if not guardian:
        return {"greeting": f"Selamat datang, {user.username}", "children": []}

    # Get children
    children_q = await db.execute(
        select(Student).where(Student.id_guardian == guardian.id_guardian, Student.deleted_at.is_(None))
    )
    children = children_q.scalars().all()

    children_data = []
    for child in children:
        # Upcoming sessions (next 7 days)
        sessions_q = await db.execute(
            select(Session).where(
                Session.id_student == child.id_student,
                Session.session_date.between(today, week_end),
            ).order_by(Session.session_date, Session.start_time).limit(5)
        )
        sessions = []
        for s in sessions_q.scalars().all():
            sessions.append({
                "session_date": str(s.session_date),
                "start_time": s.start_time.strftime("%H:%M") if s.start_time else None,
                "session_type": s.session_type,
            })

        # Recent attendance
        att_q = await db.execute(
            select(Attendance).join(Session, Attendance.id_session == Session.id_session).where(
                Session.id_student == child.id_student,
            ).order_by(Attendance.created_at.desc()).limit(3)
        )
        attendance = []
        for a in att_q.scalars().all():
            attendance.append({
                "verification_status": a.verification_status,
                "created_at": a.created_at.strftime("%d %b %Y") if a.created_at else None,
            })

        # Pending invoice
        inv_q = await db.execute(
            select(Invoice).where(
                Invoice.id_guardian == guardian.id_guardian,
                Invoice.status == "issued",
            ).order_by(Invoice.due_date).limit(1)
        )
        invoice = inv_q.scalar_one_or_none()
        invoice_data = None
        if invoice:
            invoice_data = {
                "total_amount": float(invoice.total_amount),
                "due_date": str(invoice.due_date),
                "billing_period": invoice.billing_period,
            }

        children_data.append({
            "id_student": str(child.id_student),
            "full_name": child.full_name,
            "instrument": child.instrument,
            "level": child.level,
            "upcoming_sessions": sessions,
            "recent_attendance": attendance,
            "pending_invoice": invoice_data,
        })

    return {
        "greeting": f"Selamat pagi, {user.username}",
        "date": today.isoformat(),
        "children": children_data,
    }
