"""Scheduling schemas — request/response for session management."""

from datetime import date, time
from uuid import UUID

from pydantic import BaseModel, Field


class SessionCreate(BaseModel):
    """Create new session request."""
    id_student: UUID
    id_teacher: UUID
    id_room: UUID
    session_date: date
    start_time: time
    end_time: time
    session_type: str = "reguler"


class SessionUpdate(BaseModel):
    """Update session request."""
    session_date: date | None = None
    start_time: time | None = None
    end_time: time | None = None
    session_type: str | None = None


class SessionCancel(BaseModel):
    """Cancel session request."""
    reason: str = Field(..., min_length=3, max_length=500)


class SessionResponse(BaseModel):
    """Session response."""
    id_session: UUID
    id_student: UUID
    id_teacher: UUID
    id_room: UUID
    session_date: date
    start_time: time
    end_time: time
    session_type: str
    status: str
    cancellation_reason: str | None = None

    model_config = {"from_attributes": True}


class ConflictDetail(BaseModel):
    """Conflict detail for slot validation."""
    conflict_type: str  # teacher, room, student
    conflicting_session_id: UUID
    message: str


class SlotAvailability(BaseModel):
    """Slot availability check result."""
    available: bool
    conflicts: list[ConflictDetail] = []
    alternatives: list[dict] = []
