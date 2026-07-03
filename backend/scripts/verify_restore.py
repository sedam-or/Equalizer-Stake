"""Verify database integrity after restore.

Usage: python -m scripts.verify_restore
"""
import asyncio
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from sqlalchemy import text
from app.core.database import async_session_maker

EXPECTED_TABLES = [
    "users", "guardians", "students", "teachers", "rooms",
    "sessions", "invoices", "payments", "attendances",
    "notifications", "audit_logs", "pricings", "registrations",
]


async def verify():
    print("=== Database Integrity Check ===\n")
    errors = 0

    async with async_session_maker() as db:
        # 1. Check all expected tables exist
        result = await db.execute(text(
            "SELECT table_name FROM information_schema.tables "
            "WHERE table_schema = 'public'"
        ))
        tables = {row[0] for row in result.fetchall()}

        for t in EXPECTED_TABLES:
            if t in tables:
                print(f"  ✓ Table {t} exists")
            else:
                print(f"  ✗ Table {t} MISSING")
                errors += 1

        # 2. Check row counts
        print("\n--- Row counts ---")
        for t in EXPECTED_TABLES:
            if t in tables:
                r = await db.execute(text(f"SELECT count(*) FROM {t}"))
                count = r.scalar()
                print(f"  {t}: {count} rows")

        # 3. Check foreign key constraints
        print("\n--- Foreign key constraints ---")
        r = await db.execute(text(
            "SELECT tc.table_name, kcu.column_name, ccu.table_name AS ref_table "
            "FROM information_schema.table_constraints tc "
            "JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name "
            "JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name "
            "WHERE tc.constraint_type = 'FOREIGN KEY' LIMIT 10"
        ))
        fks = r.fetchall()
        print(f"  Found {len(fks)} foreign keys (sample)")
        for fk in fks[:5]:
            print(f"    {fk[0]}.{fk[1]} → {fk[2]}")

        # 4. Check no NULL in critical columns
        print("\n--- Critical column checks ---")
        checks = [
            ("users", "username", "NOT NULL"),
            ("users", "password_hash", "NOT NULL"),
            ("guardians", "full_name", "NOT NULL"),
        ]
        for table, col, constraint in checks:
            if table in tables:
                r = await db.execute(text(f"SELECT count(*) FROM {table} WHERE {col} IS NULL"))
                nulls = r.scalar()
                if nulls == 0:
                    print(f"  ✓ {table}.{col}: no NULLs")
                else:
                    print(f"  ✗ {table}.{col}: {nulls} NULL values")
                    errors += 1

    print(f"\n=== Result: {'PASS' if errors == 0 else f'FAIL ({errors} errors)'} ===")
    return errors == 0


if __name__ == "__main__":
    success = asyncio.run(verify())
    sys.exit(0 if success else 1)
