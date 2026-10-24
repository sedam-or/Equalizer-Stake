"""Billing schemas — request/response for invoice and payment."""

from datetime import date, datetime
from decimal import Decimal
from uuid import UUID

from pydantic import BaseModel, Field


class InvoiceResponse(BaseModel):
    """Invoice response."""
    id_invoice: UUID
    id_guardian: UUID
    unique_code: str
    billing_period: str
    total_amount: Decimal
    due_date: date
    status: str
    issued_at: datetime | None = None
    paid_at: datetime | None = None
    notes: str | None = None

    model_config = {"from_attributes": True}


class PaymentVerifyRequest(BaseModel):
    """Payment verification request by Admin."""
    id_invoice: UUID
    unique_code: str = Field(..., min_length=3, max_length=3)
    amount: Decimal
    method: str = "transfer"
    proof_url: str | None = None
    notes: str | None = None


class PaymentResponse(BaseModel):
    """Payment response."""
    id_payment: UUID
    id_invoice: UUID
    amount: Decimal
    method: str
    unique_code_used: str
    is_verified: bool
    verified_at: datetime | None = None

    model_config = {"from_attributes": True}


class FineApplyRequest(BaseModel):
    """Apply fine to invoice."""
    id_invoice: UUID
    fine_amount: Decimal = Decimal("100000")
