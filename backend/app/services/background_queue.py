"""Background job queue for async notification delivery.

SRS NFR-REL-01: retry 3x with exponential backoff.
Notifications are enqueued and processed by a background worker.
Request returns immediately — no blocking on WhatsApp/email send.
"""
import asyncio
import logging
from uuid import UUID

logger = logging.getLogger("background_queue")

# In-memory queue (use Redis for production multi-worker)
_notification_queue: asyncio.Queue = asyncio.Queue()

# Worker state
_worker_task: asyncio.Task | None = None
MAX_QUEUE_SIZE = 1000


async def enqueue_notification(notification_id: UUID):
    """Enqueue a notification for background delivery. Non-blocking."""
    if _notification_queue.qsize() >= MAX_QUEUE_SIZE:
        logger.warning(f"Notification queue full ({MAX_QUEUE_SIZE}), dropping notification {notification_id}")
        return False
    await _notification_queue.put(notification_id)
    logger.debug(f"Enqueued notification {notification_id} (queue size: {_notification_queue.qsize()})")
    return True


async def _worker():
    """Background worker — processes notifications from queue with retry."""
    from app.core.database import async_session_maker
    from app.services.notification import send_notification

    logger.info("Notification worker started")
    while True:
        try:
            notification_id: UUID = await _notification_queue.get()
            try:
                async with async_session_maker() as db:
                    success = await send_notification(db, notification_id)
                    if success:
                        logger.debug(f"Notification {notification_id} sent successfully")
                    else:
                        logger.warning(f"Notification {notification_id} failed after retries")
            except Exception as e:
                logger.error(f"Worker error processing {notification_id}: {e}")
            finally:
                _notification_queue.task_done()
        except asyncio.CancelledError:
            logger.info("Notification worker stopped")
            break
        except Exception as e:
            logger.error(f"Worker loop error: {e}")
            await asyncio.sleep(1)


def start_worker():
    """Start the background notification worker. Call from FastAPI lifespan."""
    global _worker_task
    if _worker_task is None or _worker_task.done():
        _worker_task = asyncio.create_task(_worker())
        logger.info("Background notification worker started")


def stop_worker():
    """Stop the background notification worker. Call from FastAPI lifespan."""
    global _worker_task
    if _worker_task and not _worker_task.done():
        _worker_task.cancel()
        logger.info("Background notification worker stopped")


def get_queue_size() -> int:
    """Get current queue size for monitoring."""
    return _notification_queue.qsize()
