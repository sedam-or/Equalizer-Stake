"""Attendance router — submit and review attendance."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.attendance import AttendanceResponse, AttendanceReview, AttendanceSubmit
from app.services.attendance import submit_attendance, review_attendance

router = APIRouter(prefix="/api/attendance", tags=["Attendance"])


@router.post("/", response_model=AttendanceResponse, status_code=status.HTTP_201_CREATED)
async def submit_attendance_endpoint(
    data: AttendanceSubmit,
    user: User = Depends(require_role("teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Submit attendance (Guru only)."""
    try:
        attendance = await submit_attendance(
            db=db,
            id_session=data.id_session,
            gps_lat=data.gps_lat,
            gps_lng=data.gps_lng,
            face_valid=data.face_valid,
            face_confidence=data.face_confidence,
            client_timestamp=data.client_timestamp,
            is_offline_submission=data.is_offline_submission,
            teacher_id=user.id_user,
        )
        return attendance
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=str(e))


@router.put("/{attendance_id}/review", response_model=AttendanceResponse)
async def review_attendance_endpoint(
    attendance_id: str,
    data: AttendanceReview,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Review pending attendance (Admin/Owner only)."""
    attendance = await review_attendance(
        db, UUID(attendance_id), data.verification_status, user.id_user, data.rejection_reason
    )
    if not attendance:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Absensi tidak ditemukan atau sudah direview",
        )
    return attendance
