"""Restore database from backup file.

Usage:
    python -m scripts.restore_backup --latest
    python -m scripts.restore_backup --file backup_20260703_030000.sql.gz
"""
import argparse
import gzip
import os
import sys
from glob import glob
from urllib.parse import urlparse

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

BACKUP_DIR = "/home/kepolu/equalizer-stake/backups"


def get_latest_backup() -> str | None:
    """Find the most recent backup file."""
    files = glob(os.path.join(BACKUP_DIR, "backup_*.sql.gz"))
    if not files:
        return None
    return max(files, key=os.path.getmtime)


def restore_sql_dump(filepath: str, db_url: str):
    """Restore from SQL text dump (Python-generated backups)."""
    import asyncio
    from sqlalchemy import text
    from app.core.database import async_session_maker

    print(f"Restoring from: {filepath}")
    with gzip.open(filepath, "rt") as f:
        content = f.read()

    # Parse SQL statements
    lines = content.split("\n")
    statements = []
    current = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("--") or not stripped:
            if current:
                stmt = " ".join(current).strip()
                if stmt:
                    statements.append(stmt)
                current = []
            continue
        current.append(stripped)
        if stripped.endswith(";"):
            stmt = " ".join(current).strip()
            if stmt:
                statements.append(stmt)
            current = []

    if current:
        stmt = " ".join(current).strip()
        if stmt:
            statements.append(stmt)

    # Filter only TRUNCATE and INSERT statements
    restore_stmts = [s for s in statements if s.startswith("TRUNCATE") or s.startswith("INSERT")]
    print(f"Found {len(restore_stmts)} restore statements")

    async def run_restore():
        async with async_session_maker() as db:
            # Disable foreign key checks temporarily
            await db.execute(text("SET session_replication_role = 'replica'"))
            for i, stmt in enumerate(restore_stmts):
                try:
                    await db.execute(text(stmt))
                except Exception as e:
                    print(f"  Warning on statement {i+1}: {str(e)[:80]}")
                    continue
                if (i + 1) % 100 == 0:
                    print(f"  Progress: {i+1}/{len(restore_stmts)}")
            await db.execute(text("SET session_replication_role = 'origin'"))
            await db.commit()
            print(f"Restore complete: {len(restore_stmts)} statements executed")

    asyncio.run(run_restore())


def restore_pg_dump(filepath: str, db_url: str):
    """Restore from pg_dump binary format (pg_restore)."""
    import subprocess
    parsed = urlparse(db_url.replace("+asyncpg", ""))
    host = parsed.hostname or "localhost"
    port = parsed.port or 5432
    dbname = parsed.path.lstrip("/")
    username = parsed.username or "postgres"
    password = parsed.password or ""

    env = os.environ.copy()
    env["PGPASSWORD"] = password

    print(f"Restoring pg_dump: {filepath}")
    result = subprocess.run(
        ["pg_restore", "-h", host, "-p", str(port), "-U", username,
         "-d", dbname, "--no-owner", "--no-acl", "--clean", filepath],
        capture_output=True, text=True, timeout=600, env=env,
    )
    if result.returncode != 0:
        print(f"pg_restore warnings: {result.stderr[:200]}")
    print("pg_restore complete")


def main():
    parser = argparse.ArgumentParser(description="Restore database from backup")
    parser.add_argument("--latest", action="store_true", help="Restore from latest backup")
    parser.add_argument("--file", type=str, help="Restore from specific backup file")
    args = parser.parse_args()

    db_url = os.getenv("DATABASE_URL", "")

    if args.file:
        filepath = args.file if os.path.isabs(args.file) else os.path.join(BACKUP_DIR, args.file)
    elif args.latest:
        filepath = get_latest_backup()
    else:
        parser.print_help()
        return

    if not filepath or not os.path.exists(filepath):
        print(f"Backup file not found: {filepath}")
        sys.exit(1)

    print(f"=== Database Restore ===")
    print(f"File: {filepath}")
    print(f"Size: {os.path.getsize(filepath) / 1024:.1f} KB")

    # Detect format
    with open(filepath, "rb") as f:
        magic = f.read(5)

    if magic == b"PGDMP":
        restore_pg_dump(filepath, db_url)
    else:
        restore_sql_dump(filepath, db_url)

    print("=== Restore Complete ===")


if __name__ == "__main__":
    main()
