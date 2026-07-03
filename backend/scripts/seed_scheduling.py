"""Seed script — rooms + sample sessions for T-060.

Usage:
    cd backend
    python -m scripts.seed_scheduling
"""

import asyncio
import os
import sys
from datetime import date, time, timedelta

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from app.core.database import async_session_maker, engine, Base
from app.models.room import Room
from app.models.session import Session
from app.models.teacher import Teacher
from app.models.student import Student

ROOMS = [
    {"room_name": "Studio A", "room_type": "kursus", "capacity": 2},
    {"room_name": "Studio B", "room_type": "kursus", "capacity": 2},
    {"room_name": "Studio C", "room_type": "rehearsal", "capacity": 4},
]


async def seed():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    async with async_session_maker() as db:
        from sqlalchemy import select

        # Seed rooms
        for r in ROOMS:
            existing = await db.execute(select(Room).where(Room.room_name == r["room_name"]))
            if existing.scalar_one_or_none():
                print(f"  SKIP room: {r['room_name']} already exists")
                continue
            room = Room(room_name=r["room_name"], room_type=r["room_type"], capacity=r["capacity"])
            db.add(room)
            print(f"  CREATED room: {r['room_name']}")

        await db.flush()

        # Get first teacher + first 5 students for sample sessions
        teacher = (await db.execute(select(Teacher).where(Teacher.is_active == True).limit(1))).scalar_one_or_none()
        students = (await db.execute(select(Student).where(Student.deleted_at.is_(None)).limit(5))).scalars().all()
        rooms = (await db.execute(select(Room).limit(3))).scalars().all()

        if not teacher or not students or not rooms:
            print("  ERROR: Need at least 1 teacher, 5 students, 3 rooms. Run seed_accounts.py first.")
            return

        # Check if sessions already exist
        existing_count = (await db.execute(select(Session))).scalars().all()
        if len(existing_count) >= 5:
            print(f"  SKIP sessions: {len(existing_count)} already exist")
            await db.commit()
            return

        # Create 5 sample sessions over next week
        today = date.today()
        # Find next Monday
        days_until_monday = (7 - today.weekday()) % 7
        if days_until_monday == 0:
            days_until_monday = 7
        next_monday = today + timedelta(days=days_until_monday)

        session_slots = [
            (0, time(9, 0), time(10, 0), "reguler"),   # Monday 09:00-10:00
            (0, time(10, 30), time(11, 30), "reguler"),  # Monday 10:30-11:30
            (1, time(13, 0), time(14, 0), "reguler"),    # Tuesday 13:00-14:00
            (2, time(14, 0), time(15, 0), "teori"),      # Wednesday 14:00-15:00
            (3, time(16, 0), time(17, 0), "reguler"),    # Thursday 16:00-17:00
        ]

        for i, (day_offset, start, end, stype) in enumerate(session_slots):
            s = Session(
                id_teacher=teacher.id_teacher,
                id_student=students[i % len(students)].id_student,
                id_room=rooms[i % len(rooms)].id_room,
                session_date=next_monday + timedelta(days=day_offset),
                start_time=start,
                end_time=end,
                session_type=stype,
                status="pending",
            )
            db.add(s)
            print(f"  CREATED session: {s.session_date} {start.strftime('%H:%M')}-{end.strftime('%H:%M')} ({stype})")

        await db.commit()
        print("\nSeed scheduling complete.")


if __name__ == "__main__":
    asyncio.run(seed())
