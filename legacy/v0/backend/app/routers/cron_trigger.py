"""Admin endpoint to manually trigger cron jobs for testing."""

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import get_current_user
from app.models.user import User

router = APIRouter(prefix="/api/admin/cron", tags=["Cron Trigger"])


@router.post("/trigger/auto-invoice")
async def trigger_auto_invoice(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Manually trigger auto-invoice job (normally runs tanggal 5)."""
    if user.role not in ("owner", "admin"):
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Hanya admin yang bisa trigger cron job")
    
    from app.services.scheduler import job_auto_invoice
    try:
        await job_auto_invoice()
        return {"message": "Auto-invoice job berhasil dijalankan"}
    except Exception as e:
        return {"message": f"Auto-invoice job error: {str(e)}"}


@router.post("/trigger/fine-freeze")
async def trigger_fine_freeze(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Manually trigger fine+freeze job (normally runs tanggal 11)."""
    if user.role not in ("owner", "admin"):
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Hanya admin yang bisa trigger cron job")
    
    from app.services.scheduler import job_fine_freeze
    try:
        await job_fine_freeze()
        return {"message": "Fine+freeze job berhasil dijalankan"}
    except Exception as e:
        return {"message": f"Fine+freeze job error: {str(e)}"}


@router.post("/trigger/draft-cleanup")
async def trigger_draft_cleanup(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Manually trigger draft cleanup job."""
    if user.role not in ("owner", "admin"):
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Hanya admin yang bisa trigger cron job")
    
    from app.services.scheduler import job_draft_cleanup
    try:
        await job_draft_cleanup()
        return {"message": "Draft cleanup job berhasil dijalankan"}
    except Exception as e:
        return {"message": f"Draft cleanup job error: {str(e)}"}


@router.post("/trigger/session-reminder")
async def trigger_session_reminder(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Manually trigger session reminder job."""
    if user.role not in ("owner", "admin"):
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Hanya admin yang bisa trigger cron job")
    
    from app.services.scheduler import job_session_reminder
    try:
        await job_session_reminder()
        return {"message": "Session reminder job berhasil dijalankan"}
    except Exception as e:
        return {"message": f"Session reminder job error: {str(e)}"}
