"""Privacy router — UU PDP data deletion endpoints."""

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.privacy import create_deletion_request, complete_deletion

router = APIRouter(prefix="/api/v1/privacy", tags=["Privacy"])


@router.post("/delete-request")
async def request_deletion(
    reason: str,
    entity_type: str,
    entity_id: str,
    user: User = Depends(require_role("guardian", "teacher")),
    db: AsyncSession = Depends(get_db),
):
    """Submit data deletion request (UU PDP Pasal 19)."""
    from uuid import UUID

    result = await create_deletion_request(
        db=db,
        user_id=user.id_user,
        reason=reason,
        entity_type=entity_type,
        entity_id=UUID(entity_id),
    )

    if result.get("status") == "rejected":
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=result["reason"],
        )

    return {
        "message": "Permintaan hapus data diterima. Akan diproses dalam ≤14 hari kerja.",
        "request": result,
    }


@router.put("/delete-request/{request_id}/verify")
async def verify_deletion(
    request_id: str,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Admin verify and complete deletion."""
    result = await complete_deletion(
        db=db,
        request_id=request_id,
        verified_by=user.id_user,
    )

    return {
        "message": "Data berhasil dihapus dan dianonimkan.",
        "result": result,
    }
