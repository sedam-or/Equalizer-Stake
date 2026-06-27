"""Notification router — send and list notifications."""

from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.schemas.notification import NotificationResponse
from app.services.notification import get_notifications, reset_circuit_breaker

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
