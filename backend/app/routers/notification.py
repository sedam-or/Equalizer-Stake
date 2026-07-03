"""Notification router — send and list notifications."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.models.notification import Notification
from app.schemas.notification import NotificationResponse
from app.services.audit import create_audit_log
from app.services.notification import reset_circuit_breaker, group_notification_by_guardian
from app.services.whatsapp import get_circuit_status, send_with_retry
from app.services.email import get_circuit_status as get_email_cb_status

router = APIRouter(prefix="/api/notifications", tags=["Notifications"])


@router.get("/")
async def list_notifications(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner", "admin", "teacher", "guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Get notifications for current user."""
    base = select(Notification).where(Notification.recipient_id == user.id_user)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Notification.created_at.desc()).offset(offset).limit(limit))
    notifications = result.scalars().all()
    return {
        "data": [
            {
                "id_notification": str(n.id_notification),
                "recipient_id": str(n.recipient_id),
                "channel": n.channel,
                "category": n.category,
                "subject": n.subject,
                "body": n.body,
                "status": n.status,
                "retry_count": n.retry_count,
                "sent_at": n.sent_at.isoformat() if n.sent_at else None,
                "created_at": n.created_at.isoformat(),
            }
            for n in notifications
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.post("/circuit-breaker/reset")
async def reset_circuit_breaker_endpoint(
    channel: str,
    user: User = Depends(require_role("admin", "owner")),
):
    """Manually reset circuit breaker for a channel (Admin/Owner only)."""
    if channel not in ["whatsapp", "email"]:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Channel harus 'whatsapp' atau 'email'",
        )
    reset_circuit_breaker(channel)
    return {"message": f"Circuit breaker {channel} direset"}


@router.get("/circuit-breaker/status")
async def circuit_breaker_status(
    user: User = Depends(require_role("admin", "owner")),
):
    """Get WhatsApp circuit breaker status (Admin/Owner only)."""
    return {"whatsapp": get_circuit_status()}


@router.post("/test-whatsapp")
async def test_whatsapp(
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Test WhatsApp gateway connectivity (Admin/Owner only)."""
    body = await request.json()
    phone = body.get("phone")
    message = body.get("message", "Test dari Equalizer-Stake")

    if not phone:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Field 'phone' wajib diisi",
        )

    # Get WhatsApp config from settings
    from app.services.settings import get_setting
    wa_config = await get_setting(db, "whatsapp_config")
    if not wa_config or not wa_config.setting_value.get("enabled"):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="WhatsApp gateway belum dikonfigurasi atau tidak aktif",
        )

    config = wa_config.setting_value
    result = await send_with_retry(
        api_url=config.get("api_url", ""),
        api_key=config.get("api_key", ""),
        phone=phone,
        message=message,
        sender_number=config.get("sender_number"),
    )

    await create_audit_log(
        db,
        table_name="notifications",
        record_id=user.id_user,
        action="INSERT",
        changed_by=user.id_user,
        after_state={"phone": phone, "channel": "whatsapp", "success": result.get("success", False)},
        request=request,
        category="operasional",
    )
    await db.commit()

    return {
        "success": result.get("success", False),
        "message": result.get("error") if not result.get("success") else "Pesan terkirim",
        "retry_count": result.get("retry_count", 0),
    }


@router.get("/email/status")
async def email_status(
    user: User = Depends(require_role("admin", "owner")),
):
    """Get email SMTP status (Admin/Owner only)."""
    from app.core.config import get_settings
    settings = get_settings()
    return {
        "email": get_email_cb_status(),
        "smtp_configured": bool(settings.SMTP_HOST),
        "smtp_host": settings.SMTP_HOST if settings.SMTP_HOST else None,
    }


@router.post("/toggle")
async def toggle_notification_channel(
    request: Request,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Toggle notification channel on/off per category (Admin/Owner only).
    
    Body: {"channel": "whatsapp"|"email", "category": "critical"|"non_critical", "enabled": bool}
    Stores in system_settings under 'notification_toggle' key.
    """
    body = await request.json()
    channel = body.get("channel")
    category = body.get("category")
    enabled = body.get("enabled")

    if channel not in ("whatsapp", "email"):
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Channel harus 'whatsapp' atau 'email'",
        )
    if category not in ("critical", "non_critical"):
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Kategori harus 'critical' atau 'non_critical'",
        )
    if not isinstance(enabled, bool):
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Field 'enabled' harus berupa boolean",
        )

    from app.services.settings import get_setting, upsert_setting
    from app.services.audit import create_audit_log

    setting_key = "notification_toggle"
    existing = await get_setting(db, setting_key)
    toggle_value = existing.setting_value if existing else {}

    field_key = f"{channel}_{category}"
    before_state = dict(toggle_value)
    toggle_value[field_key] = enabled

    setting = await upsert_setting(db, setting_key, toggle_value, user.id_user)
    await db.flush()

    await create_audit_log(
        db,
        table_name="system_settings",
        record_id=setting.id_setting,
        action="UPDATE" if before_state else "INSERT",
        changed_by=user.id_user,
        before_state=before_state,
        after_state=toggle_value,
        request=request,
    )
    await db.commit()

    return {
        "detail": f"Notifikasi {channel} untuk kategori {category} {'diaktifkan' if enabled else 'dinonaktifkan'}",
        "toggle": toggle_value,
    }
