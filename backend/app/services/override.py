"""Override service — superadmin operational overrides."""

from datetime import datetime, timezone
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.guardian import Guardian
from app.models.invoice import Invoice
from app.models.session import Session


async def unfreeze_guardian(db: AsyncSession, guardian_id: UUID) -> Guardian | None:
    """Unfreeze guardian account. Returns None if not found or not frozen."""
    result = await db.execute(
        select(Guardian).where(
            Guardian.id_guardian == guardian_id,
            Guardian.deleted_at.is_(None),
        )
    )
    guardian = result.scalar_one_or_none()
    if not guardian or guardian.account_status != "frozen":
        return None

    guardian.account_status = "active"
    guardian.frozen_at = None
    guardian.frozen_reason = None
    guardian.updated_at = datetime.now(timezone.utc)
    await db.flush()
    return guardian


async def cancel_invoice(db: AsyncSession, invoice_id: UUID) -> Invoice | None:
    """Cancel invoice. Only issued/overdue invoices can be cancelled."""
    result = await db.execute(
        select(Invoice).where(Invoice.id_invoice == invoice_id)
    )
    invoice = result.scalar_one_or_none()
    if not invoice or invoice.status not in ("issued", "overdue"):
        return None

    old_status = invoice.status
    invoice.status = "cancelled"
    invoice.updated_at = datetime.now(timezone.utc)
    await db.flush()
    return invoice


async def revert_session(db: AsyncSession, session_id: UUID) -> Session | None:
    """Revert cancelled/rescheduled session back to pending."""
    result = await db.execute(
        select(Session).where(Session.id_session == session_id)
    )
    session = result.scalar_one_or_none()
    if not session or session.status not in ("cancelled", "rescheduled"):
        return None

    old_status = session.status
    session.status = "pending"
    session.cancellation_reason = None
    session.rescheduled_to = None
    session.updated_at = datetime.now(timezone.utc)
    await db.flush()
    return session
