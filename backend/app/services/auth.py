"""Auth service — authentication, JWT, password hashing, brute force protection."""

from datetime import datetime, timedelta, timezone
from uuid import UUID

from jose import JWTError, jwt
from passlib.context import CryptContext
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.models.user import User

settings = get_settings()

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# JWT settings
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = {
    "owner": 30,
    "admin": 30,
    "teacher": 30,
    "guardian": 60,
}

# Brute force protection
MAX_FAILED_ATTEMPTS = 5
LOCKOUT_MINUTES = 15


def hash_password(password: str) -> str:
    """Hash password using bcrypt."""
    return pwd_context.hash(password)


def verify_password(plain: str, hashed: str) -> bool:
    """Verify password against hash."""
    return pwd_context.verify(plain, hashed)


def create_access_token(user_id: UUID, role: str) -> str:
    """Create JWT access token."""
    expire = datetime.now(timezone.utc) + timedelta(
        minutes=ACCESS_TOKEN_EXPIRE_MINUTES.get(role, 30)
    )
    payload = {
        "sub": str(user_id),
        "role": role,
        "exp": expire,
        "iat": datetime.now(timezone.utc),
    }
    return jwt.encode(payload, settings.SECRET_KEY, algorithm=ALGORITHM)


def decode_access_token(token: str) -> dict | None:
    """Decode and validate JWT token."""
    try:
        payload = jwt.decode(token, settings.SECRET_KEY, algorithms=[ALGORITHM])
        return payload
    except JWTError:
        return None


async def authenticate_user(
    db: AsyncSession, username: str, password: str
) -> User | None:
    """Authenticate user with brute force protection."""
    result = await db.execute(select(User).where(User.username == username))
    user = result.scalar_one_or_none()

    if not user or user.deleted_at is not None:
        return None

    # Check if account is locked
    if user.locked_until and user.locked_until > datetime.now(timezone.utc):
        return None  # Account is locked

    # Check if account is active
    if not user.is_active:
        return None

    # Verify password
    if not verify_password(password, user.password_hash):
        user.failed_login_attempts += 1
        if user.failed_login_attempts >= MAX_FAILED_ATTEMPTS:
            user.locked_until = datetime.now(timezone.utc) + timedelta(minutes=LOCKOUT_MINUTES)
        await db.commit()
        return None

    # Successful login — reset failed attempts
    user.failed_login_attempts = 0
    user.locked_until = None
    user.last_login_at = datetime.now(timezone.utc)
    await db.commit()

    return user


async def get_user_by_id(db: AsyncSession, user_id: UUID) -> User | None:
    """Get user by ID."""
    result = await db.execute(select(User).where(User.id_user == user_id))
    return result.scalar_one_or_none()
