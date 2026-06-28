"""Admin management service — superadmin CRUD for admin accounts."""

from datetime import datetime, timezone
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.user import User
from app.services.auth import hash_password


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
    password: str,
) -> User:
    """Create new admin account. Raises ValueError on duplicate."""
    # Check duplicate username
    existing = await db.execute(
        select(User).where(User.username == username)
    )
    if existing.scalar_one_or_none():
        raise ValueError("Username sudah digunakan")

    # Check duplicate email
    existing_email = await db.execute(
        select(User).where(User.email == email)
    )
    if existing_email.scalar_one_or_none():
        raise ValueError("Email sudah digunakan")

    admin = User(
        username=username,
        email=email,
        password_hash=hash_password(password),
        role="admin",
        is_active=True,
    )
    db.add(admin)
    await db.flush()
    return admin


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
