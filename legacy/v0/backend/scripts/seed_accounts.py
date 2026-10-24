"""Seed script — create test accounts + 50 students + 10 guardians.

Usage:
    cd backend
    ADMIN_DEFAULT_PASSWORD="***" python -m scripts.seed_accounts

Accounts created:
    owner_eqs    / ADMIN_DEFAULT_PASSWORD  (owner)
    admin_eqs    / ADMIN_DEFAULT_PASSWORD  (admin)
    instr_eqs    / ADMIN_DEFAULT_PASSWORD  (teacher)
    guardian_01..guardian_10 / ADMIN_DEFAULT_PASSWORD  (guardian, each with 5 students)

Set ADMIN_DEFAULT_PASSWORD env var before running.
"""

import asyncio
import os
import sys
import random
from datetime import date, timedelta

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from app.core.database import async_session_maker, engine, Base
from app.models.user import User
from app.models.guardian import Guardian
from app.models.teacher import Teacher
from app.models.student import Student
from app.services.auth import hash_password

DEFAULT_PASSWORD = os.environ.get("ADMIN_DEFAULT_PASSWORD")
if not DEFAULT_PASSWORD:
    print("ERROR: Set ADMIN_DEFAULT_PASSWORD env var first")
    sys.exit(1)

STUDENT_NAMES = [
    "Andi Pratama", "Rina Wulandari", "Budi Santoso", "Dewi Lestari", "Eko Prasetyo",
    "Fitri Handayani", "Gilang Ramadhan", "Hana Putri", "Irfan Hakim", "Joko Widodo",
    "Kartika Sari", "Lukman Hakim", "Maya Angelina", "Nanda Pratama", "Olivia Chen",
    "Putra Wijaya", "Qori Shodiqin", "Rizky Febian", "Siti Nurhaliza", "Tono Sutrisno",
    "Ulya Maghfiroh", "Vina Panduwinata", "Wawan Setiawan", "Xena Putri", "Yusuf Mansur",
    "Zahra Amalia", "Aldi Taher", "Bunga Citra", "Cakra Khan", "Dian Sastro",
    "Ernest Prakasa", "Fatin Shidqia", "Giring Ganesha", "Hesti Purwadinata", "Iqbaal Ramadhan",
    "Jessica Mila", "Kevin Julio", "Luna Maya", "Maudy Ayunda", "Nicholas Saputra",
    "Pevita Pearce", "Raffi Ahmad", "Syahrini", "Tasya Kamila", "Ussy Sulistiawaty",
    "Vidi Aldiano", "Widyawati", "Yuki Kato", "Zaskia Sungkar", "Ariel Noah",
]

INSTRUMENTS = ["piano", "gitar", "biola", "drum", "vokal", "keyboard", "flute", "saxophone"]
LEVELS = ["pemula", "dasar", "menengah", "lanjut"]

ACCOUNTS = [
    {"username": "owner_eqs", "email": "owner@equalizer-stake.dev", "role": "owner", "full_name": "EQS Owner", "phone": "+62-812-0000-0001"},
    {"username": "admin_eqs", "email": "admin@equalizer-stake.dev", "role": "admin", "full_name": "Admin Operasional", "phone": "081234567891"},
    {"username": "instr_eqs", "email": "instructor@equalizer-stake.dev", "role": "teacher", "full_name": "Instructor EQS", "phone": "081234567892"},
]


async def seed():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    async with async_session_maker() as db:
        from sqlalchemy import select

        # Create owner/admin/teacher
        for acct in ACCOUNTS:
            existing = await db.execute(select(User).where(User.username == acct["username"]))
            if existing.scalar_one_or_none():
                print(f"  SKIP: {acct['username']} already exists")
                continue

            assert DEFAULT_PASSWORD is not None
            user = User(
                username=acct["username"],
                email=acct["email"],
                password_hash=hash_password(DEFAULT_PASSWORD),
                role=acct["role"],
                is_active=True,
            )
            db.add(user)
            await db.flush()

            if acct["role"] == "teacher":
                teacher = Teacher(
                    id_user=user.id_user,
                    full_name=acct["full_name"],
                    instruments_specialization={"primary": "gitar", "secondary": ["piano", "drum"]},
                    employment_type="tetap",
                )
                db.add(teacher)

            print(f"  CREATED: {acct['username']} ({acct['role']})")

        await db.flush()

        # Create 10 guardians with 5 students each = 50 students
        teacher_user = await db.execute(select(User).where(User.username == "instr_eqs"))
        teacher = teacher_user.scalar_one_or_none()

        for i in range(1, 11):
            username = f"guardian_{i:02d}"
            existing = await db.execute(select(User).where(User.username == username))
            if existing.scalar_one_or_none():
                print(f"  SKIP: {username} already exists")
                continue

            assert DEFAULT_PASSWORD is not None
            user = User(
                username=username,
                email=f"guardian{i}@equalizer-stake.dev",
                password_hash=hash_password(DEFAULT_PASSWORD),
                role="guardian",
                is_active=True,
            )
            db.add(user)
            await db.flush()

            guardian = Guardian(
                id_user=user.id_user,
                full_name=f"Guardian {i}",
                phone_whatsapp=f"08123456{i:04d}",
                email=f"guardian{i}@equalizer-stake.dev",
                account_status="active",
            )
            db.add(guardian)
            await db.flush()

            # 5 students per guardian
            for j in range(5):
                idx = (i - 1) * 5 + j
                student = Student(
                    id_guardian=guardian.id_guardian,
                    full_name=STUDENT_NAMES[idx % len(STUDENT_NAMES)],
                    date_of_birth=date(2010 + random.randint(0, 8), random.randint(1, 12), random.randint(1, 28)),
                    instrument=random.choice(INSTRUMENTS),
                    level=random.choice(LEVELS),
                    enrollment_status=random.choice(["active", "active", "active", "trial", "cuti"]),
                    enrollment_date=date.today() - timedelta(days=random.randint(30, 365)),
                )
                db.add(student)

            print(f"  CREATED: {username} (guardian) + 5 students")

        await db.commit()
        print("\nSeed complete.")
        print(f"  Accounts: owner_eqs, admin_eqs, instr_eqs, guardian_01..guardian_10")
        print(f"  Students: 50 (5 per guardian)")
        print(f"  Password: ADMIN_DEFAULT_PASSWORD env var")


if __name__ == "__main__":
    asyncio.run(seed())
