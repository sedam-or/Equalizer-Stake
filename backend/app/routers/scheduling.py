"""Scheduling router — session CRUD with conflict validation."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.teacher import Teacher
from app.models.user import User
from app.schemas.scheduling import (
    RecordingApproval,
    RecordingRequestCreate,
    SessionCancel,
    SessionCreate,
    SessionResponse,
    SlotAvailability,
    StudioBookingCreate,
)
from app.services.audit import create_audit_log
from app.services.scheduling import (
    ConflictError,
    approve_recording,
    book_studio_rehearsal,
    create_session,
    request_recording_session,
    suggest_alternative_slots,
    validate_slot,
)

router = APIRouter(prefix="/api/sessions", tags=["Scheduling"])


@router.get("/", response_model=list[SessionResponse])
async def list_sessions(
    teacher_id: UUID | None = Query(None),
    student_id: UUID | None = Query(None),
    date_from: str | None = Query(None),
    date_to: str | None = Query(None),
    limit: int = Query(100, le=500),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner", "teacher", "guardian")),
    db: AsyncSession = Depends(get_db),
):
    """List sessions with optional filters."""
    from app.models.session import Session as SessionModel

    query = select(SessionModel).where(SessionModel.status.notin_(["cancelled"]))
    if teacher_id:
        query = query.where(SessionModel.id_teacher == teacher_id)
    if student_id:
        query = query.where(SessionModel.id_student == student_id)
    if date_from:
        from datetime import date as dt_date
        query = query.where(SessionModel.session_date >= dt_date.fromisoformat(date_from))
    if date_to:
        from datetime import date as dt_date
        query = query.where(SessionModel.session_date <= dt_date.fromisoformat(date_to))
    query = query.order_by(SessionModel.session_date.desc(), SessionModel.start_time).offset(offset).limit(limit)
    result = await db.execute(query)
    return result.scalars().all()


@router.get("/rooms")
async def list_rooms(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner", "teacher", "guardian")),
    db: AsyncSession = Depends(get_db),
):
    """List all available rooms for dropdown."""
    from app.models.room import Room
    base = select(Room).where(Room.is_available == True)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Room.room_name).offset(offset).limit(limit))
    return {
        "data": [
            {"id_room": str(r.id_room), "room_name": r.room_name, "room_type": r.room_type}
            for r in result.scalars().all()
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.get("/teachers")
async def list_teachers(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """List all active teachers for dropdown."""
    base = select(Teacher).where(Teacher.is_active == True)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Teacher.full_name).offset(offset).limit(limit))
    return {
        "data": [
            {"id_teacher": str(t.id_teacher), "full_name": t.full_name, "instruments": t.instruments_specialization}
            for t in result.scalars().all()
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


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
    request: Request,
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

    await create_audit_log(
        db,
        table_name="sessions",
        record_id=session.id_session,
        action="INSERT",
        changed_by=user.id_user,
        after_state={
            "id_student": str(data.id_student),
            "id_teacher": str(data.id_teacher),
            "id_room": str(data.id_room),
            "session_date": str(data.session_date),
            "start_time": str(data.start_time),
            "end_time": str(data.end_time),
            "session_type": data.session_type,
        },
        request=request,
        category="operasional",
    )
    await db.commit()

    return session


@router.post("/book-studio", response_model=SessionResponse, status_code=status.HTTP_201_CREATED)
async def book_studio_endpoint(
    data: StudioBookingCreate,
    request: Request,
    user: User = Depends(require_role("guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Guardian self-booking for rehearsal studio session."""
    try:
        session = await book_studio_rehearsal(
            db,
            user_id=user.id_user,
            student_id=data.id_student,
            room_id=data.id_room,
            session_date=data.session_date,
            start_time=data.start_time,
            end_time=data.end_time,
        )
    except PermissionError as e:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=str(e))
    except ConflictError as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={
                "message": "Slot sudah dipesan. Berikut alternatif terdekat.",
                "conflicts": e.conflicts,
                "alternatives": e.alternatives,
            },
        )
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(e))

    await create_audit_log(
        db,
        table_name="sessions",
        record_id=session.id_session,
        action="INSERT",
        changed_by=user.id_user,
        after_state={
            "id_student": str(data.id_student),
            "id_room": str(data.id_room),
            "session_date": str(data.session_date),
            "start_time": str(data.start_time),
            "end_time": str(data.end_time),
            "session_type": "rehearsal",
            "status": "pending",
        },
        request=request,
        category="operasional",
    )
    await db.commit()
    return session


@router.post("/request-recording", response_model=SessionResponse, status_code=status.HTTP_201_CREATED)
async def request_recording_endpoint(
    data: RecordingRequestCreate,
    request: Request,
    user: User = Depends(require_role("guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Guardian requests recording session (pending admin approval)."""
    try:
        session = await request_recording_session(
            db,
            user_id=user.id_user,
            student_id=data.id_student,
            room_id=data.id_room,
            session_date=data.session_date,
            start_time=data.start_time,
            end_time=data.end_time,
        )
    except PermissionError as e:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=str(e))
    except ConflictError as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={
                "message": "Slot sudah dipesan. Berikut alternatif terdekat.",
                "conflicts": e.conflicts,
                "alternatives": e.alternatives,
            },
        )
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(e))

    await create_audit_log(
        db,
        table_name="sessions",
        record_id=session.id_session,
        action="INSERT",
        changed_by=user.id_user,
        after_state={
            "id_student": str(data.id_student),
            "id_room": str(data.id_room),
            "session_date": str(data.session_date),
            "start_time": str(data.start_time),
            "end_time": str(data.end_time),
            "session_type": "recording",
            "status": "pending_approval",
        },
        request=request,
        category="operasional",
    )
    await db.commit()
    return session


@router.post("/{session_id}/approve-recording", response_model=SessionResponse)
async def approve_recording_endpoint(
    session_id: str,
    data: RecordingApproval,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Admin approves pending recording session."""
    try:
        session = await approve_recording(db, session_id=UUID(session_id), teacher_id=data.id_teacher)
    except LookupError as e:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail=str(e))
    except ConflictError as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={
                "message": "Konflik slot terdeteksi saat assign guru.",
                "conflicts": e.conflicts,
            },
        )
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(e))

    await create_audit_log(
        db,
        table_name="sessions",
        record_id=session.id_session,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"status": "pending_approval"},
        after_state={"status": "approved", "id_teacher": str(data.id_teacher)},
        request=request,
        category="operasional",
    )
    await db.commit()
    return session


@router.post("/{session_id}/cancel")
async def cancel_session(
    session_id: str,
    data: SessionCancel,
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Cancel session with reason."""
    from app.services.scheduling import validate_session_ownership
    from sqlalchemy import select
    from app.models.session import Session

    result = await db.execute(select(Session).where(Session.id_session == UUID(session_id)))
    session = result.scalar_one_or_none()
    if not session:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Sesi tidak ditemukan")

    before_state = {
        "status": session.status,
        "session_type": session.session_type,
    }

    session.status = "cancelled"
    session.cancellation_reason = data.reason

    await create_audit_log(
        db,
        table_name="sessions",
        record_id=session.id_session,
        action="UPDATE",
        changed_by=user.id_user,
        before_state=before_state,
        after_state={"status": "cancelled", "cancellation_reason": data.reason},
        request=request,
        category="operasional",
    )
    await db.commit()

    return {"message": "Sesi dibatalkan", "reason": data.reason}
