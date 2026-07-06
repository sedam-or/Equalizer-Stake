"""Seed script — populate database with realistic BMD Music School data."""
import asyncio
import random
from datetime import datetime, timedelta, time
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
from sqlalchemy.orm import sessionmaker
from sqlalchemy import text

DATABASE_URL = "postgresql+asyncpg://eqs_admin:eqs_dev_password@localhost:5433/equalizer_stake"
engine = create_async_engine(DATABASE_URL)
Session = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)

INSTRUMENTS = ["piano", "gitar", "vokal", "biola", "drum", "keyboard", "flute", "saxophone", "bass"]
LEVELS = ["pemula", "dasar", "menengah", "lanjut"]
FIRST_NAMES = ["Raka", "Dewi", "Bimo", "Citra", "Eka", "Fajar", "Gita", "Hadi", "Intan", "Joko",
               "Kartika", "Lukman", "Maya", "Nando", "Olive", "Putri", "Rizky", "Sinta", "Tono", "Umi"]
LAST_NAMES = ["Pratama", "Putra", "Putri", "Sari", "Saputra", "Dewi", "Firmansyah", "Wijaya", "Setiawan", "Hidayat"]

async def safe_exec(session, sql):
    """Execute SQL with proper error handling."""
    try:
        r = await session.execute(text(sql))
        return r
    except Exception:
        await session.rollback()
        return None

async def seed():
    async with Session() as session:
        # 1. Get existing teacher IDs
        print("=== TEACHERS ===")
        r = await session.execute(text("SELECT id_teacher, full_name FROM teachers"))
        teacher_ids = [row[0] for row in r]
        print(f"  Found {len(teacher_ids)} teachers")

        # 2. Get existing rooms
        r = await session.execute(text("SELECT id_room FROM rooms"))
        room_ids = [row[0] for row in r]
        print(f"  Found {len(room_ids)} rooms")

        # 3. Create guardians
        print("\n=== GUARDIANS ===")
        guardian_ids = []
        for i in range(1, 16):
            uname = f"ortu_{i:02d}"
            try:
                r = await session.execute(text(
                    "INSERT INTO users (username, password_hash, role, is_active, created_at) "
                    "VALUES (:u, '$2b$12$LJ3m4ys4Lg0TSwMCPbNMOeEGYJqE3G3K8wJqH7z4k9v5m8n2p1q', 'guardian', true, NOW()) "
                    "RETURNING id_user"
                ), {"u": uname})
                uid = r.scalar()
                guardian_ids.append(uid)
                print(f"  + {uname}")
            except Exception:
                await session.rollback()
                r = await session.execute(text("SELECT id_user FROM users WHERE username=:u"), {"u": uname})
                row = r.fetchone()
                if row:
                    guardian_ids.append(row[0])
                    print(f"  = {uname} (exists)")
        await session.commit()

        # 4. Create students
        print("\n=== STUDENTS ===")
        student_ids = []
        for gid in guardian_ids:
            num = random.choice([1, 1, 1, 2])
            for _ in range(num):
                fname = random.choice(FIRST_NAMES) + " " + random.choice(LAST_NAMES)
                instr = random.choice(INSTRUMENTS)
                level = random.choice(LEVELS)
                try:
                    r = await session.execute(text(
                        "INSERT INTO students (id_guardian, full_name, instrument, level, is_active, created_at) "
                        "VALUES (:g, :n, :i, :l, true, NOW()) RETURNING id_student"
                    ), {"g": gid, "n": fname, "i": instr, "l": level})
                    sid = r.scalar()
                    student_ids.append(sid)
                except Exception:
                    await session.rollback()
        await session.commit()
        print(f"  Created {len(student_ids)} students")

        # Get existing students too
        r = await session.execute(text("SELECT id_student FROM students"))
        all_student_ids = [row[0] for row in r]
        print(f"  Total students: {len(all_student_ids)}")

        # 5. Create sessions (this week + next week)
        print("\n=== SESSIONS ===")
        today = datetime.now().replace(hour=0, minute=0, second=0, microsecond=0)
        timeslots = [time(8, 0), time(9, 0), time(10, 0), time(11, 0),
                     time(13, 0), time(14, 0), time(15, 0), time(16, 0), time(17, 0), time(18, 0)]
        session_count = 0
        for day_offset in range(-3, 11):
            day = today + timedelta(days=day_offset)
            if day.weekday() >= 6:
                continue
            num = random.randint(4, 7)
            slots = random.sample(timeslots, min(num, len(timeslots)))
            for st in slots:
                if not all_student_ids or not teacher_ids or not room_ids:
                    continue
                sid = random.choice(all_student_ids)
                tid = random.choice(teacher_ids)
                rid = random.choice(room_ids)
                dur = random.choice([30, 45])
                status = "completed" if day_offset < 0 else "scheduled"
                end_time = (datetime.combine(day.date(), st) + timedelta(minutes=dur)).time()
                try:
                    await session.execute(text(
                        "INSERT INTO sessions (id_student, id_teacher, id_room, session_date, "
                        "start_time, end_time, duration_minutes, session_type, status, created_at) "
                        "VALUES (:s, :t, :r, :d, :st, :et, :dur, 'private', :st2, NOW())"
                    ), {"s": sid, "t": tid, "r": rid, "d": day.date(), "st": st, "et": end_time, "dur": dur, "st2": status})
                    session_count += 1
                except Exception:
                    await session.rollback()
        await session.commit()
        print(f"  Created {session_count} sessions")

        # 6. Create attendance for completed sessions
        print("\n=== ATTENDANCE ===")
        r = await session.execute(text(
            "SELECT s.id_session, s.id_student FROM sessions s "
            "WHERE s.status='completed' AND s.id_session NOT IN (SELECT id_session FROM attendances) LIMIT 25"
        ))
        completed = [(row[0], row[1]) for row in r]
        att_count = 0
        for sess_id, stud_id in completed:
            status = random.choice(["hadir", "hadir", "hadir", "hadir", "izin", "alpha"])
            try:
                await session.execute(text(
                    "INSERT INTO attendances (id_session, id_student, status, verified_at, created_at) "
                    "VALUES (:s, :st, :st2, NOW(), NOW())"
                ), {"s": sess_id, "st": stud_id, "st2": status})
                att_count += 1
            except Exception:
                await session.rollback()
        await session.commit()
        print(f"  Created {att_count} attendances")

        # 7. Create invoices + payments
        print("\n=== INVOICES + PAYMENTS ===")
        inv_count = 0
        pay_count = 0
        for gid in guardian_ids[:12]:
            amount = random.choice([350000, 500000, 750000, 1000000])
            status = random.choice(["pending", "issued", "paid", "paid", "paid"])
            try:
                r = await session.execute(text(
                    "INSERT INTO invoices (id_guardian, billing_period, total_amount, status, due_date, created_at) "
                    "VALUES (:g, '2026-07', :a, :s, '2026-07-15', NOW()) RETURNING id_invoice"
                ), {"g": gid, "a": amount, "s": status})
                inv_id = r.scalar()
                inv_count += 1
                if status == "paid":
                    try:
                        await session.execute(text(
                            "INSERT INTO payments (id_invoice, id_guardian, amount, payment_method, status, payment_date, created_at) "
                            "VALUES (:i, :g, :a, 'transfer', 'verified', NOW(), NOW())"
                        ), {"i": inv_id, "g": gid, "a": amount})
                        pay_count += 1
                    except Exception:
                        await session.rollback()
            except Exception:
                await session.rollback()
        await session.commit()
        print(f"  Created {inv_count} invoices, {pay_count} payments")

        # Summary
        print("\n=== FINAL COUNTS ===")
        for t in ['users', 'teachers', 'students', 'sessions', 'attendances', 'invoices', 'payments']:
            r = await session.execute(text(f"SELECT COUNT(*) FROM {t}"))
            print(f"  {t}: {r.scalar()}")

asyncio.run(seed())
