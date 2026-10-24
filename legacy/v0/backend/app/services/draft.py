"""Draft service — auto-save draft CRUD with TTL cleanup."""

from datetime import datetime, timedelta, timezone
from uuid import UUID

from sqlalchemy import delete, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.draft import Draft

DRAFT_TTL_DAYS = 7


class DraftConflictError(Exception):
    """Raised when draft save conflicts with a newer server version."""

    def __init__(self, server_draft: Draft, client_data: dict):
        self.server_draft = server_draft
        self.client_data = client_data
        super().__init__("Draft konflik: versi server lebih baru")


async def save_draft(
    db: AsyncSession,
    user_id: UUID,
    form_key: str,
    draft_data: dict,
    client_updated_at: datetime | None = None,
) -> Draft:
    """Save or update draft for user+form combination."""
    result = await db.execute(
        select(Draft).where(
            Draft.id_user == user_id,
            Draft.form_key == form_key,
        )
    )
    existing = result.scalar_one_or_none()

    if existing:
        # Conflict check: if client has stale timestamp, reject
        if client_updated_at and existing.updated_at and existing.updated_at > client_updated_at:
            raise DraftConflictError(existing, draft_data)
        existing.draft_data = draft_data
        existing.updated_at = datetime.now(timezone.utc)
        return existing
    else:
        draft = Draft(
            id_user=user_id,
            form_key=form_key,
            draft_data=draft_data,
        )
        db.add(draft)
        return draft


async def get_draft(
    db: AsyncSession,
    user_id: UUID,
    form_key: str,
) -> Draft | None:
    """Get draft for user+form."""
    result = await db.execute(
        select(Draft).where(
            Draft.id_user == user_id,
            Draft.form_key == form_key,
        )
    )
    return result.scalar_one_or_none()


async def get_user_drafts(
    db: AsyncSession,
    user_id: UUID,
) -> list[Draft]:
    """Get all drafts for a user."""
    result = await db.execute(
        select(Draft).where(Draft.id_user == user_id)
        .order_by(Draft.updated_at.desc())
    )
    return list(result.scalars().all())


async def delete_draft(
    db: AsyncSession,
    user_id: UUID,
    form_key: str,
) -> bool:
    """Delete draft after permanent save."""
    result = await db.execute(
        delete(Draft).where(
            Draft.id_user == user_id,
            Draft.form_key == form_key,
        )
    )
    return result.rowcount > 0


async def cleanup_expired_drafts(db: AsyncSession) -> int:
    """Delete drafts older than TTL. Returns count deleted."""
    cutoff = datetime.now(timezone.utc) - timedelta(days=DRAFT_TTL_DAYS)
    result = await db.execute(
        delete(Draft).where(Draft.updated_at < cutoff)
    )
    return result.rowcount
