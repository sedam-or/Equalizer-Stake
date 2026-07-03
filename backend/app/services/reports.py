"""Reports service — owner dashboard KPI aggregation."""

from datetime import datetime, timezone
from calendar import monthrange

from sqlalchemy import func, select, case, extract
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.attendance import Attendance
from app.models.audit_log import AuditLog
from app.models.invoice import Invoice
from app.models.payment import Payment
from app.models.room import Room
from app.models.session import Session
from app.models.student import Student
from app.models.user import User


def _month_range(year: int, month: int) -> tuple[str, str]:
    """Return (YYYY-MM-01, YYYY-MM-DD) for given year/month."""
    _, last_day = monthrange(year, month)
    return f"{year}-{month:02d}-01", f"{year}-{month:02d}-{last_day:02d}"


async def get_owner_dashboard(
    db: AsyncSession,
    start_date: str | None = None,
    end_date: str | None = None,
) -> dict:
    """Aggregate KPIs for owner read-only dashboard.

    Returns: revenue, attendance_rate, frozen_accounts, studio_utilization,
    audit_summary, total_students, total_invoices,
    revenue_trend (6-month), studio_breakdown, anomaly_flags.
    """
    now = datetime.now(timezone.utc)
    current_period = now.strftime("%Y-%m")

    # --- date range filter (default: current month) ---
    if start_date and end_date:
        filter_start = start_date
        filter_end = end_date
        filter_period = start_date[:7]  # YYYY-MM
    else:
        y, m = now.year, now.month
        filter_start, filter_end = _month_range(y, m)
        filter_period = current_period

    # Revenue: sum of verified payments within date range
    revenue_q = await db.execute(
        select(func.coalesce(func.sum(Payment.amount), 0)).where(
            Payment.is_verified == True,  # noqa: E712
        )
    )
    total_revenue = float(revenue_q.scalar() or 0)

    # Revenue in date range
    revenue_range_q = await db.execute(
        select(func.coalesce(func.sum(Payment.amount), 0))
        .join(Invoice, Payment.id_invoice == Invoice.id_invoice)
        .where(
            Payment.is_verified == True,  # noqa: E712
            Invoice.billing_period == filter_period,
        )
    )
    revenue_in_range = float(revenue_range_q.scalar() or 0)

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

    total_users_q = await db.execute(
        select(func.count(User.id_user)).where(User.deleted_at.is_(None))
    )
    total_users = int(total_users_q.scalar() or 0)

    # Inactive students
    inactive_q = await db.execute(
        select(func.count(Student.id_student)).where(
            Student.enrollment_status == "inactive"
        )
    )
    inactive_students = int(inactive_q.scalar() or 0)

    # Studio utilization (overall)
    rooms_q = await db.execute(select(func.count(Room.id_room)))
    total_rooms = int(rooms_q.scalar() or 0)

    sessions_month_q = await db.execute(
        select(func.count(Session.id_session)).where(
            func.to_char(Session.session_date, "YYYY-MM") == filter_period
        )
    )
    sessions_this_month = int(sessions_month_q.scalar() or 0)

    max_slots = total_rooms * 30 * 8 if total_rooms > 0 else 1
    studio_utilization = round(sessions_this_month / max_slots * 100, 1)

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

    total_invoices_q = await db.execute(
        select(func.count(Invoice.id_invoice))
    )
    total_invoices_count = int(total_invoices_q.scalar() or 0)

    # Total students
    total_students_q = await db.execute(
        select(func.count(Student.id_student)).where(
            Student.deleted_at.is_(None)
        )
    )
    total_students = int(total_students_q.scalar() or 0)

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

    # ============================================================
    # 2. Revenue trend: last 6 months invoices grouped by month
    # ============================================================
    trend_months = []
    for i in range(5, -1, -1):
        # walk backwards from current month
        m = now.month - i
        y = now.year
        while m <= 0:
            m += 12
            y -= 1
        period_str = f"{y}-{m:02d}"
        trend_months.append(period_str)

    trend_q = await db.execute(
        select(
            Invoice.billing_period,
            func.coalesce(func.sum(Invoice.total_amount), 0).label("total"),
            func.count(Invoice.id_invoice).label("count"),
        )
        .where(Invoice.billing_period.in_(trend_months))
        .group_by(Invoice.billing_period)
        .order_by(Invoice.billing_period)
    )
    trend_rows = {row[0]: {"total": float(row[1]), "count": row[2]} for row in trend_q.all()}
    revenue_trend = []
    for period in trend_months:
        entry = trend_rows.get(period, {"total": 0.0, "count": 0})
        revenue_trend.append({
            "period": period,
            "total_amount": entry["total"],
            "invoice_count": entry["count"],
        })

    # ============================================================
    # 3. Per-studio breakdown: sessions by room with utilization %
    # ============================================================
    studio_q = await db.execute(
        select(
            Room.id_room,
            Room.room_name,
            Room.room_type,
            func.count(Session.id_session).label("session_count"),
        )
        .outerjoin(
            Session,
            (Session.id_room == Room.id_room)
            & (func.to_char(Session.session_date, "YYYY-MM") == filter_period),
        )
        .group_by(Room.id_room, Room.room_name, Room.room_type)
        .order_by(Room.room_name)
    )
    studio_breakdown = []
    for row in studio_q.all():
        session_count = row[3]
        room_util = round(session_count / (30 * 8) * 100, 1)  # ponytail: fixed 8 slots/day
        studio_breakdown.append({
            "id_room": str(row[0]),
            "room_name": row[1],
            "room_type": row[2],
            "sessions": session_count,
            "utilization_pct": room_util,
        })

    # ============================================================
    # 4. Anomaly flags
    # ============================================================
    anomaly_flags = []

    # attendance < 5 sessions (per student in period)
    low_att_q = await db.execute(
        select(
            Student.id_student,
            func.count(Attendance.id_attendance).label("att_count"),
        )
        .join(Session, Session.id_student == Student.id_student)
        .outerjoin(Attendance, Attendance.id_session == Session.id_session)
        .where(
            Student.deleted_at.is_(None),
            func.to_char(Session.session_date, "YYYY-MM") == filter_period,
        )
        .group_by(Student.id_student)
        .having(func.count(Attendance.id_attendance) < 5)
    )
    low_att_students = low_att_q.all()
    if low_att_students:
        anomaly_flags.append({
            "type": "low_attendance",
            "threshold": "< 5 sesi",
            "count": len(low_att_students),
            "detail": f"{len(low_att_students)} siswa memiliki kurang dari 5 sesi kehadiran*",
        })

    # frozen > 20% of users
    frozen_pct = round(frozen_accounts / total_users * 100, 1) if total_users > 0 else 0.0
    if frozen_pct > 20:
        anomaly_flags.append({
            "type": "high_frozen",
            "threshold": "> 20%",
            "value": f"{frozen_pct}%",
            "detail": f"Akun beku mencapai {frozen_pct}% dari total pengguna",
        })

    # overdue > 10% of invoices
    overdue_pct = round(overdue_invoices / total_invoices_count * 100, 1) if total_invoices_count > 0 else 0.0
    if overdue_pct > 10:
        anomaly_flags.append({
            "type": "high_overdue",
            "threshold": "> 10%",
            "value": f"{overdue_pct}%",
            "detail": f"Tagihan jatuh tempo mencapai {overdue_pct}% dari total invoice",
        })

    # ============================================================
    # 6. Attendance <5 sessions asterisk
    # ============================================================
    low_att_ids = {str(row[0]) for row in low_att_students}

    return {
        "total_revenue": total_revenue,
        "revenue_this_month": revenue_in_range,
        "revenue_in_range": revenue_in_range,
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
        "billing_period": filter_period,
        "date_range": {"start": filter_start, "end": filter_end},
        "revenue_trend": revenue_trend,
        "studio_breakdown": studio_breakdown,
        "anomaly_flags": anomaly_flags,
        "low_attendance_students": list(low_att_ids),
        "low_attendance_note": "* Siswa dengan kurang dari 5 sesi kehadiran ditandai (*)",
    }


def render_dashboard_html(data: dict) -> str:
    """Render owner dashboard as printable HTML for PDF export."""
    trend_rows = "".join(
        f"<tr><td>{t['period']}</td><td>Rp {t['total_amount']:,.0f}</td><td>{t['invoice_count']}</td></tr>"
        for t in data.get("revenue_trend", [])
    )
    studio_rows = "".join(
        f"<tr><td>{s['room_name']}</td><td>{s['room_type']}</td>"
        f"<td>{s['sessions']}</td><td>{s['utilization_pct']}%</td></tr>"
        for s in data.get("studio_breakdown", [])
    )
    anomaly_li = "".join(
        f"<li>{a['detail']}</li>" for a in data.get("anomaly_flags", [])
    ) or "<li>Tidak ada anomali terdeteksi</li>"

    dr = data.get("date_range", {})
    return f"""<!DOCTYPE html>
<html lang="id">
<head>
<meta charset="utf-8">
<title>Laporan Dashboard Owner — BMD Musik</title>
<style>
  body {{ font-family: Arial, sans-serif; margin: 24px; color: #222; }}
  h1 {{ font-size: 20px; border-bottom: 2px solid #333; padding-bottom: 6px; }}
  h2 {{ font-size: 16px; margin-top: 24px; }}
  table {{ border-collapse: collapse; width: 100%; margin: 8px 0 16px; }}
  th, td {{ border: 1px solid #999; padding: 6px 10px; text-align: left; font-size: 13px; }}
  th {{ background: #eee; }}
  .kpi-grid {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin: 12px 0; }}
  .kpi {{ border: 1px solid #ccc; border-radius: 6px; padding: 12px; }}
  .kpi .label {{ font-size: 12px; color: #666; }}
  .kpi .value {{ font-size: 22px; font-weight: bold; }}
  .anomaly {{ color: #c00; }}
  .note {{ font-size: 12px; color: #666; margin-top: 8px; }}
  @media print {{ body {{ margin: 12px; }} }}
</style>
</head>
<body>
<h1>Laporan Dashboard Owner — BMD Musik Yogyakarta</h1>
<p>Periode: <strong>{data.get('billing_period', '-')}</strong>
 | Rentang: {dr.get('start', '-')} s/d {dr.get('end', '-')}</p>

<h2>Ringkasan KPI</h2>
<div class="kpi-grid">
  <div class="kpi"><div class="label">Total Pendapatan</div><div class="value">Rp {data['total_revenue']:,.0f}</div></div>
  <div class="kpi"><div class="label">Pendapatan Periode Ini</div><div class="value">Rp {data['revenue_in_range']:,.0f}</div></div>
  <div class="kpi"><div class="label">Tingkat Kehadiran</div><div class="value">{data['attendance_rate']}%</div></div>
  <div class="kpi"><div class="label">Akun Beku</div><div class="value">{data['frozen_accounts']}</div></div>
  <div class="kpi"><div class="label">Utilisasi Studio</div><div class="value">{data['studio_utilization']}%</div></div>
  <div class="kpi"><div class="label">Total Siswa</div><div class="value">{data['total_students']}</div></div>
  <div class="kpi"><div class="label">Invoice Belum Bayar</div><div class="value">{data['unpaid_invoices']}</div></div>
  <div class="kpi"><div class="label">Invoice Jatuh Tempo</div><div class="value">{data['overdue_invoices']}</div></div>
  <div class="kpi"><div class="label">Sesi Bulan Ini</div><div class="value">{data['sessions_this_month']}</div></div>
</div>

<h2>Tren Pendapatan 6 Bulan</h2>
<table>
<tr><th>Bulan</th><th>Total Tagihan</th><th>Jumlah Invoice</th></tr>
{trend_rows}
</table>

<h2>Breakdown Per Studio</h2>
<table>
<tr><th>Ruangan</th><th>Tipe</th><th>Sesi</th><th>Utilisasi</th></tr>
{studio_rows}
</table>

<h2 class="anomaly">Flag Anomali</h2>
<ul class="anomaly">{anomaly_li}</ul>

<p class="note">* Siswa dengan kurang dari 5 sesi kehadiran ditandai (*).
 Dicetak pada {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}</p>
</body>
</html>"""
