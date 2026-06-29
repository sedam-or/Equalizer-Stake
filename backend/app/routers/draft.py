"""Draft router — auto-save draft endpoints."""

from fastapi import APIRouter, Depends, HTTPException, Request, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.draft import delete_draft, get_draft, get_user_drafts, save_draft

router = APIRouter(prefix="/api/drafts", tags=["Drafts"])


@router.get("/")
async def list_my_drafts(
    user: User = Depends(require_role("owner", "admin", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """List all drafts for current user."""
    drafts = await get_user_drafts(db, user.id_user)
    return {
        "drafts": [
            {
                "id_draft": str(d.id_draft),
                "form_key": d.form_key,
                "draft_data": d.draft_data,
                "updated_at": d.updated_at.isoformat() if d.updated_at else None,
            }
            for d in drafts
        ]
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

    draft = await save_draft(db, user.id_user, form_key, draft_data)
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
