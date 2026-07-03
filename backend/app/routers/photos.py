"""Photo upload/retrieval endpoints."""

import base64
from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import get_current_user
from app.models.user import User
from app.services.photo_storage import (
    validate_photo,
    save_attendance_photo,
    save_profile_photo,
    load_photo,
    delete_photo,
)

router = APIRouter()


class PhotoUploadRequest(BaseModel):
    photo_base64: str


class PhotoUploadResponse(BaseModel):
    message: str
    photo_path: str


@router.post("/attendance/{session_id}/photo", response_model=PhotoUploadResponse)
async def upload_attendance_photo(
    session_id: str,
    body: PhotoUploadRequest,
    user: User = Depends(get_current_user),
):
    """Upload attendance photo (encrypted)."""
    valid, error = validate_photo(body.photo_base64)
    if not valid:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=error)
    
    try:
        path = save_attendance_photo(body.photo_base64, session_id)
        return PhotoUploadResponse(message="Foto berhasil diupload", photo_path=path)
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail=str(e))


@router.get("/attendance/{session_id}/photo")
async def get_attendance_photo(
    session_id: str,
    user: User = Depends(get_current_user),
):
    """Get decrypted attendance photo."""
    # Find photo file
    from pathlib import Path
    import os
    
    photo_dir = Path(os.getenv("UPLOAD_DIR", "/home/kepolu/bmdmusic/backend/uploads")) / "attendance"
    files = list(photo_dir.glob(f"{session_id}_*.enc"))
    
    if not files:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Foto tidak ditemukan")
    
    photo_data = load_photo(str(files[0]))
    if photo_data is None:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Gagal membaca foto")
    
    return {"photo_data": photo_data}


@router.post("/profile/photo", response_model=PhotoUploadResponse)
async def upload_profile_photo(
    body: PhotoUploadRequest,
    user: User = Depends(get_current_user),
):
    """Upload profile photo (encrypted)."""
    valid, error = validate_photo(body.photo_base64)
    if not valid:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=error)
    
    try:
        path = save_profile_photo(body.photo_base64, str(user.id_user))
        return PhotoUploadResponse(message="Foto profil berhasil diupload", photo_path=path)
    except ValueError as e:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail=str(e))


@router.get("/profile/photo")
async def get_profile_photo(
    user: User = Depends(get_current_user),
):
    """Get decrypted profile photo."""
    import os
    from pathlib import Path
    
    photo_dir = Path(os.getenv("UPLOAD_DIR", "/home/kepolu/bmdmusic/backend/uploads")) / "profiles"
    filepath = photo_dir / f"{user.id_user}.enc"
    
    if not filepath.exists():
        return {"photo_data": None}
    
    photo_data = load_photo(str(filepath))
    return {"photo_data": photo_data}


@router.delete("/profile/photo")
async def delete_profile_photo(
    user: User = Depends(get_current_user),
):
    """Delete profile photo."""
    import os
    from pathlib import Path
    
    photo_dir = Path(os.getenv("UPLOAD_DIR", "/home/kepolu/bmdmusic/backend/uploads")) / "profiles"
    filepath = photo_dir / f"{user.id_user}.enc"
    
    if filepath.exists():
        delete_photo(str(filepath))
    
    return {"message": "Foto profil berhasil dihapus"}
