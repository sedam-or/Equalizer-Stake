"""Seed master data — Instrumen, Level, Tipe Kelas, Harga.

Usage:
    cd backend
    python -m scripts.seed_pricing

Data dari addendum SRS (srs/99-addendum-business-rules.md).
"""

import asyncio
import os
import sys
from datetime import date

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from app.core.database import async_session_maker, engine, Base
from app.models.pricing import Pricing
from sqlalchemy import select

EFFECTIVE_DATE = date(2024, 1, 1)

# Registration fees per instrument
REGISTRATION_FEES = {
    "piano_klasik": 420000,
    "piano_pop": 420000,
    "vokal": 420000,
    "biola": 540000,
    "gitar": 420000,
    "keyboard": 480000,
    "drum": 420000,
    "flute": 420000,
    "saxophone": 420000,
}

# Pricing data: (instrument, class_type, level, monthly_fee, duration_minutes)
PRICING_DATA = [
    # === VOKAL ===
    ("vokal", "private", "basic", 600000, 45),
    ("vokal", "private", "intermediate", 600000, 45),
    ("vokal", "private", "advance", 600000, 45),
    ("vokal", "intensif_2", "basic", 390000, 45),
    ("vokal", "intensif_2", "intermediate", 390000, 45),
    ("vokal", "intensif_2", "advance", 390000, 45),
    ("vokal", "intensif_3", "basic", 330000, 45),
    ("vokal", "intensif_3", "intermediate", 330000, 45),
    ("vokal", "intensif_3", "advance", 330000, 45),
    ("vokal", "semi_intensif", "basic", 300000, 45),
    ("vokal", "semi_intensif", "intermediate", 300000, 45),
    ("vokal", "semi_intensif", "advance", 300000, 45),
    ("vokal", "expatriate_private", "basic", 840000, 45),
    ("vokal", "expatriate_private", "intermediate", 840000, 45),
    ("vokal", "expatriate_private", "advance", 840000, 45),
    ("vokal", "expatriate_2", "basic", 660000, 45),
    ("vokal", "expatriate_2", "intermediate", 660000, 45),
    ("vokal", "expatriate_3", "basic", 600000, 45),
    ("vokal", "expatriate_3", "intermediate", 600000, 45),

    # === PIANO KLASIK ===
    ("piano_klasik", "private", "pra_elementary", 420000, 30),
    ("piano_klasik", "private", "elementary", 420000, 30),
    ("piano_klasik", "private", "grade_1", 480000, 45),
    ("piano_klasik", "private", "grade_2", 480000, 45),
    ("piano_klasik", "private", "grade_3", 510000, 45),
    ("piano_klasik", "private", "grade_4", 540000, 45),
    ("piano_klasik", "private", "grade_5", 570000, 45),
    ("piano_klasik", "private", "grade_6", 630000, 45),
    ("piano_klasik", "tunas", "pra_elementary", 600000, 30),
    ("piano_klasik", "tunas", "elementary", 600000, 30),

    # === PIANO EXPATRIATE ===
    ("piano_klasik", "expatriate_private", "beginner", 540000, 45),
    ("piano_klasik", "expatriate_private", "grade_1", 600000, 45),
    ("piano_klasik", "expatriate_private", "grade_2", 630000, 45),
    ("piano_klasik", "expatriate_private", "grade_3", 660000, 45),
    ("piano_klasik", "expatriate_private", "grade_4", 720000, 45),
    ("piano_klasik", "expatriate_private", "grade_5", 780000, 45),
    ("piano_klasik", "expatriate_private", "grade_6", 840000, 45),

    # === PIANO POP ===
    ("piano_pop", "private", "beginner", 600000, 45),
    ("piano_pop", "private", "intermediate", 600000, 45),
    ("piano_pop", "private", "intermediate_2", 600000, 45),
    ("piano_pop", "expatriate_private", "beginner", 720000, 45),
    ("piano_pop", "expatriate_private", "intermediate", 720000, 45),
    ("piano_pop", "expatriate_private", "intermediate_2", 720000, 45),

    # === BIOLA ===
    ("biola", "private", "basic", 420000, 45),
    ("biola", "private", "intermediate", 420000, 45),
    ("biola", "private", "advance", 420000, 45),

    # === GITAR ===
    ("gitar", "private", "basic", 420000, 45),
    ("gitar", "private", "intermediate", 420000, 45),
    ("gitar", "private", "advance", 420000, 45),

    # === KEYBOARD ===
    ("keyboard", "private", "basic", 420000, 45),
    ("keyboard", "private", "intermediate", 420000, 45),
    ("keyboard", "private", "advance", 420000, 45),

    # === DRUM ===
    ("drum", "private", "basic", 420000, 45),
    ("drum", "private", "intermediate", 420000, 45),
    ("drum", "private", "advance", 420000, 45),

    # === FLUTE ===
    ("flute", "private", "basic", 420000, 45),
    ("flute", "private", "intermediate", 420000, 45),
    ("flute", "private", "advance", 420000, 45),

    # === SAXOPHONE ===
    ("saxophone", "private", "basic", 420000, 45),
    ("saxophone", "private", "intermediate", 420000, 45),
    ("saxophone", "private", "advance", 420000, 45),

    # === PAKET LIBURAN (all instruments) ===
    ("vokal", "paket_liburan", "basic", 780000, 45),
    ("vokal", "paket_liburan", "intermediate", 780000, 45),
    ("vokal", "paket_liburan", "advance", 780000, 45),
    ("piano_klasik", "paket_liburan", "basic", 780000, 45),
    ("piano_klasik", "paket_liburan", "intermediate", 780000, 45),
    ("gitar", "paket_liburan", "basic", 780000, 45),
    ("biola", "paket_liburan", "basic", 780000, 45),
]


async def seed():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    async with async_session_maker() as db:
        # Check if pricing already exists
        existing = await db.execute(select(Pricing).limit(1))
        if existing.scalar_one_or_none():
            print("Pricing data already exists. Skipping.")
            return

        count = 0
        for instrument, class_type, level, fee, duration in PRICING_DATA:
            reg_fee = REGISTRATION_FEES.get(instrument, 420000)
            pricing = Pricing(
                instrument=instrument,
                class_type=class_type,
                level=level,
                monthly_fee=fee,
                duration_minutes=duration,
                registration_fee=reg_fee,
                effective_date=EFFECTIVE_DATE,
                is_active=True,
            )
            db.add(pricing)
            count += 1

        await db.commit()
        print(f"Seeded {count} pricing entries.")
        print(f"Instruments: {len(REGISTRATION_FEES)}")
        print(f"Class types: private, intensif_2, intensif_3, semi_intensif, expatriate_private, expatriate_2, expatriate_3, tunas, paket_liburan")


if __name__ == "__main__":
    asyncio.run(seed())
