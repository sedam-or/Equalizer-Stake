"""Import service — CSV import for students, guardians, sessions."""

import csv
import io
from uuid import UUID
from datetime import date, datetime, timezone

from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select


def parse_csv(content: str) -> list[dict]:
    """Parse CSV string to list of dicts."""
    reader = csv.DictReader(io.StringIO(content))
    return list(reader)


def validate_student_row(row: dict, idx: int) -> list[str]:
    """Validate a student CSV row. Returns list of errors."""
    errors = []
    required = ["full_name", "instrument", "level"]
    for field in required:
        if not row.get(field, "").strip():
            errors.append(f"Baris {idx}: kolom '{field}' wajib diisi")

    valid_instruments = ["gitar", "piano", "biola", "drum", "keyboard", "flute", "saxophone", "vokal"]
    if row.get("instrument", "").lower().strip() not in valid_instruments:
        errors.append(f"Baris {idx}: instrumen '{row.get('instrument')}' tidak valid. Pilihan: {', '.join(valid_instruments)}")

    return errors


def validate_guardian_row(row: dict, idx: int) -> list[str]:
    """Validate a guardian CSV row."""
    errors = []
    if not row.get("full_name", "").strip():
        errors.append(f"Baris {idx}: kolom 'full_name' wajib diisi")
    if not row.get("phone_whatsapp", "").strip():
        errors.append(f"Baris {idx}: kolom 'phone_whatsapp' wajib diisi")
    return errors


async def preview_import(db: AsyncSession, entity_type: str, rows: list[dict]) -> dict:
    """Preview import — validate all rows without saving."""
    errors = []
    valid_count = 0

    for idx, row in enumerate(rows, 1):
        if entity_type == "students":
            row_errors = validate_student_row(row, idx)
        elif entity_type == "guardians":
            row_errors = validate_guardian_row(row, idx)
        else:
            row_errors = [f"Baris {idx}: tipe '{entity_type}' tidak didukung"]

        if row_errors:
            errors.extend(row_errors)
        else:
            valid_count += 1

    return {
        "total_rows": len(rows),
        "valid_rows": valid_count,
        "invalid_rows": len(rows) - valid_count,
        "errors": errors[:20],  # max 20 errors shown
        "preview": rows[:5],  # first 5 rows
    }


async def execute_import(
    db: AsyncSession,
    entity_type: str,
    rows: list[dict],
    imported_by: UUID,
) -> dict:
    """Execute import — save valid rows to DB."""
    from app.models.student import Student
    from app.models.guardian import Guardian

    imported = 0
    skipped = 0

    if entity_type == "students":
        for idx, row in enumerate(rows, 1):
            errors = validate_student_row(row, idx)
            if errors:
                skipped += 1
                continue

            # Lookup guardian by phone
            guardian_phone = row.get("guardian_phone", "").strip()
            if guardian_phone:
                from app.models.guardian import Guardian
                g_result = await db.execute(
                    select(Guardian).where(Guardian.phone_whatsapp == guardian_phone)
                )
                guardian = g_result.scalar_one_or_none()
                if not guardian:
                    skipped += 1
                    continue
                guardian_id = guardian.id_guardian
            else:
                skipped += 1
                continue

            student = Student(
                full_name=row["full_name"].strip(),
                instrument=row["instrument"].strip().lower(),
                level=row.get("level", "pemula").strip(),
                enrollment_status=row.get("enrollment_status", "active").strip(),
                id_guardian=guardian_id,
                enrollment_date=date.today(),
            )
            db.add(student)
            imported += 1

    elif entity_type == "guardians":
        for idx, row in enumerate(rows, 1):
            errors = validate_guardian_row(row, idx)
            if errors:
                skipped += 1
                continue

            guardian = Guardian(
                full_name=row["full_name"].strip(),
                phone_whatsapp=row["phone_whatsapp"].strip(),
            )
            db.add(guardian)
            imported += 1

    await db.commit()

    return {
        "imported": imported,
        "skipped": skipped,
        "total": len(rows),
    }
