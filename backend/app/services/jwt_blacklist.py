"""JWT blacklist — invalidate refresh tokens via Redis.

SRS: Refresh token yang sudah logout harus ditolak.
Uses Redis for O(1) lookup and automatic TTL expiry.
"""
import os
import logging

logger = logging.getLogger("jwt_blacklist")

_client = None


def _get_redis():
    """Get Redis client (lazy init)."""
    global _client
    if _client is not None:
        return _client
    from app.core.config import get_settings
    redis_url = get_settings().REDIS_URL
    if not redis_url:
        logger.warning("REDIS_URL not set, JWT blacklist disabled")
        return None
    try:
        import redis
        _client = redis.from_url(redis_url, decode_responses=True)
        _client.ping()
        return _client
    except Exception as e:
        logger.error(f"Redis connection failed: {e}")
        return None


def blacklist_token(token: str, ttl_seconds: int = 86400 * 7):
    """Add token to blacklist. TTL = 7 days (max refresh token lifetime)."""
    r = _get_redis()
    if r:
        r.set(f"blacklist:{token}", "1", ex=ttl_seconds)
        logger.debug(f"Token blacklisted (TTL {ttl_seconds}s)")


def is_blacklisted(token: str) -> bool:
    """Check if token is blacklisted."""
    r = _get_redis()
    if not r:
        return False  # No Redis = no blacklist (fail open)
    return r.exists(f"blacklist:{token}") > 0


def get_blacklist_size() -> int:
    """Get number of blacklisted tokens (for monitoring)."""
    r = _get_redis()
    if not r:
        return 0
    return len(r.keys("blacklist:*"))
