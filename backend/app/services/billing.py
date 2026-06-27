"""Billing service — invoice generation, fine, payment verification."""

import random
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal
from uuid import UUID

from sqlalchemy import and_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.guardian import Guardian
from app.models.invoice import Invoice
from app.models.payment import Payment
from app.models.student import Student

UNIQUE_CODE_MAX_RETRIES = 3


async def generate_unique_code(db: AsyncSession, billing_period: str, guardian_id: UUID) -> str:
    """Generate unique 3-digit code with collision handling."""
    for _ in range(UNIQUE_CODE_MAX_RETRIES):
        code = f"{random.randint(0, 999):03d}"
        existing = await db.execute(
            select(Invoice).where(
                and_(
                    Invoice.billing_period == billing_period,
                    Invoice.id_guardian == guardian_id,
                    Invoice.unique_code == code,
                )
            )
        )
        if not existing.scalar_one_or_none():
            return code
    # After 3 retries, return manual assignment marker
    return "999"  # Will be flagged for manual assignment


async def create_invoice(db: AsyncSession, guardian_id: UUID, billing_period: str, total_amount: Decimal) -> Invoice:
    """Create invoice for guardian."""
    unique_code = await generate_unique_code(db, billing_period, guardian_id)
    due_date = date.today().replace(day=10)  # Due on 10th

    invoice = Invoice(
        id_guardian=guardian_id,
        unique_code=unique_code,
        billing_period=billing_period,
        total_amount=total_amount,
        due_date=due_date,
        status="issued",
        issued_at=datetime.now(timezone.utc),
    )
    db.add(invoice)
    await db.commit()
    await db.refresh(invoice)
    return invoice


async def verify_payment(
    db: AsyncSession,
    invoice_id: UUID,
    unique_code: str,
    amount: Decimal,
    verified_by: UUID,
) -> Payment | None:
    """Verify payment by matching unique code."""
    result = await db.execute(select(Invoice).where(Invoice.id_invoice == invoice_id))
    invoice = result.scalar_one_or_none()

    if not invoice:
        return None

    # Check unique code match
    if invoice.unique_code != unique_code:
        return None

    # Check idempotency — already verified?
    existing_payment = await db.execute(
        select(Payment).where(
            and_(
                Payment.id_invoice == invoice_id,
                Payment.unique_code_used == unique_code,
                Payment.is_verified == True,
            )
        )
    )
    if existing_payment.scalar_one_or_none():
        return None  # Already verified

    # Create payment record
    payment = Payment(
        id_invoice=invoice_id,
        amount=amount,
        method="transfer",
        payment_date=datetime.now(timezone.utc),
        unique_code_used=unique_code,
        is_verified=True,
        verified_by=verified_by,
        verified_at=datetime.now(timezone.utc),
    )
    db.add(payment)

    # Update invoice status
    invoice.status = "paid"
    invoice.paid_at = datetime.now(timezone.utc)

    await db.commit()
    await db.refresh(payment)
    return payment


async def apply_fine(db: AsyncSession, invoice_id: UUID) -> Invoice | None:
    """Apply Rp100k fine and freeze account."""
    result = await db.execute(select(Invoice).where(Invoice.id_invoice == invoice_id))
    invoice = result.scalar_one_or_none()

    if not invoice or invoice.status != "issued":
        return None

    # Apply fine
    invoice.total_amount += Decimal("100000")
    invoice.status = "overdue"

    # Freeze guardian account
    guardian_result = await db.execute(
        select(Guardian).where(Guardian.id_guardian == invoice.id_guardian)
    )
    guardian = guardian_result.scalar_one_or_none()
    if guardian:
        guardian.account_status = "frozen"
        guardian.frozen_at = datetime.now(timezone.utc)
        guardian.frozen_reason = f"Tagihan {invoice.billing_period} belum lunas"

    await db.commit()
    await db.refresh(invoice)
    return invoice
