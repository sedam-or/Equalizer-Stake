"""Attendance model — teacher attendance per session."""

import uuid
from datetime import datetime

from sqlalchemy import Boolean, DateTime, Enum, Float, ForeignKey, Numeric, Text, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base
from app.services.encryption import EncryptedString


class Attendance(Base):
    """Teacher attendance record per session with GPS/face validation."""

    __tablename__ = "attendances"

    id_attendance: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    id_session: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("sessions.id_session"), unique=True, index=True, nullable=False)
    id_teacher: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("teachers.id_teacher"), nullable=False, index=True)
    photo_url: Mapped[str] = mapped_column(EncryptedString(1000), nullable=False)  # AES-256-GCM
    gps_lat: Mapped[float] = mapped_column(Numeric(10, 7), nullable=False)  # SRS: DECIMAL(10,7)
    gps_lng: Mapped[float] = mapped_column(Numeric(10, 7), nullable=False)  # SRS: DECIMAL(10,7)
    gps_distance_m: Mapped[float] = mapped_column(Float, nullable=False)
    face_valid: Mapped[bool] = mapped_column(Boolean, nullable=False)
    face_confidence: Mapped[float | None] = mapped_column(Float, nullable=True)
    client_timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    server_timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    verification_status: Mapped[str] = mapped_column(
        Enum("verified", "pending_review", "rejected", name="verification_status"),
        nullable=False, default="pending_review"
    )
    rejection_reason: Mapped[str | None] = mapped_column(Text, nullable=True)
    reviewed_by: Mapped[uuid.UUID | None] = mapped_column(UUID(as_uuid=True), ForeignKey("users.id_user"), nullable=True)
    reviewed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    is_offline_submission: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
