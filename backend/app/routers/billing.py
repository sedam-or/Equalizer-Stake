"""Billing router — invoice and payment endpoints. T-094: audit logs."""

from datetime import datetime, timezone
from decimal import Decimal
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.models.invoice import Invoice
from app.schemas.billing import InvoiceResponse, PaymentResponse, PaymentVerifyRequest
from app.services.billing import apply_fine, verify_payment
from app.services.audit import create_audit_log

router = APIRouter(prefix="/api/billing", tags=["Billing"])


def _invoice_dict(inv: Invoice) -> dict:
    """Serialize invoice for audit log."""
    return {
        "id_invoice": str(inv.id_invoice),
        "id_guardian": str(inv.id_guardian),
        "billing_period": inv.billing_period,
        "total_amount": float(inv.total_amount),
        "status": inv.status,
        "due_date": inv.due_date.isoformat() if inv.due_date else None,
        "paid_at": inv.paid_at.isoformat() if inv.paid_at else None,
    }


@router.get("/invoices")
async def list_invoices(
    status_filter: str | None = Query(None, alias="status"),
    guardian_id: UUID | None = Query(None),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """List invoices with optional filters."""
    base = select(Invoice).where(Invoice.deleted_at.is_(None))
    if status_filter:
        base = base.where(Invoice.status == status_filter)
    if guardian_id:
        base = base.where(Invoice.id_guardian == guardian_id)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Invoice.created_at.desc()).offset(offset).limit(limit))
    invoices = result.scalars().all()
    return {
        "data": [
            {
                "id_invoice": str(inv.id_invoice),
                "id_guardian": str(inv.id_guardian),
                "billing_period": inv.billing_period,
                "total_amount": float(inv.total_amount),
                "due_date": inv.due_date.isoformat() if inv.due_date else None,
                "status": inv.status,
                "issued_at": inv.issued_at.isoformat() if inv.issued_at else None,
                "paid_at": inv.paid_at.isoformat() if inv.paid_at else None,
            }
            for inv in invoices
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.post("/payments/verify", response_model=PaymentResponse)
async def verify_payment_endpoint(
    data: PaymentVerifyRequest,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Verify payment by matching unique code."""
    # Get invoice before verification for audit
    result = await db.execute(select(Invoice).where(Invoice.id_invoice == data.id_invoice))
    invoice_before = result.scalar_one_or_none()
    before_state = _invoice_dict(invoice_before) if invoice_before else None

    payment = await verify_payment(
        db,
        invoice_id=data.id_invoice,
        unique_code=data.unique_code,
        amount=data.amount,
        verified_by=user.id_user,
    )

    if not payment:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Kode unik tidak ditemukan atau sudah terverifikasi",
        )

    # Get invoice after verification for audit
    result = await db.execute(select(Invoice).where(Invoice.id_invoice == data.id_invoice))
    invoice_after = result.scalar_one_or_none()
    after_state = _invoice_dict(invoice_after) if invoice_after else None

    await create_audit_log(
        db, table_name="payments", record_id=payment.id_payment,
        action="INSERT", changed_by=user.id_user,
        before_state=before_state, after_state=after_state, request=request,
    )
    await db.commit()

    return payment


@router.post("/invoices/{invoice_id}/fine")
async def apply_fine_endpoint(
    invoice_id: str,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Apply fine to overdue invoice (called by cron job)."""
    # Get invoice before fine for audit
    result = await db.execute(select(Invoice).where(Invoice.id_invoice == UUID(invoice_id)))
    invoice_before = result.scalar_one_or_none()
    before_state = _invoice_dict(invoice_before) if invoice_before else None

    invoice = await apply_fine(db, UUID(invoice_id))
    if not invoice:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invoice tidak valid atau sudah lunas",
        )

    after_state = _invoice_dict(invoice)
    await create_audit_log(
        db, table_name="invoices", record_id=invoice.id_invoice,
        action="UPDATE", changed_by=user.id_user,
        before_state=before_state, after_state=after_state, request=request,
    )
    await db.commit()

    return {"message": f"Denda Rp 100.000 diterapkan, akun dibekukan"}


@router.delete("/invoices/{invoice_id}")
async def delete_invoice(
    invoice_id: str,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Soft-delete invoice."""
    result = await db.execute(
        select(Invoice).where(Invoice.id_invoice == UUID(invoice_id), Invoice.deleted_at.is_(None))
    )
    invoice = result.scalar_one_or_none()
    if not invoice:
        raise HTTPException(status_code=404, detail="Tagihan tidak ditemukan")
    invoice.deleted_at = datetime.now(timezone.utc)
    await db.commit()
    return {"detail": "Tagihan berhasil dihapus"}
