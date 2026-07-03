"""Reports router — eligibility + owner dashboard KPIs."""

from uuid import UUID

from fastapi import APIRouter, Depends, Query
from fastapi.responses import HTMLResponse
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import create_audit_log
from app.services.eligibility import check_eligibility
from app.services.reports import get_owner_dashboard, render_dashboard_html

router = APIRouter(prefix="/api/reports", tags=["Reports"])


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
