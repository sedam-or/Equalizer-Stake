"""Student model — registered student."""

import uuid
from datetime import date, datetime

from sqlalchemy import Boolean, Date, DateTime, Enum, ForeignKey, String, Text, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base
from app.services.encryption import EncryptedString


class Student(Base):
    """Registered student linked to one Guardian."""

    __tablename__ = "students"

    id_student: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    id_guardian: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("guardians.id_guardian"), nullable=False, index=True)
    full_name: Mapped[str] = mapped_column(EncryptedString(500), nullable=False)  # AES-256-GCM
    date_of_birth: Mapped[date | None] = mapped_column(Date, nullable=True)
    instrument: Mapped[str] = mapped_column(
        Enum("biola", "piano", "gitar", "drum", "keyboard", "flute", "saxophone", "vokal", "bass", "ensemble", name="instrument_type"),
        nullable=False
    )
    level: Mapped[str] = mapped_column(
        String(50), nullable=False, default="pemula"
    )  # Per-instrumen: Piano Klasik=pra_elementary..grade_6, lain=basic/intermediate/advance
    enrollment_status: Mapped[str] = mapped_column(
        Enum("active", "cuti", "inactive", "trial", name="enrollment_status"),
        nullable=False, default="active"
    )
    enrollment_date: Mapped[date] = mapped_column(Date, nullable=False)
    trial_end_date: Mapped[date | None] = mapped_column(Date, nullable=True)
    eligible_for_exam: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
    deleted_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
