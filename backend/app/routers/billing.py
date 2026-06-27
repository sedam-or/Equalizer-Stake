"""Billing router — invoice and payment endpoints."""

from decimal import Decimal
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.billing import InvoiceResponse, PaymentResponse, PaymentVerifyRequest
from app.services.billing import apply_fine, verify_payment

router = APIRouter(prefix="/api/billing", tags=["Billing"])


@router.post("/payments/verify", response_model=PaymentResponse)
async def verify_payment_endpoint(
    data: PaymentVerifyRequest,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Verify payment by matching unique code."""
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

    return payment


@router.post("/invoices/{invoice_id}/fine")
async def apply_fine_endpoint(
    invoice_id: str,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Apply fine to overdue invoice (called by cron job)."""
    invoice = await apply_fine(db, UUID(invoice_id))
    if not invoice:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invoice tidak valid atau sudah lunas",
        )
    return {"message": f"Denda Rp 100.000 diterapkan, akun dibekukan"}
