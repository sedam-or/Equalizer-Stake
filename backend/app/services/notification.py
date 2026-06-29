"""Notification service — multi-channel with retry, circuit breaker, fallback."""

import asyncio
from datetime import datetime, timezone
from uuid import UUID

from sqlalchemy import select, update
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.notification import Notification
from app.services.whatsapp import send_with_retry, reset_circuit_breaker as reset_wa_cb, get_circuit_status as get_wa_cb_status

# Retry settings
MAX_RETRIES = 3
RETRY_DELAYS = [10, 20, 40]  # seconds (exponential backoff)

# Circuit breaker
CIRCUIT_BREAKER_THRESHOLD = 5  # consecutive failures
CIRCUIT_BREAKER_COOLDOWN = 1800  # 30 minutes in seconds

# In-memory circuit breaker state (should use Redis in production)
_circuit_breaker_state = {
    "whatsapp": {"failures": 0, "open_until": None},
    "email": {"failures": 0, "open_until": None},
}


def is_circuit_breaker_open(channel: str) -> bool:
    """Check if circuit breaker is open for a channel."""
    if channel == "whatsapp":
        return get_wa_cb_status()["is_open"]
    state = _circuit_breaker_state.get(channel, {})
    if state.get("open_until"):
        if datetime.now(timezone.utc) < state["open_until"]:
            return True
        # Cooldown expired, reset
        state["open_until"] = None
        state["failures"] = 0
    return False


def record_circuit_breaker_failure(channel: str):
    """Record a failure for circuit breaker."""
    if channel == "whatsapp":
        return  # Handled by whatsapp module
    state = _circuit_breaker_state.setdefault(channel, {"failures": 0, "open_until": None})
    state["failures"] += 1
    if state["failures"] >= CIRCUIT_BREAKER_THRESHOLD:
        state["open_until"] = datetime.now(timezone.utc).replace(
            second=0, microsecond=0
        ) + __import__("datetime").timedelta(seconds=CIRCUIT_BREAKER_COOLDOWN)


def reset_circuit_breaker(channel: str):
    """Manually reset circuit breaker."""
    if channel == "whatsapp":
        reset_wa_cb()
    else:
        _circuit_breaker_state[channel] = {"failures": 0, "open_until": None}


def get_channel_priority(category: str) -> list[str]:
    """Get channel priority based on notification category."""
    if category == "critical":
        return ["email", "whatsapp"]  # Email primary for critical
    else:
        return ["whatsapp", "email"]  # WhatsApp primary for non-critical


async def create_notification(
    db: AsyncSession,
    recipient_id: UUID,
    channel: str,
    category: str,
    body: str,
    subject: str | None = None,
    related_entity_type: str | None = None,
    related_entity_id: UUID | None = None,
) -> Notification:
    """Create a notification record."""
    notification = Notification(
        recipient_id=recipient_id,
        channel=channel,
        category=category,
        subject=subject,
        body=body,
        status="pending",
        related_entity_type=related_entity_type,
        related_entity_id=related_entity_id,
    )
    db.add(notification)
    await db.commit()
    await db.refresh(notification)
    return notification


async def send_notification(
    db: AsyncSession,
    notification_id: UUID,
) -> bool:
    """Send notification with retry and fallback logic."""
    result = await db.execute(
        select(Notification).where(Notification.id_notification == notification_id)
    )
    notification = result.scalar_one_or_none()

    if not notification or notification.status == "sent":
        return True

    channel = notification.channel
    category = notification.category

    # Check circuit breaker
    if is_circuit_breaker_open(channel):
        # Fallback to alternative channel
        channels = get_channel_priority(category)
        alt_channel = next((c for c in channels if c != channel and not is_circuit_breaker_open(c)), None)
        if alt_channel:
            notification.channel = alt_channel
            notification.status = "retrying"
            await db.commit()
            return await send_notification(db, notification_id)
        return False

    # Try sending with retries
    for attempt in range(MAX_RETRIES):
        try:
            if channel == "whatsapp":
                # Get WhatsApp config from system_settings
                from app.services.settings import get_setting
                wa_config = await get_setting(db, "whatsapp_config")
                if not wa_config or not wa_config.setting_value.get("enabled"):
                    # WhatsApp not configured, skip to fallback
                    break
                
                result = await send_with_retry(
                    api_url=wa_config.setting_value.get("api_url", ""),
                    api_key=wa_config.setting_value.get("api_key", ""),
                    phone=notification.body.split("Phone: ")[1].split("\n")[0] if "Phone: " in notification.body else "",
                    message=notification.body,
                    sender_number=wa_config.setting_value.get("sender_number"),
                )
                success = result.get("success", False)
                if not success:
                    notification.error_message = result.get("error", "Unknown error")
            else:
                # Email or other channel — simulate for now
                success = True  # ponytail: email integration in T-023

            if success:
                notification.status = "sent"
                notification.sent_at = datetime.now(timezone.utc)
                notification.retry_count = attempt
                await db.commit()
                _circuit_breaker_state[channel]["failures"] = 0
                return True

        except Exception:
            if attempt < MAX_RETRIES - 1:
                await asyncio.sleep(RETRY_DELAYS[attempt])
                notification.retry_count = attempt + 1
                notification.status = "retrying"
                await db.commit()

    # All retries failed
    notification.status = "failed"
    notification.error_message = f"Gagal mengirim via {channel} setelah {MAX_RETRIES} percobaan"
    await db.commit()
    record_circuit_breaker_failure(channel)

    # Fallback to alternative channel
    channels = get_channel_priority(category)
    alt_channel = next((c for c in channels if c != channel and not is_circuit_breaker_open(c)), None)
    if alt_channel:
        notification.channel = alt_channel
        notification.status = "retrying"
        notification.retry_count = 0
        await db.commit()
        return await send_notification(db, notification_id)

    return False


async def get_notifications(
    db: AsyncSession,
    recipient_id: UUID,
    limit: int = 50,
) -> list[Notification]:
    """Get notifications for a user."""
    result = await db.execute(
        select(Notification)
        .where(Notification.recipient_id == recipient_id)
        .order_by(Notification.created_at.desc())
        .limit(limit)
    )
    return list(result.scalars().all())
