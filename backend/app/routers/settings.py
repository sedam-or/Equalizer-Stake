"""Settings router — system configuration (Owner only)."""

from fastapi import APIRouter, Depends, HTTPException, Request, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import create_audit_log
from app.services.settings import (
    ALLOWED_SETTINGS,
    get_all_settings,
    get_setting,
    upsert_setting,
    validate_setting_value,
)

router = APIRouter(prefix="/api/settings", tags=["Settings"])


@router.get("")
async def list_settings(
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """List all system settings (Owner only)."""
    settings = await get_all_settings(db)
    return {
        "settings": [
            {
                "key": s.setting_key,
                "value": s.setting_value,
                "description": s.description,
                "updated_at": s.updated_at.isoformat() if s.updated_at else None,
            }
            for s in settings
        ],
        "available_keys": list(ALLOWED_SETTINGS.keys()),
    }


@router.get("/{key}")
async def get_single_setting(
    key: str,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Get single setting by key (Owner only)."""
    setting = await get_setting(db, key)
    if not setting:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Setting '{key}' belum dikonfigurasi",
        )
    return {
        "key": setting.setting_key,
        "value": setting.setting_value,
        "description": setting.description,
        "updated_at": setting.updated_at.isoformat() if setting.updated_at else None,
    }


@router.put("/{key}")
async def update_setting(
    key: str,
    request: Request,
    user: User = Depends(require_role("owner")),
    db: AsyncSession = Depends(get_db),
):
    """Update or create a system setting (Owner only)."""
    body = await request.json()
    value = body.get("value")
    if not value or not isinstance(value, dict):
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Field 'value' harus berupa object JSON",
        )

    # Validate
    error = validate_setting_value(key, value)
    if error:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail=error,
        )

    # Get before state for audit
    before = await get_setting(db, key)
    before_state = before.setting_value if before else None

    setting = await upsert_setting(db, key, value, user.id_user)
    await db.flush()  # Ensure setting.id_setting is populated

    await create_audit_log(
        db,
        table_name="system_settings",
        record_id=setting.id_setting,
        action="UPDATE" if before_state else "INSERT",
        changed_by=user.id_user,
        before_state=before_state,
        after_state=value,
        request=request,
    )
    await db.commit()

    return {
        "detail": f"Setting '{key}' berhasil diperbarui",
        "key": setting.setting_key,
        "value": setting.setting_value,
    }
