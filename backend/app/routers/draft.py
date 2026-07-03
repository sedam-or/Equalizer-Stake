"""Draft router — auto-save draft endpoints."""

from datetime import datetime

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.models.draft import Draft
from app.services.draft import DraftConflictError, delete_draft, get_draft, save_draft

router = APIRouter(prefix="/api/v1/drafts", tags=["Drafts"])


@router.get("")
async def list_my_drafts(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner", "admin", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """List all drafts for current user."""
    base = select(Draft).where(Draft.id_user == user.id_user)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Draft.updated_at.desc()).offset(offset).limit(limit))
    drafts = result.scalars().all()
    return {
        "data": [
            {
                "id_draft": str(d.id_draft),
                "form_key": d.form_key,
                "draft_data": d.draft_data,
                "updated_at": d.updated_at.isoformat() if d.updated_at else None,
            }
            for d in drafts
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.get("/{form_key}")
async def get_form_draft(
    form_key: str,
    user: User = Depends(require_role("owner", "admin", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Get draft for specific form."""
    draft = await get_draft(db, user.id_user, form_key)
    if not draft:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Draft tidak ditemukan",
        )
    return {
        "id_draft": str(draft.id_draft),
        "form_key": draft.form_key,
        "draft_data": draft.draft_data,
        "updated_at": draft.updated_at.isoformat() if draft.updated_at else None,
    }


@router.post("/{form_key}")
async def save_form_draft(
    form_key: str,
    request: Request,
    user: User = Depends(require_role("owner", "admin", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Save or update draft for a form."""
    body = await request.json()
    draft_data = body.get("draft_data")
    if not draft_data or not isinstance(draft_data, dict):
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Field 'draft_data' harus berupa object JSON",
        )

    # Parse optional client timestamp for conflict detection
    client_updated_at = None
    raw_ts = body.get("updated_at")
    if raw_ts:
        try:
            client_updated_at = datetime.fromisoformat(raw_ts)
        except (ValueError, TypeError):
            raise HTTPException(
                status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
                detail="Format 'updated_at' tidak valid, gunakan ISO 8601",
            )

    try:
        draft = await save_draft(db, user.id_user, form_key, draft_data, client_updated_at)
    except DraftConflictError as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={
                "message": "Draft konflik: versi server lebih baru",
                "server_draft": {
                    "draft_data": e.server_draft.draft_data,
                    "updated_at": e.server_draft.updated_at.isoformat(),
                },
                "client_draft": {
                    "draft_data": e.client_data,
                },
            },
        )
    await db.commit()

    return {
        "detail": "Draft tersimpan",
        "id_draft": str(draft.id_draft),
        "form_key": form_key,
        "updated_at": draft.updated_at.isoformat() if draft.updated_at else None,
    }


@router.delete("/{form_key}")
async def delete_form_draft(
    form_key: str,
    user: User = Depends(require_role("owner", "admin", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Delete draft after permanent save."""
    deleted = await delete_draft(db, user.id_user, form_key)
    await db.commit()
    if not deleted:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Draft tidak ditemukan",
        )
    return {"detail": "Draft dihapus"}
