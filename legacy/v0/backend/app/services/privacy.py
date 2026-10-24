"""Privacy service — UU PDP data deletion (Pasal 19)."""

from datetime import datetime, timedelta, timezone
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

MAX_BUSINESS_DAYS = 14


async def create_deletion_request(
    db: AsyncSession,
    user_id: UUID,
    reason: str,
    entity_type: str,
    entity_id: UUID,
) -> dict:
    """Create a data deletion request (UU PDP Pasal 19)."""
    from app.models.deletion_request import DeletionRequest

    # Check if entity has active invoices
    if entity_type == "guardian":
        from app.models.invoice import Invoice
        result = await db.execute(
            select(Invoice).where(
                Invoice.id_guardian == entity_id,
                Invoice.status.in_(["issued", "overdue"]),
                Invoice.deleted_at.is_(None),
            )
        )
        active_invoices = result.scalars().all()
        if active_invoices:
            return {
                "status": "rejected",
                "reason": "Masih ada tagihan aktif. Lunasi terlebih dahulu.",
            }

    # Create deletion request record
    request = DeletionRequest(
        id_user=user_id,
        entity_type=entity_type,
        entity_id=entity_id,
        reason=reason,
        status="pending",
    )
    db.add(request)
    await db.commit()
    await db.refresh(request)

    return {
        "status": "pending",
        "id_request": str(request.id_request),
        "entity_type": entity_type,
        "entity_id": str(entity_id),
        "reason": reason,
        "created_at": request.created_at.isoformat() if request.created_at else None,
        "deadline": (datetime.now(timezone.utc) + timedelta(days=MAX_BUSINESS_DAYS)).isoformat(),
    }


async def complete_deletion(
    db: AsyncSession,
    request_id: str,
    verified_by: UUID,
) -> dict:
    """Admin verify and complete deletion."""
    from app.models.deletion_request import DeletionRequest

    result = await db.execute(
        select(DeletionRequest).where(DeletionRequest.id_request == UUID(request_id))
    )
    request = result.scalar_one_or_none()

    if not request:
        return {"error": "Request tidak ditemukan"}

    if request.status != "pending":
        return {"error": f"Request sudah {request.status}"}

    # Soft delete + anonymize entity
    if request.entity_type == "guardian":
        from app.models.guardian import Guardian
        entity_result = await db.execute(
            select(Guardian).where(Guardian.id_guardian == request.entity_id)
        )
        entity = entity_result.scalar_one_or_none()
        if entity:
            # Anonymize PII
            anonymize_pii(entity, "guardian")
            entity.account_status = "inactive"
            entity.deleted_at = datetime.now(timezone.utc)

    elif request.entity_type == "teacher":
        from app.models.teacher import Teacher
        entity_result = await db.execute(
            select(Teacher).where(Teacher.id_teacher == request.entity_id)
        )
        entity = entity_result.scalar_one_or_none()
        if entity:
            anonymize_pii(entity, "teacher")
            entity.deleted_at = datetime.now(timezone.utc)

    # Update request status
    request.status = "approved"
    request.processed_by = verified_by
    request.processed_at = datetime.now(timezone.utc)

    await db.commit()

    return {
        "status": "approved",
        "id_request": str(request.id_request),
        "entity_type": request.entity_type,
        "entity_id": str(request.entity_id),
        "processed_at": request.processed_at.isoformat() if request.processed_at else None,
    }


def anonymize_pii(entity, entity_type: str) -> dict:
    """Anonymize PII fields with REDACTED-[hash] pattern."""
    import hashlib

    changes = {}

    if entity_type == "guardian":
        if entity.full_name:
            hash_val = hashlib.sha256(entity.full_name.encode()).hexdigest()[:8]
            entity.full_name = f"REDACTED-{hash_val}"
            changes["full_name"] = True
        if entity.phone_whatsapp:
            hash_val = hashlib.sha256(entity.phone_whatsapp.encode()).hexdigest()[:8]
            entity.phone_whatsapp = f"REDACTED-{hash_val}"
            changes["phone_whatsapp"] = True
        if entity.email:
            hash_val = hashlib.sha256(entity.email.encode()).hexdigest()[:8]
            entity.email = f"REDACTED-{hash_val}"
            changes["email"] = True

    elif entity_type == "teacher":
        if entity.full_name:
            hash_val = hashlib.sha256(entity.full_name.encode()).hexdigest()[:8]
            entity.full_name = f"REDACTED-{hash_val}"
            changes["full_name"] = True

    return changes
