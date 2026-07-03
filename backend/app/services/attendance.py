"""Attendance service — GPS/face validation, submission, review."""

import math
from datetime import datetime, timezone, timedelta
from uuid import UUID

from sqlalchemy import and_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.attendance import Attendance
from app.models.session import Session
from app.models.teacher import Teacher

# GPS constants
STUDIO_LAT = -7.737428
STUDIO_LNG = 110.429669
GPS_RADIUS_M = 150
GPS_TOLERANCE_M = 15  # jitter compensation


def haversine_distance(lat1: float, lng1: float, lat2: float, lng2: float) -> float:
    """Calculate distance in meters between two GPS points using Haversine formula."""
    R = 6371000  # Earth radius in meters
    phi1 = math.radians(lat1)
    phi2 = math.radians(lat2)
    delta_phi = math.radians(lat2 - lat1)
    delta_lambda = math.radians(lng2 - lng1)

    a = math.sin(delta_phi / 2) ** 2 + math.cos(phi1) * math.cos(phi2) * math.sin(delta_lambda / 2) ** 2
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))

    return R * c


def validate_gps(gps_lat: float, gps_lng: float) -> tuple[bool, float]:
    """Validate GPS distance from studio. Returns (valid, distance_m)."""
    distance = haversine_distance(gps_lat, gps_lng, STUDIO_LAT, STUDIO_LNG)
    effective_radius = GPS_RADIUS_M + GPS_TOLERANCE_M
    return distance <= effective_radius, distance


def validate_server_timestamp(client_ts: datetime, session_date, session_start_time) -> bool:
    """Validate server timestamp within [-30min, +15min] of session start."""
    now = datetime.now(timezone.utc)
    # Combine date + time into datetime
    session_dt = datetime.combine(session_date, session_start_time, tzinfo=timezone.utc)

    earliest = session_dt - timedelta(minutes=30)
    latest = session_dt + timedelta(minutes=15)

    return earliest <= now <= latest


async def submit_attendance(
    db: AsyncSession,
    id_session: UUID,
    gps_lat: float,
    gps_lng: float,
    face_valid: bool,
    face_confidence: float | None,
    client_timestamp: datetime,
    is_offline_submission: bool,
    teacher_id: UUID,
) -> Attendance:
    """Submit attendance with GPS/face validation."""
    # Get session
    result = await db.execute(select(Session).where(Session.id_session == id_session))
    session = result.scalar_one_or_none()

    if not session:
        raise ValueError("Sesi tidak ditemukan")

    # Session ownership: verify teacher owns this session
    ownership_q = (
        select(Session)
        .join(Teacher, Session.id_teacher == Teacher.id_teacher)
        .where(and_(Session.id_session == id_session, Teacher.id_user == teacher_id))
    )
    ownership_result = await db.execute(ownership_q)
    if ownership_result.scalar_one_or_none() is None:
        raise ValueError("Guru bukan pemilik sesi ini")

    # Validate GPS — reject if outside 150m
    gps_valid, gps_distance = validate_gps(gps_lat, gps_lng)
    if not gps_valid:
        raise ValueError(f"Anda di luar jangkauan studio ({gps_distance:.0f}m). Maksimal 150m dari studio.")

    # Validate server timestamp
    time_valid = validate_server_timestamp(client_timestamp, session.session_date, session.start_time)

    # Determine verification status
    all_valid = gps_valid and time_valid and face_valid
    verification_status = "verified" if all_valid else "pending_review"

    # Create attendance record
    attendance = Attendance(
        id_session=id_session,
        id_teacher=teacher_id,
        photo_url="pending_upload",  # Will be updated when photo is uploaded
        gps_lat=gps_lat,
        gps_lng=gps_lng,
        gps_distance_m=gps_distance,
        face_valid=face_valid,
        face_confidence=face_confidence,
        client_timestamp=client_timestamp,
        verification_status=verification_status,
        is_offline_submission=is_offline_submission,
    )
    db.add(attendance)
    await db.commit()
    await db.refresh(attendance)
    return attendance


async def review_attendance(
    db: AsyncSession,
    attendance_id: UUID,
    status: str,
    reviewed_by: UUID,
    reason: str | None = None,
) -> Attendance | None:
    """Admin review of pending attendance."""
    result = await db.execute(
        select(Attendance).where(Attendance.id_attendance == attendance_id)
    )
    attendance = result.scalar_one_or_none()

    if not attendance or attendance.verification_status != "pending_review":
        return None

    attendance.verification_status = status
    attendance.reviewed_by = reviewed_by
    attendance.reviewed_at = datetime.now(timezone.utc)
    attendance.rejection_reason = reason

    await db.commit()
    await db.refresh(attendance)
    return attendance
