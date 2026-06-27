"""Privacy service — UU PDP data deletion flow."""

from datetime import datetime, timezone, timedelta
from uuid import UUID
import hashlib

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.user import User
from app.models.guardian import Guardian
from app.models.student import Student


# Business rule: max 14 business days to complete deletion
MAX_BUSINESS_DAYS = 14


async def create_deletion_request(
    db: AsyncSession,
    user_id: UUID,
    reason: str,
    entity_type: str,  # "guardian" or "teacher"
    entity_id: UUID,
) -> dict:
    """Create a data deletion request (UU PDP Pasal 19)."""
    # Check if entity has active invoices
    if entity_type == "guardian":
        from app.models.invoice import Invoice
        result = await db.execute(
            select(Invoice).where(
                Invoice.id_guardian == entity_id,
                Invoice.status.in_(["issued", "overdue"]),
            )
        )
        active_invoices = result.scalars().all()
        if active_invoices:
            return {
                "status": "rejected",
                "reason": "Masih ada tagihan aktif. Lunasi terlebih dahulu.",
            }

    # Create deletion request record
    request = {
        "id": str(UUID()),
        "user_id": str(user_id),
        "entity_type": entity_type,
        "entity_id": str(entity_id),
        "reason": reason,
        "status": "pending",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "deadline": (datetime.now(timezone.utc) + timedelta(days=MAX_BUSINESS_DAYS)).isoformat(),
    }

    return request


async def anonymize_pii(entity, entity_type: str) -> dict:
    """Anonymize PII fields with REDACTED-[hash] pattern."""
    changes = {}

    if entity_type == "guardian":
        # Hash name for anonymization
        name_hash = hashlib.sha256(str(entity.id_guardian).encode()).hexdigest()[:8]
        changes["full_name"] = f"REDACTED-{name_hash}"
        changes["phone"] = f"REDACTED-{name_hash}"
        changes["email"] = f"REDACTED-{name_hash}@deleted.local"
        changes["address"] = f"REDACTED-{name_hash}"
        changes["ktp_number"] = f"REDACTED-{name_hash}"
        changes["is_anonymized"] = True
        changes["anonymized_at"] = datetime.now(timezone.utc).isoformat()

    elif entity_type == "student":
        name_hash = hashlib.sha256(str(entity.id_student).encode()).hexdigest()[:8]
        changes["full_name"] = f"REDACTED-{name_hash}"
        changes["is_anonymized"] = True
        changes["anonymized_at"] = datetime.now(timezone.utc).isoformat()

    elif entity_type == "teacher":
        name_hash = hashlib.sha256(str(entity.id_teacher).encode()).hexdigest()[:8]
        changes["full_name"] = f"REDACTED-{name_hash}"
        changes["phone"] = f"REDACTED-{name_hash}"
        changes["email"] = f"REDACTED-{name_hash}@deleted.local"
        changes["is_anonymized"] = True
        changes["anonymized_at"] = datetime.now(timezone.utc).isoformat()

    return changes


async def delete_photos(entity_type: str, entity_id: UUID) -> int:
    """Permanently delete photos for entity."""
    # TODO: Delete from object storage
    # Return count of deleted photos
    return 0


async def complete_deletion(
    db: AsyncSession,
    request_id: str,
    verified_by: UUID,
) -> dict:
    """Complete the deletion process with audit log."""
    # 1. Anonymize PII
    # 2. Delete photos
    # 3. Create audit log with category "Tindakan Kepatuhan"
    # 4. Update request status to "completed"

    return {
        "status": "completed",
        "completed_at": datetime.now(timezone.utc).isoformat(),
        "verified_by": str(verified_by),
    }
