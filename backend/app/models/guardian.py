"""Guardian model — Wali Murid profile."""

import uuid
from datetime import datetime

from sqlalchemy import DateTime, Enum, ForeignKey, String, Text, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.core.database import Base


class Guardian(Base):
    """Parent/guardian profile. 1 Guardian : N Students."""

    __tablename__ = "guardians"

    id_guardian: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    id_user: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), ForeignKey("users.id_user"), unique=True, nullable=False)
    full_name: Mapped[str] = mapped_column(Text, nullable=False)  # KRITIS: AES-256-GCM
    phone_whatsapp: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)  # +62xxx
    email: Mapped[str | None] = mapped_column(String(255), nullable=True)
    address: Mapped[str | None] = mapped_column(Text, nullable=True)
    account_status: Mapped[str] = mapped_column(
        Enum("active", "frozen", "inactive", "pending_deletion", name="account_status"),
        nullable=False, default="active"
    )
    frozen_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    frozen_reason: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
    deleted_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
