"""Teacher model — music teacher profile."""

import uuid
from datetime import datetime

from sqlalchemy import Boolean, DateTime, Enum, ForeignKey, Integer, String, func
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base


class Teacher(Base):
    """Music teacher profile linked to User."""

    __tablename__ = "teachers"

    id_teacher: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    id_user: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("users.id_user"), unique=True, nullable=False)
    full_name: Mapped[str] = mapped_column(String(255), nullable=False)
    instruments_specialization: Mapped[dict] = mapped_column(JSONB, nullable=False)  # ["biola","piano"]
    employment_type: Mapped[str] = mapped_column(Enum("tetap", "freelance", name="employment_type"), nullable=False)
    max_students_per_session: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
    deleted_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
