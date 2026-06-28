"""Admin management router — superadmin CRUD for admin accounts."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.admin_mgmt import (
    AdminResponse,
    CreateAdminRequest,
    DeleteAdminRequest,
)
from app.services.admin_mgmt import (
    activate_admin,
    create_admin,
    deactivate_admin,
    delete_admin,
    list_admins,
)
from app.services.audit import create_audit_log

router = APIRouter(prefix="/api/admin-mgmt", tags=["Admin Management"])


def _to_response(admin: User) -> dict:
    return {
        "id_user": str(admin.id_user),
        "username": admin.username,
        "email": admin.email,
        "is_active": admin.is_active,
        "created_at": admin.created_at.isoformat(),
    }


@router.get("/admins")
async def list_admins_endpoint(
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """List all admin accounts (Owner only)."""
    admins = await list_admins(db)
    return {"admins": [_to_response(a) for a in admins], "total": len(admins)}


@router.post("/admins", status_code=status.HTTP_201_CREATED)
async def create_admin_endpoint(
    data: CreateAdminRequest,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Create new admin account (Owner only)."""
    try:
        admin = await create_admin(db, data.username, data.email, data.password)
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=str(e))

    await create_audit_log(
        db,
        table_name="users",
        record_id=admin.id_user,
        action="INSERT",
        changed_by=user.id_user,
        after_state={"username": admin.username, "email": admin.email, "role": "admin"},
    )
    await db.commit()
    return _to_response(admin)


@router.put("/admins/{admin_id}/deactivate")
async def deactivate_admin_endpoint(
    admin_id: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Deactivate admin account (Owner only)."""
    admin = await deactivate_admin(db, UUID(admin_id))
    if not admin:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Admin tidak ditemukan",
        )

    await create_audit_log(
        db,
        table_name="users",
        record_id=admin.id_user,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"is_active": True},
        after_state={"is_active": False},
    )
    await db.commit()
    return _to_response(admin)


@router.put("/admins/{admin_id}/activate")
async def activate_admin_endpoint(
    admin_id: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Re-activate admin account (Owner only)."""
    admin = await activate_admin(db, UUID(admin_id))
    if not admin:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Admin tidak ditemukan",
        )

    await create_audit_log(
        db,
        table_name="users",
        record_id=admin.id_user,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"is_active": False},
        after_state={"is_active": True},
    )
    await db.commit()
    return _to_response(admin)


@router.delete("/admins/{admin_id}")
async def delete_admin_endpoint(
    admin_id: str,
    data: DeleteAdminRequest,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Soft-delete admin + anonymize PII (Owner only).

    Requires username confirmation in request body.
    """
    try:
        admin = await delete_admin(db, UUID(admin_id), data.confirm_username)
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=str(e))

    if not admin:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Admin tidak ditemukan",
        )

    await create_audit_log(
        db,
        table_name="users",
        record_id=admin.id_user,
        action="DELETE",
        changed_by=user.id_user,
        before_state={"username": data.confirm_username, "role": "admin"},
        after_state={"anonymized": True, "deleted_at": str(admin.deleted_at)},
    )
    await db.commit()
    return {"detail": f"Admin '{data.confirm_username}' berhasil dihapus", "id_user": str(admin.id_user)}
