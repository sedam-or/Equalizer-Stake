"""Session model — course/studio session schedule."""

import uuid
from datetime import date, datetime, time

from sqlalchemy import DateTime, Enum, ForeignKey, Text, Time, UniqueConstraint, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base


class Session(Base):
    """Course or studio session schedule."""

    __tablename__ = "sessions"
    __table_args__ = (
        UniqueConstraint("id_room", "session_date", "start_time", name="uq_session_room_date_start"),
    )

    id_session: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    id_teacher: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("teachers.id_teacher"), nullable=False)
    id_student: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("students.id_student"), nullable=False)
    id_room: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("rooms.id_room"), nullable=False)
    session_date: Mapped[date] = mapped_column(nullable=False)
    start_time: Mapped[time] = mapped_column(Time, nullable=False)
    end_time: Mapped[time] = mapped_column(Time, nullable=False)
    session_type: Mapped[str] = mapped_column(
        Enum("reguler", "teori", "rehearsal", "recording", "kompensasi", name="session_type"),
        nullable=False, default="reguler"
    )
    status: Mapped[str] = mapped_column(
        Enum("draft", "pending", "locked", "completed", "cancelled", "rescheduled", "pending_approval", "approved", name="session_status"),
        nullable=False, default="draft"
    )
    rescheduled_to: Mapped[uuid.UUID | None] = mapped_column(UUID(as_uuid=True), ForeignKey("sessions.id_session"), nullable=True)
    cancellation_reason: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
