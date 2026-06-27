# Equalizer-Stake — Integration Architecture

## WhatsApp Gateway

### REST API Integration

```python
# app/services/notification_service.py
import httpx

class WhatsAppGateway:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url
        self.headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        }
        self.client = httpx.AsyncClient(
            base_url=base_url,
            headers=self.headers,
            timeout=30.0,
        )

    async def send_message(
        self, phone: str, message: str
    ) -> dict:
        """Send WhatsApp message via unofficial gateway (e.g., WABlas, Fonnte)."""
        response = await self.client.post(
            "/api/v1/send-message",
            json={
                "phone": self._normalize_phone(phone),
                "message": message,
            },
        )
        response.raise_for_status()
        return response.json()

    def _normalize_phone(self, phone: str) -> str:
        """Ensure Indonesian format: 62xxxxxxxxxx."""
        phone = phone.strip().replace("+", "").replace("-", "")
        if phone.startswith("0"):
            phone = "62" + phone[1:]
        return phone
```

### Retry Strategy (Exponential Backoff)

```python
# app/services/notification_service.py
from tenacity import retry, stop_after_attempt, wait_exponential, retry_if_exception_type

class NotificationService:
    def __init__(self, db: AsyncSession):
        self.db = db
        self.whatsapp = WhatsAppGateway(
            settings.WHATSAPP_GATEWAY_URL,
            settings.WHATSAPP_API_KEY,
        )

    @retry(
        stop=stop_after_attempt(3),
        wait=wait_exponential(multiplier=1, min=2, max=30),
        retry=retry_if_exception_type((httpx.HTTPStatusError, httpx.ConnectTimeout)),
    )
    async def _send_whatsapp_with_retry(self, phone: str, message: str) -> bool:
        try:
            result = await self.whatsapp.send_message(phone, message)
            return result.get("status") == "sent"
        except (httpx.HTTPStatusError, httpx.ConnectTimeout) as e:
            logger.warning(f"WhatsApp send failed (will retry): {e}")
            raise

    async def send_notification(
        self, user_id: uuid.UUID, type: str, subject: str, message: str
    ) -> Notification:
        """Send notification with WhatsApp primary, email fallback."""
        user = await self.db.get(User, user_id)
        notification = Notification(
            user_id=user_id,
            type=type,
            subject=subject,
            body=message,
            status="pending",
        )
        self.db.add(notification)

        # Try WhatsApp first
        sent = False
        try:
            guardian = await self._get_guardian(user_id)
            if guardian and guardian.phone:
                phone = enc.decrypt(guardian.phone)
                sent = await self._send_whatsapp_with_retry(phone, message)
                notification.channel = "whatsapp"
        except Exception as e:
            logger.error(f"WhatsApp failed after retries: {e}")

        # Fallback to email
        if not sent:
            try:
                await self._send_email(user.email, subject, message)
                notification.channel = "email"
                sent = True
            except Exception as e:
                logger.error(f"Email fallback failed: {e}")

        notification.status = "sent" if sent else "failed"
        notification.sent_at = datetime.utcnow() if sent else None
        await self.db.flush()
        return notification
```

### Circuit Breaker

```python
# app/services/circuit_breaker.py
import time

class CircuitBreaker:
    """Simple circuit breaker for WhatsApp gateway."""
    def __init__(self, failure_threshold: int = 5, recovery_timeout: int = 60):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.failure_count = 0
        self.last_failure_time = 0.0
        self.state = "closed"  # closed | open | half_open

    def record_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.time()
        if self.failure_count >= self.failure_threshold:
            self.state = "open"
            logger.warning(f"Circuit breaker OPEN after {self.failure_count} failures")

    def record_success(self):
        self.failure_count = 0
        self.state = "closed"

    def allow_request(self) -> bool:
        if self.state == "closed":
            return True
        if self.state == "open":
            if time.time() - self.last_failure_time > self.recovery_timeout:
                self.state = "half_open"
                return True
            return False
        return True  # half_open: allow one probe request

# Usage in NotificationService
_whatsapp_circuit = CircuitBreaker(failure_threshold=5, recovery_timeout=60)

async def send_notification(self, ...):
    if not _whatsapp_circuit.allow_request():
        logger.info("Circuit open, skipping WhatsApp, using email")
        # go straight to email fallback
    else:
        try:
            sent = await self._send_whatsapp_with_retry(phone, message)
            _whatsapp_circuit.record_success()
        except Exception:
            _whatsapp_circuit.record_failure()
            # fall through to email
```

## SMTP Email Fallback

```python
# app/services/email_service.py
import aiosmtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart

class EmailService:
    async def send(
        self, to: str, subject: str, html_body: str
    ) -> bool:
        msg = MIMEMultipart("alternative")
        msg["From"] = settings.SMTP_FROM
        msg["To"] = to
        msg["Subject"] = subject
        msg.attach(MIMEText(html_body, "html"))

        await aiosmtplib.send(
            msg,
            hostname=settings.SMTP_HOST,
            port=settings.SMTP_PORT,
            username=settings.SMTP_USER,
            password=settings.SMTP_PASS,
            use_tls=True,
        )
        return True
```

Email templates (Jinja2):

```
templates/emails/
├── payment_reminder.html      # Invoice due reminder
├── payment_confirmed.html     # Payment verified
├── invoice_generated.html     # New invoice notification
├── attendance_alert.html      # Student missed session
└── welcome.html               # New guardian registration
```

## face-api.js / MediaPipe Client-Side Detection

Detection runs entirely in browser — no biometric data transmitted to server.

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Webcam     │───►│  face-api.js │───►│  Result:    │
│   Video      │    │  Detection   │    │  {detected: │
│   Stream     │    │  (browser)   │    │   true,     │
│              │    │              │    │   confidence│
│              │    │  Models:     │    │   : 0.87}   │
│              │    │  - TinyFace  │    │             │
│              │    │  - Landmark  │    │  Send ONLY  │
│              │    │  - 68pt      │    │  result,    │
│              │    │              │    │  NOT image  │
└─────────────┘    └──────────────┘    └──────┬──────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │  POST /attendance│
                                    │  {session_id,   │
                                    │   gps_lat,      │
                                    │   gps_lng,      │
                                    │   face_verified: │
                                    │    true}        │
                                    └─────────────────┘
```

### Model Loading Strategy

```typescript
// Models loaded once on attendance page mount
// TinyFace: ~200KB, fast on mobile
// Landmark68: ~350KB, optional refinement

// Lazy load: models loaded only when user navigates to /attendance
// Cached: service worker caches model files (CacheFirst, 30-day TTL)
```

### GPS + Face Combined Validation

```typescript
// Frontend: combined check before API call
async function submitAttendance(sessionId: string) {
  // Step 1: GPS check (client-side)
  const position = await getCurrentPosition()
  const distance = haversine(position, roomGps)
  if (distance > 150) {
    throw new Error(`Too far from room: ${Math.round(distance)}m (max 150m)`)
  }

  // Step 2: Face check (client-side)
  const faceResult = await detectFace(videoElement)
  if (!faceResult.detected || faceResult.confidence < 0.6) {
    throw new Error('Face not detected. Position face in camera.')
  }

  // Step 3: Send only verification results (no image, no raw GPS)
  await api.post('/attendance', {
    session_id: sessionId,
    gps_lat: position.coords.latitude,
    gps_lng: position.coords.longitude,
    face_verified: true,
  })
}
```

## Notification Queue Design

Notifications queued and processed asynchronously to avoid blocking API responses.

### Architecture

```
API Request (create invoice)
    │
    ▼
NotificationService.queue_notification()
    │
    ▼
Redis List: "notification_queue"
    │
    ▼
Background Worker (APScheduler or asyncio.Task)
    │
    ├── WhatsApp Gateway (primary)
    │     │
    │     ├── Success → update Notification.status = "sent"
    │     └── Failure → retry (3x exponential backoff)
    │           └── All retries failed → Email fallback
    │
    └── SMTP (fallback)
          └── Success → update Notification.status = "sent"
```

### Queue Implementation

```python
# app/services/notification_service.py
import json

NOTIFICATION_QUEUE = "notification:queue"

async def enqueue_notification(redis: Redis, notification_id: uuid.UUID):
    await redis.rpush(NOTIFICATION_QUEUE, str(notification_id))

async def process_notification_queue(redis: Redis):
    """Background task: process queued notifications."""
    while True:
        # Block until notification available (timeout 5s to allow shutdown)
        result = await redis.blpop(NOTIFICATION_QUEUE, timeout=5)
        if result is None:
            continue
        _, notification_id_bytes = result
        notification_id = uuid.UUID(notification_id_bytes.decode())

        async with async_session_maker() as db:
            notification = await db.get(Notification, notification_id)
            if not notification:
                continue
            service = NotificationService(db)
            await service.deliver(notification)
            await db.commit()
```

### Notification Types

| Type | Trigger | Channel | Template |
|------|---------|---------|----------|
| `invoice_generated` | Cron (5th) | WhatsApp + email | Invoice details, amount, due date |
| `payment_reminder` | Cron (11th) | WhatsApp + email | Overdue invoice, amount |
| `payment_confirmed` | Admin verifies | WhatsApp + email | Payment receipt |
| `session_reminder` | Cron (1hr before) | WhatsApp | Session time, room |
| `attendance_alert` | Attendance service | WhatsApp | Student missed session |

### Delivery Tracking

```python
class Notification(Base):
    __tablename__ = "notifications"
    # ...
    status: Mapped[str] = mapped_column(String(20), default="pending")
    # pending → queued → sending → sent | failed
    channel: Mapped[str | None] = mapped_column(String(20))  # whatsapp | email
    retry_count: Mapped[int] = mapped_column(default=0)
    last_retry_at: Mapped[datetime | None]
    error_message: Mapped[str | None]
```
