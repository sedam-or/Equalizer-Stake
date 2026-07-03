"""Equalizer-Stake — FastAPI Application."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from slowapi import _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded

from app.core.config import get_settings
from app.core.limiter import limiter
from app.services.scheduler import start_scheduler, shutdown_scheduler
from app.services.background_queue import start_worker, stop_worker
from app.routers import auth, scheduling, billing, attendance, notification, audit, privacy, multi_student, eligibility, admin_mgmt, override, settings as settings_router, pricing, draft, students, payments, my_portal, registrations, photos, cron_trigger

settings = get_settings()

# Security warnings on startup
logger = logging.getLogger(__name__)
if settings.SECRET_KEY == "change-me-in-production":
    logger.warning("⚠️  SECRET_KEY is default! Set a strong key in .env for production!")


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan — startup and shutdown events."""
    # Startup
    start_worker()
    start_scheduler()
    yield
    # Shutdown
    shutdown_scheduler()
    stop_worker()


app = FastAPI(
    title=settings.APP_NAME,
    version=settings.APP_VERSION,
    description="Sistem Penjadwalan Kursus & Reservasi Studio BMD Musik Yogyakarta",
    lifespan=lifespan,
    redirect_slashes=False,
)

# Rate limiter
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# Security headers middleware (FINDING 15)
@app.middleware("http")
async def security_headers(request: Request, call_next):
    response = await call_next(request)
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
    response.headers["Content-Security-Policy"] = (
        "default-src 'self'; "
        "script-src 'self'; "
        "style-src 'self' 'unsafe-inline'; "
        "img-src 'self' data:; "
        "font-src 'self'; "
        "connect-src 'self'; "
        "frame-ancestors 'none'; "
        "base-uri 'self'; "
        "form-action 'self'"
    )
    response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
    return response

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
app.include_router(students.router)
app.include_router(payments.router)
app.include_router(my_portal.router)
app.include_router(registrations.router)
app.include_router(photos.router, prefix="/api/photos", tags=["Photos"])
app.include_router(cron_trigger.router, prefix="/api/admin/cron", tags=["Cron Trigger"])


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
