"""Payment router — list + verify + create payments. T-096: audit logs."""

from uuid import UUID
from datetime import datetime, timedelta, timezone

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from pydantic import BaseModel
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.core.limiter import limiter
from app.middleware.auth import require_role
from app.models.payment import Payment
from app.models.user import User
from app.services.audit import create_audit_log

router = APIRouter(prefix="/api/v1/payments", tags=["Payments"])

# Circuit breaker for payment verification: 5 consecutive failures → open 30 min
_CB_THRESHOLD = 5
_CB_COOLDOWN = 1800
_cb_state: dict = {"failures": 0, "open_until": None}


def _is_cb_open() -> bool:
    if _cb_state["open_until"]:
        if datetime.now(timezone.utc) < _cb_state["open_until"]:
            return True
        _cb_state["failures"] = 0
        _cb_state["open_until"] = None
    return False


def _record_cb_failure():
    _cb_state["failures"] += 1
    if _cb_state["failures"] >= _CB_THRESHOLD:
        _cb_state["open_until"] = datetime.now(timezone.utc) + timedelta(seconds=_CB_COOLDOWN)


def _reset_cb():
    _cb_state["failures"] = 0
    _cb_state["open_until"] = None


def _payment_dict(p: Payment) -> dict:
    """Serialize payment for audit log."""
    return {
        "id_payment": str(p.id_payment),
        "id_invoice": str(p.id_invoice),
        "amount": float(p.amount),
        "method": p.method,
        "is_verified": p.is_verified,
        "verified_by": str(p.verified_by) if p.verified_by else None,
        "notes": p.notes,
    }


class PaymentVerify(BaseModel):
    unique_code: str
    notes: str | None = None


@router.get("")
async def list_payments(
    verified: bool | None = Query(None),
    method: str | None = Query(None),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """List payments (admin/owner)."""
    base = select(Payment).where(Payment.deleted_at.is_(None))
    if verified is not None:
        base = base.where(Payment.is_verified == verified)
    if method:
        base = base.where(Payment.method == method)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Payment.created_at.desc()).offset(offset).limit(limit))
    payments = result.scalars().all()
    return {
        "data": [
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
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.post("/{payment_id}/verify")
@limiter.limit("20/minute")
async def verify_payment(
    request: Request,
    payment_id: UUID,
    body: PaymentVerify,
    user: User = Depends(require_role("admin")),
    db: AsyncSession = Depends(get_db),
):
    """Verify a payment (admin only)."""
    if _is_cb_open():
        raise HTTPException(
            status_code=503,
            detail="Verifikasi pembayaran tidak tersedia. Terlalu banyak kegagalan. Coba lagi dalam 30 menit.",
        )
    try:
        result = await db.execute(select(Payment).where(Payment.id_payment == payment_id))
        payment = result.scalar_one_or_none()
        if not payment:
            raise HTTPException(status_code=404, detail="Pembayaran tidak ditemukan")
        if payment.is_verified:
            raise HTTPException(status_code=400, detail="Pembayaran sudah diverifikasi")

        before_state = _payment_dict(payment)

        # Validate unique code from invoice
        from app.models.invoice import Invoice
        invoice_result = await db.execute(select(Invoice).where(Invoice.id_invoice == payment.id_invoice))
        invoice = invoice_result.scalar_one_or_none()
        if invoice and invoice.unique_code != body.unique_code:
            raise HTTPException(status_code=400, detail="Kode unik tidak cocok. Periksa kembali bukti transfer.")

        payment.is_verified = True
        payment.verified_by = user.id_user
        payment.verified_at = datetime.now(timezone.utc)
        if body.notes:
            payment.notes = body.notes

        after_state = _payment_dict(payment)
        await create_audit_log(
            db, table_name="payments", record_id=payment_id,
            action="UPDATE", changed_by=user.id_user,
            before_state=before_state, after_state=after_state, request=request,
        )
        await db.commit()
        _reset_cb()
        return {"detail": "Pembayaran berhasil diverifikasi"}
    except HTTPException:
        raise
    except Exception:
        _record_cb_failure()
        raise HTTPException(status_code=500, detail="Gagal memverifikasi pembayaran. Silakan coba lagi.")


@router.post("")
async def create_payment(
    id_invoice: str,
    amount: float,
    method: str,
    proof_base64: str | None = None,
    user: User = Depends(require_role("guardian")),
    db: AsyncSession = Depends(get_db),
    request: Request = Depends(),
):
    """Guardian submits payment with optional proof photo."""
    from app.models.invoice import Invoice
    from app.models.payment import Payment
    from app.models.guardian import Guardian
    from app.services.photo_storage import validate_photo, save_attendance_photo
    from datetime import datetime, timezone

    # Get guardian from user
    guardian_result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
    guardian = guardian_result.scalar_one_or_none()
    if not guardian:
        raise HTTPException(status_code=403, detail="Hanya guardian yang bisa membayar")

    # Verify invoice belongs to guardian
    result = await db.execute(
        select(Invoice).where(
            Invoice.id_invoice == id_invoice,
            Invoice.id_guardian == guardian.id_guardian,
        )
    )
    invoice = result.scalar_one_or_none()
    if not invoice:
        raise HTTPException(status_code=404, detail="Tagihan tidak ditemukan")

    # Save proof photo if provided
    proof_url = None
    if proof_base64:
        valid, error = validate_photo(proof_base64)
        if not valid:
            raise HTTPException(status_code=400, detail=error)
        proof_url = save_attendance_photo(proof_base64, str(invoice.id_invoice))

    payment = Payment(
        id_invoice=id_invoice,
        amount=amount,
        method=method,
        proof_url=proof_url,
        status="pending",
    )
    db.add(payment)

    # Update invoice status
    invoice.status = "pending"

    await db.flush()
    await create_audit_log(
        db, table_name="payments", record_id=payment.id_payment,
        action="INSERT", changed_by=user.id_user,
        after_state=_payment_dict(payment), request=request,
    )
    await db.commit()

    return {"message": "Pembayaran berhasil dikirim. Menunggu verifikasi admin.", "payment_id": str(payment.id_payment)}


@router.delete("/{payment_id}")
async def delete_payment(
    payment_id: UUID,
    request: Request,
    user: User = Depends(require_role("admin")),
    db: AsyncSession = Depends(get_db),
):
    """Soft-delete payment."""
    result = await db.execute(
        select(Payment).where(Payment.id_payment == payment_id, Payment.deleted_at.is_(None))
    )
    payment = result.scalar_one_or_none()
    if not payment:
        raise HTTPException(status_code=404, detail="Pembayaran tidak ditemukan")
    payment.deleted_at = datetime.now(timezone.utc)
    await create_audit_log(
        db, table_name="payments", record_id=payment_id,
        action="DELETE", changed_by=user.id_user,
        before_state=_payment_dict(payment), request=request,
    )
    await db.commit()
    return {"detail": "Pembayaran berhasil dihapus"}
