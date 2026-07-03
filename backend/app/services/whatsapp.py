"""WhatsApp Gateway client — REST API integration with retry + circuit breaker + rate limiting."""

import asyncio
from datetime import datetime, timedelta, timezone
from typing import Any

import httpx

# Rate limiting: max 100 msg/sec per SRS
_rate_semaphore = asyncio.Semaphore(100)

# Circuit breaker state (in-memory, use Redis in production)
_cb_state: dict[str, Any] = {
    "failures": 0,
    "open_until": None,
}

CB_THRESHOLD = 5
CB_COOLDOWN = 1800  # 30 minutes


def is_circuit_open() -> bool:
    """Check if WhatsApp circuit breaker is open."""
    if _cb_state["open_until"]:
        if datetime.now(timezone.utc) < _cb_state["open_until"]:
            return True
        # Cooldown expired
        _cb_state["failures"] = 0
        _cb_state["open_until"] = None
    return False


def record_failure():
    """Record a failure for circuit breaker."""
    _cb_state["failures"] += 1
    if _cb_state["failures"] >= CB_THRESHOLD:
        _cb_state["open_until"] = datetime.now(timezone.utc) + timedelta(seconds=CB_COOLDOWN)


def reset_circuit_breaker():
    """Manually reset circuit breaker."""
    _cb_state["failures"] = 0
    _cb_state["open_until"] = None


def get_circuit_status() -> dict:
    """Get circuit breaker status."""
    return {
        "failures": _cb_state["failures"],
        "is_open": is_circuit_open(),
        "open_until": _cb_state["open_until"].isoformat() if _cb_state["open_until"] else None,
    }


def normalize_phone(phone: str) -> str:
    """Normalize Indonesian phone number to 62xxxxxxxxxx format."""
    phone = phone.strip().replace("+", "").replace("-", "").replace(" ", "")
    if phone.startswith("0"):
        phone = "62" + phone[1:]
    if not phone.startswith("62"):
        phone = "62" + phone
    return phone


async def send_whatsapp_message(
    api_url: str,
    api_key: str,
    phone: str,
    message: str,
    sender_number: str | None = None,
) -> dict:
    """Send WhatsApp message via gateway API.
    
    Supports common Indonesian WhatsApp gateways (WABlas, Fonnte, etc).
    Returns dict with 'success', 'message_id', 'error' keys.
    """
    if is_circuit_open():
        return {"success": False, "error": "Circuit breaker open", "circuit_open": True}

    normalized = normalize_phone(phone)

    # Common gateway payload formats
    payload = {
        "phone": normalized,
        "message": message,
    }
    if sender_number:
        payload["sender"] = sender_number

    headers = {
        "Authorization": api_key,
        "Content-Type": "application/json",
    }

    try:
        async with _rate_semaphore:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{api_url.rstrip('/')}/api/v1/send-message",
                    json=payload,
                    headers=headers,
                )
                response.raise_for_status()
                data = response.json()

                # Reset circuit breaker on success
                _cb_state["failures"] = 0
                _cb_state["open_until"] = None

                return {
                    "success": True,
                    "message_id": data.get("message_id") or data.get("id"),
                    "raw_response": data,
                }

    except httpx.TimeoutException:
        record_failure()
        return {"success": False, "error": "Timeout: gateway tidak merespons"}

    except httpx.HTTPStatusError as e:
        record_failure()
        return {"success": False, "error": f"HTTP {e.response.status_code}: {e.response.text[:200]}"}

    except Exception as e:
        record_failure()
        return {"success": False, "error": str(e)[:200]}


async def send_with_retry(
    api_url: str,
    api_key: str,
    phone: str,
    message: str,
    sender_number: str | None = None,
    max_retries: int = 3,
) -> dict:
    """Send WhatsApp message with exponential backoff retry.
    
    Delays: 10s, 20s, 40s per SRS.
    """
    delays = [10, 20, 40]
    result: dict = {"success": False, "error": "No attempts made"}

    for attempt in range(max_retries):
        result = await send_whatsapp_message(api_url, api_key, phone, message, sender_number)

        if result.get("success"):
            result["retry_count"] = attempt
            return result

        # Don't retry if circuit breaker is open
        if result.get("circuit_open"):
            result["retry_count"] = attempt
            return result

        # Wait before retry (except last attempt)
        if attempt < max_retries - 1:
            await asyncio.sleep(delays[attempt])

    result["retry_count"] = max_retries
    return result
