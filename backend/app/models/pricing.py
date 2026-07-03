"""Pricing model — tarif kursus BMD MS per instrument, class_type, level.

SRS Addendum: 9 instrumen, 9 tipe kelas, level per instrumen.
Harga asli BMD MS +20% (update 2024).
"""

import uuid
from datetime import date, datetime

from sqlalchemy import Boolean, Date, DateTime, Integer, Numeric, String, Text, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base


class Pricing(Base):
    """Course pricing per instrument × class_type × level."""

    __tablename__ = "pricings"

    id_pricing: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    instrument: Mapped[str] = mapped_column(String(50), nullable=False)  # piano_klasik, piano_pop, vokal, biola, gitar, keyboard, drum, flute, saxophone
    class_type: Mapped[str] = mapped_column(String(50), nullable=False)  # private, intensif_2, intensif_3, semi_intensif, expatriate_private, expatriate_2, expatriate_3, tunas, paket_liburan
    level: Mapped[str] = mapped_column(String(50), nullable=False)  # per instrument (pra_elementary, elementary, grade_1..6, basic, intermediate, advance, etc)
    monthly_fee: Mapped[float] = mapped_column(Numeric(12, 2), nullable=False)  # SPP per bulan
    duration_minutes: Mapped[int] = mapped_column(Integer, nullable=False, default=45)  # 30 atau 45
    registration_fee: Mapped[float] = mapped_column(Numeric(12, 2), nullable=False, default=0)  # biaya daftar
    effective_date: Mapped[date] = mapped_column(Date, nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    created_by: Mapped[uuid.UUID | None] = mapped_column(UUID(as_uuid=True), nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
