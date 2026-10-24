"""Shared rate limiter instance for use across routers.

NFR-SEC-04: Rate limit 100 request/menit per IP/user.
Uses Redis storage backend for persistence across restarts.
"""
import os
from slowapi import Limiter
from slowapi.util import get_remote_address

_redis_url = os.getenv("REDIS_URL", "")

limiter = Limiter(
    key_func=get_remote_address,
    default_limits=["100/minute"],
    storage_uri=_redis_url or "memory://",
)
