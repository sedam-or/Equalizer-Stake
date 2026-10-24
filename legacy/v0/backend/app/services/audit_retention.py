"""Audit log retention service — archive old logs, anonymize PII.

SRS T-052: 7-year retention, PII anonymization after 2 years.
Actual cron scheduling will be added to scheduler.py.
"""

from datetime import datetime, timedelta, timezone

from sqlalchemy import delete, select, text, update
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.audit_log import AuditLog


ARCHIVE_YEARS = 7
ANONYMIZE_YEARS = 2

# PII fields to anonymize in after_state/before_state JSON
PII_KEYS = {"phone", "email", "address", "nik", "npwp", "date_of_birth", "full_name", "ip_address"}


def _anonymize_state(state: dict | None) -> dict | None:
    """Replace PII values with [REDACTED] in audit state dict."""
    if not state:
        return state
    return {k: "[REDACTED]" if k.lower() in PII_KEYS else v for k, v in state.items()}


async def archive_old_logs(db: AsyncSession) -> int:
    """Move logs older than 7 years to audit_logs_archive.

    Returns count of archived rows.
    """
    cutoff = datetime.now(timezone.utc) - timedelta(days=ARCHIVE_YEARS * 365)

    # Insert old logs into archive
    await db.execute(
        text(
            """
            INSERT INTO audit_logs_archive
            SELECT * FROM audit_logs WHERE created_at < :cutoff
            """
        ),
        {"cutoff": cutoff},
    )

    # Delete from main table
    result = await db.execute(
        text("DELETE FROM audit_logs WHERE created_at < :cutoff"),
        {"cutoff": cutoff},
    )
    await db.commit()
    return result.rowcount


async def anonymize_pii_logs(db: AsyncSession) -> int:
    """Anonymize PII in logs older than 2 years.

    Overwrites ip_address, user_agent, before_state, after_state PII fields.
    Returns count of anonymized rows.
    """
    cutoff = datetime.now(timezone.utc) - timedelta(days=ANONYMIZE_YEARS * 365)

    # Fetch candidate rows
    result = await db.execute(
        select(AuditLog).where(AuditLog.created_at < cutoff, AuditLog.ip_address.isnot(None))
    )
    logs = result.scalars().all()

    count = 0
    for log in logs:
        log.ip_address = None
        log.user_agent = "[REDACTED]"
        log.before_state = _anonymize_state(log.before_state)
        log.after_state = _anonymize_state(log.after_state)
        count += 1

    if count:
        await db.commit()
    return count
