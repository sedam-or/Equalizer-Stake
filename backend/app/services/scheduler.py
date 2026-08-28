"""APScheduler jobs — auto-invoice, fine+freeze, draft cleanup, session reminders.

Timezone: Asia/Jakarta (UTC+7). All cron times are WIB.
"""

import logging
import uuid
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal

from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger
from apscheduler.triggers.interval import IntervalTrigger
from sqlalchemy import or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import async_session_maker
from app.models.guardian import Guardian
from app.models.invoice import Invoice
from app.models.pricing import Pricing
from app.models.session import Session
from app.models.student import Student
from app.services.audit import create_audit_log
from app.services.billing import apply_fine, create_invoice
from app.services.draft import cleanup_expired_drafts
from app.services.notification import create_notification, send_notification

logger = logging.getLogger("scheduler")

# Sentinel UUID for system-initiated actions (audit changed_by)
SYSTEM_USER_ID = uuid.UUID("00000000-0000-0000-0000-000000000000")

WIB = timezone(timedelta(hours=7))


# ---------------------------------------------------------------------------
# 1. Auto-invoice — tanggal 5, 00:05 WIB
# ---------------------------------------------------------------------------
async def job_auto_invoice():
    """Create invoices for all active guardians on the 5th of each month."""
    now_wib = datetime.now(WIB)
    if now_wib.day != 5:
        return

    billing_period = f"{now_wib.year}-{now_wib.month - 1:02d}" if now_wib.month > 1 else f"{now_wib.year - 1}-12"
    logger.info("[auto-invoice] START billing_period=%s", billing_period)

    async with async_session_maker() as db:
        try:
            # Get active guardians
            result = await db.execute(
                select(Guardian).where(Guardian.account_status == "active")
            )
            guardians = list(result.scalars().all())

            created = 0
            for guardian in guardians:
                # Skip if invoice already exists for this period
                existing = await db.execute(
                    select(Invoice).where(
                        Invoice.id_guardian == guardian.id_guardian,
                        Invoice.billing_period == billing_period,
                    )
                )
                if existing.scalar_one_or_none():
                    continue

                # Sum pricing for guardian's students
                students_result = await db.execute(
                    select(Student).where(Student.id_guardian == guardian.id_guardian)
                )
                students = list(students_result.scalars().all())
                if not students:
                    continue

                total = Decimal("0")
                for student in students:
                    pricing_result = await db.execute(
                        select(Pricing).where(
                            Pricing.instrument == student.instrument,
                            Pricing.level == student.level,
                            Pricing.is_active == True,  # noqa: E712
                        )
                    )
                    pricing = pricing_result.scalar_one_or_none()
                    if pricing:
                        total += Decimal(str(pricing.price))

                if total <= 0:
                    continue

                invoice = await create_invoice(db, guardian.id_guardian, billing_period, total)
                await create_audit_log(
                    db,
                    table_name="invoices",
                    record_id=invoice.id_invoice,
                    action="AUTO_INVOICE",
                    changed_by=SYSTEM_USER_ID,
                    after_state={"billing_period": billing_period, "total_amount": str(total)},
                )
                created += 1

            await db.commit()
            logger.info("[auto-invoice] DONE — %d invoices created", created)
        except Exception:
            await db.rollback()
            logger.exception("[auto-invoice] FAILED")


# ---------------------------------------------------------------------------
# 2. Fine + Freeze — tanggal 11, 00:05 WIB
# ---------------------------------------------------------------------------
async def job_fine_freeze():
    """Apply Rp 100k fine and freeze unpaid guardians on the 11th."""
    now_wib = datetime.now(WIB)
    if now_wib.day != 11:
        return

    logger.info("[fine-freeze] START")

    async with async_session_maker() as db:
        try:
            result = await db.execute(
                select(Invoice).where(
                    Invoice.status == "issued",
                    Invoice.due_date < date.today(),
                )
            )
            overdue_invoices = list(result.scalars().all())

            processed = 0
            for invoice in overdue_invoices:
                before_amount = str(invoice.total_amount)
                updated = await apply_fine(db, invoice.id_invoice)
                if updated:
                    await create_audit_log(
                        db,
                        table_name="invoices",
                        record_id=invoice.id_invoice,
                        action="FINE_FREEZE",
                        changed_by=SYSTEM_USER_ID,
                        before_state={"amount": before_amount},
                        after_state={"amount": str(updated.total_amount), "status": "overdue"},
                    )
                    processed += 1

            await db.commit()
            logger.info("[fine-freeze] DONE — %d invoices fined+frozen", processed)
        except Exception:
            await db.rollback()
            logger.exception("[fine-freeze] FAILED")


# ---------------------------------------------------------------------------
# 3. Draft cleanup — daily 03:00 WIB
# ---------------------------------------------------------------------------
async def job_draft_cleanup():
    """Delete drafts older than 7 days."""
    logger.info("[draft-cleanup] START")

    async with async_session_maker() as db:
        try:
            count = await cleanup_expired_drafts(db)
            await create_audit_log(
                db,
                table_name="drafts",
                record_id=SYSTEM_USER_ID,
                action="DRAFT_CLEANUP",
                changed_by=SYSTEM_USER_ID,
                after_state={"deleted_count": count},
            )
            await db.commit()
            logger.info("[draft-cleanup] DONE — %d drafts deleted", count)
        except Exception:
            await db.rollback()
            logger.exception("[draft-cleanup] FAILED")


# ---------------------------------------------------------------------------
# 4. Session reminder — every 15 minutes
# ---------------------------------------------------------------------------
async def job_session_reminder():
    """Send reminder 1 hour before scheduled sessions."""
    now_utc = datetime.now(timezone.utc)
    window_start = now_utc + timedelta(minutes=45)
    window_end = now_utc + timedelta(minutes=60)

    # Convert to date/time for comparison with session model
    start_date = window_start.date()
    end_date = window_end.date()
    start_time = window_start.time()
    end_time = window_end.time()

    async with async_session_maker() as db:
        try:
            # Find sessions in the reminder window
            # Sessions can span midnight so check both dates
            conditions = []
            if start_date == end_date:
                conditions.append(
                    (Session.session_date == start_date)
                    & (Session.start_time >= start_time)
                    & (Session.start_time <= end_time)
                )
            else:
                conditions.append(
                    (Session.session_date == start_date) & (Session.start_time >= start_time)
                )
                conditions.append(
                    (Session.session_date == end_date) & (Session.start_time <= end_time)
                )

            result = await db.execute(
                select(Session).where(
                    Session.status.in_(["pending", "locked"]),
                    or_(*conditions),
                )
            )
            sessions = list(result.scalars().all())

            if not sessions:
                return

            sent = 0
            # Group sessions by guardian for consolidated notifications (SRS FR-05)
            guardian_sessions: dict[UUID, list] = {}
            guardian_map: dict[UUID, Guardian] = {}
            student_map: dict[UUID, Student] = {}

            for session in sessions:
                student_result = await db.execute(
                    select(Student).where(Student.id_student == session.id_student)
                )
                student = student_result.scalar_one_or_none()
                if not student:
                    continue

                guardian_result = await db.execute(
                    select(Guardian).where(Guardian.id_guardian == student.id_guardian)
                )
                guardian = guardian_result.scalar_one_or_none()
                if not guardian or guardian.account_status == "frozen":
                    continue

                gid = guardian.id_user
                if gid not in guardian_sessions:
                    guardian_sessions[gid] = []
                    guardian_map[gid] = guardian
                guardian_sessions[gid].append({
                    "session": session,
                    "student_name": student.full_name,
                })
                student_map[session.id_student] = student

            # Send one grouped notification per guardian
            for gid, items in guardian_sessions.items():
                guardian = guardian_map[gid]
                if len(items) == 1:
                    s = items[0]
                    body = (
                        f"Pengingat: Sesi {s['session'].session_type} untuk {s['student_name']} "
                        f"akan dimulai pukul {s['session'].start_time.strftime('%H:%M')} WIB. "
                        f"Harap hadir tepat waktu.\n"
                        f"Phone: {guardian.phone_whatsapp}\n"
                    )
                else:
                    student_names = ", ".join(i["student_name"] for i in items)
                    times = ", ".join(i["session"].start_time.strftime("%H:%M") for i in items)
                    body = (
                        f"Pengingat: Anda memiliki {len(items)} sesi hari ini "
                        f"untuk {student_names} "
                        f"pukul {times} WIB. "
                        f"Harap hadir tepat waktu.\n"
                        f"Phone: {guardian.phone_whatsapp}\n"
                    )

                notif = await create_notification(
                    db,
                    recipient_id=gid,
                    channel="whatsapp",
                    category="non_critical",
                    body=body,
                    subject="Pengingat Sesi",
                    related_entity_type="session",
                    related_entity_id=items[0]["session"].id_session,
                )
                await send_notification(db, notif.id_notification)
                sent += 1

            await db.commit()
            if sent:
                logger.info("[session-reminder] DONE — %d reminders sent", sent)
        except Exception:
            await db.rollback()
            logger.exception("[session-reminder] FAILED")


# ---------------------------------------------------------------------------
# ---------------------------------------------------------------------------
# 5. Database backup — daily 03:00 WIB
# ---------------------------------------------------------------------------
BACKUP_DIR = "/home/kepolu/equalizer-stake/backups"
BACKUP_RETENTION_DAYS = 30


async def job_database_backup():
    """Daily database backup via pg_dump or Python fallback. Retention: 30 days."""
    import os
    import subprocess
    import gzip
    from glob import glob
    from urllib.parse import urlparse

    os.makedirs(BACKUP_DIR, exist_ok=True)

    db_url = os.getenv("DATABASE_URL", "")
    parsed = urlparse(db_url.replace("+asyncpg", ""))
    host = parsed.hostname or "localhost"
    port = parsed.port or 5432
    dbname = parsed.path.lstrip("/")
    username = parsed.username or "postgres"
    password = parsed.password or ""

    now = datetime.now(WIB)
    filename = f"backup_{now.strftime('%Y%m%d_%H%M%S')}.sql.gz"
    filepath = os.path.join(BACKUP_DIR, filename)

    # Try pg_dump first, fall back to Python-based dump
    env = os.environ.copy()
    env["PGPASSWORD"] = password

    try:
        result = subprocess.run(
            ["pg_dump", "-h", host, "-p", str(port), "-U", username, "-d", dbname,
             "--no-owner", "--no-acl", "-Fc"],
            capture_output=True, timeout=300, env=env,
        )

        if result.returncode == 0 and result.stdout:
            with gzip.open(filepath, "wb") as f:
                f.write(result.stdout)
        else:
            # Fallback: SQL text dump via Python
            await _python_dump(db_url, filepath)
    except (FileNotFoundError, subprocess.TimeoutExpired):
        await _python_dump(db_url, filepath)
    except Exception as e:
        logger.error(f"Backup failed: {e}")
        return

    if os.path.exists(filepath):
        size_mb = os.path.getsize(filepath) / (1024 * 1024)
        logger.info(f"Database backup completed: {filename} ({size_mb:.1f} MB)")

        # Cleanup old backups
        cutoff = now - timedelta(days=BACKUP_RETENTION_DAYS)
        for old_file in glob(os.path.join(BACKUP_DIR, "backup_*.sql.gz")):
            if os.path.getmtime(old_file) < cutoff.timestamp():
                os.remove(old_file)
                logger.info(f"Removed old backup: {os.path.basename(old_file)}")


async def _python_dump(db_url: str, filepath: str):
    """Python-based SQL dump fallback using SQLAlchemy."""
    import gzip
    from sqlalchemy import text
    from app.core.database import async_session_maker

    try:
        async with async_session_maker() as db:
            # Get all table names
            result = await db.execute(text(
                "SELECT table_name FROM information_schema.tables "
                "WHERE table_schema = 'public' ORDER BY table_name"
            ))
            tables = [row[0] for row in result.fetchall()]

            with gzip.open(filepath, "wt") as f:
                f.write("-- Database backup\\n")
                f.write(f"-- Tables: {len(tables)}\\n\\n")
                for table in tables:
                    f.write(f"\\n-- Table: {table}\\n")
                    f.write(f"TRUNCATE TABLE {table} CASCADE;\\n")
                    # Dump data as INSERT statements
                    rows = await db.execute(text(f"SELECT * FROM {table}"))
                    for row in rows:
                        vals = ", ".join(repr(str(v)) if v is not None else "NULL" for v in row)
                        f.write(f"INSERT INTO {table} VALUES ({vals});\\n")
            logger.info(f"Python-based backup completed: {len(tables)} tables")
    except Exception as e:
        logger.error(f"Python dump failed: {e}")


# Scheduler setup
# ---------------------------------------------------------------------------
scheduler = AsyncIOScheduler(timezone="Asia/Jakarta")


def start_scheduler():
    """Register all jobs and start the scheduler. Call from FastAPI lifespan."""
    # Auto-invoice: daily 00:05 WIB
    scheduler.add_job(
        job_auto_invoice,
        CronTrigger(hour=0, minute=5, timezone="Asia/Jakarta"),
        id="auto_invoice",
        replace_existing=True,
    )

    # Fine+freeze: daily 00:05 WIB
    scheduler.add_job(
        job_fine_freeze,
        CronTrigger(hour=0, minute=5, timezone="Asia/Jakarta"),
        id="fine_freeze",
        replace_existing=True,
    )

    # Draft cleanup: daily 03:00 WIB
    scheduler.add_job(
        job_draft_cleanup,
        CronTrigger(hour=3, minute=0, timezone="Asia/Jakarta"),
        id="draft_cleanup",
        replace_existing=True,
    )

    # Database backup: daily 03:00 WIB
    scheduler.add_job(
        job_database_backup,
        CronTrigger(hour=3, minute=0, timezone="Asia/Jakarta"),
        id="database_backup",
        replace_existing=True,
    )

    # Session reminder: every 15 minutes
    scheduler.add_job(
        job_session_reminder,
        IntervalTrigger(minutes=15),
        id="session_reminder",
        replace_existing=True,
    )

    scheduler.start()
    logger.info("Scheduler started with 5 jobs")


def shutdown_scheduler():
    """Gracefully shut down the scheduler."""
    scheduler.shutdown(wait=False)
    logger.info("Scheduler shut down")
