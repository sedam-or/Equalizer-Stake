"""Notification schemas — request/response for notification management."""

from datetime import datetime
from uuid import UUID

from pydantic import BaseModel


class NotificationResponse(BaseModel):
    """Notification response."""
    id_notification: UUID
    recipient_id: UUID
    channel: str
    category: str
    subject: str | None = None
    body: str
    status: str
    retry_count: int
    sent_at: datetime | None = None
    created_at: datetime

    model_config = {"from_attributes": True}


class NotificationToggle(BaseModel):
    """Admin toggle notification channel."""
    channel: str  # whatsapp, email
    enabled: bool
    category: str | None = None  # critical, non_critical, or None for all
