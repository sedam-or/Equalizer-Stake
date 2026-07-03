"""Import router — CSV import for students, guardians."""

from fastapi import APIRouter, Depends, UploadFile, File
from sqlalchemy.ext.asyncio import AsyncSession
from pydantic import BaseModel

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.user import User
from app.services.audit import create_audit_log
from app.services.import_service import parse_csv, preview_import, execute_import

router = APIRouter(prefix="/api/v1/admin/import", tags=["Data Import"])


class ImportPreviewRequest(BaseModel):
    entity_type: str  # students, guardians
    csv_content: str


@router.post("/preview")
async def preview(
    data: ImportPreviewRequest,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Preview CSV import — validate without saving."""
    rows = parse_csv(data.csv_content)
    result = await preview_import(db, data.entity_type, rows)
    return result


class ImportExecuteRequest(BaseModel):
    entity_type: str
    csv_content: str


@router.post("/execute")
async def execute(
    data: ImportExecuteRequest,
    user: User = Depends(require_role("admin", "owner")),
    db: AsyncSession = Depends(get_db),
):
    """Execute CSV import — save valid rows to DB."""
    rows = parse_csv(data.csv_content)
    result = await execute_import(db, data.entity_type, rows, user.id_user)

    await create_audit_log(
        db,
        table_name="data_import",
        record_id=user.id_user,
        action="INSERT",
        changed_by=user.id_user,
        after_state={
            "entity_type": data.entity_type,
            "imported": result["imported"],
            "skipped": result["skipped"],
        },
        category="keamanan",
    )
    await db.commit()

    return {
        "message": f"Import selesai: {result['imported']} data masuk, {result['skipped']} dilewati",
        "result": result,
    }
