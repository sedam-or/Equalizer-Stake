"""Payment router — list + verify payments (admin)."""

from uuid import UUID
from datetime import datetime, timezone

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.payment import Payment
from app.models.user import User

router = APIRouter(prefix="/api/payments", tags=["Payments"])


class PaymentVerify(BaseModel):
    notes: str | None = None


@router.get("/")
async def list_payments(
    verified: bool | None = Query(None),
    method: str | None = Query(None),
    limit: int = Query(100, le=500),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """List payments (admin/owner)."""
    query = select(Payment)
    if verified is not None:
        query = query.where(Payment.is_verified == verified)
    if method:
        query = query.where(Payment.method == method)
    query = query.order_by(Payment.created_at.desc()).offset(offset).limit(limit)
    result = await db.execute(query)
    payments = result.scalars().all()
    return [
        {
            "id_payment": str(p.id_payment),
            "id_invoice": str(p.id_invoice),
            "amount": float(p.amount),
            "method": p.method,
            "payment_date": p.payment_date.isoformat(),
            "is_verified": p.is_verified,
            "verified_at": p.verified_at.isoformat() if p.verified_at else None,
            "notes": p.notes,
            "unique_code_used": p.unique_code_used,
        }
        for p in payments
    ]


@router.post("/{payment_id}/verify")
async def verify_payment(
    payment_id: UUID,
    body: PaymentVerify,
    user: User = Depends(require_role("admin")),
    db: AsyncSession = Depends(get_db),
):
    """Verify a payment (admin only)."""
    result = await db.execute(select(Payment).where(Payment.id_payment == payment_id))
    payment = result.scalar_one_or_none()
    if not payment:
        raise HTTPException(status_code=404, detail="Pembayaran tidak ditemukan")
    if payment.is_verified:
        raise HTTPException(status_code=400, detail="Pembayaran sudah diverifikasi")
    payment.is_verified = True
    payment.verified_by = user.id_user
    payment.verified_at = datetime.now(timezone.utc)
    if body.notes:
        payment.notes = body.notes
    await db.commit()
    return {"detail": "Pembayaran berhasil diverifikasi"}
