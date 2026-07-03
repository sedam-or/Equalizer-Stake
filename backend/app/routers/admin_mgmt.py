"""Admin management router — superadmin CRUD for admin accounts."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.admin_mgmt import (
    AdminResponse,
    CreateAdminRequest,
    CreateAdminResponse,
    DeleteAdminRequest,
    ResendOnboardingResponse,
)
from app.services.admin_mgmt import (
    activate_admin,
    create_admin,
    deactivate_admin,
    delete_admin,
    resend_onboarding,
)
from app.templates.emails import onboarding_credentials_email
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
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """List all admin accounts (Owner only)."""
    base = select(User).where(User.role == "admin", User.deleted_at.is_(None))
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(User.created_at.desc()).offset(offset).limit(limit))
    admins = result.scalars().all()
    return {"data": [_to_response(a) for a in admins], "total": total, "limit": limit, "offset": offset}


@router.post("/admins", status_code=status.HTTP_201_CREATED, response_model=CreateAdminResponse)
async def create_admin_endpoint(
    data: CreateAdminRequest,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Create new admin account (Owner only)."""
    try:
        admin, temp_pw = await create_admin(db, data.username, data.email)
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
    return {
        **_to_response(admin),
        "temp_password": temp_pw,
    }


@router.post("/admins/{admin_id}/resend-onboarding", response_model=ResendOnboardingResponse)
async def resend_onboarding_endpoint(
    admin_id: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Regenerate and resend temp password for admin (Owner only)."""
    try:
        result = await resend_onboarding(db, UUID(admin_id))
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=str(e))

    if not result:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Admin tidak ditemukan",
        )

    admin, temp_pw = result
    await create_audit_log(
        db,
        table_name="users",
        record_id=admin.id_user,
        action="UPDATE",
        changed_by=user.id_user,
        after_state={"onboarding_resent": True},
    )
    await db.commit()
    return {
        "id_user": str(admin.id_user),
        "username": admin.username,
        "email": admin.email,
        "temp_password": temp_pw,
        "password_expires_at": admin.password_expires_at.isoformat() if admin.password_expires_at else "",
    }


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
