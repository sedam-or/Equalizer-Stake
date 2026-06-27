"""Scheduling router — session CRUD with conflict validation."""

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.scheduling import SessionCancel, SessionCreate, SessionResponse, SlotAvailability
from app.services.scheduling import create_session, validate_slot

router = APIRouter(prefix="/api/sessions", tags=["Scheduling"])


@router.post("/validate", response_model=SlotAvailability)
async def validate_slot_endpoint(
    data: SessionCreate,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Validate slot availability before creating session."""
    result = await validate_slot(
        db,
        teacher_id=data.id_teacher,
        room_id=data.id_room,
        student_id=data.id_student,
        session_date=data.session_date,
        start_time=data.start_time,
        end_time=data.end_time,
    )
    return result


@router.post("/", response_model=SessionResponse, status_code=status.HTTP_201_CREATED)
async def create_session_endpoint(
    data: SessionCreate,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Create new session with conflict validation."""
    # Validate first
    validation = await validate_slot(
        db,
        teacher_id=data.id_teacher,
        room_id=data.id_room,
        student_id=data.id_student,
        session_date=data.session_date,
        start_time=data.start_time,
        end_time=data.end_time,
    )

    if not validation.available:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"message": "Konflik slot terdeteksi", "conflicts": validation.conflicts},
        )

    session = await create_session(db, data)
    return session


@router.post("/{session_id}/cancel")
async def cancel_session(
    session_id: str,
    data: SessionCancel,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Cancel session with reason."""
    # TODO: implement cancel logic with credit refund rules
    return {"message": "Sesi dibatalkan", "reason": data.reason}
