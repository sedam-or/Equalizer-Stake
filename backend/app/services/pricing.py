"""Pricing service — tarif management."""

from datetime import date
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.pricing import Pricing

VALID_INSTRUMENTS = {"biola", "piano", "gitar", "drum", "keyboard", "flute", "saxophone", "vokal", "ensemble"}
VALID_LEVELS = {"pemula", "dasar", "menengah", "lanjut"}


async def get_all_pricing(db: AsyncSession) -> list[Pricing]:
    """Get all pricing entries ordered by instrument, level, effective_date."""
    result = await db.execute(
        select(Pricing).order_by(
            Pricing.instrument, Pricing.level, Pricing.effective_date.desc()
        )
    )
    return list(result.scalars().all())


async def get_pricing(db: AsyncSession, pricing_id: UUID) -> Pricing | None:
    """Get single pricing by ID."""
    result = await db.execute(
        select(Pricing).where(Pricing.id_pricing == pricing_id)
    )
    return result.scalar_one_or_none()


async def create_pricing(
    db: AsyncSession,
    instrument: str,
    level: str,
    price: float,
    effective_date: date,
    description: str | None,
    created_by: UUID,
) -> Pricing:
    """Create new pricing entry."""
    pricing = Pricing(
        instrument=instrument,
        level=level,
        price=price,
        effective_date=effective_date,
        description=description,
        created_by=created_by,
    )
    db.add(pricing)
    return pricing


async def update_pricing(
    db: AsyncSession,
    pricing: Pricing,
    **kwargs,
) -> Pricing:
    """Update pricing fields."""
    for key, value in kwargs.items():
        if value is not None and hasattr(pricing, key):
            setattr(pricing, key, value)
    return pricing


def validate_pricing_input(instrument: str, level: str, price: float) -> str | None:
    """Validate pricing input. Returns error message or None."""
    if instrument not in VALID_INSTRUMENTS:
        return f"Instrumen '{instrument}' tidak valid. Pilihan: {', '.join(sorted(VALID_INSTRUMENTS))}"
    if level not in VALID_LEVELS:
        return f"Level '{level}' tidak valid. Pilihan: {', '.join(sorted(VALID_LEVELS))}"
    if price < 0:
        return "Harga tidak boleh negatif"
    return None
