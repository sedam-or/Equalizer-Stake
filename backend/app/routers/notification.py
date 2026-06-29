"""Notification router — send and list notifications."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Request, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.notification import NotificationResponse
from app.services.notification import get_notifications, reset_circuit_breaker
from app.services.whatsapp import get_circuit_status, send_with_retry

router = APIRouter(prefix="/api/notifications", tags=["Notifications"])


@router.get("/", response_model=list[NotificationResponse])
async def list_notifications(
    user: User = Depends(require_role("owner", "admin", "teacher", "guardian")),
    db: AsyncSession = Depends(get_db),
):
    """Get notifications for current user."""
    notifications = await get_notifications(db, user.id_user)
    return notifications


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

    return {
        "success": result.get("success", False),
        "message": result.get("error") if not result.get("success") else "Pesan terkirim",
        "retry_count": result.get("retry_count", 0),
    }
