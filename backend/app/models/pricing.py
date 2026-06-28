"""Pricing model — tarif kursus/studio per instrument & level."""

import uuid
from datetime import date, datetime

from sqlalchemy import Boolean, Date, DateTime, Enum, Numeric, String, Text, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base


class Pricing(Base):
    """Course/studio pricing per instrument and level with effective date."""

    __tablename__ = "pricings"

    id_pricing: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid.uuid4
    )
    instrument: Mapped[str] = mapped_column(
        Enum(
            "biola", "piano", "gitar", "drum", "keyboard",
            "flute", "saxophone", "vokal", "ensemble",
            name="instrument_type",
        ),
        nullable=False,
    )
    level: Mapped[str] = mapped_column(
        Enum("pemula", "dasar", "menengah", "lanjut", name="student_level"),
        nullable=False,
    )
    price: Mapped[float] = mapped_column(Numeric(12, 2), nullable=False)
    effective_date: Mapped[date] = mapped_column(Date, nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    created_by: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), nullable=True
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False
    )
