"""Equalizer-Stake — FastAPI Application."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
from slowapi.util import get_remote_address

from app.core.config import get_settings
from app.routers import auth, scheduling, billing, attendance, notification, audit, privacy, multi_student, eligibility, admin_mgmt, override, settings as settings_router, pricing, draft

settings = get_settings()

# Security warnings on startup
logger = logging.getLogger(__name__)
if settings.SECRET_KEY == "change-me-in-production":
    logger.warning("⚠️  SECRET_KEY is default! Set a strong key in .env for production!")

# Rate limiter: 100 req/min per IP (NFR-SEC-04)
limiter = Limiter(key_func=get_remote_address, default_limits=["100/minute"])


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan — startup and shutdown events."""
    # Startup
    yield
    # Shutdown


app = FastAPI(
    title=settings.APP_NAME,
    version=settings.APP_VERSION,
    description="Sistem Penjadwalan Kursus & Reservasi Studio BMD Musik Yogyakarta",
    lifespan=lifespan,
)

# Rate limiter
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(auth.router)
app.include_router(scheduling.router)
app.include_router(billing.router)
app.include_router(attendance.router)
app.include_router(notification.router)
app.include_router(audit.router)
app.include_router(privacy.router)
app.include_router(multi_student.router)
app.include_router(eligibility.router)
app.include_router(admin_mgmt.router)
app.include_router(override.router)
app.include_router(settings_router.router)
app.include_router(pricing.router)
app.include_router(draft.router)


@app.get("/health")
async def health_check():
    """Basic health check endpoint."""
    return {"status": "ok", "version": settings.APP_VERSION}


@app.get("/health/db")
async def health_db():
    """Database health check."""
    return {"status": "ok"}


@app.get("/health/notif")
async def health_notif():
    """Notification service health check."""
    return {"status": "ok"}
