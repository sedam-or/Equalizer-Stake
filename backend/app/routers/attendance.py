"""Attendance router — submit and review attendance."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.attendance import AttendanceResponse, AttendanceReview, AttendanceSubmit
from app.services.attendance import submit_attendance, review_attendance
from app.models.attendance import Attendance

router = APIRouter(prefix="/api/attendance", tags=["Attendance"])


@router.get("/")
async def list_attendance(
    session_id: UUID | None = Query(None),
    teacher_id: UUID | None = Query(None),
    verification_status: str | None = Query(None),
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """List attendance records with optional filters."""
    base = select(Attendance)
    if session_id:
        base = base.where(Attendance.id_session == session_id)
    if teacher_id:
        base = base.where(Attendance.id_teacher == teacher_id)
    if verification_status:
        base = base.where(Attendance.verification_status == verification_status)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Attendance.created_at.desc()).offset(offset).limit(limit))
    records = result.scalars().all()
    return {
        "data": [
            {
                "id_attendance": str(a.id_attendance),
                "id_session": str(a.id_session),
                "id_teacher": str(a.id_teacher),
                "gps_lat": float(a.gps_lat),
                "gps_lng": float(a.gps_lng),
                "gps_distance_m": float(a.gps_distance_m) if a.gps_distance_m else None,
                "face_valid": a.face_valid,
                "face_confidence": float(a.face_confidence) if a.face_confidence else None,
                "verification_status": a.verification_status,
                "is_offline_submission": a.is_offline_submission,
                "created_at": a.created_at.isoformat() if a.created_at else None,
            }
            for a in records
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


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
