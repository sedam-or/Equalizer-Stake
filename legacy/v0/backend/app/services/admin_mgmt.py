"""Admin management service — superadmin CRUD for admin accounts."""

from datetime import datetime, timezone
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.user import User
from app.services.auth import hash_password
from app.services.encryption import hash_pii
import secrets
import string
from datetime import timedelta


def generate_temp_password(length: int = 8) -> str:
    """Generate alphanumeric temporary password."""
    alphabet = string.ascii_letters + string.digits
    return ''.join(secrets.choice(alphabet) for _ in range(length))


async def list_admins(db: AsyncSession) -> list[User]:
    """List all admin accounts (excluding soft-deleted)."""
    result = await db.execute(
        select(User)
        .where(User.role == "admin", User.deleted_at.is_(None))
        .order_by(User.created_at.desc())
    )
    return list(result.scalars().all())


async def create_admin(
    db: AsyncSession,
    username: str,
    email: str,
) -> tuple[User, str]:
    """Create new admin with auto-generated temp password.

    Returns (user, temp_password_plain).
    """
    # Check duplicate username
    existing = await db.execute(
        select(User).where(User.username == username)
    )
    if existing.scalar_one_or_none():
        raise ValueError("Username sudah digunakan")

    # Check duplicate email (blind index)
    existing_email = await db.execute(
        select(User).where(User.email_hash == hash_pii(email))
    )
    if existing_email.scalar_one_or_none():
        raise ValueError("Email sudah digunakan")

    temp_pw = generate_temp_password()
    now = datetime.now(timezone.utc)
    admin = User(
        username=username,
        email=email,
        email_hash=hash_pii(email),
        password_hash=hash_password(temp_pw),
        role="admin",
        is_active=True,
        first_login=True,
        is_temp_password=True,
        password_expires_at=now + timedelta(days=7),
    )
    db.add(admin)
    await db.flush()
    return admin, temp_pw


async def resend_onboarding(
    db: AsyncSession, admin_id: UUID
) -> tuple[User, str] | None:
    """Regenerate temp password for admin if expired. Returns (user, new_temp_pw) or None."""
    result = await db.execute(
        select(User).where(
            User.id_user == admin_id,
            User.role == "admin",
            User.deleted_at.is_(None),
        )
    )
    admin = result.scalar_one_or_none()
    if not admin:
        return None

    now = datetime.now(timezone.utc)

    # Only allow resend if temp password expired or user never logged in
    if admin.password_expires_at and admin.password_expires_at > now and not admin.first_login:
        raise ValueError("Password sementara masih berlaku. Tidak perlu dikirim ulang.")

    temp_pw = generate_temp_password()
    admin.password_hash = hash_password(temp_pw)
    admin.first_login = True
    admin.is_temp_password = True
    admin.password_expires_at = now + timedelta(days=7)
    admin.updated_at = now
    await db.flush()
    return admin, temp_pw


async def deactivate_admin(db: AsyncSession, admin_id: UUID) -> User | None:
    """Deactivate admin account (is_active=False)."""
    result = await db.execute(
        select(User).where(
            User.id_user == admin_id,
            User.role == "admin",
            User.deleted_at.is_(None),
        )
    )
    admin = result.scalar_one_or_none()
    if not admin:
        return None

    admin.is_active = False
    admin.updated_at = datetime.now(timezone.utc)
    await db.flush()
    return admin


async def activate_admin(db: AsyncSession, admin_id: UUID) -> User | None:
    """Re-activate admin account."""
    result = await db.execute(
        select(User).where(
            User.id_user == admin_id,
            User.role == "admin",
            User.deleted_at.is_(None),
        )
    )
    admin = result.scalar_one_or_none()
    if not admin:
        return None

    admin.is_active = True
    admin.updated_at = datetime.now(timezone.utc)
    await db.flush()
    return admin


async def delete_admin(
    db: AsyncSession, admin_id: UUID, confirm_username: str
) -> User | None:
    """Soft-delete admin + anonymize PII. Requires username confirmation.

    - Sets deleted_at
    - Anonymizes: username -> 'deleted_<id>', email -> 'deleted_<id>@removed.local'
    - Clears refresh_token (force logout)
    - Sets is_active = False
    """
    result = await db.execute(
        select(User).where(
            User.id_user == admin_id,
            User.role == "admin",
            User.deleted_at.is_(None),
        )
    )
    admin = result.scalar_one_or_none()
    if not admin:
        return None

    if admin.username != confirm_username:
        raise ValueError(
            f"Konfirmasi gagal. Ketik '{admin.username}' untuk menghapus."
        )

    # Anonymize PII
    short_id = str(admin.id_user)[:8]
    admin.username = f"deleted_{short_id}"
    admin.email = f"deleted_{short_id}@removed.local"
    admin.is_active = False
    admin.refresh_token = None
    admin.token_expires_at = None
    admin.deleted_at = datetime.now(timezone.utc)
    admin.updated_at = datetime.now(timezone.utc)
    await db.flush()
    return admin
