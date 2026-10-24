"""Equalizer-Stake — Application Settings."""

import logging
from functools import lru_cache

from pydantic_settings import BaseSettings

logger = logging.getLogger(__name__)


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""

    # App
    APP_NAME: str = "Equalizer-Stake"
    APP_VERSION: str = "0.1.0"
    DEBUG: bool = True

    # Database
    DATABASE_URL: str = "postgresql+asyncpg://eqs_admin:eqs_dev_password@localhost:5432/equalizer_stake"

    # Redis
    REDIS_URL: str = "redis://localhost:6379/0"

    # JWT Auth
    SECRET_KEY: str = "change-me-in-production"
    JWT_ALGORITHM: str = "HS256"

    # Session timeouts (seconds)
    SESSION_TIMEOUT_OWNER: int = 1800  # 30 min
    SESSION_TIMEOUT_ADMIN: int = 1800  # 30 min
    SESSION_TIMEOUT_TEACHER: int = 1800  # 30 min
    SESSION_TIMEOUT_GUARDIAN: int = 3600  # 60 min

    # Rate limiting
    RATE_LIMIT_PER_MINUTE: int = 100

    # CORS
    CORS_ORIGINS: str = "http://localhost:3000"

    # Studio GPS
    STUDIO_LAT: float = -7.7265
    STUDIO_LNG: float = 110.4162
    STUDIO_GPS_RADIUS: int = 150  # meters

    # WhatsApp Gateway
    WHATSAPP_API_URL: str = ""
    WHATSAPP_API_KEY: str = ""

    # SMTP
    SMTP_HOST: str = ""
    SMTP_PORT: int = 587
    SMTP_USER: str = ""
    SMTP_PASSWORD: str = ""

    model_config = {"env_file": ".env", "extra": "ignore"}


@lru_cache
def get_settings() -> Settings:
    s = Settings()
    if not s.DEBUG and s.SECRET_KEY == "change-me-in-production":
        raise ValueError(
            "SECRET_KEY is default! Cannot start in production mode. "
            "Set a strong SECRET_KEY in .env for production!"
        )
    return s
