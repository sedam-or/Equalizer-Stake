"""Seed script — create mock data to simulate running system.

Usage:
    cd backend
    python -m scripts.seed_mock_data

Creates:
- 5 teachers with instruments
- 10 students (wali_01..wali_05, 2 children each)
- 20 sessions (past + future)
- 5 invoices
- Sample notifications
"""

import asyncio
import os
import sys
import random
from datetime import date, time, timedelta, datetime, timezone

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from app.core.database import async_session_maker, engine, Base
from app.models.user import User
from app.models.guardian import Guardian
from app.models.teacher import Teacher
from app.models.student import Student
from app.models.session import Session
from app.models.room import Room
from app.models.invoice import Invoice
from app.models.notification import Notification
from app.services.billing import generate_unique_code
from sqlalchemy import select

TEACHERS = [
    {"username": "guru_bmd", "full_name": "Pak Dimas Pratama", "instruments": ["piano", "keyboard"], "type": "tetap"},
    {"username": "guru_sari", "full_name": "Mbak Sari Wulandari", "instruments": ["vokal", "biola"], "type": "tetap"},
    {"username": "guru_rizky", "full_name": "Pak Rizky Febian", "instruments": ["gitar", "gitar"], "type": "tetap"},
    {"username": "guru_andi", "full_name": "Pak Andi Drum", "instruments": ["drum"], "type": "freelance"},
    {"username": "guru_hana", "full_name": "Mbak Hana Flute", "instruments": ["flute", "saxophone"], "type": "freelance"},
]

STUDENTS = [
    {"guardian": "wali_01", "name": "Andi Pratama", "instrument": "piano", "level": "pemula"},
    {"guardian": "wali_01", "name": "Rina Wulandari", "instrument": "vokal", "level": "dasar"},
    {"guardian": "wali_02", "name": "Budi Santoso", "instrument": "gitar", "level": "menengah"},
    {"guardian": "wali_02", "name": "Dewi Lestari", "instrument": "biola", "level": "pemula"},
    {"guardian": "wali_03", "name": "Eko Prasetyo", "instrument": "drum", "level": "dasar"},
    {"guardian": "wali_03", "name": "Fitri Handayani", "instrument": "piano", "level": "menengah"},
    {"guardian": "wali_04", "name": "Gilang Ramadhan", "instrument": "keyboard", "level": "pemula"},
    {"guardian": "wali_04", "name": "Hana Putri", "instrument": "flute", "level": "dasar"},
    {"guardian": "wali_05", "name": "Irfan Hakim", "instrument": "saxophone", "level": "lanjut"},
    {"guardian": "wali_05", "name": "Joko Widodo", "instrument": "gitar", "level": "pemula"},
]


async def seed():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    async with async_session_maker() as db:
        # 1. Create teachers
        print("Creating teachers...")
        teachers = []
        for t_data in TEACHERS:
            user_result = await db.execute(select(User).where(User.username == t_data["username"]))
            user = user_result.scalar_one_or_none()
            if not user:
                print(f"  SKIP: {t_data['username']} not found")
                continue
            existing = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
            if existing.scalar_one_or_none():
                print(f"  SKIP: teacher profile exists for {t_data['username']}")
                continue
            teacher = Teacher(
                id_user=user.id_user,
                full_name=t_data["full_name"],
                instruments_specialization=t_data["instruments"],
                employment_type=t_data["type"],
                max_students_per_session=3 if t_data["type"] == "tetap" else 1,
            )
            db.add(teacher)
            await db.flush()
            teachers.append(teacher)
            print(f"  Created: {t_data['full_name']}")

        # 2. Create students
        print("Creating students...")
        students = []
        for s_data in STUDENTS:
            guardian_user = await db.execute(select(User).where(User.username == s_data["guardian"]))
            user = guardian_user.scalar_one_or_none()
            if not user:
                print(f"  SKIP: {s_data['guardian']} not found")
                continue
            guardian_result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
            guardian = guardian_result.scalar_one_or_none()
            if not guardian:
                print(f"  SKIP: guardian profile for {s_data['guardian']}")
                continue
            existing = await db.execute(select(Student).where(
                Student.id_guardian == guardian.id_guardian,
                Student.full_name == s_data["name"]
            ))
            if existing.scalar_one_or_none():
                print(f"  SKIP: {s_data['name']} exists")
                continue
            student = Student(
                id_guardian=guardian.id_guardian,
                full_name=s_data["name"],
                instrument=s_data["instrument"],
                level=s_data["level"],
                enrollment_status="active",
                enrollment_date=date.today() - timedelta(days=random.randint(30, 180)),
            )
            db.add(student)
            await db.flush()
            students.append(student)
            print(f"  Created: {s_data['name']} ({s_data['instrument']})")

        # 3. Get rooms
        rooms_result = await db.execute(select(Room))
        rooms = list(rooms_result.scalars().all())
        if not rooms:
            print("No rooms found, skipping sessions")
            return

        # 4. Create sessions (past + future)
        print("Creating sessions...")
        if teachers and students:
            for i in range(20):
                teacher = random.choice(teachers)
                student = random.choice(students)
                room = random.choice(rooms)
                days_offset = random.randint(-14, 14)
                session_date = date.today() + timedelta(days=days_offset)
                hour = random.choice([9, 10, 14, 15, 16])
                start = time(hour, 0)
                end = time(hour + 1, 0)

                # Check for duplicate
                existing = await db.execute(select(Session).where(
                    Session.session_date == session_date,
                    Session.start_time == start,
                    Session.id_teacher == teacher.id_teacher,
                ))
                if existing.scalar_one_or_none():
                    continue

                session = Session(
                    id_teacher=teacher.id_teacher,
                    id_student=student.id_student,
                    id_room=room.id_room,
                    session_date=session_date,
                    start_time=start,
                    end_time=end,
                    session_type="reguler",
                    status="completed" if days_offset < 0 else "pending",
                )
                db.add(session)
            await db.flush()
            print(f"  Created 20 sessions")

        # 5. Create invoices
        print("Creating invoices...")
        guardians_result = await db.execute(select(Guardian).where(Guardian.account_status == "active"))
        guardians = list(guardians_result.scalars().all())
        for guardian in guardians:
            students_result = await db.execute(select(Student).where(
                Student.id_guardian == guardian.id_guardian,
                Student.enrollment_status == "active",
            ))
            student_list = list(students_result.scalars().all())
            if not student_list:
                continue
            total = sum(random.randint(300, 600) * 1000 for _ in student_list)
            billing_period = f"{date.today().year}-{date.today().month - 1:02d}" if date.today().month > 1 else f"{date.today().year - 1}-12"
            unique_code = await generate_unique_code(db, billing_period, guardian.id_guardian)
            invoice = Invoice(
                id_guardian=guardian.id_guardian,
                unique_code=unique_code,
                billing_period=billing_period,
                total_amount=total,
                due_date=date.today().replace(day=10),
                status=random.choice(["issued", "paid", "issued"]),
                issued_at=datetime.now(timezone.utc) - timedelta(days=random.randint(1, 10)),
            )
            db.add(invoice)
        await db.flush()
        print(f"  Created invoices for {len(guardians)} guardians")

        # 6. Create notifications
        print("Creating notifications...")
        for guardian in guardians[:3]:
            notif = Notification(
                recipient_id=guardian.id_user,
                channel="email",
                category="non_critical",
                subject="Invoice Baru",
                body=f"Invoice untuk bulan ini telah diterbitkan. Silakan cek halaman Pembayaran.",
                status="sent",
            )
            db.add(notif)
        await db.flush()
        print(f"  Created notifications")

        await db.commit()
        print("\n✅ Seed data created successfully!")


if __name__ == "__main__":
    asyncio.run(seed())
