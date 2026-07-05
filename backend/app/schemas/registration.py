"""Registration schemas — Pydantic models for visitor registration."""

from pydantic import BaseModel, Field


class RegistrationCreate(BaseModel):
    """Visitor submits registration form (public, no auth)."""

    full_name: str = Field(..., min_length=2, max_length=255)
    phone_whatsapp: str = Field(..., min_length=8, max_length=20)
    email: str | None = None
    address: str | None = None
    instrument: str = Field(..., max_length=50)
    level: str = Field(..., max_length=50)
    class_type: str = Field(..., max_length=50)
    schedule_id: str | None = None


class RegistrationReject(BaseModel):
    """Admin rejects registration with reason."""

    notes: str = Field(..., min_length=1, max_length=1000)
