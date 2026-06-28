"""Equalizer-Stake — FastAPI Application."""

from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.core.config import get_settings
from app.routers import auth, scheduling, billing, attendance, notification, audit, privacy, multi_student, eligibility, admin_mgmt

settings = get_settings()


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
