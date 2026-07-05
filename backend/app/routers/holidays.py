"""Holiday router — admin CRUD for holidays."""

from datetime import date
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from pydantic import BaseModel, Field

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User

router = APIRouter(prefix="/api/v1/holidays", tags=["Holidays"])

# In-memory holidays (ponytail: use DB table when migration ready)
_HOLIDAYS: list[dict] = [
    {"date": "2026-01-01", "name": "Tahun Baru", "type": "nasional"},
    {"date": "2026-08-17", "name": "Kemerdekaan RI", "type": "nasional"},
    {"date": "2026-12-25", "name": "Natal", "type": "nasional"},
]


class HolidayCreate(BaseModel):
    date: str = Field(..., pattern=r"^\d{4}-\d{2}-\d{2}$")
    name: str = Field(..., min_length=2, max_length=100)
    type: str = Field("nasional", pattern=r"^(nasional|libur)$")


@router.get("")
async def list_holidays(
    limit: int = 20,
    offset: int = 0,
    user: User = Depends(require_role("admin", "owner")),
):
    """List all holidays."""
    total = len(_HOLIDAYS)
    data = _HOLIDAYS[offset:offset + limit]
    return {"data": data, "total": total, "limit": limit, "offset": offset}


@router.post("", status_code=status.HTTP_201_CREATED)
async def create_holiday(
    body: HolidayCreate,
    user: User = Depends(require_role("admin")),
):
    """Add new holiday."""
    # Check duplicate date
    if any(h["date"] == body.date for h in _HOLIDAYS):
        raise HTTPException(status_code=400, detail="Tanggal sudah terdaftar")
    holiday = {"date": body.date, "name": body.name, "type": body.type}
    _HOLIDAYS.append(holiday)
    return holiday


@router.delete("/{date}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_holiday(
    date: str,
    user: User = Depends(require_role("admin")),
):
    """Delete holiday by date."""
    global _HOLIDAYS
    _HOLIDAYS = [h for h in _HOLIDAYS if h["date"] != date]
