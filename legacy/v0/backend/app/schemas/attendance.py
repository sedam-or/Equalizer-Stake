"""Attendance schemas — request/response for attendance submission."""

from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, Field


class AttendanceSubmit(BaseModel):
    """Attendance submission request."""
    id_session: UUID
    photo_base64: str | None = None  # base64 encoded photo, encrypted before storage
    gps_lat: float = Field(..., ge=-90, le=90)
    gps_lng: float = Field(..., ge=-180, le=180)
    face_valid: bool
    face_confidence: float | None = None
    client_timestamp: datetime
    is_offline_submission: bool = False


class AttendanceResponse(BaseModel):
    """Attendance response."""
    id_attendance: UUID
    id_session: UUID
    gps_lat: float
    gps_lng: float
    gps_distance_m: float
    face_valid: bool
    verification_status: str
    server_timestamp: datetime

    model_config = {"from_attributes": True}


class AttendanceReview(BaseModel):
    """Admin review of pending attendance."""
    verification_status: str = Field(..., pattern="^(verified|rejected)$")
    rejection_reason: str | None = None
