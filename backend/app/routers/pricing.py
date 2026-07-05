"""Pricing router — tarif management (Owner + Admin)."""

from datetime import date
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.models.pricing import Pricing
from app.services.audit import create_audit_log
from app.services.pricing import (
    create_pricing,
    get_pricing,
    update_pricing,
    validate_pricing_input,
)

router = APIRouter(prefix="/api/v1/pricing", tags=["Pricing"])


@router.get("")
async def list_pricing(
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """List all pricing entries (Owner + Admin)."""
    base = select(Pricing)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(
        base.order_by(Pricing.instrument, Pricing.level, Pricing.effective_date.desc())
        .offset(offset).limit(limit)
    )
    pricings = result.scalars().all()
    return {
        "data": [
            {
                "id_pricing": str(p.id_pricing),
                "instrument": p.instrument,
                "level": p.level,
                "price": float(p.monthly_fee),
                "effective_date": p.effective_date.isoformat(),
                "description": p.description,
                "is_active": p.is_active,
                "created_at": p.created_at.isoformat() if p.created_at else None,
            }
            for p in pricings
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


@router.get("/{pricing_id}")
async def get_single_pricing(
    pricing_id: str,
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Get single pricing entry (Owner + Admin)."""
    pricing = await get_pricing(db, UUID(pricing_id))
    if not pricing:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Tarif tidak ditemukan",
        )
    return {
        "id_pricing": str(pricing.id_pricing),
        "instrument": pricing.instrument,
        "level": pricing.level,
        "price": float(pricing.monthly_fee),
        "effective_date": pricing.effective_date.isoformat(),
        "description": pricing.description,
        "is_active": pricing.is_active,
        "created_at": pricing.created_at.isoformat() if pricing.created_at else None,
    }


@router.post("")
async def create_pricing_endpoint(
    request: Request,
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Create new pricing entry (Owner + Admin)."""
    body = await request.json()
    instrument = body.get("instrument")
    level = body.get("level")
    price = body.get("price")
    effective_date = body.get("effective_date")
    description = body.get("description")

    # Validate required fields
    if not instrument or not level or price is None or not effective_date:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Field 'instrument', 'level', 'price', dan 'effective_date' wajib diisi",
        )

    # Validate values
    error = validate_pricing_input(instrument, level, float(price))
    if error:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail=error,
        )

    try:
        eff_date = date.fromisoformat(effective_date)
    except ValueError:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Format tanggal tidak valid. Gunakan YYYY-MM-DD",
        )

    pricing = await create_pricing(
        db,
        instrument=instrument,
        level=level,
        price=float(price),
        effective_date=eff_date,
        description=description,
        created_by=user.id_user,
    )
    await db.flush()

    await create_audit_log(
        db,
        table_name="pricings",
        record_id=pricing.id_pricing,
        action="INSERT",
        changed_by=user.id_user,
        after_state={
            "instrument": instrument,
            "level": level,
            "price": float(price),
            "effective_date": effective_date,
        },
        request=request,
    )
    await db.commit()

    return {
        "detail": "Tarif berhasil ditambahkan",
        "id_pricing": str(pricing.id_pricing),
        "instrument": pricing.instrument,
        "level": pricing.level,
        "price": float(pricing.monthly_fee),
        "effective_date": pricing.effective_date.isoformat(),
    }


@router.put("/{pricing_id}")
async def update_pricing_endpoint(
    pricing_id: str,
    request: Request,
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Update pricing entry (Owner + Admin)."""
    pricing = await get_pricing(db, UUID(pricing_id))
    if not pricing:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Tarif tidak ditemukan",
        )

    body = await request.json()
    before_state = {
        "instrument": pricing.instrument,
        "level": pricing.level,
        "price": float(pricing.monthly_fee),
        "effective_date": pricing.effective_date.isoformat(),
        "is_active": pricing.is_active,
    }

    # Validate if instrument/level/price provided
    instrument = body.get("instrument", pricing.instrument)
    level = body.get("level", pricing.level)
    price = float(body.get("price", pricing.monthly_fee))
    error = validate_pricing_input(instrument, level, price)
    if error:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail=error,
        )

    eff_date = None
    if "effective_date" in body:
        try:
            eff_date = date.fromisoformat(body["effective_date"])
        except ValueError:
            raise HTTPException(
                status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
                detail="Format tanggal tidak valid",
            )

    await update_pricing(
        db,
        pricing,
        instrument=instrument,
        level=level,
        price=price,
        effective_date=eff_date,
        description=body.get("description"),
        is_active=body.get("is_active"),
    )
    await db.flush()

    after_state = {
        "instrument": pricing.instrument,
        "level": pricing.level,
        "price": float(pricing.monthly_fee),
        "effective_date": pricing.effective_date.isoformat(),
        "is_active": pricing.is_active,
    }

    await create_audit_log(
        db,
        table_name="pricings",
        record_id=pricing.id_pricing,
        action="UPDATE",
        changed_by=user.id_user,
        before_state=before_state,
        after_state=after_state,
        request=request,
    )
    await db.commit()

    return {
        "detail": "Tarif berhasil diperbarui",
        "id_pricing": str(pricing.id_pricing),
        "instrument": pricing.instrument,
        "level": pricing.level,
        "price": float(pricing.monthly_fee),
        "effective_date": pricing.effective_date.isoformat(),
    }
