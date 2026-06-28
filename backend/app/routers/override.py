"""Override router — superadmin operational overrides."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import create_audit_log
from app.services.override import cancel_invoice, revert_session, unfreeze_guardian

router = APIRouter(prefix="/api/override", tags=["Override"])


@router.put("/unfreeze/{guardian_id}")
async def unfreeze_guardian_endpoint(
    guardian_id: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Unfreeze guardian account (Owner only)."""
    guardian = await unfreeze_guardian(db, UUID(guardian_id))
    if not guardian:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Guardian tidak ditemukan atau tidak dalam status beku",
        )

    await create_audit_log(
        db,
        table_name="guardians",
        record_id=guardian.id_guardian,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"account_status": "frozen"},
        after_state={"account_status": "active", "override": True},
    )
    await db.commit()
    return {
        "detail": "Akun berhasil di-unfreeze",
        "id_guardian": str(guardian.id_guardian),
        "account_status": guardian.account_status,
    }


@router.put("/cancel-invoice/{invoice_id}")
async def cancel_invoice_endpoint(
    invoice_id: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Cancel invoice (Owner only). Only issued/overdue can be cancelled."""
    invoice = await cancel_invoice(db, UUID(invoice_id))
    if not invoice:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Invoice tidak ditemukan atau tidak bisa dibatalkan",
        )

    await create_audit_log(
        db,
        table_name="invoices",
        record_id=invoice.id_invoice,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"status": "issued/overdue"},
        after_state={"status": "cancelled", "override": True},
    )
    await db.commit()
    return {
        "detail": "Invoice berhasil dibatalkan",
        "id_invoice": str(invoice.id_invoice),
        "status": invoice.status,
    }


@router.put("/revert-session/{session_id}")
async def revert_session_endpoint(
    session_id: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Revert cancelled/rescheduled session to pending (Owner only)."""
    session = await revert_session(db, UUID(session_id))
    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Sesi tidak ditemukan atau tidak bisa di-revert",
        )

    await create_audit_log(
        db,
        table_name="sessions",
        record_id=session.id_session,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"status": "cancelled/rescheduled"},
        after_state={"status": "pending", "override": True},
    )
    await db.commit()
    return {
        "detail": "Sesi berhasil di-revert ke pending",
        "id_session": str(session.id_session),
        "status": session.status,
    }
